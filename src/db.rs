//! SQLite database layer for note CRUD operations with full-text search.

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

// Database performance configuration
const DB_CACHE_SIZE_KB: i32 = -64000; // 64MB cache (negative = KB)
const DB_JOURNAL_MODE: &str = "WAL";
const DB_SYNCHRONOUS: &str = "NORMAL";
const DB_TEMP_STORE: &str = "MEMORY";

/// A note with title, content, tags, and timestamps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
	pub id:         Option<i64>,
	pub title:      String,
	pub content:    String,
	pub tags:       Vec<String>,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

impl Note {
	/// Creates a new note with current timestamp (id is None until saved).
	pub fn new(title: String, content: String, tags: Vec<String>) -> Self {
		let now = Utc::now();
		Self { id: None, title, content, tags, created_at: now, updated_at: now }
	}
}

/// SQLite database wrapper for note storage and retrieval.
pub struct Database {
	conn: Connection,
}

impl Database {
	/// Opens or creates a database with WAL mode and FTS5 support.
	pub fn new(path: &str) -> Result<Self> {
		let conn = Connection::open(path)?;

		// Enable WAL mode for concurrent reads
		conn.pragma_update(None, "journal_mode", DB_JOURNAL_MODE)?;
		conn.pragma_update(None, "synchronous", DB_SYNCHRONOUS)?;
		conn.pragma_update(None, "cache_size", DB_CACHE_SIZE_KB)?;
		conn.pragma_update(None, "temp_store", DB_TEMP_STORE)?;

		let db = Self { conn };
		db.init_schema()?;
		Ok(db)
	}

	/// Converts a database row to a Note.
	fn row_to_note(row: &rusqlite::Row) -> rusqlite::Result<Note> {
		let tags_json: String = row.get(3)?;
		let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

		let created_at = DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
			.map_err(|e| rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e)))?
			.with_timezone(&Utc);
		let updated_at = DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
			.map_err(|e| rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(e)))?
			.with_timezone(&Utc);

		Ok(Note { id: Some(row.get(0)?), title: row.get(1)?, content: row.get(2)?, tags, created_at, updated_at })
	}

	/// Initializes database schema with FTS5 triggers (idempotent).
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

		// Indices for sorting
		self.conn.execute_batch(
			"CREATE INDEX IF NOT EXISTS idx_notes_updated_at ON notes(updated_at DESC);
             CREATE INDEX IF NOT EXISTS idx_notes_created_at ON notes(created_at DESC);
             CREATE INDEX IF NOT EXISTS idx_notes_title ON notes(title COLLATE NOCASE);",
		)?;

		// FTS5 virtual table for full-text search
		self.conn.execute(
			"CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
                title, content, tags, content='notes', content_rowid='id'
            )",
			[],
		)?;

		// Triggers to sync FTS table
		self.conn.execute_batch(
			"CREATE TRIGGER IF NOT EXISTS notes_ai AFTER INSERT ON notes BEGIN
                INSERT INTO notes_fts(rowid, title, content, tags)
                VALUES (new.id, new.title, new.content, new.tags);
             END;

             CREATE TRIGGER IF NOT EXISTS notes_ad AFTER DELETE ON notes BEGIN
                DELETE FROM notes_fts WHERE rowid = old.id;
             END;

             CREATE TRIGGER IF NOT EXISTS notes_au AFTER UPDATE ON notes BEGIN
                UPDATE notes_fts SET title=new.title, content=new.content, tags=new.tags
                WHERE rowid=new.id;
             END;",
		)?;

		// Rebuild FTS index if empty (migration case)
		let notes_count: i64 = self.conn.query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))?;
		let fts_count: i64 = match self.conn.query_row("SELECT COUNT(*) FROM notes_fts", [], |row| row.get(0)) {
			Ok(count) => count,
			Err(rusqlite::Error::QueryReturnedNoRows) => 0,
			Err(e) => return Err(e.into()),
		};

		if notes_count > 0 && fts_count == 0 {
			self
				.conn
				.execute("INSERT INTO notes_fts(rowid, title, content, tags) SELECT id, title, content, tags FROM notes", [])?;
		}

		Ok(())
	}

	/// Inserts a note and returns its assigned ID.
	pub fn create_note(&self, note: &Note) -> Result<i64> {
		let tags_json = serde_json::to_string(&note.tags)?;
		self.conn.execute(
			"INSERT INTO notes (title, content, tags, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
			params![&note.title, &note.content, &tags_json, &note.created_at.to_rfc3339(), &note.updated_at.to_rfc3339(),],
		)?;
		Ok(self.conn.last_insert_rowid())
	}

	/// Retrieves a note by ID.
	pub fn get_note(&self, id: i64) -> Result<Option<Note>> {
		let mut stmt =
			self.conn.prepare("SELECT id, title, content, tags, created_at, updated_at FROM notes WHERE id = ?1")?;

		let note = stmt.query_row(params![id], Self::row_to_note);

		match note {
			Ok(n) => Ok(Some(n)),
			Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
			Err(e) => Err(e.into()),
		}
	}

	/// Returns all notes ordered by most recently updated.
	pub fn list_notes(&self) -> Result<Vec<Note>> {
		let mut stmt = self
			.conn
			.prepare("SELECT id, title, content, tags, created_at, updated_at FROM notes ORDER BY updated_at DESC")?;

		let notes = stmt.query_map([], Self::row_to_note)?.collect::<Result<Vec<Note>, rusqlite::Error>>()?;

		Ok(notes)
	}

	/// Updates a note's title, content, and tags.
	pub fn update_note(&self, id: i64, title: &str, content: &str, tags: &[String]) -> Result<()> {
		let tags_json = serde_json::to_string(tags)?;
		let updated_at = Utc::now();

		self.conn.execute(
			"UPDATE notes SET title = ?1, content = ?2, tags = ?3, updated_at = ?4 WHERE id = ?5",
			params![title, content, &tags_json, &updated_at.to_rfc3339(), id],
		)?;
		Ok(())
	}

	/// Deletes a note by ID.
	pub fn delete_note(&self, id: i64) -> Result<()> {
		self.conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
		Ok(())
	}

	/// Searches notes using LIKE pattern matching (case-insensitive substring
	/// search).
	pub fn search_notes(&self, query: &str) -> Result<Vec<Note>> {
		if query.is_empty() {
			return self.list_notes();
		}

		let search_pattern = format!("%{query}%");
		let mut stmt = self.conn.prepare(
			"SELECT id, title, content, tags, created_at, updated_at
             FROM notes
             WHERE title LIKE ?1 OR content LIKE ?1 OR tags LIKE ?1
             ORDER BY updated_at DESC",
		)?;

		let notes =
			stmt.query_map(params![&search_pattern], Self::row_to_note)?.collect::<Result<Vec<Note>, rusqlite::Error>>()?;

		Ok(notes)
	}
}
