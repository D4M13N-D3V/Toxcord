use std::sync::Arc;

use tokio::sync::{oneshot, Mutex};
use tracing::{error, info};

use crate::db::message_store::{ChannelMessageRecord, ChannelRecord, GuildRecord};
use crate::db::MessageStore;
use crate::managers::tox_manager::{ToxCommand, ToxManager};

/// Higher-level guild abstraction that maps NGC groups to guilds.
///
/// Each guild uses a single NGC group. Channels are a logical separation
/// at the application layer — all messages go to the same group, tagged
/// with a channel_id.
pub struct GuildManager {
    store: Arc<MessageStore>,
}

impl GuildManager {
    pub fn new(store: Arc<MessageStore>) -> Self {
        Self { store }
    }

    /// Create a new guild. Creates an NGC group and persists the guild + default "general" channel.
    pub async fn create_guild(
        &self,
        name: &str,
        tox_manager: &Arc<Mutex<ToxManager>>,
    ) -> Result<GuildRecord, String> {
        // Create the NGC group
        let (tx, rx) = oneshot::channel();
        tox_manager
            .lock()
            .await
            .send_command(ToxCommand::GroupNew(name.to_string(), tx))
            .await?;
        let group_number = rx.await.map_err(|_| "Failed to receive response".to_string())??;

        // Get our public key for the owner field
        let (pk_tx, pk_rx) = oneshot::channel();
        tox_manager
            .lock()
            .await
            .send_command(ToxCommand::GroupGetSelfPk(group_number, pk_tx))
            .await?;
        let owner_pk = pk_rx
            .await
            .map_err(|_| "Failed to receive response".to_string())?
            .unwrap_or_default();

        let guild_id = uuid::Uuid::new_v4().to_string();

        // Persist guild
        self.store
            .insert_guild(&guild_id, name, Some(group_number as i64), &owner_pk, "server")?;

        // Create default "general" channel
        let channel_id = uuid::Uuid::new_v4().to_string();
        self.store
            .insert_channel(&channel_id, &guild_id, "general", "text", 0)?;

        info!("Created guild '{name}' with group_number={group_number}");

        self.store
            .get_guild(&guild_id)?
            .ok_or_else(|| "Guild not found after creation".to_string())
    }

    /// Get all guilds from the database.
    pub fn get_guilds(&self) -> Result<Vec<GuildRecord>, String> {
        self.store.get_guilds()
    }

    /// Get channels for a guild.
    pub fn get_guild_channels(&self, guild_id: &str) -> Result<Vec<ChannelRecord>, String> {
        self.store.get_channels(guild_id)
    }

    /// Add a new channel to a guild.
    pub fn add_channel(
        &self,
        guild_id: &str,
        name: &str,
    ) -> Result<ChannelRecord, String> {
        let position = self.store.get_channel_count(guild_id)?;
        let channel_id = uuid::Uuid::new_v4().to_string();
        self.store
            .insert_channel(&channel_id, guild_id, name, "text", position)?;

        let channels = self.store.get_channels(guild_id)?;
        channels
            .into_iter()
            .find(|c| c.id == channel_id)
            .ok_or_else(|| "Channel not found after creation".to_string())
    }

    /// Remove a channel from a guild.
    pub fn remove_channel(&self, _guild_id: &str, channel_id: &str) -> Result<(), String> {
        self.store.delete_channel(channel_id)
    }

    /// Update a guild's name.
    pub fn update_guild_name(&self, guild_id: &str, name: &str) -> Result<(), String> {
        self.store.update_guild_name(guild_id, name)
    }

    /// Rename a channel.
    pub fn rename_channel(&self, channel_id: &str, name: &str) -> Result<(), String> {
        self.store.rename_channel(channel_id, name)
    }

