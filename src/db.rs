//! Database layer for qnote using `SQLite`.
//!
//! Handles all CRUD operations for notes with the following schema:
//! - id: Auto-incrementing primary key
//! - title: Note heading
//! - content: Note body text
//! - tags: JSON array of tag strings
//! - `created_at`: RFC3339 timestamp of creation
//! - `updated_at`: RFC3339 timestamp of last modification

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

// Database performance configuration
const DB_CACHE_SIZE_KB: i32 = -64000; // 64MB cache (negative = KB)
const DB_JOURNAL_MODE: &str = "WAL";
const DB_SYNCHRONOUS: &str = "NORMAL";
const DB_TEMP_STORE: &str = "MEMORY";

/// Represents a single note with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: Option<i64>,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Note {
    /// Creates a new note with the current timestamp for both `created_at` and `updated_at`.
    /// The id field is None until the note is saved to the database.
    pub fn new(title: String, content: String, tags: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            title,
            content,
            tags,
            created_at: now,
            updated_at: now,
        }
    }
}

/// `SQLite` database wrapper for note storage and retrieval.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Opens or creates a `SQLite` database at the given path.
    /// Initializes the schema if the notes table doesn't exist.
    /// Enables WAL mode for better performance and concurrency.
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;

        // Enable WAL mode for better performance (allows concurrent reads during writes)
        conn.pragma_update(None, "journal_mode", DB_JOURNAL_MODE)?;
        conn.pragma_update(None, "synchronous", DB_SYNCHRONOUS)?;
        conn.pragma_update(None, "cache_size", DB_CACHE_SIZE_KB)?;
        conn.pragma_update(None, "temp_store", DB_TEMP_STORE)?;

        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Helper function to convert a database row into a Note struct.
    /// Handles JSON deserialization of tags and RFC3339 timestamp parsing.
    ///
    /// Expected row columns (in order):
    /// 0. id (i64)
    /// 1. title (String)
    /// 2. content (String)
    /// 3. tags (JSON String)
    /// 4. `created_at` (RFC3339 String)
    /// 5. `updated_at` (RFC3339 String)
    fn row_to_note(row: &rusqlite::Row) -> rusqlite::Result<Note> {
        let tags_json: String = row.get(3)?;
        let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

        let created_at = DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    4,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?
            .with_timezone(&Utc);
        let updated_at = DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    5,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?
            .with_timezone(&Utc);

        Ok(Note {
            id: Some(row.get(0)?),
            title: row.get(1)?,
            content: row.get(2)?,
            tags,
            created_at,
            updated_at,
        })
    }

    /// Creates the notes table if it doesn't exist.
    /// Safe to call multiple times - uses CREATE TABLE IF NOT EXISTS.
    fn init_schema(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                tags TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Add indices for frequently queried columns to speed up sorting
        self.conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_notes_updated_at ON notes(updated_at DESC);
             CREATE INDEX IF NOT EXISTS idx_notes_created_at ON notes(created_at DESC);
             CREATE INDEX IF NOT EXISTS idx_notes_title ON notes(title COLLATE NOCASE);",
        )?;

        Ok(())
    }

    /// Inserts a new note into the database and returns its assigned ID.
    /// Tags are serialized to JSON before storage.
    pub fn create_note(&self, note: &Note) -> Result<i64> {
        let tags_json = serde_json::to_string(&note.tags)?;
        self.conn.execute(
            "INSERT INTO notes (title, content, tags, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                &note.title,
                &note.content,
                &tags_json,
                &note.created_at.to_rfc3339(),
                &note.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Retrieves a single note by its ID.
    /// Returns None if the note doesn't exist, otherwise Some(Note).
    pub fn get_note(&self, id: i64) -> Result<Option<Note>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, tags, created_at, updated_at FROM notes WHERE id = ?1",
        )?;

        let note = stmt.query_row(params![id], Self::row_to_note);

        match note {
            Ok(n) => Ok(Some(n)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Returns all notes ordered by `updated_at` descending (most recently updated first).
    /// Tags are deserialized from JSON storage format.
    pub fn list_notes(&self) -> Result<Vec<Note>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, tags, created_at, updated_at FROM notes ORDER BY updated_at DESC",
        )?;

        let notes = stmt
            .query_map([], Self::row_to_note)?
            .collect::<Result<Vec<Note>, rusqlite::Error>>()?;

        Ok(notes)
    }

    /// Updates an existing note's title, content, and tags.
    /// Automatically updates the `updated_at` timestamp to the current time.
    /// Tags are serialized to JSON before storage.
    pub fn update_note(
        &self,
        id: i64,
        title: impl Into<String>,
        content: impl Into<String>,
        tags: &[String],
    ) -> Result<()> {
        let title = title.into();
        let content = content.into();
        let tags_json = serde_json::to_string(tags)?;
        let updated_at = Utc::now();

        self.conn.execute(
            "UPDATE notes SET title = ?1, content = ?2, tags = ?3, updated_at = ?4 WHERE id = ?5",
            params![&title, &content, &tags_json, &updated_at.to_rfc3339(), id],
        )?;
        Ok(())
    }

    /// Permanently deletes a note from the database by its ID.
    /// No error is raised if the note doesn't exist.
    pub fn delete_note(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM notes WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Searches for notes matching the query string using SQL LIKE.
    /// Searches across title, content, and tags fields (case-insensitive).
    /// Returns results ordered by `updated_at` descending.
    pub fn search_notes(&self, query: &str) -> Result<Vec<Note>> {
        let search_pattern = format!("%{query}%");
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, tags, created_at, updated_at
             FROM notes
             WHERE title LIKE ?1 OR content LIKE ?1 OR tags LIKE ?1
             ORDER BY updated_at DESC",
        )?;

        let notes = stmt
            .query_map(params![&search_pattern], Self::row_to_note)?
            .collect::<Result<Vec<Note>, rusqlite::Error>>()?;

        Ok(notes)
    }
}
