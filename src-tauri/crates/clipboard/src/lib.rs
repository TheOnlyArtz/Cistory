use chrono::Utc;
use domain::NewClipboardEntry;
use parking_lot::Mutex;
use std::{
    borrow::Cow,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClipboardError {
    #[error(transparent)]
    Backend(#[from] arboard::Error),
    #[error(transparent)]
    Domain(#[from] domain::DomainError),
    #[error("clipboard snapshot kind requires specialized entry mapping")]
    UnsupportedSnapshotKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardSnapshot {
    Text {
        content: String,
        content_hash: String,
    },
    Image {
        bytes: Vec<u8>,
        width: usize,
        height: usize,
        content_hash: String,
    },
}

pub trait ClipboardBackend: Send + Sync + 'static {
    fn get_text(&mut self) -> Result<String, arboard::Error>;
    fn set_text(&mut self, content: String) -> Result<(), arboard::Error>;
    fn get_image(&mut self) -> Result<arboard::ImageData<'static>, arboard::Error>;
    fn set_image(&mut self, image: arboard::ImageData<'_>) -> Result<(), arboard::Error>;
}

pub struct ArboardBackend {
    clipboard: arboard::Clipboard,
}

impl ArboardBackend {
    pub fn new() -> Result<Self, arboard::Error> {
        Ok(Self {
            clipboard: arboard::Clipboard::new()?,
        })
    }
}

impl ClipboardBackend for ArboardBackend {
    fn get_text(&mut self) -> Result<String, arboard::Error> {
        self.clipboard.get_text()
    }

    fn set_text(&mut self, content: String) -> Result<(), arboard::Error> {
        self.clipboard.set_text(content)
    }

    fn get_image(&mut self) -> Result<arboard::ImageData<'static>, arboard::Error> {
        self.clipboard.get_image()
    }

    fn set_image(&mut self, image: arboard::ImageData<'_>) -> Result<(), arboard::Error> {
        self.clipboard.set_image(image)
    }
}

pub struct ClipboardService<B: ClipboardBackend> {
    backend: Mutex<B>,
    last_written_hash: Mutex<Option<String>>,
    last_seen_hash: Mutex<Option<String>>,
}

impl<B: ClipboardBackend> ClipboardService<B> {
    pub fn new(backend: B) -> Self {
        Self {
            backend: Mutex::new(backend),
            last_written_hash: Mutex::new(None),
            last_seen_hash: Mutex::new(None),
        }
    }

    pub fn snapshot(&self) -> Result<Option<ClipboardSnapshot>, ClipboardError> {
        let mut backend = self.backend.lock();
        let snapshot = match backend.get_text() {
            Ok(content) if !content.trim().is_empty() => {
                let content_hash = hash_text(&content);
                ClipboardSnapshot::Text {
                    content,
                    content_hash,
                }
            }
            Ok(_) | Err(arboard::Error::ContentNotAvailable) => {
                let image = match backend.get_image() {
                    Ok(image) => image,
                    Err(arboard::Error::ContentNotAvailable) => return Ok(None),
                    Err(error) => return Err(ClipboardError::Backend(error)),
                };

                let content_hash = hash_bytes(image.bytes.as_ref());
                ClipboardSnapshot::Image {
                    bytes: image.bytes.into_owned(),
                    width: image.width,
                    height: image.height,
                    content_hash,
                }
            }
            Err(error) => return Err(ClipboardError::Backend(error)),
        };
        let content_hash = match &snapshot {
            ClipboardSnapshot::Text { content_hash, .. } => content_hash,
            ClipboardSnapshot::Image { content_hash, .. } => content_hash,
        };

        if self
            .last_written_hash
            .lock()
            .as_ref()
            .is_some_and(|last_written| last_written == content_hash)
        {
            return Ok(None);
        }

        {
            let mut last_seen_hash = self.last_seen_hash.lock();
            if last_seen_hash
                .as_ref()
                .is_some_and(|last_seen| last_seen == content_hash)
            {
                return Ok(None);
            }
            *last_seen_hash = Some(content_hash.clone());
        }

        Ok(Some(snapshot))
    }

    pub fn write_text(&self, content: String) -> Result<(), ClipboardError> {
        let hash = hash_text(&content);
        self.backend.lock().set_text(content)?;
        *self.last_written_hash.lock() = Some(hash);
        *self.last_seen_hash.lock() = None;
        Ok(())
    }

    pub fn write_image(
        &self,
        width: usize,
        height: usize,
        bytes: Vec<u8>,
    ) -> Result<(), ClipboardError> {
        let hash = hash_bytes(&bytes);
        let image = arboard::ImageData {
            width,
            height,
            bytes: Cow::Owned(bytes),
        };
        self.backend.lock().set_image(image)?;
        *self.last_written_hash.lock() = Some(hash);
        *self.last_seen_hash.lock() = None;
        Ok(())
    }

    pub fn to_entry(
        snapshot: ClipboardSnapshot,
        source_app: Option<String>,
    ) -> Result<NewClipboardEntry, ClipboardError> {
        match snapshot {
            ClipboardSnapshot::Text {
                content,
                content_hash,
            } => Ok(NewClipboardEntry::new_text(
                content,
                content_hash,
                source_app,
                Utc::now(),
            )?),
            ClipboardSnapshot::Image { .. } => Err(ClipboardError::UnsupportedSnapshotKind),
        }
    }
}

pub fn hash_text(value: &str) -> String {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn hash_bytes(value: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[derive(Default)]
    struct FakeBackend {
        current: String,
        image: Option<(usize, usize, Vec<u8>)>,
    }

    impl ClipboardBackend for FakeBackend {
        fn get_text(&mut self) -> Result<String, arboard::Error> {
            Ok(self.current.clone())
        }

        fn set_text(&mut self, content: String) -> Result<(), arboard::Error> {
            self.current = content;
            Ok(())
        }

        fn get_image(&mut self) -> Result<arboard::ImageData<'static>, arboard::Error> {
            let Some((width, height, bytes)) = &self.image else {
                return Err(arboard::Error::ContentNotAvailable);
            };

            Ok(arboard::ImageData {
                width: *width,
                height: *height,
                bytes: Cow::Owned(bytes.clone()),
            })
        }

        fn set_image(&mut self, image: arboard::ImageData<'_>) -> Result<(), arboard::Error> {
            self.image = Some((image.width, image.height, image.bytes.into_owned()));
            Ok(())
        }
    }

    #[test]
    fn skips_empty_snapshots() {
        let service = ClipboardService::new(FakeBackend::default());
        assert_eq!(service.snapshot().expect("snapshot"), None);
    }

    #[test]
    fn suppresses_last_written_content() {
        let backend = FakeBackend {
            current: "hello".to_string(),
            image: None,
        };
        let service = ClipboardService::new(backend);
        service.write_text("hello".to_string()).expect("write");
        assert_eq!(service.snapshot().expect("snapshot"), None);
    }

    #[test]
    fn suppresses_repeated_unmodified_snapshots() {
        let backend = FakeBackend {
            current: "hello".to_string(),
            image: None,
        };
        let service = ClipboardService::new(backend);

        let first = service.snapshot().expect("snapshot one");
        let second = service.snapshot().expect("snapshot two");

        assert!(first.is_some());
        assert_eq!(second, None);
    }

    #[test]
    fn builds_domain_entry_from_snapshot() {
        let snapshot = ClipboardSnapshot::Text {
                content: "hello".to_string(),
                content_hash: hash_text("hello"),
        };

        let entry = ClipboardService::<FakeBackend>::to_entry(snapshot, Some("terminal".to_string()))
            .expect("entry");
        assert_eq!(entry.content, "hello");
        assert_eq!(entry.source_app.as_deref(), Some("terminal"));
    }

    struct SharedBackend {
        current: Arc<Mutex<String>>,
    }

    impl ClipboardBackend for SharedBackend {
        fn get_text(&mut self) -> Result<String, arboard::Error> {
            Ok(self.current.lock().clone())
        }

        fn set_text(&mut self, content: String) -> Result<(), arboard::Error> {
            *self.current.lock() = content;
            Ok(())
        }

        fn get_image(&mut self) -> Result<arboard::ImageData<'static>, arboard::Error> {
            Err(arboard::Error::ContentNotAvailable)
        }

        fn set_image(&mut self, _image: arboard::ImageData<'_>) -> Result<(), arboard::Error> {
            Ok(())
        }
    }

    #[test]
    fn captures_rapid_distinct_changes_once_each() {
        let current = Arc::new(Mutex::new("first".to_string()));
        let service = ClipboardService::new(SharedBackend {
            current: Arc::clone(&current),
        });

        let first = service.snapshot().expect("first snapshot");
        *current.lock() = "second".to_string();
        let second = service.snapshot().expect("second snapshot");
        let third = service.snapshot().expect("third snapshot");

        assert_eq!(
            first.as_ref().and_then(|value| match value {
                ClipboardSnapshot::Text { content, .. } => Some(content.as_str()),
                _ => None,
            }),
            Some("first")
        );
        assert_eq!(
            second.as_ref().and_then(|value| match value {
                ClipboardSnapshot::Text { content, .. } => Some(content.as_str()),
                _ => None,
            }),
            Some("second")
        );
        assert_eq!(third, None);
    }

    #[test]
    fn captures_image_when_text_is_unavailable() {
        let backend = FakeBackend {
            current: String::new(),
            image: Some((1, 1, vec![255, 0, 0, 255])),
        };
        let service = ClipboardService::new(backend);

        let snapshot = service.snapshot().expect("snapshot").expect("image snapshot");
        match snapshot {
            ClipboardSnapshot::Image {
                width,
                height,
                bytes,
                ..
            } => {
                assert_eq!(width, 1);
                assert_eq!(height, 1);
                assert_eq!(bytes, vec![255, 0, 0, 255]);
            }
            _ => panic!("expected image snapshot"),
        }
    }
}