    /// Invite a friend to the guild's NGC group.
    pub async fn invite_to_guild(
        &self,
        guild_id: &str,
        friend_number: u32,
        tox_manager: &Arc<Mutex<ToxManager>>,
    ) -> Result<(), String> {
        let guild = self
            .store
            .get_guild(guild_id)?
            .ok_or("Guild not found")?;

        let group_number = guild
            .metadata_group_number
            .ok_or("Guild has no group number")? as u32;

        info!(
            "Inviting friend {} to guild '{}' (group_number={})",
            friend_number, guild.name, group_number
        );

        // Verify the group exists in the tox instance before attempting invite
        let (check_tx, check_rx) = oneshot::channel();
        tox_manager
            .lock()
            .await
            .send_command(ToxCommand::GroupGetInfo(group_number, check_tx))
            .await?;
        if let Err(e) = check_rx.await.map_err(|_| "Failed to receive response".to_string())? {
            return Err(format!(
                "Group {} no longer exists in tox instance (guild '{}'): {}. \
                 The guild record may be stale — try recreating the server.",
                group_number, guild.name, e
            ));
        }

        let (tx, rx) = oneshot::channel();
        tox_manager
            .lock()
            .await
            .send_command(ToxCommand::GroupInviteFriend(group_number, friend_number, tx))
            .await?;
        rx.await
            .map_err(|_| "Failed to receive response".to_string())?
    }

    /// Accept a guild invite. Creates a local guild record from the NGC group.
    pub async fn accept_guild_invite(
        &self,
        friend_number: u32,
        invite_data: &[u8],
        group_name: &str,
        tox_manager: &Arc<Mutex<ToxManager>>,
    ) -> Result<GuildRecord, String> {
        let (tx, rx) = oneshot::channel();
        tox_manager
            .lock()
            .await
            .send_command(ToxCommand::GroupInviteAccept(
                friend_number,
                invite_data.to_vec(),
                tx,
            ))
            .await?;
        let group_number = rx.await.map_err(|_| "Failed to receive response".to_string())??;

        // Use the group name from the invite (more reliable than querying immediately)
        let raw_name = if group_name.is_empty() {
            // Fallback: try to query from Tox
            let (info_tx, info_rx) = oneshot::channel();
            tox_manager
                .lock()
                .await
                .send_command(ToxCommand::GroupGetInfo(group_number, info_tx))
                .await?;
            match info_rx.await {
                Ok(Ok(info)) => info.name,
                _ => format!("Guild #{group_number}"),
            }
        } else {
            group_name.to_string()
        };

        // Detect if this is a DM group by checking for [DM] prefix
        let (final_name, guild_type) = if raw_name.starts_with("[DM]") {
            (raw_name.strip_prefix("[DM]").unwrap_or(&raw_name).to_string(), "dm_group")
        } else {
            (raw_name, "server")
        };

        let guild_id = uuid::Uuid::new_v4().to_string();
        self.store
            .insert_guild(&guild_id, &final_name, Some(group_number as i64), "", guild_type)?;

        // Create default channel - use "messages" for DM groups, "general" for servers
        let channel_name = if guild_type == "dm_group" { "messages" } else { "general" };
        let channel_id = uuid::Uuid::new_v4().to_string();
        self.store
            .insert_channel(&channel_id, &guild_id, channel_name, "text", 0)?;

        info!("Accepted guild invite, group_number={group_number}, guild_type={guild_type}");

        self.store
            .get_guild(&guild_id)?
            .ok_or_else(|| "Guild not found after creation".to_string())
    }

