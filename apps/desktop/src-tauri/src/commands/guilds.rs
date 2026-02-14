use tauri::State;
use tokio::sync::oneshot;

use crate::managers::guild_manager::GuildManager;
use crate::managers::tox_manager::ToxCommand;
use crate::AppState;

// ─── Response types ────────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct GuildInfo {
    pub id: String,
    pub name: String,
    pub group_number: Option<i64>,
    pub owner_public_key: String,
    pub guild_type: String,
    pub created_at: String,
}

#[derive(serde::Serialize)]
pub struct ChannelInfo {
    pub id: String,
    pub guild_id: String,
    pub name: String,
    pub topic: String,
    pub channel_type: String,
    pub position: i64,
}

#[derive(serde::Serialize)]
pub struct ChannelMessageInfo {
    pub id: String,
    pub channel_id: String,
    pub sender_public_key: String,
    pub sender_name: String,
    pub content: String,
    pub message_type: String,
    pub timestamp: String,
    pub is_own: bool,
}

#[derive(serde::Serialize)]
pub struct MemberInfo {
    pub peer_id: u32,
    pub name: String,
    pub public_key: String,
    pub role: String,
    pub status: String,
}

// ─── Commands ──────────────────────────────────────────────────────

#[tauri::command]
pub async fn create_guild(
    name: String,
    state: State<'_, AppState>,
) -> Result<GuildInfo, String> {
    let store = state
        .message_store
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;
    let tox = state
        .tox_manager
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;

    let gm = GuildManager::new(store);
    let record = gm.create_guild(&name, &tox).await?;

    Ok(GuildInfo {
        id: record.id,
        name: record.name,
        group_number: record.metadata_group_number,
        owner_public_key: record.owner_public_key,
        guild_type: record.guild_type,
        created_at: record.created_at,
    })
}

#[tauri::command]
pub async fn get_guilds(state: State<'_, AppState>) -> Result<Vec<GuildInfo>, String> {
    let store = state
        .message_store
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;

    let gm = GuildManager::new(store);
    let guilds = gm.get_guilds()?;

    // Filter to only return server type guilds (not dm_groups)
    Ok(guilds
        .into_iter()
        .filter(|g| g.guild_type == "server")
        .map(|g| GuildInfo {
            id: g.id,
            name: g.name,
            group_number: g.metadata_group_number,
            owner_public_key: g.owner_public_key,
            guild_type: g.guild_type,
            created_at: g.created_at,
        })
        .collect())
}

