use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentKind {
    Text,
    Image,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub id: i64,
    pub content: String,
    pub content_hash: String,
    pub content_kind: ContentKind,
    pub image_path: Option<String>,
    pub captured_at: DateTime<Utc>,
    pub source_app: Option<String>,
    pub pinned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewClipboardEntry {
    pub content: String,
    pub content_hash: String,
    pub content_kind: ContentKind,
    pub image_path: Option<String>,
    pub source_app: Option<String>,
    pub captured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    pub hotkey_binding: String,
    pub autostart_enabled: bool,
    pub retention_days: u16,
    pub ignore_sensitive_apps: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkey_binding: "Super+V".to_string(),
            autostart_enabled: false,
            retention_days: 7,
            ignore_sensitive_apps: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryQuery {
    pub query: Option<String>,
    pub limit: usize,
}

impl Default for EntryQuery {
    fn default() -> Self {
        Self {
            query: None,
            limit: 100,
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("clipboard entry content must not be empty")]
    EmptyContent,
    #[error("clipboard image path must not be empty")]
    EmptyImagePath,
    #[error("clipboard entry content exceeds {max_bytes} bytes")]
    ContentTooLarge { max_bytes: usize },
    #[error("retention days must be between 1 and 365")]
    InvalidRetentionDays,
    #[error("hotkey binding must not be empty")]
    InvalidHotkeyBinding,
}

impl ClipboardEntry {
    pub fn from_new(id: i64, value: NewClipboardEntry, pinned: bool) -> Self {
        Self {
            id,
            content: value.content,
            content_hash: value.content_hash,
            content_kind: value.content_kind,
            image_path: value.image_path,
            captured_at: value.captured_at,
            source_app: value.source_app,
            pinned,
        }
    }
}

impl NewClipboardEntry {
    pub const MAX_TEXT_BYTES: usize = 1024 * 1024;

    pub fn new_text(
        content: String,
        content_hash: String,
        source_app: Option<String>,
        captured_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        Self::new(
            content,
            content_hash,
            ContentKind::Text,
            None,
            source_app,
            captured_at,
        )
    }

    pub fn new_image(
        content_hash: String,
        image_path: String,
        source_app: Option<String>,
        captured_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        Self::new(
            String::new(),
            content_hash,
            ContentKind::Image,
            Some(image_path),
            source_app,
            captured_at,
        )
    }

    pub fn new(
        content: String,
        content_hash: String,
        content_kind: ContentKind,
        image_path: Option<String>,
        source_app: Option<String>,
        captured_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        match content_kind {
            ContentKind::Text => {
                if content.trim().is_empty() {
                    return Err(DomainError::EmptyContent);
                }

                if content.len() > Self::MAX_TEXT_BYTES {
                    return Err(DomainError::ContentTooLarge {
                        max_bytes: Self::MAX_TEXT_BYTES,
                    });
                }
            }
            ContentKind::Image => {
                if image_path
                    .as_deref()
                    .is_none_or(|path| path.trim().is_empty())
                {
                    return Err(DomainError::EmptyImagePath);
                }
            }
        }

        Ok(Self {
            content,
            content_hash,
            content_kind,
            image_path,
            source_app,
            captured_at,
        })
    }
}

impl Settings {
    pub fn validate(self) -> Result<Self, DomainError> {
        if self.hotkey_binding.trim().is_empty() {
            return Err(DomainError::InvalidHotkeyBinding);
        }

        if !(1..=365).contains(&self.retention_days) {
            return Err(DomainError::InvalidRetentionDays);
        }

        Ok(self)
    }
}

impl fmt::Display for ContentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text => write!(f, "text"),
            Self::Image => write!(f, "image"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_content() {
        let result = NewClipboardEntry::new(
            "   ".to_string(),
            "hash".to_string(),
            ContentKind::Text,
            None,
            None,
            Utc::now(),
        );

        assert_eq!(result, Err(DomainError::EmptyContent));
    }

    #[test]
    fn rejects_invalid_retention_days() {
        let settings = Settings {
            retention_days: 0,
            ..Settings::default()
        };

        assert_eq!(settings.validate(), Err(DomainError::InvalidRetentionDays));
    }

    #[test]
    fn accepts_default_settings() {
        assert!(Settings::default().validate().is_ok());
    }

    #[test]
    fn rejects_image_entry_without_path() {
        let result = NewClipboardEntry::new(
            String::new(),
            "img-hash".to_string(),
            ContentKind::Image,
            None,
            None,
            Utc::now(),
        );

        assert_eq!(result, Err(DomainError::EmptyImagePath));
    }

    #[test]
    fn accepts_image_entry_with_path() {
        let result = NewClipboardEntry::new_image(
            "img-hash".to_string(),
            "/tmp/cistory-images/img-hash.png".to_string(),
            None,
            Utc::now(),
        );

        assert!(result.is_ok());
    }
}
