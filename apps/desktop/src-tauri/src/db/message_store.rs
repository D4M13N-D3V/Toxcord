use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::Connection;
use tracing::info;

use super::schema;

/// Thread-safe wrapper around an SQLCipher-encrypted SQLite database.
/// All database operations go through this struct.
pub struct MessageStore {
    conn: Mutex<Connection>,
}

/// A friend record from the database
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FriendRecord {
    pub friend_number: i64,
    pub public_key: String,
    pub name: String,
    pub status_message: String,
    pub user_status: String,
    pub connection_status: String,
    pub last_seen: Option<String>,
    pub added_at: String,
    pub notes: String,
}

/// A pending friend request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FriendRequestRecord {
    pub public_key: String,
    pub message: String,
    pub received_at: String,
}

/// A guild record
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GuildRecord {
    pub id: String,
    pub name: String,
    pub metadata_group_number: Option<i64>,
    pub icon_hash: Option<String>,
    pub owner_public_key: String,
    pub guild_type: String, // "server" or "dm_group"
    pub created_at: String,
}

/// A channel record
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChannelRecord {
    pub id: String,
    pub guild_id: String,
    pub name: String,
    pub topic: String,
    pub channel_type: String,
    pub category: Option<String>,
    pub position: i64,
    pub group_number: Option<i64>,
    pub created_at: String,
}

/// A channel message record
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChannelMessageRecord {
    pub id: String,
    pub channel_id: String,
    pub sender_public_key: String,
    pub sender_name: String,
    pub content: String,
    pub message_type: String,
    pub timestamp: String,
}

/// A direct message record
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DirectMessageRecord {
    pub id: String,
    pub friend_number: i64,
    pub sender: String,
    pub content: String,
    pub message_type: String,
    pub timestamp: String,
    pub is_outgoing: bool,
    pub delivered: bool,
    pub read: bool,
}

