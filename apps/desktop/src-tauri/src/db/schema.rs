use rusqlite::Connection;
use tracing::info;

const _CURRENT_SCHEMA_VERSION: i32 = 3;

/// Initialize the database schema, running migrations as needed.
pub fn initialize(conn: &Connection) -> rusqlite::Result<()> {
    let version = get_schema_version(conn);
    info!("Database schema version: {version}");

    if version < 1 {
        migrate_v1(conn)?;
    }
    if version < 2 {
        migrate_v2(conn)?;
    }
    if version < 3 {
        migrate_v3(conn)?;
    }

    Ok(())
}

fn get_schema_version(conn: &Connection) -> i32 {
    conn.pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap_or(0)
}

fn set_schema_version(conn: &Connection, version: i32) -> rusqlite::Result<()> {
    conn.pragma_update(None, "user_version", version)
}

/// Version 1: Initial schema
fn migrate_v1(conn: &Connection) -> rusqlite::Result<()> {
    info!("Running migration v1: initial schema");

    conn.execute_batch(
        "
        -- Local profile metadata
        CREATE TABLE IF NOT EXISTS profile (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            tox_id TEXT NOT NULL,
            name TEXT NOT NULL DEFAULT '',
            status_message TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Friends list (local cache, mirrors Tox friend list)
        CREATE TABLE IF NOT EXISTS friends (
            friend_number INTEGER PRIMARY KEY,
            public_key TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL DEFAULT '',
            status_message TEXT NOT NULL DEFAULT '',
            user_status TEXT NOT NULL DEFAULT 'none',
            connection_status TEXT NOT NULL DEFAULT 'none',
            last_seen TEXT,
            added_at TEXT NOT NULL DEFAULT (datetime('now')),
            notes TEXT NOT NULL DEFAULT ''
        );

        -- Pending friend requests (inbound)
        CREATE TABLE IF NOT EXISTS friend_requests (
            public_key TEXT PRIMARY KEY,
            message TEXT NOT NULL DEFAULT '',
            received_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Direct messages
        CREATE TABLE IF NOT EXISTS direct_messages (
            id TEXT PRIMARY KEY,
            friend_number INTEGER NOT NULL,
            sender TEXT NOT NULL,
            content TEXT NOT NULL,
            message_type TEXT NOT NULL DEFAULT 'normal',
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            is_outgoing INTEGER NOT NULL DEFAULT 0,
            delivered INTEGER NOT NULL DEFAULT 0,
            read INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (friend_number) REFERENCES friends(friend_number) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_dm_friend ON direct_messages(friend_number, timestamp);

        -- Guilds
        CREATE TABLE IF NOT EXISTS guilds (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            metadata_group_number INTEGER,
            metadata_doc BLOB,
            icon_hash TEXT,
            owner_public_key TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            last_synced TEXT
        );

        -- Channels within guilds
        CREATE TABLE IF NOT EXISTS channels (
            id TEXT PRIMARY KEY,
            guild_id TEXT NOT NULL,
            name TEXT NOT NULL,
            topic TEXT NOT NULL DEFAULT '',
            channel_type TEXT NOT NULL DEFAULT 'text',
            category TEXT,
            position INTEGER NOT NULL DEFAULT 0,
            group_number INTEGER,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (guild_id) REFERENCES guilds(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_channel_guild ON channels(guild_id, position);

        -- Channel messages (text channels in guilds)
        CREATE TABLE IF NOT EXISTS channel_messages (
            id TEXT PRIMARY KEY,
            channel_id TEXT NOT NULL,
            sender_public_key TEXT NOT NULL,
            sender_name TEXT NOT NULL DEFAULT '',
            content TEXT NOT NULL,
            message_type TEXT NOT NULL DEFAULT 'normal',
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            edited_at TEXT,
            thread_id TEXT,
            reply_to TEXT,
            FOREIGN KEY (channel_id) REFERENCES channels(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_cmsg_channel ON channel_messages(channel_id, timestamp);

        -- Reactions
        CREATE TABLE IF NOT EXISTS reactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            message_id TEXT NOT NULL,
            message_table TEXT NOT NULL DEFAULT 'channel_messages',
            emoji TEXT NOT NULL,
            reactor_public_key TEXT NOT NULL,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(message_id, emoji, reactor_public_key)
        );
        CREATE INDEX IF NOT EXISTS idx_reaction_msg ON reactions(message_id);

        -- Pinned messages
        CREATE TABLE IF NOT EXISTS pinned_messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            message_id TEXT NOT NULL,
            channel_id TEXT NOT NULL,
            pinned_by TEXT NOT NULL,
            pinned_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(message_id, channel_id)
        );

        -- File transfers
        CREATE TABLE IF NOT EXISTS file_transfers (
            id TEXT PRIMARY KEY,
            friend_number INTEGER,
            file_number INTEGER,
            filename TEXT NOT NULL,
            file_size INTEGER NOT NULL DEFAULT 0,
            file_path TEXT,
            direction TEXT NOT NULL DEFAULT 'incoming',
            status TEXT NOT NULL DEFAULT 'pending',
            bytes_transferred INTEGER NOT NULL DEFAULT 0,
            started_at TEXT NOT NULL DEFAULT (datetime('now')),
            completed_at TEXT
        );

        -- Offline message queue
        CREATE TABLE IF NOT EXISTS offline_queue (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            target_type TEXT NOT NULL DEFAULT 'friend',
            target_id TEXT NOT NULL,
            message_type TEXT NOT NULL DEFAULT 'text',
            content TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            attempts INTEGER NOT NULL DEFAULT 0,
            last_attempt TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_offline_target ON offline_queue(target_type, target_id);

        -- Full-text search for messages
        CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
            content,
            message_id UNINDEXED,
            source_table UNINDEXED,
            content='',
            tokenize='unicode61'
        );

        -- Triggers to keep FTS in sync with direct_messages
        CREATE TRIGGER IF NOT EXISTS dm_fts_insert AFTER INSERT ON direct_messages BEGIN
            INSERT INTO messages_fts(content, message_id, source_table)
            VALUES (NEW.content, NEW.id, 'direct_messages');
        END;

        CREATE TRIGGER IF NOT EXISTS dm_fts_delete AFTER DELETE ON direct_messages BEGIN
            INSERT INTO messages_fts(messages_fts, content, message_id, source_table)
            VALUES ('delete', OLD.content, OLD.id, 'direct_messages');
        END;

        -- Triggers to keep FTS in sync with channel_messages
        CREATE TRIGGER IF NOT EXISTS cmsg_fts_insert AFTER INSERT ON channel_messages BEGIN
            INSERT INTO messages_fts(content, message_id, source_table)
            VALUES (NEW.content, NEW.id, 'channel_messages');
        END;

        CREATE TRIGGER IF NOT EXISTS cmsg_fts_delete AFTER DELETE ON channel_messages BEGIN
            INSERT INTO messages_fts(messages_fts, content, message_id, source_table)
            VALUES ('delete', OLD.content, OLD.id, 'channel_messages');
        END;
        ",
    )?;

    set_schema_version(conn, 1)?;
    info!("Migration v1 complete");
    Ok(())
}

/// Version 2: Ensure guild/channel tables exist (for databases created before Phase 4)
fn migrate_v2(conn: &Connection) -> rusqlite::Result<()> {
    info!("Running migration v2: ensure guild tables");

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS guilds (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            metadata_group_number INTEGER,
            metadata_doc BLOB,
            icon_hash TEXT,
            owner_public_key TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            last_synced TEXT
        );

        CREATE TABLE IF NOT EXISTS channels (
            id TEXT PRIMARY KEY,
            guild_id TEXT NOT NULL,
            name TEXT NOT NULL,
            topic TEXT NOT NULL DEFAULT '',
            channel_type TEXT NOT NULL DEFAULT 'text',
            category TEXT,
            position INTEGER NOT NULL DEFAULT 0,
            group_number INTEGER,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (guild_id) REFERENCES guilds(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_channel_guild ON channels(guild_id, position);

        CREATE TABLE IF NOT EXISTS channel_messages (
            id TEXT PRIMARY KEY,
            channel_id TEXT NOT NULL,
            sender_public_key TEXT NOT NULL,
            sender_name TEXT NOT NULL DEFAULT '',
            content TEXT NOT NULL,
            message_type TEXT NOT NULL DEFAULT 'normal',
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            edited_at TEXT,
            thread_id TEXT,
            reply_to TEXT,
            FOREIGN KEY (channel_id) REFERENCES channels(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_cmsg_channel ON channel_messages(channel_id, timestamp);

        CREATE TRIGGER IF NOT EXISTS cmsg_fts_insert AFTER INSERT ON channel_messages BEGIN
            INSERT INTO messages_fts(content, message_id, source_table)
            VALUES (NEW.content, NEW.id, 'channel_messages');
        END;

        CREATE TRIGGER IF NOT EXISTS cmsg_fts_delete AFTER DELETE ON channel_messages BEGIN
            INSERT INTO messages_fts(messages_fts, content, message_id, source_table)
            VALUES ('delete', OLD.content, OLD.id, 'channel_messages');
        END;
        ",
    )?;

    set_schema_version(conn, 2)?;
    info!("Migration v2 complete");
    Ok(())
}

/// Version 3: Add guild_type column to distinguish servers from DM groups
fn migrate_v3(conn: &Connection) -> rusqlite::Result<()> {
    info!("Running migration v3: add guild_type column");

    conn.execute_batch(
        "
        ALTER TABLE guilds ADD COLUMN guild_type TEXT NOT NULL DEFAULT 'server';
        ",
    )?;

    set_schema_version(conn, 3)?;
    info!("Migration v3 complete");
    Ok(())
}
