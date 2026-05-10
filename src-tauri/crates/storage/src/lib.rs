use chrono::{Duration, Utc};
use domain::{ClipboardEntry, ContentKind, EntryQuery, NewClipboardEntry, Settings};
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use rusqlite_migration::{M, Migrations};
use std::path::Path;
use thiserror::Error;

fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(
            "
            CREATE TABLE IF NOT EXISTS entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                content_kind TEXT NOT NULL,
                source_app TEXT,
                captured_at TEXT NOT NULL,
                pinned INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_entries_captured_at ON entries(captured_at DESC);
            CREATE INDEX IF NOT EXISTS idx_entries_hash ON entries(content_hash);
            CREATE TABLE IF NOT EXISTS settings (
                singleton INTEGER PRIMARY KEY CHECK(singleton = 1),
                hotkey_binding TEXT NOT NULL,
                autostart_enabled INTEGER NOT NULL,
                retention_days INTEGER NOT NULL,
                ignore_sensitive_apps INTEGER NOT NULL
            );
            INSERT OR IGNORE INTO settings (
                singleton,
                hotkey_binding,
                autostart_enabled,
                retention_days,
                ignore_sensitive_apps
            ) VALUES (1, 'Super+V', 0, 2, 0);
            ",
        ),
        M::up(
            "
            ALTER TABLE entries ADD COLUMN image_path TEXT;
            CREATE INDEX IF NOT EXISTS idx_entries_image_path ON entries(image_path);
            ",
        ),
    ])
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error(transparent)]
    Sql(#[from] rusqlite::Error),
    #[error(transparent)]
    Migration(#[from] rusqlite_migration::Error),
    #[error("entry {0} was not found")]
    MissingEntry(i64),
}

pub struct SqliteStore {
    connection: Mutex<Connection>,
}

impl SqliteStore {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let mut connection = Connection::open(path)?;
        configure_connection(&connection)?;
        migrations().to_latest(&mut connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    pub fn open_in_memory() -> Result<Self, StorageError> {
        let mut connection = Connection::open_in_memory()?;
        configure_connection(&connection)?;
        migrations().to_latest(&mut connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    pub fn upsert_entry(&self, entry: &NewClipboardEntry) -> Result<ClipboardEntry, StorageError> {
        let mut connection = self.connection.lock();
        let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;

        let existing = tx
            .query_row(
                "
                SELECT id, content, content_hash, content_kind, image_path, source_app, captured_at, pinned
                FROM entries
                WHERE content_hash = ?1
                AND content_kind = ?2
                AND (
                    (content_kind = 'text' AND content = ?3)
                    OR (content_kind = 'image' AND image_path = ?4)
                )
                ORDER BY captured_at DESC
                LIMIT 1
                ",
                params![
                    entry.content_hash,
                    entry.content_kind.to_string(),
                    entry.content,
                    entry.image_path,
                ],
                map_entry,
            )
            .optional()?;

        let result = if let Some(existing) = existing {
            tx.execute(
                "UPDATE entries SET captured_at = ?2 WHERE id = ?1",
                params![existing.id, entry.captured_at.to_rfc3339()],
            )?;
            ClipboardEntry {
                captured_at: entry.captured_at,
                ..existing
            }
        } else {
            tx.execute(
                "
                INSERT INTO entries (content, content_hash, content_kind, image_path, source_app, captured_at, pinned)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0)
                ",
                params![
                    entry.content,
                    entry.content_hash,
                    entry.content_kind.to_string(),
                    entry.image_path,
                    entry.source_app,
                    entry.captured_at.to_rfc3339(),
                ],
            )?;

            ClipboardEntry::from_new(tx.last_insert_rowid(), entry.clone(), false)
        };

        tx.commit()?;
        Ok(result)
    }

    pub fn list_entries(&self, query: &EntryQuery) -> Result<Vec<ClipboardEntry>, StorageError> {
        let connection = self.connection.lock();
        let like_query = query
            .query
            .as_ref()
            .map(|value| format!("%{}%", value.trim()))
            .unwrap_or_else(|| "%".to_string());

        let mut statement = connection.prepare(
            "
            SELECT id, content, content_hash, content_kind, image_path, source_app, captured_at, pinned
            FROM entries
            WHERE content_kind = 'image' OR content LIKE ?1
            ORDER BY pinned DESC, captured_at DESC
            LIMIT ?2
            ",
        )?;

        statement
            .query_map(params![like_query, query.limit as i64], map_entry)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(StorageError::from)
    }

    pub fn get_entry(&self, entry_id: i64) -> Result<ClipboardEntry, StorageError> {
        let connection = self.connection.lock();
        connection
            .query_row(
                "
                SELECT id, content, content_hash, content_kind, image_path, source_app, captured_at, pinned
                FROM entries
                WHERE id = ?1
                ",
                params![entry_id],
                map_entry,
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => StorageError::MissingEntry(entry_id),
                other => StorageError::Sql(other),
            })
    }

    pub fn toggle_pin(&self, entry_id: i64) -> Result<(), StorageError> {
        let connection = self.connection.lock();
        connection.execute(
            "UPDATE entries SET pinned = CASE WHEN pinned = 1 THEN 0 ELSE 1 END WHERE id = ?1",
            params![entry_id],
        )?;
        Ok(())
    }

    pub fn prune_expired_entries(&self, retention_days: u16) -> Result<usize, StorageError> {
        let cutoff = Utc::now() - Duration::days(retention_days as i64);
        let connection = self.connection.lock();
        connection
            .execute(
                "DELETE FROM entries WHERE pinned = 0 AND captured_at < ?1",
                params![cutoff.to_rfc3339()],
            )
            .map_err(StorageError::from)
    }

    pub fn prune_expired_entries_with_paths(
        &self,
        retention_days: u16,
    ) -> Result<(usize, Vec<String>), StorageError> {
        let cutoff = Utc::now() - Duration::days(retention_days as i64);
        let connection = self.connection.lock();
        let mut statement = connection.prepare(
            "
            SELECT image_path
            FROM entries
            WHERE pinned = 0
              AND captured_at < ?1
              AND image_path IS NOT NULL
            ",
        )?;
        let image_paths = statement
            .query_map(params![cutoff.to_rfc3339()], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;

        let deleted = connection.execute(
            "DELETE FROM entries WHERE pinned = 0 AND captured_at < ?1",
            params![cutoff.to_rfc3339()],
        )?;
        Ok((deleted, image_paths))
    }

    pub fn load_settings(&self) -> Result<Settings, StorageError> {
        let connection = self.connection.lock();
        Ok(connection.query_row(
            "
            SELECT hotkey_binding, autostart_enabled, retention_days, ignore_sensitive_apps
            FROM settings WHERE singleton = 1
            ",
            [],
            |row| {
                Ok(Settings {
                    hotkey_binding: row.get(0)?,
                    autostart_enabled: row.get::<_, i64>(1)? != 0,
                    retention_days: row.get::<_, u16>(2)?,
                    ignore_sensitive_apps: row.get::<_, i64>(3)? != 0,
                })
            },
        )?)
    }

    pub fn save_settings(&self, settings: &Settings) -> Result<Settings, StorageError> {
        let connection = self.connection.lock();
        connection.execute(
            "
            UPDATE settings
            SET hotkey_binding = ?1,
                autostart_enabled = ?2,
                retention_days = ?3,
                ignore_sensitive_apps = ?4
            WHERE singleton = 1
            ",
            params![
                settings.hotkey_binding,
                settings.autostart_enabled as i64,
                settings.retention_days as i64,
                settings.ignore_sensitive_apps as i64,
            ],
        )?;
        Ok(settings.clone())
    }
}

fn configure_connection(connection: &Connection) -> Result<(), StorageError> {
    connection.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA busy_timeout = 5000;
        PRAGMA foreign_keys = ON;
        ",
    )?;
    Ok(())
}

fn map_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<ClipboardEntry> {
    let content_kind = match row.get::<_, String>(3)?.as_str() {
        "text" => ContentKind::Text,
        "image" => ContentKind::Image,
        _ => ContentKind::Text,
    };

    Ok(ClipboardEntry {
        id: row.get(0)?,
        content: row.get(1)?,
        content_hash: row.get(2)?,
        content_kind,
        image_path: row.get(4)?,
        source_app: row.get(5)?,
        captured_at: row.get(6)?,
        pinned: row.get::<_, i64>(7)? != 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use domain::ContentKind;
    use std::{fs, time::{SystemTime, UNIX_EPOCH}};

    fn sample_entry(content: &str) -> NewClipboardEntry {
        NewClipboardEntry::new_text(
            content.to_string(),
            format!("hash-{content}"),
            None,
            Utc::now(),
        )
        .expect("valid test entry")
    }

    #[test]
    fn stores_and_lists_entries() {
        let store = SqliteStore::open_in_memory().expect("store");
        store.upsert_entry(&sample_entry("hello")).expect("insert");

        let entries = store.list_entries(&EntryQuery::default()).expect("list");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content, "hello");
    }

    #[test]
    fn deduplicates_same_content() {
        let store = SqliteStore::open_in_memory().expect("store");
        let first = store.upsert_entry(&sample_entry("hello")).expect("insert one");
        let second = store.upsert_entry(&sample_entry("hello")).expect("insert two");
        let entries = store.list_entries(&EntryQuery::default()).expect("list");

        assert_eq!(entries.len(), 1);
        assert_eq!(first.id, second.id);
    }

    #[test]
    fn prunes_expired_unpinned_entries() {
        let store = SqliteStore::open_in_memory().expect("store");
        let old_entry = NewClipboardEntry::new(
            "old".to_string(),
            "old-hash".to_string(),
            ContentKind::Text,
            None,
            None,
            Utc::now() - Duration::days(10),
        )
        .expect("valid");
        store.upsert_entry(&old_entry).expect("insert");

        let deleted = store.prune_expired_entries(7).expect("prune");

        assert_eq!(deleted, 1);
        assert!(store.list_entries(&EntryQuery::default()).expect("list").is_empty());
    }

    #[test]
    fn persists_entries_across_reopen() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let database_path = std::env::temp_dir().join(format!("cistory-storage-{unique}.sqlite3"));

        {
            let store = SqliteStore::open(&database_path).expect("store");
            store.upsert_entry(&sample_entry("persisted")).expect("insert");
        }

        let reopened = SqliteStore::open(&database_path).expect("reopen store");
        let entries = reopened.list_entries(&EntryQuery::default()).expect("list");

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content, "persisted");

        let _ = fs::remove_file(database_path);
    }

    #[test]
    fn stores_image_entry_path() {
        let store = SqliteStore::open_in_memory().expect("store");
        let image_entry = NewClipboardEntry::new_image(
            "image-hash".to_string(),
            "/tmp/cistory-images/image-hash.png".to_string(),
            None,
            Utc::now(),
        )
        .expect("valid image entry");

        let inserted = store.upsert_entry(&image_entry).expect("insert image");
        assert_eq!(inserted.content_kind, ContentKind::Image);
        assert_eq!(
            inserted.image_path.as_deref(),
            Some("/tmp/cistory-images/image-hash.png")
        );

        let fetched = store.get_entry(inserted.id).expect("fetch image");
        assert_eq!(fetched.content_kind, ContentKind::Image);
        assert_eq!(
            fetched.image_path.as_deref(),
            Some("/tmp/cistory-images/image-hash.png")
        );
    }

    #[test]
    fn deduplicates_same_image_entry() {
        let store = SqliteStore::open_in_memory().expect("store");
        let first_entry = NewClipboardEntry::new_image(
            "image-hash".to_string(),
            "/tmp/cistory-images/image-hash.png".to_string(),
            None,
            Utc::now(),
        )
        .expect("valid image entry");
        let second_entry = NewClipboardEntry::new_image(
            "image-hash".to_string(),
            "/tmp/cistory-images/image-hash.png".to_string(),
            None,
            Utc::now(),
        )
        .expect("valid image entry");

        let first = store.upsert_entry(&first_entry).expect("insert one");
        let second = store.upsert_entry(&second_entry).expect("insert two");

        assert_eq!(first.id, second.id);
        let entries = store.list_entries(&EntryQuery::default()).expect("list");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content_kind, ContentKind::Image);
    }

    #[test]
    fn lists_mixed_text_and_image_entries() {
        let store = SqliteStore::open_in_memory().expect("store");
        store.upsert_entry(&sample_entry("hello world")).expect("insert text");
        store
            .upsert_entry(
                &NewClipboardEntry::new_image(
                    "image-hash".to_string(),
                    "/tmp/cistory-images/image-hash.png".to_string(),
                    None,
                    Utc::now(),
                )
                .expect("image entry"),
            )
            .expect("insert image");

        let all_entries = store.list_entries(&EntryQuery::default()).expect("list all");
        assert_eq!(all_entries.len(), 2);
        assert!(all_entries.iter().any(|entry| entry.content_kind == ContentKind::Text));
        assert!(all_entries.iter().any(|entry| entry.content_kind == ContentKind::Image));

        let text_entries = store
            .list_entries(&EntryQuery {
                query: Some("hello".to_string()),
                limit: 100,
            })
            .expect("list filtered");
        assert_eq!(text_entries.len(), 2);
    }

    #[test]
    fn returns_stale_image_paths_during_pruning() {
        let store = SqliteStore::open_in_memory().expect("store");
        let old_image = NewClipboardEntry::new_image(
            "old-image-hash".to_string(),
            "/tmp/cistory-images/old-image-hash.png".to_string(),
            None,
            Utc::now() - Duration::days(10),
        )
        .expect("old image");
        store.upsert_entry(&old_image).expect("insert old image");

        let (deleted, paths) = store
            .prune_expired_entries_with_paths(7)
            .expect("prune with paths");

        assert_eq!(deleted, 1);
        assert_eq!(paths, vec!["/tmp/cistory-images/old-image-hash.png".to_string()]);
    }

    #[test]
    fn initializes_settings_with_two_day_retention_default() {
        let store = SqliteStore::open_in_memory().expect("store");

        let settings = store.load_settings().expect("load settings");

        assert_eq!(settings.retention_days, 2);
    }

    #[test]
    fn preserves_custom_retention_override() {
        let store = SqliteStore::open_in_memory().expect("store");
        let mut settings = store.load_settings().expect("load settings");
        settings.retention_days = 14;
        store.save_settings(&settings).expect("save settings");

        let reloaded = store.load_settings().expect("reload settings");

        assert_eq!(reloaded.retention_days, 14);
    }
}