impl MessageStore {
    /// Open or create a database at the given path, encrypted with the given key.
    pub fn open(path: &PathBuf, encryption_key: &str) -> Result<Self, String> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create database directory: {e}"))?;
        }

        let conn = Connection::open(path)
            .map_err(|e| format!("Failed to open database: {e}"))?;

        // Set the encryption key (SQLCipher pragma)
        if !encryption_key.is_empty() {
            conn.pragma_update(None, "key", encryption_key)
                .map_err(|e| format!("Failed to set encryption key: {e}"))?;
        }

        // Performance pragmas
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA cache_size = -8000;",
        )
        .map_err(|e| format!("Failed to set pragmas: {e}"))?;

        // Run migrations
        schema::initialize(&conn)
            .map_err(|e| format!("Failed to initialize schema: {e}"))?;

        info!("Database opened at {}", path.display());

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    // ─── Profile ───────────────────────────────────────────────────────

    pub fn upsert_profile(&self, tox_id: &str, name: &str, status_message: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO profile (id, tox_id, name, status_message) VALUES (1, ?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET tox_id = ?1, name = ?2, status_message = ?3",
            rusqlite::params![tox_id, name, status_message],
        )
        .map_err(|e| format!("Failed to upsert profile: {e}"))?;
        Ok(())
    }

    // ─── Friends ───────────────────────────────────────────────────────

    pub fn upsert_friend(
        &self,
        friend_number: u32,
        public_key: &str,
        name: &str,
        status_message: &str,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO friends (friend_number, public_key, name, status_message)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(friend_number) DO UPDATE SET
                public_key = ?2, name = ?3, status_message = ?4",
            rusqlite::params![friend_number, public_key, name, status_message],
        )
        .map_err(|e| format!("Failed to upsert friend: {e}"))?;
        Ok(())
    }

    pub fn update_friend_name(&self, friend_number: u32, name: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE friends SET name = ?1 WHERE friend_number = ?2",
            rusqlite::params![name, friend_number],
        )
        .map_err(|e| format!("Failed to update friend name: {e}"))?;
        Ok(())
    }

    pub fn update_friend_status_message(
        &self,
        friend_number: u32,
        message: &str,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE friends SET status_message = ?1 WHERE friend_number = ?2",
            rusqlite::params![message, friend_number],
        )
        .map_err(|e| format!("Failed to update friend status message: {e}"))?;
        Ok(())
    }

    pub fn update_friend_status(&self, friend_number: u32, status: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE friends SET user_status = ?1 WHERE friend_number = ?2",
            rusqlite::params![status, friend_number],
        )
        .map_err(|e| format!("Failed to update friend status: {e}"))?;
        Ok(())
    }

    pub fn update_friend_connection_status(
        &self,
        friend_number: u32,
        status: &str,
        update_last_seen: bool,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        if update_last_seen {
            conn.execute(
                "UPDATE friends SET connection_status = ?1, last_seen = datetime('now')
                 WHERE friend_number = ?2",
                rusqlite::params![status, friend_number],
            )
        } else {
            conn.execute(
                "UPDATE friends SET connection_status = ?1 WHERE friend_number = ?2",
                rusqlite::params![status, friend_number],
            )
        }
        .map_err(|e| format!("Failed to update friend connection: {e}"))?;
        Ok(())
    }

    pub fn remove_friend(&self, friend_number: u32) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM friends WHERE friend_number = ?1",
            rusqlite::params![friend_number],
        )
        .map_err(|e| format!("Failed to remove friend: {e}"))?;
        Ok(())
    }

    pub fn get_friends(&self) -> Result<Vec<FriendRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT friend_number, public_key, name, status_message,
                        user_status, connection_status, last_seen, added_at, notes
                 FROM friends ORDER BY name COLLATE NOCASE",
            )
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let friends = stmt
            .query_map([], |row| {
                Ok(FriendRecord {
                    friend_number: row.get(0)?,
                    public_key: row.get(1)?,
                    name: row.get(2)?,
                    status_message: row.get(3)?,
                    user_status: row.get(4)?,
                    connection_status: row.get(5)?,
                    last_seen: row.get(6)?,
                    added_at: row.get(7)?,
                    notes: row.get(8)?,
                })
            })
            .map_err(|e| format!("Failed to query friends: {e}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect friends: {e}"))?;

        Ok(friends)
    }

    // ─── Friend Requests ───────────────────────────────────────────────

    pub fn add_friend_request(&self, public_key: &str, message: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO friend_requests (public_key, message) VALUES (?1, ?2)",
            rusqlite::params![public_key, message],
        )
        .map_err(|e| format!("Failed to add friend request: {e}"))?;
        Ok(())
    }

    pub fn remove_friend_request(&self, public_key: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM friend_requests WHERE public_key = ?1",
            rusqlite::params![public_key],
        )
        .map_err(|e| format!("Failed to remove friend request: {e}"))?;
        Ok(())
    }

    pub fn get_friend_requests(&self) -> Result<Vec<FriendRequestRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT public_key, message, received_at FROM friend_requests ORDER BY received_at DESC")
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let requests = stmt
            .query_map([], |row| {
                Ok(FriendRequestRecord {
                    public_key: row.get(0)?,
                    message: row.get(1)?,
                    received_at: row.get(2)?,
                })
            })
            .map_err(|e| format!("Failed to query friend requests: {e}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect friend requests: {e}"))?;

        Ok(requests)
    }

    // ─── Direct Messages ───────────────────────────────────────────────

    pub fn insert_direct_message(&self, msg: &DirectMessageRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO direct_messages (id, friend_number, sender, content, message_type, timestamp, is_outgoing, delivered, read)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                msg.id,
                msg.friend_number,
                msg.sender,
                msg.content,
                msg.message_type,
                msg.timestamp,
                msg.is_outgoing,
                msg.delivered,
                msg.read,
            ],
        )
        .map_err(|e| format!("Failed to insert message: {e}"))?;
        Ok(())
    }

    pub fn get_direct_messages(
        &self,
        friend_number: u32,
        limit: i64,
        before_timestamp: Option<&str>,
    ) -> Result<Vec<DirectMessageRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = if let Some(before) = before_timestamp {
            (
                "SELECT id, friend_number, sender, content, message_type, timestamp, is_outgoing, delivered, read
                 FROM direct_messages
                 WHERE friend_number = ?1 AND timestamp < ?2
                 ORDER BY timestamp DESC LIMIT ?3",
                vec![
                    Box::new(friend_number as i64),
                    Box::new(before.to_string()),
                    Box::new(limit),
                ],
            )
        } else {
            (
                "SELECT id, friend_number, sender, content, message_type, timestamp, is_outgoing, delivered, read
                 FROM direct_messages
                 WHERE friend_number = ?1
                 ORDER BY timestamp DESC LIMIT ?2",
                vec![
                    Box::new(friend_number as i64),
                    Box::new(limit),
                ],
            )
        };

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let params_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let messages = stmt
            .query_map(params_refs.as_slice(), |row| {
                Ok(DirectMessageRecord {
                    id: row.get(0)?,
                    friend_number: row.get(1)?,
                    sender: row.get(2)?,
                    content: row.get(3)?,
                    message_type: row.get(4)?,
                    timestamp: row.get(5)?,
                    is_outgoing: row.get(6)?,
                    delivered: row.get(7)?,
                    read: row.get(8)?,
                })
            })
            .map_err(|e| format!("Failed to query messages: {e}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect messages: {e}"))?;

        Ok(messages)
    }

    pub fn mark_message_delivered(&self, message_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE direct_messages SET delivered = 1 WHERE id = ?1",
            rusqlite::params![message_id],
        )
        .map_err(|e| format!("Failed to mark delivered: {e}"))?;
        Ok(())
    }

    pub fn mark_messages_read(&self, friend_number: u32) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE direct_messages SET read = 1
             WHERE friend_number = ?1 AND is_outgoing = 0 AND read = 0",
            rusqlite::params![friend_number],
        )
        .map_err(|e| format!("Failed to mark messages read: {e}"))?;
        Ok(())
    }

    pub fn get_unread_counts(&self) -> Result<Vec<(i64, i64)>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT friend_number, COUNT(*) FROM direct_messages
                 WHERE is_outgoing = 0 AND read = 0
                 GROUP BY friend_number",
            )
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let counts = stmt
            .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))
            .map_err(|e| format!("Failed to query unread counts: {e}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect unread counts: {e}"))?;

        Ok(counts)
    }

    // ─── Search ────────────────────────────────────────────────────────

    pub fn search_messages(&self, query: &str, limit: i64) -> Result<Vec<(String, String)>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT message_id, source_table FROM messages_fts
                 WHERE content MATCH ?1 ORDER BY rank LIMIT ?2",
            )
            .map_err(|e| format!("Failed to prepare search: {e}"))?;

        let results = stmt
            .query_map(rusqlite::params![query, limit], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| format!("Failed to search: {e}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect results: {e}"))?;

        Ok(results)
    }

    // ─── Offline Queue ─────────────────────────────────────────────────

    pub fn queue_offline_message(
        &self,
        target_type: &str,
        target_id: &str,
        message_type: &str,
        content: &str,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO offline_queue (target_type, target_id, message_type, content)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![target_type, target_id, message_type, content],
        )
        .map_err(|e| format!("Failed to queue offline message: {e}"))?;
        Ok(())
    }

    pub fn get_offline_messages_for(
        &self,
        target_type: &str,
        target_id: &str,
    ) -> Result<Vec<(i64, String, String)>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, message_type, content FROM offline_queue
                 WHERE target_type = ?1 AND target_id = ?2 ORDER BY created_at",
            )
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let messages = stmt
            .query_map(rusqlite::params![target_type, target_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| format!("Failed to query offline queue: {e}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect offline messages: {e}"))?;

        Ok(messages)
    }

    pub fn remove_offline_message(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM offline_queue WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| format!("Failed to remove offline message: {e}"))?;
        Ok(())
    }

    // ─── Guilds ───────────────────────────────────────────────────────

    pub fn insert_guild(
        &self,
        id: &str,
        name: &str,
        group_number: Option<i64>,
        owner_pk: &str,
        guild_type: &str,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO guilds (id, name, metadata_group_number, owner_public_key, guild_type)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, name, group_number, owner_pk, guild_type],
        )
        .map_err(|e| format!("Failed to insert guild: {e}"))?;
        Ok(())
    }

    pub fn get_guilds(&self) -> Result<Vec<GuildRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, metadata_group_number, icon_hash, owner_public_key, guild_type, created_at
                 FROM guilds ORDER BY created_at",
            )
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let guilds = stmt
            .query_map([], |row| {
                Ok(GuildRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    metadata_group_number: row.get(2)?,
                    icon_hash: row.get(3)?,
                    owner_public_key: row.get(4)?,
                    guild_type: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|e| format!("Failed to query guilds: {e}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect guilds: {e}"))?;

        Ok(guilds)
    }

    pub fn get_guild(&self, id: &str) -> Result<Option<GuildRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, metadata_group_number, icon_hash, owner_public_key, guild_type, created_at
                 FROM guilds WHERE id = ?1",
            )
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let mut rows = stmt
            .query_map(rusqlite::params![id], |row| {
                Ok(GuildRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    metadata_group_number: row.get(2)?,
                    icon_hash: row.get(3)?,
                    owner_public_key: row.get(4)?,
                    guild_type: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|e| format!("Failed to query guild: {e}"))?;

        match rows.next() {
            Some(Ok(record)) => Ok(Some(record)),
            Some(Err(e)) => Err(format!("Failed to read guild: {e}")),
            None => Ok(None),
        }
    }

    pub fn get_guild_by_group_number(&self, group_number: i64) -> Result<Option<GuildRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, metadata_group_number, icon_hash, owner_public_key, guild_type, created_at
                 FROM guilds WHERE metadata_group_number = ?1",
            )
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let mut rows = stmt
            .query_map(rusqlite::params![group_number], |row| {
                Ok(GuildRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    metadata_group_number: row.get(2)?,
                    icon_hash: row.get(3)?,
                    owner_public_key: row.get(4)?,
                    guild_type: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|e| format!("Failed to query guild: {e}"))?;

        match rows.next() {
            Some(Ok(record)) => Ok(Some(record)),
            Some(Err(e)) => Err(format!("Failed to read guild: {e}")),
            None => Ok(None),
        }
    }

    pub fn get_guild_by_group_number_and_type(&self, group_number: i64, guild_type: &str) -> Result<Option<GuildRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, metadata_group_number, icon_hash, owner_public_key, guild_type, created_at
                 FROM guilds WHERE metadata_group_number = ?1 AND guild_type = ?2",
            )
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let mut rows = stmt
            .query_map(rusqlite::params![group_number, guild_type], |row| {
                Ok(GuildRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    metadata_group_number: row.get(2)?,
                    icon_hash: row.get(3)?,
                    owner_public_key: row.get(4)?,
                    guild_type: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|e| format!("Failed to query guild: {e}"))?;

        match rows.next() {
            Some(Ok(record)) => Ok(Some(record)),
            Some(Err(e)) => Err(format!("Failed to read guild: {e}")),
            None => Ok(None),
        }
    }

    pub fn update_guild_name(&self, id: &str, name: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE guilds SET name = ?1 WHERE id = ?2",
            rusqlite::params![name, id],
        )
        .map_err(|e| format!("Failed to update guild name: {e}"))?;
        Ok(())
    }

    pub fn update_guild_group_number(&self, id: &str, group_number: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE guilds SET metadata_group_number = ?1 WHERE id = ?2",
            rusqlite::params![group_number, id],
        )
        .map_err(|e| format!("Failed to update guild group_number: {e}"))?;
        Ok(())
    }

    pub fn get_guild_by_name(&self, name: &str) -> Result<Option<GuildRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, metadata_group_number, icon_hash, owner_public_key, guild_type, created_at
                 FROM guilds WHERE name = ?1",
            )
            .map_err(|e| format!("Failed to prepare statement: {e}"))?;

        let mut rows = stmt
            .query_map(rusqlite::params![name], |row| {
                Ok(GuildRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    metadata_group_number: row.get(2)?,
                    icon_hash: row.get(3)?,
                    owner_public_key: row.get(4)?,
                    guild_type: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|e| format!("Failed to query guilds: {e}"))?;

        match rows.next() {
            Some(Ok(record)) => Ok(Some(record)),
            Some(Err(e)) => Err(format!("Failed to read guild: {e}")),
            None => Ok(None),
        }
    }

    pub fn delete_guild(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM guilds WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| format!("Failed to delete guild: {e}"))?;
        Ok(())
    }

    // ─── Channels ─────────────────────────────────────────────────────

    pub fn insert_channel(
        &self,
        id: &str,
        guild_id: &str,
        name: &str,
        channel_type: &str,
        position: i64,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO channels (id, guild_id, name, channel_type, position)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, guild_id, name, channel_type, position],
        )
        .map_err(|e| format!("Failed to insert channel: {e}"))?;
        Ok(())
    }

    pub fn get_channels(&self, guild_id: &str) -> Result<Vec<ChannelRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, guild_id, name, topic, channel_type, category, position, group_number, created_at
                 FROM channels WHERE guild_id = ?1 ORDER BY position",
            )
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let channels = stmt
            .query_map(rusqlite::params![guild_id], |row| {
                Ok(ChannelRecord {
                    id: row.get(0)?,
                    guild_id: row.get(1)?,
                    name: row.get(2)?,
                    topic: row.get(3)?,
                    channel_type: row.get(4)?,
                    category: row.get(5)?,
                    position: row.get(6)?,
                    group_number: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })
            .map_err(|e| format!("Failed to query channels: {e}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect channels: {e}"))?;

        Ok(channels)
    }

    pub fn update_channel(&self, id: &str, name: &str, topic: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE channels SET name = ?1, topic = ?2 WHERE id = ?3",
            rusqlite::params![name, topic, id],
        )
        .map_err(|e| format!("Failed to update channel: {e}"))?;
        Ok(())
    }

    pub fn rename_channel(&self, id: &str, name: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE channels SET name = ?1 WHERE id = ?2",
            rusqlite::params![name, id],
        )
        .map_err(|e| format!("Failed to rename channel: {e}"))?;
        Ok(())
    }

    pub fn delete_channel(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM channels WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| format!("Failed to delete channel: {e}"))?;
        Ok(())
    }

    pub fn get_channel_count(&self, guild_id: &str) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM channels WHERE guild_id = ?1",
                rusqlite::params![guild_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to count channels: {e}"))?;
        Ok(count)
    }

    /// Get or create a channel by name within a guild.
    /// Returns the channel_id.
    pub fn get_or_create_channel_by_name(&self, guild_id: &str, channel_name: &str) -> Result<String, String> {
        // First, try to find existing channel
        let channels = self.get_channels(guild_id)?;
        if let Some(channel) = channels.iter().find(|c| c.name == channel_name) {
            return Ok(channel.id.clone());
        }

        // Channel doesn't exist, create it
        let channel_id = uuid::Uuid::new_v4().to_string();
        let position = channels.len() as i64;
        self.insert_channel(&channel_id, guild_id, channel_name, "text", position)?;

        Ok(channel_id)
    }

    // ─── Channel Messages ─────────────────────────────────────────────

    pub fn insert_channel_message(&self, msg: &ChannelMessageRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO channel_messages (id, channel_id, sender_public_key, sender_name, content, message_type, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                msg.id,
                msg.channel_id,
                msg.sender_public_key,
                msg.sender_name,
                msg.content,
                msg.message_type,
                msg.timestamp,
            ],
        )
        .map_err(|e| format!("Failed to insert channel message: {e}"))?;
        Ok(())
    }

    pub fn get_channel_messages(
        &self,
        channel_id: &str,
        limit: i64,
        before_timestamp: Option<&str>,
    ) -> Result<Vec<ChannelMessageRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = if let Some(before) = before_timestamp {
            (
                "SELECT id, channel_id, sender_public_key, sender_name, content, message_type, timestamp
                 FROM channel_messages
                 WHERE channel_id = ?1 AND timestamp < ?2
                 ORDER BY timestamp DESC LIMIT ?3",
                vec![
                    Box::new(channel_id.to_string()),
                    Box::new(before.to_string()),
                    Box::new(limit),
                ],
            )
        } else {
            (
                "SELECT id, channel_id, sender_public_key, sender_name, content, message_type, timestamp
                 FROM channel_messages
                 WHERE channel_id = ?1
                 ORDER BY timestamp DESC LIMIT ?2",
                vec![
                    Box::new(channel_id.to_string()),
                    Box::new(limit),
                ],
            )
        };

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let params_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let messages = stmt
            .query_map(params_refs.as_slice(), |row| {
                Ok(ChannelMessageRecord {
                    id: row.get(0)?,
                    channel_id: row.get(1)?,
                    sender_public_key: row.get(2)?,
                    sender_name: row.get(3)?,
                    content: row.get(4)?,
                    message_type: row.get(5)?,
                    timestamp: row.get(6)?,
                })
            })
            .map_err(|e| format!("Failed to query channel messages: {e}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect channel messages: {e}"))?;

        Ok(messages)
    }
}