    /// Create a DM group chat with selected friends.
    pub async fn create_dm_group(
        &self,
        name: &str,
        friend_numbers: &[u32],
        tox_manager: &Arc<Mutex<ToxManager>>,
    ) -> Result<GuildRecord, String> {
        // Create the NGC group with [DM] prefix so recipients know it's a DM group
        let tox_group_name = format!("[DM]{}", name);
        let (tx, rx) = oneshot::channel();
        tox_manager
            .lock()
            .await
            .send_command(ToxCommand::GroupNew(tox_group_name, tx))
            .await?;
        let group_number = rx.await.map_err(|_| "Failed to receive response".to_string())??;

        // Get our public key for the owner field
        let (pk_tx, pk_rx) = oneshot::channel();
        tox_manager
            .lock()
            .await
            .send_command(ToxCommand::GroupGetSelfPk(group_number, pk_tx))
            .await?;
        let owner_pk = pk_rx
            .await
            .map_err(|_| "Failed to receive response".to_string())?
            .unwrap_or_default();

        let guild_id = uuid::Uuid::new_v4().to_string();

        // Persist as dm_group type
        self.store
            .insert_guild(&guild_id, name, Some(group_number as i64), &owner_pk, "dm_group")?;

        // Create a single "messages" channel for DM groups
        let channel_id = uuid::Uuid::new_v4().to_string();
        self.store
            .insert_channel(&channel_id, &guild_id, "messages", "text", 0)?;

        // Invite all selected friends
        for &friend_number in friend_numbers {
            let (inv_tx, inv_rx) = oneshot::channel();
            tox_manager
                .lock()
                .await
                .send_command(ToxCommand::GroupInviteFriend(group_number, friend_number, inv_tx))
                .await?;
            if let Err(e) = inv_rx.await.map_err(|_| "Failed to receive response".to_string())? {
                error!("Failed to invite friend {friend_number} to DM group: {e}");
            }
        }

        info!("Created DM group '{name}' with group_number={group_number}");

        self.store
            .get_guild(&guild_id)?
            .ok_or_else(|| "DM group not found after creation".to_string())
    }

    /// Send a message to a DM group (uses [DM] prefix).
    pub async fn send_dm_group_message(
        &self,
        guild_id: &str,
        content: &str,
        tox_manager: &Arc<Mutex<ToxManager>>,
    ) -> Result<ChannelMessageRecord, String> {
        let guild = self
            .store
            .get_guild(guild_id)?
            .ok_or("DM group not found")?;

        if guild.guild_type != "dm_group" {
            return Err("Not a DM group".to_string());
        }

        let group_number = guild
            .metadata_group_number
            .ok_or("DM group has no group number")? as u32;

        // Prefix message with [DM] for DM group routing
        let prefixed_content = format!("[DM]{}", content);

        let (tx, rx) = oneshot::channel();
        tox_manager
            .lock()
            .await
            .send_command(ToxCommand::GroupSendMessage(
                group_number,
                prefixed_content,
                tx,
            ))
            .await?;
        rx.await
            .map_err(|_| "Failed to receive response".to_string())??;

        // Get our own public key
        let (pk_tx, pk_rx) = oneshot::channel();
        tox_manager
            .lock()
            .await
            .send_command(ToxCommand::GroupGetSelfPk(group_number, pk_tx))
            .await?;
        let self_pk = pk_rx
            .await
            .map_err(|_| "Failed to receive response".to_string())?
            .unwrap_or_default();

        // Get our display name
        let (info_tx, info_rx) = oneshot::channel();
        tox_manager
            .lock()
            .await
            .send_command(ToxCommand::GetProfileInfo(info_tx))
            .await?;
        let self_name = info_rx
            .await
            .map(|p| p.name)
            .unwrap_or_default();

        // Get the messages channel for this DM group
        let channels = self.store.get_channels(guild_id)?;
        let channel_id = channels
            .first()
            .map(|c| c.id.clone())
            .unwrap_or_else(|| format!("dm_group_{group_number}"));

        let msg_id = uuid::Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().to_rfc3339();

        let record = ChannelMessageRecord {
            id: msg_id,
            channel_id,
            sender_public_key: self_pk,
            sender_name: self_name,
            content: content.to_string(),
            message_type: "normal".to_string(),
            timestamp,
        };

        self.store.insert_channel_message(&record)?;
        Ok(record)
    }