#[tauri::command]
pub async fn get_guild_channels(
    guild_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ChannelInfo>, String> {
    let store = state
        .message_store
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;

    let gm = GuildManager::new(store);
    let channels = gm.get_guild_channels(&guild_id)?;

    Ok(channels
        .into_iter()
        .map(|c| ChannelInfo {
            id: c.id,
            guild_id: c.guild_id,
            name: c.name,
            topic: c.topic,
            channel_type: c.channel_type,
            position: c.position,
        })
        .collect())
}

#[tauri::command]
pub async fn create_channel(
    guild_id: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<ChannelInfo, String> {
    let store = state
        .message_store
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;

    let gm = GuildManager::new(store);
    let channel = gm.add_channel(&guild_id, &name)?;

    Ok(ChannelInfo {
        id: channel.id,
        guild_id: channel.guild_id,
        name: channel.name,
        topic: channel.topic,
        channel_type: channel.channel_type,
        position: channel.position,
    })
}

#[tauri::command]
pub async fn delete_channel(
    guild_id: String,
    channel_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let store = state
        .message_store
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;

    let gm = GuildManager::new(store);
    gm.remove_channel(&guild_id, &channel_id)
}

#[tauri::command]
pub async fn send_channel_message(
    guild_id: String,
    channel_id: String,
    message: String,
    state: State<'_, AppState>,
) -> Result<ChannelMessageInfo, String> {
    let store = state
        .message_store
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;
    let tox = state
        .tox_manager
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;

    let gm = GuildManager::new(store);
    let record = gm
        .send_channel_message(&guild_id, &channel_id, &message, &tox)
        .await?;

    Ok(ChannelMessageInfo {
        id: record.id,
        channel_id: record.channel_id,
        sender_public_key: record.sender_public_key,
        sender_name: record.sender_name,
        content: record.content,
        message_type: record.message_type,
        timestamp: record.timestamp,
        is_own: true,
    })
}

#[tauri::command]
pub async fn get_channel_messages(
    channel_id: String,
    limit: Option<i64>,
    before_timestamp: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<ChannelMessageInfo>, String> {
    let store = state
        .message_store
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;

    let gm = GuildManager::new(store);
    let messages = gm.get_channel_messages(
        &channel_id,
        limit.unwrap_or(50),
        before_timestamp.as_deref(),
    )?;

    // We need our own public key to determine is_own.
    // Get it from tox_manager if available.
    let self_pk = if let Some(tox) = state.tox_manager.lock().await.clone() {
        let (tx, rx) = oneshot::channel();
        if tox
            .lock()
            .await
            .send_command(ToxCommand::GetProfileInfo(tx))
            .await
            .is_ok()
        {
            rx.await.ok().map(|p| {
                // ProfileInfo has tox_id (address), we need the public key (first 64 chars)
                p.tox_id.as_str()[..64].to_uppercase()
            })
        } else {
            None
        }
    } else {
        None
    };

    Ok(messages
        .into_iter()
        .map(|m| {
            let is_own = self_pk
                .as_ref()
                .map(|pk| m.sender_public_key.to_uppercase() == *pk)
                .unwrap_or(false);
            ChannelMessageInfo {
                id: m.id,
                channel_id: m.channel_id,
                sender_public_key: m.sender_public_key,
                sender_name: m.sender_name,
                content: m.content,
                message_type: m.message_type,
                timestamp: m.timestamp,
                is_own,
            }
        })
        .collect())
}

#[tauri::command]
pub async fn invite_to_guild(
    guild_id: String,
    friend_number: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let store = state
        .message_store
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;
    let tox = state
        .tox_manager
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;

    let gm = GuildManager::new(store);
    gm.invite_to_guild(&guild_id, friend_number, &tox).await
}

#[tauri::command]
pub async fn accept_guild_invite(
    friend_number: u32,
    invite_data: Vec<u8>,
    group_name: String,
    state: State<'_, AppState>,
) -> Result<GuildInfo, String> {
    let store = state
        .message_store
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;
    let tox = state
        .tox_manager
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;

    let gm = GuildManager::new(store);
    let record = gm
        .accept_guild_invite(friend_number, &invite_data, &group_name, &tox)
        .await?;

    Ok(GuildInfo {
        id: record.id,
        name: record.name,
        group_number: record.metadata_group_number,
        owner_public_key: record.owner_public_key,
        guild_type: record.guild_type,
        created_at: record.created_at,
    })
}

#[tauri::command]
pub async fn get_guild_members(
    guild_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<MemberInfo>, String> {
    let store = state
        .message_store
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;
    let tox = state
        .tox_manager
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;

    let guild = GuildManager::new(store.clone())
        .get_guilds()?
        .into_iter()
        .find(|g| g.id == guild_id)
        .ok_or("Guild not found")?;

    let group_number = guild
        .metadata_group_number
        .ok_or("Guild has no group number")? as u32;

    let (tx, rx) = oneshot::channel();
    tox.lock()
        .await
        .send_command(ToxCommand::GroupGetPeerList(group_number, tx))
        .await?;
    let peers = rx
        .await
        .map_err(|_| "Failed to receive response".to_string())?;

    Ok(peers
        .into_iter()
        .map(|p| {
            let role_str = match p.role {
                toxcord_tox::GroupRole::Founder => "founder",
                toxcord_tox::GroupRole::Moderator => "moderator",
                toxcord_tox::GroupRole::User => "user",
                toxcord_tox::GroupRole::Observer => "observer",
            };
            let status_str = match p.status {
                toxcord_tox::UserStatus::None => "online",
                toxcord_tox::UserStatus::Away => "away",
                toxcord_tox::UserStatus::Busy => "busy",
            };
            MemberInfo {
                peer_id: p.peer_id,
                name: p.name,
                public_key: p.public_key,
                role: role_str.to_string(),
                status: status_str.to_string(),
            }
        })
        .collect())
}

#[tauri::command]
pub async fn set_channel_topic(
    guild_id: String,
    _channel_id: String,
    topic: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let store = state
        .message_store
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;
    let tox = state
        .tox_manager
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;

    let guild = GuildManager::new(store)
        .get_guilds()?
        .into_iter()
        .find(|g| g.id == guild_id)
        .ok_or("Guild not found")?;

    let group_number = guild
        .metadata_group_number
        .ok_or("Guild has no group number")? as u32;

    let (tx, rx) = oneshot::channel();
    tox.lock()
        .await
        .send_command(ToxCommand::GroupSetTopic(group_number, topic, tx))
        .await?;
    rx.await
        .map_err(|_| "Failed to receive response".to_string())?
}

#[tauri::command]
pub async fn kick_member(
    guild_id: String,
    peer_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let store = state
        .message_store
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;
    let tox = state
        .tox_manager
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;

    let guild = GuildManager::new(store)
        .get_guilds()?
        .into_iter()
        .find(|g| g.id == guild_id)
        .ok_or("Guild not found")?;

    let group_number = guild
        .metadata_group_number
        .ok_or("Guild has no group number")? as u32;

    let (tx, rx) = oneshot::channel();
    tox.lock()
        .await
        .send_command(ToxCommand::GroupKickPeer(group_number, peer_id, tx))
        .await?;
    rx.await
        .map_err(|_| "Failed to receive response".to_string())?
}

#[tauri::command]
pub async fn set_member_role(
    guild_id: String,
    peer_id: u32,
    role: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let store = state
        .message_store
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;
    let tox = state
        .tox_manager
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;

    let guild = GuildManager::new(store)
        .get_guilds()?
        .into_iter()
        .find(|g| g.id == guild_id)
        .ok_or("Guild not found")?;

    let group_number = guild
        .metadata_group_number
        .ok_or("Guild has no group number")? as u32;

    let role_num: u8 = match role.as_str() {
        "moderator" => 1,
        "user" => 2,
        "observer" => 3,
        _ => return Err("Invalid role".to_string()),
    };

    let (tx, rx) = oneshot::channel();
    tox.lock()
        .await
        .send_command(ToxCommand::GroupSetRole(group_number, peer_id, role_num, tx))
        .await?;
    rx.await
        .map_err(|_| "Failed to receive response".to_string())?
}

#[tauri::command]
pub async fn rename_guild(
    guild_id: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let store = state
        .message_store
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;

    let gm = GuildManager::new(store);
    gm.update_guild_name(&guild_id, &name)
}

#[tauri::command]
pub async fn rename_channel(
    channel_id: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let store = state
        .message_store
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;

    let gm = GuildManager::new(store);
    gm.rename_channel(&channel_id, &name)
}

#[tauri::command]
pub async fn leave_guild(
    guild_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let store = state
        .message_store
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;
    let tox = state
        .tox_manager
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;

    let gm = GuildManager::new(store);
    gm.delete_guild(&guild_id, &tox).await
}

#[tauri::command]
pub async fn create_dm_group(
    name: String,
    friend_numbers: Vec<u32>,
    state: State<'_, AppState>,
) -> Result<GuildInfo, String> {
    let store = state
        .message_store
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;
    let tox = state
        .tox_manager
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;

    let gm = GuildManager::new(store);
    let record = gm.create_dm_group(&name, &friend_numbers, &tox).await?;

    Ok(GuildInfo {
        id: record.id,
        name: record.name,
        group_number: record.metadata_group_number,
        owner_public_key: record.owner_public_key,
        guild_type: record.guild_type,
        created_at: record.created_at,
    })
}

#[tauri::command]
pub async fn send_dm_group_message(
    guild_id: String,
    message: String,
    state: State<'_, AppState>,
) -> Result<ChannelMessageInfo, String> {
    let store = state
        .message_store
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;
    let tox = state
        .tox_manager
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;

    let gm = GuildManager::new(store);
    let record = gm.send_dm_group_message(&guild_id, &message, &tox).await?;

    Ok(ChannelMessageInfo {
        id: record.id,
        channel_id: record.channel_id,
        sender_public_key: record.sender_public_key,
        sender_name: record.sender_name,
        content: record.content,
        message_type: record.message_type,
        timestamp: record.timestamp,
        is_own: true,
    })
}

#[tauri::command]
pub async fn get_dm_groups(state: State<'_, AppState>) -> Result<Vec<GuildInfo>, String> {
    let store = state
        .message_store
        .lock()
        .await
        .clone()
        .ok_or("Not logged in")?;

    let gm = GuildManager::new(store);
    let guilds = gm.get_guilds()?;

    Ok(guilds
        .into_iter()
        .filter(|g| g.guild_type == "dm_group")
        .map(|g| GuildInfo {
            id: g.id,
            name: g.name,
            group_number: g.metadata_group_number,
            owner_public_key: g.owner_public_key,
            guild_type: g.guild_type,
            created_at: g.created_at,
        })
        .collect())
}