    /// Send a message to a channel in a guild.
    pub async fn send_channel_message(
        &self,
        guild_id: &str,
        channel_id: &str,
        content: &str,
        tox_manager: &Arc<Mutex<ToxManager>>,
    ) -> Result<ChannelMessageRecord, String> {
        let guild = self
            .store
            .get_guild(guild_id)?
            .ok_or("Guild not found")?;

        let group_number = guild
            .metadata_group_number
            .ok_or("Guild has no group number")? as u32;

        // Get channel name for routing prefix
        let channels = self.store.get_channels(guild_id)?;
        let channel_name = channels
            .iter()
            .find(|c| c.id == channel_id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "general".to_string());

        // Prefix message with channel name: [CH:general]content
        let prefixed_content = format!("[CH:{}]{}", channel_name, content);

        info!("Sending message to group {} channel '{}': {:?}",
              group_number, channel_name, content.chars().take(50).collect::<String>());

        let (tx, rx) = oneshot::channel();
        tox_manager
            .lock()
            .await
            .send_command(ToxCommand::GroupSendMessage(
                group_number,
                prefixed_content,
                tx,
            ))
            .await?;

        match rx.await {
            Ok(Ok(msg_id)) => {
                info!("Message sent to group {} (tox_msg_id={})", group_number, msg_id);
            }
            Ok(Err(e)) => {
                error!("Failed to send message to group {}: {}", group_number, e);
                return Err(format!("Failed to send message: {}", e));
            }
            Err(_) => {
                error!("Channel closed when sending to group {}", group_number);
                return Err("Failed to receive response from Tox thread".to_string());
            }
        }

        // Get our own public key
        let (pk_tx, pk_rx) = oneshot::channel();
        tox_manager
            .lock()
            .await
            .send_command(ToxCommand::GroupGetSelfPk(group_number, pk_tx))
            .await?;
        let self_pk = pk_rx
            .await
            .map_err(|_| "Failed to receive response".to_string())?
            .unwrap_or_default();

        // Get our display name
        let (info_tx, info_rx) = oneshot::channel();
        tox_manager
            .lock()
            .await
            .send_command(ToxCommand::GetProfileInfo(info_tx))
            .await?;
        let self_name = info_rx
            .await
            .map(|p| p.name)
            .unwrap_or_default();

        let msg_id = uuid::Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().to_rfc3339();

        let record = ChannelMessageRecord {
            id: msg_id,
            channel_id: channel_id.to_string(),
            sender_public_key: self_pk,
            sender_name: self_name,
            content: content.to_string(),
            message_type: "normal".to_string(),
            timestamp,
        };

        self.store.insert_channel_message(&record)?;
        Ok(record)
    }

    /// Get channel messages with pagination.
    pub fn get_channel_messages(
        &self,
        channel_id: &str,
        limit: i64,
        before_timestamp: Option<&str>,
    ) -> Result<Vec<ChannelMessageRecord>, String> {
        self.store
            .get_channel_messages(channel_id, limit, before_timestamp)
    }

    /// Get the guild associated with a group number (for mapping incoming events).
    #[allow(dead_code)]
    pub fn get_guild_by_group_number(&self, group_number: i64) -> Result<Option<GuildRecord>, String> {
        self.store.get_guild_by_group_number(group_number)
    }

    /// Delete a guild and leave its NGC group.
    pub async fn delete_guild(
        &self,
        guild_id: &str,
        tox_manager: &Arc<Mutex<ToxManager>>,
    ) -> Result<(), String> {
        let guild = self
            .store
            .get_guild(guild_id)?
            .ok_or("Guild not found")?;

        if let Some(group_number) = guild.metadata_group_number {
            let (tx, rx) = oneshot::channel();
            tox_manager
                .lock()
                .await
                .send_command(ToxCommand::GroupLeave(group_number as u32, tx))
                .await?;
            if let Err(e) = rx.await.map_err(|_| "Failed to receive response".to_string())? {
                error!("Failed to leave NGC group: {e}");
            }
        }

        self.store.delete_guild(guild_id)
    }
}
