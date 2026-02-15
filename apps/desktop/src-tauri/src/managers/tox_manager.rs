use std::path::PathBuf;
use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{debug, error, info, warn};

use toxcord_tox::callbacks::ToxEventHandler;
use toxcord_tox::tox::{decrypt_savedata, default_bootstrap_nodes, encrypt_savedata, is_data_encrypted};
use toxcord_tox::types::*;
use toxcord_tox::{AudioFrame, ProxyType, ToxAvEventHandler, ToxAvInstance, ToxInstance, ToxOptionsBuilder, VideoFrame};

use super::av_manager::{AvManager, CallState, CallStatus, TauriAvEventHandler, ToxAvEvent};
use crate::audio::{AudioCapture, AudioMixer, AudioPlayback};
use crate::video::{ScreenCapture, VideoCapture, VideoCaptureError, VideoFrameData};
use crate::AppState;

/// Proxy configuration for Tox connections
#[derive(Clone, Debug, Default)]
pub struct ProxyConfig {
    pub proxy_type: ProxyType,
    pub host: Option<String>,
    pub port: u16,
}

impl ProxyConfig {
    /// No proxy (default)
    pub fn none() -> Self {
        Self::default()
    }

    /// SOCKS5 proxy (recommended for I2P/Tor)
    pub fn socks5(host: &str, port: u16) -> Self {
        Self {
            proxy_type: ProxyType::Socks5,
            host: Some(host.to_string()),
            port,
        }
    }

    /// HTTP proxy
    pub fn http(host: &str, port: u16) -> Self {
        Self {
            proxy_type: ProxyType::Http,
            host: Some(host.to_string()),
            port,
        }
    }

    /// Load proxy config from environment variables
    /// Set TOXCORD_PROXY_TYPE=socks5 or http
    /// Set TOXCORD_PROXY_HOST=127.0.0.1
    /// Set TOXCORD_PROXY_PORT=4447
    pub fn from_env() -> Self {
        let proxy_type = std::env::var("TOXCORD_PROXY_TYPE").ok();
        let host = std::env::var("TOXCORD_PROXY_HOST").ok();
        let port = std::env::var("TOXCORD_PROXY_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(0);

        match (proxy_type.as_deref(), host) {
            (Some("socks5"), Some(h)) if port > 0 => Self::socks5(&h, port),
            (Some("http"), Some(h)) if port > 0 => Self::http(&h, port),
            _ => Self::none(),
        }
    }

    /// Create proxy config from embedded I2P router
    #[cfg(feature = "i2p")]
    pub fn from_i2p(i2p_manager: &super::i2p_manager::I2pManager) -> Self {
        Self::socks5("127.0.0.1", i2p_manager.socks_port())
    }
}

use crate::db::MessageStore;

/// Commands sent to the Tox thread via mpsc channel
pub enum ToxCommand {
    GetAddress(oneshot::Sender<ToxAddress>),
    GetConnectionStatus(oneshot::Sender<ConnectionStatus>),
    GetProfileInfo(oneshot::Sender<ProfileInfo>),
    SetName(String, oneshot::Sender<Result<(), String>>),
    SetStatusMessage(String, oneshot::Sender<Result<(), String>>),
    FriendAdd(String, String, oneshot::Sender<Result<u32, String>>),
    FriendAccept([u8; 32], oneshot::Sender<Result<u32, String>>),
    FriendDelete(u32, oneshot::Sender<Result<(), String>>),
    FriendList(oneshot::Sender<Vec<FriendInfo>>),
    FriendSendMessage(u32, String, oneshot::Sender<Result<u32, String>>),
    SetTyping(u32, bool, oneshot::Sender<Result<(), String>>),
    SaveProfile(oneshot::Sender<Result<(), String>>),
    Shutdown(oneshot::Sender<()>),
    // Group commands
    GroupNew(String, oneshot::Sender<Result<u32, String>>),
    GroupJoin([u8; 32], String, oneshot::Sender<Result<u32, String>>),
    GroupLeave(u32, oneshot::Sender<Result<(), String>>),
    GroupInviteFriend(u32, u32, oneshot::Sender<Result<(), String>>),
    GroupInviteAccept(u32, Vec<u8>, oneshot::Sender<Result<u32, String>>),
    GroupSendMessage(u32, String, oneshot::Sender<Result<u32, String>>),
    GroupSendCustomPacket(u32, Vec<u8>, oneshot::Sender<Result<(), String>>),
    GroupGetList(oneshot::Sender<Vec<GroupInfo>>),
    GroupGetPeerList(u32, oneshot::Sender<Vec<GroupPeerInfo>>),
    GroupSetTopic(u32, String, oneshot::Sender<Result<(), String>>),
    GroupSetRole(u32, u32, u8, oneshot::Sender<Result<(), String>>),
    GroupKickPeer(u32, u32, oneshot::Sender<Result<(), String>>),
    GroupGetInfo(u32, oneshot::Sender<Result<GroupInfo, String>>),
    GroupGetSelfPk(u32, oneshot::Sender<Result<String, String>>),
    GroupReconnect(u32, oneshot::Sender<Result<(), String>>),
    // ToxAV commands
    AvCall {
        friend_number: u32,
        audio_bit_rate: u32,
        video_bit_rate: u32,
        reply: oneshot::Sender<Result<(), String>>,
    },
    AvAnswer {
        friend_number: u32,
        audio_bit_rate: u32,
        video_bit_rate: u32,
        reply: oneshot::Sender<Result<(), String>>,
    },
    AvHangup {
        friend_number: u32,
        reply: oneshot::Sender<Result<(), String>>,
    },
    AvMuteAudio {
        friend_number: u32,
        reply: oneshot::Sender<Result<(), String>>,
    },
    AvUnmuteAudio {
        friend_number: u32,
        reply: oneshot::Sender<Result<(), String>>,
    },
    AvHideVideo {
        friend_number: u32,
        reply: oneshot::Sender<Result<(), String>>,
    },
    AvShowVideo {
        friend_number: u32,
        reply: oneshot::Sender<Result<(), String>>,
    },
    AvSendAudioFrame {
        friend_number: u32,
        pcm: Vec<i16>,
        sample_count: usize,
        channels: u8,
        sampling_rate: u32,
    },
    AvGetCallState {
        friend_number: u32,
        reply: oneshot::Sender<Option<CallState>>,
    },
}

/// Events emitted to the frontend via Tauri
#[derive(Clone, serde::Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ToxEvent {
    ConnectionStatus { connected: bool, status: String },
    FriendRequest { public_key: String, message: String },
    FriendMessage { friend_number: u32, message_type: String, message: String, id: String, timestamp: String },
    FriendName { friend_number: u32, name: String },
    FriendStatusMessage { friend_number: u32, message: String },
    FriendStatus { friend_number: u32, status: String },
    FriendConnectionStatus { friend_number: u32, connected: bool, status: String },
    FriendTyping { friend_number: u32, is_typing: bool },
    // Group events
    GroupInvite { friend_number: u32, invite_data: Vec<u8>, group_name: String },
    GroupSelfJoin { group_number: u32 },
    GroupJoinFail { group_number: u32, fail_type: String },
    GroupPeerJoin { group_number: u32, peer_id: u32, name: String, public_key: String },
    GroupPeerExit { group_number: u32, peer_id: u32, name: String },
    GroupPeerName { group_number: u32, peer_id: u32, name: String },
    GroupMessage { group_number: u32, peer_id: u32, sender_name: String, sender_pk: String, message: String, message_type: String, id: String, timestamp: String, channel_id: String },
    GroupTopicChange { group_number: u32, topic: String },
    GroupCustomPacket { group_number: u32, peer_id: u32, data: Vec<u8> },
    GroupPeerStatus { group_number: u32, peer_id: u32, status: String },
}

/// ToxEventHandler implementation that emits Tauri events and persists to DB
struct TauriEventHandler {
    app_handle: AppHandle,
    store: Arc<MessageStore>,
    /// Sender to queue offline flushes for the tox thread to process
    offline_flush_tx: std::sync::mpsc::Sender<u32>,
    /// Raw tox pointer for querying peer info during callbacks.
    /// SAFETY: Only accessed on the tox thread during iterate_with_userdata.
    tox_raw: *mut toxcord_tox_sys::Tox,
}

// SAFETY: TauriEventHandler is only ever accessed on the tox thread.
// The raw pointer is only used during callbacks on that same thread.
unsafe impl Send for TauriEventHandler {}

impl TauriEventHandler {
    fn emit(&self, event: ToxEvent) {
        if let Err(e) = self.app_handle.emit("tox://event", &event) {
            error!("Failed to emit Tauri event: {e}");
        }
    }

    /// Query a peer's name from the tox instance during a callback.
    fn query_peer_name(&self, group_number: u32, peer_id: u32) -> String {
        unsafe {
            let mut err = toxcord_tox_sys::Tox_Err_Group_Peer_Query::default();
            let size = toxcord_tox_sys::tox_group_peer_get_name_size(
                self.tox_raw, group_number, peer_id, &mut err,
            );
            if err != 0 || size == 0 {
                return String::new();
            }
            let mut name = vec![0u8; size];
            toxcord_tox_sys::tox_group_peer_get_name(
                self.tox_raw, group_number, peer_id, name.as_mut_ptr(), &mut err,
            );
            String::from_utf8_lossy(&name).to_string()
        }
    }

    /// Query a peer's public key from the tox instance during a callback.
    fn query_peer_public_key(&self, group_number: u32, peer_id: u32) -> String {
        unsafe {
            let mut pk = [0u8; 32];
            let mut err = toxcord_tox_sys::Tox_Err_Group_Peer_Query::default();
            let ok = toxcord_tox_sys::tox_group_peer_get_public_key(
                self.tox_raw, group_number, peer_id, pk.as_mut_ptr(), &mut err,
            );
            if ok {
                pk.iter().map(|b| format!("{b:02X}")).collect()
            } else {
                String::new()
            }
        }
    }

    /// Parse group message prefix and return (channel_id, content).
    /// Supports: [CH:name] for guild channels, [DM] for DM groups, or no prefix (fallback).
    fn parse_group_message(&self, group_number: u32, message: &str) -> (String, String) {
        info!("parse_group_message: group={} msg_preview={:?}",
              group_number, message.chars().take(30).collect::<String>());

        // Try to parse [CH:name] prefix for guild channel messages
        if message.starts_with("[CH:") {
            if let Some(end) = message.find(']') {
                let channel_name = &message[4..end];
                let content = message[end + 1..].to_string();
                info!("[CH] Parsed [CH:{}] prefix, looking up server by group_number={}", channel_name, group_number);

                // Look up server specifically by guild_type="server" to avoid collision with DM groups
                let guild_result = self.store.get_guild_by_group_number_and_type(group_number as i64, "server");
                info!("[CH] Guild lookup result: {:?}", guild_result.as_ref().map(|g| g.as_ref().map(|gg| &gg.name)));

                if let Some(channel_id) = guild_result
                    .ok()
                    .flatten()
                    .and_then(|guild| {
                        let ch_result = self.store.get_or_create_channel_by_name(&guild.id, channel_name);
                        info!("[CH] get_or_create_channel_by_name result for '{}': {:?}", channel_name, ch_result);
                        ch_result.ok()
                    })
                {
                    info!("[CH] Successfully routed to channel_id={}", channel_id);
                    return (channel_id, content);
                }
                warn!("[CH] Failed to route [CH:{}] message - server or channel lookup failed", channel_name);
            }
        }

        // Try to parse [DM] prefix for DM group messages
        if message.starts_with("[DM]") {
            let content = message[4..].to_string();
            info!("[DM] Parsing DM group message for group_number={}", group_number);

            // For DM groups, look up specifically by guild_type="dm_group" to avoid collision with servers
            let guild_result = self.store.get_guild_by_group_number_and_type(group_number as i64, "dm_group");
            info!("[DM] Guild lookup result: {:?}", guild_result.as_ref().map(|g| g.as_ref().map(|gg| (&gg.id, &gg.name, &gg.guild_type))));

            if let Some(channel_id) = guild_result
                .ok()
                .flatten()
                .and_then(|guild| {
                    let channels_result = self.store.get_channels(&guild.id);
                    info!("[DM] Channels lookup for guild {}: {:?}", guild.id, channels_result.as_ref().map(|chs| chs.iter().map(|c| (&c.id, &c.name)).collect::<Vec<_>>()));
                    channels_result
                        .ok()
                        .and_then(|channels| channels.first().map(|c| c.id.clone()))
                })
            {
                info!("[DM] Successfully routed to channel_id={}", channel_id);
                return (channel_id, content);
            }
            warn!("[DM] Failed to find dm_group for group_number={}, using fallback", group_number);
            return (format!("dm_group_{group_number}"), content);
        }

        // Fallback: no prefix, route to first channel of guild
        let channel_id = self
            .store
            .get_guild_by_group_number(group_number as i64)
            .ok()
            .flatten()
            .and_then(|guild| {
                self.store
                    .get_channels(&guild.id)
                    .ok()
                    .and_then(|channels| channels.first().map(|c| c.id.clone()))
            })
            .unwrap_or_else(|| format!("group_{group_number}"));

        (channel_id, message.to_string())
    }
}

impl ToxEventHandler for TauriEventHandler {
    fn on_self_connection_status(&self, status: ConnectionStatus) {
        let status_str = match status {
            ConnectionStatus::None => "none",
            ConnectionStatus::Tcp => "tcp",
            ConnectionStatus::Udp => "udp",
        };
        info!("Connection status: {status_str}");

        // I2P/Proxy verification logging
        // When using I2P or any SOCKS/HTTP proxy, UDP is disabled and only TCP should be used
        match status {
            ConnectionStatus::Udp => {
                warn!("[I2P-CHECK] UDP connection detected - traffic is NOT routed through I2P/proxy!");
            }
            ConnectionStatus::Tcp => {
                info!("[I2P-CHECK] TCP connection confirmed - traffic is routed through proxy (I2P/Tor if configured)");
            }
            ConnectionStatus::None => {
                debug!("[I2P-CHECK] No connection yet");
            }
        }

        self.emit(ToxEvent::ConnectionStatus {
            connected: status.is_connected(),
            status: status_str.to_string(),
        });
    }

    fn on_friend_request(&self, public_key: &[u8; 32], message: &str) {
        let pk_hex: String = public_key.iter().map(|b| format!("{b:02X}")).collect();
        info!("Friend request from {pk_hex}");

        // Persist to DB
        if let Err(e) = self.store.add_friend_request(&pk_hex, message) {
            error!("Failed to persist friend request: {e}");
        }

        self.emit(ToxEvent::FriendRequest {
            public_key: pk_hex,
            message: message.to_string(),
        });
    }

    fn on_friend_message(&self, friend_number: u32, message_type: MessageType, message: &str) {
        let mt = match message_type {
            MessageType::Normal => "normal",
            MessageType::Action => "action",
        };

        let msg_id = uuid::Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Persist incoming message to DB
        let record = crate::db::message_store::DirectMessageRecord {
            id: msg_id.clone(),
            friend_number: friend_number as i64,
            sender: "friend".to_string(),
            content: message.to_string(),
            message_type: mt.to_string(),
            timestamp: timestamp.clone(),
            is_outgoing: false,
            delivered: true,
            read: false,
        };
        if let Err(e) = self.store.insert_direct_message(&record) {
            error!("Failed to persist incoming message: {e}");
        }

        self.emit(ToxEvent::FriendMessage {
            friend_number,
            message_type: mt.to_string(),
            message: message.to_string(),
            id: msg_id,
            timestamp,
        });
    }

    fn on_friend_name(&self, friend_number: u32, name: &str) {
        // Persist to DB
        if let Err(e) = self.store.update_friend_name(friend_number, name) {
            error!("Failed to persist friend name: {e}");
        }

        self.emit(ToxEvent::FriendName {
            friend_number,
            name: name.to_string(),
        });
    }

    fn on_friend_status_message(&self, friend_number: u32, message: &str) {
        if let Err(e) = self.store.update_friend_status_message(friend_number, message) {
            error!("Failed to persist friend status message: {e}");
        }

        self.emit(ToxEvent::FriendStatusMessage {
            friend_number,
            message: message.to_string(),
        });
    }

    fn on_friend_status(&self, friend_number: u32, status: UserStatus) {
        let s = match status {
            UserStatus::None => "online",
            UserStatus::Away => "away",
            UserStatus::Busy => "busy",
        };

        if let Err(e) = self.store.update_friend_status(friend_number, s) {
            error!("Failed to persist friend status: {e}");
        }

        self.emit(ToxEvent::FriendStatus {
            friend_number,
            status: s.to_string(),
        });
    }

    fn on_friend_connection_status(&self, friend_number: u32, status: ConnectionStatus) {
        let s = match status {
            ConnectionStatus::None => "none",
            ConnectionStatus::Tcp => "tcp",
            ConnectionStatus::Udp => "udp",
        };

        // Update last_seen when going offline
        let going_offline = matches!(status, ConnectionStatus::None);
        if let Err(e) = self.store.update_friend_connection_status(friend_number, s, going_offline) {
            error!("Failed to persist friend connection status: {e}");
        }

        // If friend came online, request offline queue flush
        if status.is_connected() {
            let _ = self.offline_flush_tx.send(friend_number);
        }

        self.emit(ToxEvent::FriendConnectionStatus {
            friend_number,
            connected: status.is_connected(),
            status: s.to_string(),
        });
    }

    fn on_friend_typing(&self, friend_number: u32, is_typing: bool) {
        self.emit(ToxEvent::FriendTyping {
            friend_number,
            is_typing,
        });
    }

    fn on_friend_read_receipt(&self, friend_number: u32, message_id: u32) {
        debug!("Read receipt: friend={friend_number} msg_id={message_id}");
        // Read receipts from Tox use sequential IDs, not our UUIDs.
        // We could map tox_msg_id -> uuid, but for now this is a no-op.
        // The message is already marked delivered=true on successful send.
    }
    fn on_file_recv_control(&self, _friend_number: u32, _file_number: u32, _control: u32) {}
    fn on_file_chunk_request(&self, _friend_number: u32, _file_number: u32, _position: u64, _length: usize) {}
    fn on_file_recv(&self, _friend_number: u32, _file_number: u32, _kind: u32, _file_size: u64, _filename: &str) {}
    fn on_file_recv_chunk(&self, _friend_number: u32, _file_number: u32, _position: u64, _data: &[u8]) {}
    fn on_group_invite(&self, friend_number: u32, invite_data: &[u8], group_name: &str) {
        info!("Group invite from friend {friend_number}: {group_name}");
        self.emit(ToxEvent::GroupInvite {
            friend_number,
            invite_data: invite_data.to_vec(),
            group_name: group_name.to_string(),
        });
    }

    fn on_group_peer_join(&self, group_number: u32, peer_id: u32) {
        let name = self.query_peer_name(group_number, peer_id);
        let public_key = self.query_peer_public_key(group_number, peer_id);
        info!("Peer joined group {group_number}: {name} ({peer_id})");
        self.emit(ToxEvent::GroupPeerJoin {
            group_number,
            peer_id,
            name,
            public_key,
        });
    }

    fn on_group_peer_exit(&self, group_number: u32, peer_id: u32, _exit_type: u32, name: &str, _message: &str) {
        info!("Peer left group {group_number}: {name} ({peer_id})");
        self.emit(ToxEvent::GroupPeerExit {
            group_number,
            peer_id,
            name: name.to_string(),
        });
    }

    fn on_group_peer_name(&self, group_number: u32, peer_id: u32, name: &str) {
        self.emit(ToxEvent::GroupPeerName {
            group_number,
            peer_id,
            name: name.to_string(),
        });
    }

    fn on_group_message(&self, group_number: u32, peer_id: u32, message_type: MessageType, message: &str, _message_id: u32) {
        let mt = match message_type {
            MessageType::Normal => "normal",
            MessageType::Action => "action",
        };
        let sender_name = self.query_peer_name(group_number, peer_id);
        let sender_pk = self.query_peer_public_key(group_number, peer_id);
        let msg_id = uuid::Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Parse message prefix: [CH:N] for channel, [DM] for DM group
        let (channel_id, content) = self.parse_group_message(group_number, message);

        info!("Group message received: group={} peer={} sender='{}' channel={} content_len={}",
              group_number, peer_id, sender_name, channel_id, content.len());

        if let Err(e) = self.store.insert_channel_message(
            &crate::db::message_store::ChannelMessageRecord {
                id: msg_id.clone(),
                channel_id: channel_id.clone(),
                sender_public_key: sender_pk.clone(),
                sender_name: sender_name.clone(),
                content: content.clone(),
                message_type: mt.to_string(),
                timestamp: timestamp.clone(),
            },
        ) {
            error!("Failed to persist group message: {e}");
        } else {
            info!("Group message persisted successfully to channel {}", channel_id);
        }

        self.emit(ToxEvent::GroupMessage {
            group_number,
            peer_id,
            sender_name,
            sender_pk,
            message: content,
            message_type: mt.to_string(),
            id: msg_id,
            timestamp,
            channel_id,
        });
    }

    fn on_group_custom_packet(&self, group_number: u32, peer_id: u32, data: &[u8]) {
        self.emit(ToxEvent::GroupCustomPacket {
            group_number,
            peer_id,
            data: data.to_vec(),
        });
    }

    fn on_group_custom_private_packet(&self, _group_number: u32, _peer_id: u32, _data: &[u8]) {
        // Custom private packets will be handled by protocol routing layer
    }

    fn on_group_self_join(&self, group_number: u32) {
        info!("Self joined group {group_number}");
        self.emit(ToxEvent::GroupSelfJoin { group_number });
    }

    fn on_group_join_fail(&self, group_number: u32, fail_type: u32) {
        let ft = match fail_type {
            0 => "peer_limit",
            1 => "invalid_password",
            _ => "unknown",
        };
        warn!("Failed to join group {group_number}: {ft}");
        self.emit(ToxEvent::GroupJoinFail {
            group_number,
            fail_type: ft.to_string(),
        });
    }

    fn on_group_topic(&self, group_number: u32, _peer_id: u32, topic: &str) {
        self.emit(ToxEvent::GroupTopicChange {
            group_number,
            topic: topic.to_string(),
        });
    }

    fn on_group_peer_status(&self, group_number: u32, peer_id: u32, status: UserStatus) {
        let s = match status {
            UserStatus::None => "online",
            UserStatus::Away => "away",
            UserStatus::Busy => "busy",
        };
        self.emit(ToxEvent::GroupPeerStatus {
            group_number,
            peer_id,
            status: s.to_string(),
        });
    }
}

/// Manages the Tox instance on a dedicated thread
pub struct ToxManager {
    cmd_tx: mpsc::Sender<ToxCommand>,
    #[allow(dead_code)]
    profile_path: PathBuf,
}

impl ToxManager {
    /// Start a new ToxManager with a fresh profile
    pub fn create_profile(
        app_handle: AppHandle,
        profile_name: &str,
        password: &str,
        display_name: &str,
        store: Arc<MessageStore>,
    ) -> Result<Arc<Mutex<Self>>, String> {
        let profile_dir = get_profiles_dir();
        std::fs::create_dir_all(&profile_dir).map_err(|e| format!("Failed to create profile dir: {e}"))?;

        let profile_path = profile_dir.join(format!("{profile_name}.tox"));

        if profile_path.exists() {
            return Err(format!("Profile '{profile_name}' already exists"));
        }

        let (cmd_tx, cmd_rx) = mpsc::channel(256);
        let password = password.to_string();
        let display_name = display_name.to_string();
        let path = profile_path.clone();

        // Load proxy config from environment variables
        let proxy_config = ProxyConfig::from_env();

        std::thread::spawn(move || {
            run_tox_thread(app_handle, cmd_rx, None, &password, &path, Some(&display_name), store, None, proxy_config);
        });

        Ok(Arc::new(Mutex::new(Self {
            cmd_tx,
            profile_path,
        })))
    }

    /// Start a ToxManager from an existing profile
    pub fn load_profile(
        app_handle: AppHandle,
        profile_name: &str,
        password: &str,
        store: Arc<MessageStore>,
    ) -> Result<Arc<Mutex<Self>>, String> {
        let profile_dir = get_profiles_dir();
        let profile_path = profile_dir.join(format!("{profile_name}.tox"));

        if !profile_path.exists() {
            return Err(format!("Profile '{profile_name}' not found"));
        }

        let savedata = std::fs::read(&profile_path)
            .map_err(|e| format!("Failed to read profile: {e}"))?;

        let savedata = if is_data_encrypted(&savedata) {
            decrypt_savedata(&savedata, password)
                .map_err(|e| format!("Failed to decrypt profile: {e}"))?
        } else {
            savedata
        };

        let (cmd_tx, cmd_rx) = mpsc::channel(256);
        let (sync_tx, sync_rx) = std::sync::mpsc::channel::<()>();
        let password = password.to_string();
        let path = profile_path.clone();

        // Load proxy config from environment variables
        let proxy_config = ProxyConfig::from_env();

        std::thread::spawn(move || {
            run_tox_thread(app_handle, cmd_rx, Some(savedata), &password, &path, None, store, Some(sync_tx), proxy_config);
        });

        // Wait for the sync to complete before returning
        let _ = sync_rx.recv_timeout(std::time::Duration::from_secs(5));

        Ok(Arc::new(Mutex::new(Self {
            cmd_tx,
            profile_path,
        })))
    }

    /// Send a command to the Tox thread
    pub async fn send_command(&self, cmd: ToxCommand) -> Result<(), String> {
        self.cmd_tx
            .send(cmd)
            .await
            .map_err(|_| "Tox thread has shut down".to_string())
    }

    /// Get the Tox address
    pub async fn get_address(&self) -> Result<ToxAddress, String> {
        let (tx, rx) = oneshot::channel();
        self.send_command(ToxCommand::GetAddress(tx)).await?;
        rx.await.map_err(|_| "Failed to receive response".to_string())
    }

    /// Get connection status
    pub async fn get_connection_status(&self) -> Result<ConnectionStatus, String> {
        let (tx, rx) = oneshot::channel();
        self.send_command(ToxCommand::GetConnectionStatus(tx)).await?;
        rx.await.map_err(|_| "Failed to receive response".to_string())
    }

    /// Get profile info
    pub async fn get_profile_info(&self) -> Result<ProfileInfo, String> {
        let (tx, rx) = oneshot::channel();
        self.send_command(ToxCommand::GetProfileInfo(tx)).await?;
        rx.await.map_err(|_| "Failed to receive response".to_string())
    }

    /// Shutdown the Tox thread
    pub async fn shutdown(&self) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.send_command(ToxCommand::Shutdown(tx)).await?;
        rx.await.map_err(|_| "Failed to shutdown".to_string())
    }

    // ─── ToxAV Methods ───────────────────────────────────────────────────────

    /// Start a call with a friend
    pub async fn call(&self, friend_number: u32, with_video: bool) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        let audio_bit_rate = 64; // Default audio bit rate (64 kbit/s)
        let video_bit_rate = if with_video { 400 } else { 0 }; // 400 kbit/s for video
        self.send_command(ToxCommand::AvCall {
            friend_number,
            audio_bit_rate,
            video_bit_rate,
            reply: tx,
        })
        .await?;
        rx.await.map_err(|_| "Failed to receive response".to_string())?
    }

    /// Answer an incoming call
    pub async fn answer(&self, friend_number: u32, with_video: bool) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        let audio_bit_rate = 64;
        let video_bit_rate = if with_video { 400 } else { 0 };
        self.send_command(ToxCommand::AvAnswer {
            friend_number,
            audio_bit_rate,
            video_bit_rate,
            reply: tx,
        })
        .await?;
        rx.await.map_err(|_| "Failed to receive response".to_string())?
    }

    /// Hangup a call
    pub async fn hangup(&self, friend_number: u32) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.send_command(ToxCommand::AvHangup {
            friend_number,
            reply: tx,
        })
        .await?;
        rx.await.map_err(|_| "Failed to receive response".to_string())?
    }

    /// Mute audio for a call
    pub async fn mute_audio(&self, friend_number: u32) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.send_command(ToxCommand::AvMuteAudio {
            friend_number,
            reply: tx,
        })
        .await?;
        rx.await.map_err(|_| "Failed to receive response".to_string())?
    }

    /// Unmute audio for a call
    pub async fn unmute_audio(&self, friend_number: u32) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.send_command(ToxCommand::AvUnmuteAudio {
            friend_number,
            reply: tx,
        })
        .await?;
        rx.await.map_err(|_| "Failed to receive response".to_string())?
    }

    /// Hide video for a call
    pub async fn hide_video(&self, friend_number: u32) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.send_command(ToxCommand::AvHideVideo {
            friend_number,
            reply: tx,
        })
        .await?;
        rx.await.map_err(|_| "Failed to receive response".to_string())?
    }

    /// Show video for a call
    pub async fn show_video(&self, friend_number: u32) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.send_command(ToxCommand::AvShowVideo {
            friend_number,
            reply: tx,
        })
        .await?;
        rx.await.map_err(|_| "Failed to receive response".to_string())?
    }

    /// Get call state for a friend
    pub async fn get_call_state(&self, friend_number: u32) -> Option<CallState> {
        let (tx, rx) = oneshot::channel();
        if self
            .send_command(ToxCommand::AvGetCallState {
                friend_number,
                reply: tx,
            })
            .await
            .is_err()
        {
            return None;
        }
        rx.await.ok().flatten()
    }

    /// List available profiles
    pub fn list_profiles() -> Vec<String> {
        let profile_dir = get_profiles_dir();
        if !profile_dir.exists() {
            return vec![];
        }

        std::fs::read_dir(profile_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path()
                            .extension()
                            .map(|ext| ext == "tox")
                            .unwrap_or(false)
                    })
                    .filter_map(|e| {
                        e.path()
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// The main Tox event loop running on a dedicated thread
fn run_tox_thread(
    app_handle: AppHandle,
    mut cmd_rx: mpsc::Receiver<ToxCommand>,
    savedata: Option<Vec<u8>>,
    password: &str,
    profile_path: &PathBuf,
    display_name: Option<&str>,
    store: Arc<MessageStore>,
    sync_complete_tx: Option<std::sync::mpsc::Sender<()>>,
    proxy_config: ProxyConfig,
) {
    // Build Tox options with proxy configuration
    let mut builder = ToxOptionsBuilder::new();

    // Apply proxy settings if configured
    if let Some(ref host) = proxy_config.host {
        match proxy_config.proxy_type {
            ProxyType::Socks5 => {
                info!("Using SOCKS5 proxy: {}:{}", host, proxy_config.port);
                builder = builder.proxy_socks5(host, proxy_config.port);
            }
            ProxyType::Http => {
                info!("Using HTTP proxy: {}:{}", host, proxy_config.port);
                builder = builder.proxy_http(host, proxy_config.port);
            }
            ProxyType::None => {}
        }
    }

    // Apply savedata if loading existing profile
    let tox = if let Some(data) = savedata {
        builder.savedata(data).build()
    } else {
        builder.build()
    };

    let tox = match tox {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to create Tox instance: {e}");
            return;
        }
    };

    // Set display name if creating new profile
    if let Some(name) = display_name {
        if let Err(e) = tox.set_name(name) {
            error!("Failed to set name: {e}");
        }
        if let Err(e) = tox.set_status_message("Using Toxcord") {
            error!("Failed to set status message: {e}");
        }
    }

    // Register callbacks
    tox.register_callbacks();

    // Channel for offline queue flush requests from callbacks
    let (offline_flush_tx, offline_flush_rx) = std::sync::mpsc::channel::<u32>();

    // Create event handler with DB persistence
    let handler: Box<dyn ToxEventHandler> = Box::new(TauriEventHandler {
        app_handle: app_handle.clone(),
        store: store.clone(),
        offline_flush_tx,
        tox_raw: tox.raw(),
    });
    let handler_ptr = Box::into_raw(Box::new(handler));

    // Create ToxAV instance (must be on same thread as Tox)
    let toxav = match ToxAvInstance::new(&tox) {
        Ok(av) => {
            info!("ToxAV instance created");
            Some(av)
        }
        Err(e) => {
            error!("Failed to create ToxAV instance: {e} - calls will be disabled");
            None
        }
    };

    // Create shared audio mixer for combining received audio from multiple peers
    let mixer = Arc::new(std::sync::Mutex::new(AudioMixer::default()));

    // Create AV manager and event handler for ToxAV callbacks
    let av_manager = Arc::new(std::sync::Mutex::new(AvManager::new()));
    let av_handler: Option<*mut Box<dyn ToxAvEventHandler>> = if toxav.is_some() {
        let handler: Box<dyn ToxAvEventHandler> = Box::new(TauriAvEventHandler::new(
            app_handle.clone(),
            av_manager.clone(),
            mixer.clone(),
        ));
        let handler_ptr = Box::into_raw(Box::new(handler));
        // Register ToxAV callbacks with our handler
        if let Some(ref av) = toxav {
            av.register_callbacks_with_userdata(handler_ptr as *mut std::ffi::c_void);
        }
        Some(handler_ptr)
    } else {
        None
    };

    // Audio capture channel - capture thread sends frames here
    let (audio_tx, mut audio_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<i16>>();

    // Audio capture and playback (managed on this thread, started when calls are active)
    let mut audio_capture: Option<AudioCapture> = None;
    let mut audio_playback: Option<AudioPlayback> = None;
    let mut audio_active = false;

    // Video capture channel - capture thread sends frames here
    let (video_tx, mut video_rx) = tokio::sync::mpsc::unbounded_channel::<VideoFrameData>();
    // Video capture error channel - capture thread sends errors here
    let (video_error_tx, mut video_error_rx) = tokio::sync::mpsc::unbounded_channel::<VideoCaptureError>();

    // Video capture (managed on this thread, started when video calls are active)
    let mut video_capture: Option<VideoCapture> = None;
    let mut screen_capture: Option<ScreenCapture> = None;
    let mut video_active = false;
    let mut video_capture_failed = false; // Tracks if capture failed, to avoid retry loop

    // Bootstrap to DHT nodes and add TCP relays for NAT traversal fallback
    for node in default_bootstrap_nodes() {
        // Bootstrap for DHT discovery (UDP)
        if let Err(e) = tox.bootstrap(&node.address, node.port, &node.public_key) {
            warn!("Failed to bootstrap to {}: {e}", node.address);
        }

        // Add TCP relay for each supported port - essential for NAT traversal
        // when direct UDP connection fails (common behind symmetric NATs/firewalls)
        for tcp_port in &node.tcp_ports {
            if let Err(e) = tox.add_tcp_relay(&node.address, *tcp_port, &node.public_key) {
                warn!("Failed to add TCP relay {}:{}: {e}", node.address, tcp_port);
            } else {
                debug!("Added TCP relay {}:{}", node.address, tcp_port);
            }
        }
    }

    info!("Bootstrap complete: {} nodes configured with TCP relay support",
          default_bootstrap_nodes().len());

    // I2P/Proxy verification logging
    match proxy_config.proxy_type {
        ProxyType::Socks5 => {
            info!("[I2P-CHECK] All Tox traffic routed through SOCKS5 proxy at {}:{}",
                  proxy_config.host.as_deref().unwrap_or("127.0.0.1"), proxy_config.port);
            info!("[I2P-CHECK] UDP disabled - using TCP relay mode only (required for I2P/Tor)");
        }
        ProxyType::Http => {
            info!("[I2P-CHECK] All Tox traffic routed through HTTP proxy at {}:{}",
                  proxy_config.host.as_deref().unwrap_or("127.0.0.1"), proxy_config.port);
            info!("[I2P-CHECK] UDP disabled - using TCP relay mode only");
        }
        ProxyType::None => {
            debug!("[I2P-CHECK] No proxy configured - direct connections enabled");
        }
    }

    info!("Tox thread started, address: {}", tox.self_address());

    // Sync existing friends to DB
    for friend_num in tox.friend_list() {
        let pk = tox.friend_public_key(friend_num).unwrap_or(ToxPublicKey(String::new()));
        let name = tox.friend_name(friend_num).unwrap_or_default();
        if let Err(e) = store.upsert_friend(friend_num, &pk.0, &name, "") {
            error!("Failed to sync friend {friend_num} to DB: {e}");
        }
    }

    // Log all existing guilds before sync
    if let Ok(all_guilds) = store.get_guilds() {
        info!("Guilds before sync:");
        for g in &all_guilds {
            info!("  Guild '{}' (id={}) -> group_number={:?}", g.name, g.id, g.metadata_group_number);
        }
    }

    // Log all Tox groups
    let group_count = tox.group_get_number_groups();
    let tox_groups = tox.group_list();
    info!("Tox group count: {}, groups found: {:?}", group_count, tox_groups);

    // Sync existing groups to DB - match by name and update group_number
    // Also reconnect each group to ensure it can send/receive messages
    for group_num in tox_groups {
        if let Ok(group_info) = tox.group_get_info(group_num) {
            info!("Tox group {}: name='{}' peers={}", group_num, group_info.name, group_info.peer_count);
            // Try to find existing guild by name
            match store.get_guild_by_name(&group_info.name) {
                Ok(Some(guild)) => {
                    // Update group_number if it changed
                    if guild.metadata_group_number != Some(group_num as i64) {
                        info!("Updating guild '{}' group_number: {:?} -> {}",
                              guild.name, guild.metadata_group_number, group_num);
                        if let Err(e) = store.update_guild_group_number(&guild.id, group_num as i64) {
                            error!("Failed to update guild group_number: {e}");
                        }
                    }
                }
                Ok(None) => {
                    // No guild found, create one
                    info!("Creating guild record for Tox group '{}' ({})", group_info.name, group_num);
                    let guild_id = uuid::Uuid::new_v4().to_string();
                    if let Err(e) = store.insert_guild(&guild_id, &group_info.name, Some(group_num as i64), "", "server") {
                        error!("Failed to create guild for group {}: {e}", group_num);
                    } else {
                        // Create default channel
                        let channel_id = uuid::Uuid::new_v4().to_string();
                        if let Err(e) = store.insert_channel(&channel_id, &guild_id, "general", "text", 0) {
                            error!("Failed to create default channel: {e}");
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to lookup guild by name: {e}");
                }
            }

            // Reconnect the group to ensure it can send/receive messages after restart
            if let Err(e) = tox.group_reconnect(group_num) {
                warn!("Failed to reconnect group {}: {e}", group_num);
            } else {
                info!("Reconnected group {} to DHT", group_num);
            }
        }
    }

    // Log guilds after sync
    if let Ok(all_guilds) = store.get_guilds() {
        info!("Guilds after sync:");
        for g in &all_guilds {
            info!("  Guild '{}' (id={}) -> group_number={:?}", g.name, g.id, g.metadata_group_number);
        }
        // Check for duplicate group_numbers
        let mut seen_group_nums = std::collections::HashMap::new();
        for g in &all_guilds {
            if let Some(gn) = g.metadata_group_number {
                seen_group_nums.entry(gn).or_insert_with(Vec::new).push(g.name.clone());
            }
        }
        for (gn, names) in &seen_group_nums {
            if names.len() > 1 {
                warn!("DUPLICATE group_number {}: guilds {:?} - this will cause routing issues!", gn, names);
            }
        }
    }

    // Signal that sync is complete
    if let Some(tx) = sync_complete_tx {
        let _ = tx.send(());
    }

    // Save the initial profile
    let password = password.to_string();
    let profile_path = profile_path.clone();
    save_profile(&tox, &password, &profile_path);

    // Main event loop
    loop {
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                ToxCommand::GetAddress(reply) => {
                    let _ = reply.send(tox.self_address());
                }
                ToxCommand::GetConnectionStatus(reply) => {
                    let _ = reply.send(tox.self_connection_status());
                }
                ToxCommand::GetProfileInfo(reply) => {
                    let _ = reply.send(tox.profile_info());
                }
                ToxCommand::SetName(name, reply) => {
                    let result = tox.set_name(&name).map_err(|e| e.to_string());
                    if result.is_ok() {
                        save_profile(&tox, &password, &profile_path);
                    }
                    let _ = reply.send(result);
                }
                ToxCommand::SetStatusMessage(msg, reply) => {
                    let result = tox.set_status_message(&msg).map_err(|e| e.to_string());
                    if result.is_ok() {
                        save_profile(&tox, &password, &profile_path);
                    }
                    let _ = reply.send(result);
                }
                ToxCommand::FriendAdd(address, message, reply) => {
                    let result = tox.friend_add(&address, &message).map_err(|e| e.to_string());
                    if let Ok(friend_num) = &result {
                        save_profile(&tox, &password, &profile_path);
                        // Persist new friend to DB
                        let pk = tox.friend_public_key(*friend_num).unwrap_or(ToxPublicKey(String::new()));
                        if let Err(e) = store.upsert_friend(*friend_num, &pk.0, "", "") {
                            error!("Failed to persist friend: {e}");
                        }
                    }
                    let _ = reply.send(result);
                }
                ToxCommand::FriendAccept(pk, reply) => {
                    let result = tox.friend_add_norequest(&pk).map_err(|e| e.to_string());
                    if let Ok(friend_num) = &result {
                        save_profile(&tox, &password, &profile_path);
                        let pk_hex: String = pk.iter().map(|b| format!("{b:02X}")).collect();
                        if let Err(e) = store.upsert_friend(*friend_num, &pk_hex, "", "") {
                            error!("Failed to persist accepted friend: {e}");
                        }
                    }
                    let _ = reply.send(result);
                }
                ToxCommand::FriendDelete(num, reply) => {
                    let result = tox.friend_delete(num).map_err(|e| e.to_string());
                    if result.is_ok() {
                        save_profile(&tox, &password, &profile_path);
                        if let Err(e) = store.remove_friend(num) {
                            error!("Failed to remove friend from DB: {e}");
                        }
                    }
                    let _ = reply.send(result);
                }
                ToxCommand::FriendList(reply) => {
                    let friends: Vec<FriendInfo> = tox
                        .friend_list()
                        .into_iter()
                        .map(|num| FriendInfo {
                            number: num,
                            public_key: tox.friend_public_key(num).unwrap_or(ToxPublicKey(String::new())),
                            name: tox.friend_name(num).unwrap_or_default(),
                            status_message: String::new(),
                            status: UserStatus::None,
                            connection_status: tox.friend_connection_status(num),
                        })
                        .collect();
                    let _ = reply.send(friends);
                }
                ToxCommand::FriendSendMessage(num, msg, reply) => {
                    let result = tox
                        .friend_send_message(num, MessageType::Normal, &msg)
                        .map_err(|e| e.to_string());
                    let _ = reply.send(result);
                }
                ToxCommand::SetTyping(num, typing, reply) => {
                    let result = tox.self_set_typing(num, typing).map_err(|e| e.to_string());
                    let _ = reply.send(result);
                }
                ToxCommand::GroupNew(name, reply) => {
                    let self_name = tox.self_name();
                    let result = tox
                        .group_new(GroupPrivacyState::Private, &name, &self_name)
                        .map_err(|e| e.to_string());
                    if result.is_ok() {
                        save_profile(&tox, &password, &profile_path);
                    }
                    let _ = reply.send(result);
                }
                ToxCommand::GroupJoin(chat_id, pwd, reply) => {
                    let self_name = tox.self_name();
                    let result = tox
                        .group_join(&chat_id, &self_name, &pwd)
                        .map_err(|e| e.to_string());
                    if result.is_ok() {
                        save_profile(&tox, &password, &profile_path);
                    }
                    let _ = reply.send(result);
                }
                ToxCommand::GroupLeave(group_number, reply) => {
                    let result = tox.group_leave(group_number, "").map_err(|e| e.to_string());
                    if result.is_ok() {
                        save_profile(&tox, &password, &profile_path);
                    }
                    let _ = reply.send(result);
                }
                ToxCommand::GroupInviteFriend(group_number, friend_number, reply) => {
                    let result = tox
                        .group_invite_friend(group_number, friend_number)
                        .map_err(|e| e.to_string());
                    let _ = reply.send(result);
                }
                ToxCommand::GroupInviteAccept(friend_number, invite_data, reply) => {
                    let self_name = tox.self_name();
                    let result = tox
                        .group_invite_accept(friend_number, &invite_data, &self_name, "")
                        .map_err(|e| e.to_string());
                    if result.is_ok() {
                        save_profile(&tox, &password, &profile_path);
                    }
                    let _ = reply.send(result);
                }
                ToxCommand::GroupSendMessage(group_number, msg, reply) => {
                    let result = tox
                        .group_send_message(group_number, MessageType::Normal, &msg)
                        .map_err(|e| e.to_string());
                    let _ = reply.send(result);
                }
                ToxCommand::GroupSendCustomPacket(group_number, data, reply) => {
                    let result = tox
                        .group_send_custom_packet(group_number, true, &data)
                        .map_err(|e| e.to_string());
                    let _ = reply.send(result);
                }
                ToxCommand::GroupGetList(reply) => {
                    let groups: Vec<GroupInfo> = tox
                        .group_list()
                        .into_iter()
                        .filter_map(|num| tox.group_get_info(num).ok())
                        .collect();
                    let _ = reply.send(groups);
                }
                ToxCommand::GroupGetPeerList(group_number, reply) => {
                    // Get our own peer ID to know the range; iterate from 0 up
                    let mut peers = Vec::new();
                    // c-toxcore doesn't have a direct peer list API; we iterate possible peer IDs
                    // Using self peer ID + reasonable range. A practical limit:
                    let limit = tox.group_peer_count(group_number).unwrap_or(100);
                    for peer_id in 0..limit {
                        if let Ok(info) = tox.group_get_peer_info(group_number, peer_id) {
                            if !info.public_key.is_empty() {
                                peers.push(info);
                            }
                        }
                    }
                    let _ = reply.send(peers);
                }
                ToxCommand::GroupSetTopic(group_number, topic, reply) => {
                    let result = tox
                        .group_set_topic(group_number, &topic)
                        .map_err(|e| e.to_string());
                    let _ = reply.send(result);
                }
                ToxCommand::GroupSetRole(group_number, peer_id, role, reply) => {
                    let group_role = GroupRole::from_raw(role as u32);
                    let result = tox
                        .group_set_role(group_number, peer_id, group_role)
                        .map_err(|e| e.to_string());
                    let _ = reply.send(result);
                }
                ToxCommand::GroupKickPeer(group_number, peer_id, reply) => {
                    let result = tox
                        .group_kick_peer(group_number, peer_id)
                        .map_err(|e| e.to_string());
                    let _ = reply.send(result);
                }
                ToxCommand::GroupGetInfo(group_number, reply) => {
                    let result = tox
                        .group_get_info(group_number)
                        .map_err(|e| e.to_string());
                    let _ = reply.send(result);
                }
                ToxCommand::GroupGetSelfPk(group_number, reply) => {
                    let result = tox
                        .group_self_get_public_key(group_number)
                        .map(|pk| pk.iter().map(|b| format!("{b:02X}")).collect())
                        .map_err(|e| e.to_string());
                    let _ = reply.send(result);
                }
                ToxCommand::GroupReconnect(group_number, reply) => {
                    let result = tox
                        .group_reconnect(group_number)
                        .map_err(|e| e.to_string());
                    let _ = reply.send(result);
                }
                // ToxAV commands
                ToxCommand::AvCall {
                    friend_number,
                    audio_bit_rate,
                    video_bit_rate,
                    reply,
                } => {
                    let result = if let Some(ref av) = toxav {
                        match av.call(friend_number, audio_bit_rate, video_bit_rate) {
                            Ok(()) => {
                                // Register the call with the manager
                                let with_video = video_bit_rate > 0;
                                if let Ok(mut mgr) = av_manager.lock() {
                                    mgr.start_call(friend_number, with_video);
                                }
                                Ok(())
                            }
                            Err(e) => Err(e.to_string()),
                        }
                    } else {
                        Err("ToxAV not available".to_string())
                    };
                    let _ = reply.send(result);
                }
                ToxCommand::AvAnswer {
                    friend_number,
                    audio_bit_rate,
                    video_bit_rate,
                    reply,
                } => {
                    let result = if let Some(ref av) = toxav {
                        match av.answer(friend_number, audio_bit_rate, video_bit_rate) {
                            Ok(()) => {
                                info!("Successfully answered call from friend {}", friend_number);
                                // Manually transition call to InProgress since ToxAV callback may not fire
                                if let Ok(mut mgr) = av_manager.lock() {
                                    // Create a synthetic "active" state to transition the call
                                    let active_state = toxcord_tox::CallStateFlags {
                                        error: false,
                                        finished: false,
                                        sending_audio: true,
                                        sending_video: video_bit_rate > 0,
                                        accepting_audio: true,
                                        accepting_video: video_bit_rate > 0,
                                    };
                                    mgr.update_call_state(friend_number, active_state);
                                    info!("Transitioned call with friend {} to InProgress after answer", friend_number);
                                }
                                // Emit state change to frontend
                                use crate::managers::av_manager::ToxAvEvent;
                                let event = ToxAvEvent::CallStateChange {
                                    friend_number,
                                    state: "in_progress".to_string(),
                                    sending_audio: true,
                                    sending_video: video_bit_rate > 0,
                                    accepting_audio: true,
                                    accepting_video: video_bit_rate > 0,
                                };
                                if let Err(e) = app_handle.emit("toxav://event", &event) {
                                    error!("Failed to emit call state change: {e}");
                                }
                                Ok(())
                            }
                            Err(e) => {
                                error!("Failed to answer call from friend {}: {}", friend_number, e);
                                Err(e.to_string())
                            }
                        }
                    } else {
                        Err("ToxAV not available".to_string())
                    };
                    let _ = reply.send(result);
                }
                ToxCommand::AvHangup { friend_number, reply } => {
                    let result = if let Some(ref av) = toxav {
                        match av.hangup(friend_number) {
                            Ok(()) => {
                                // Clean up the call in the manager
                                if let Ok(mut mgr) = av_manager.lock() {
                                    mgr.end_call(friend_number);
                                }
                                // Clean up mixer source
                                if let Ok(mut m) = mixer.lock() {
                                    m.remove_source(friend_number);
                                }
                                Ok(())
                            }
                            Err(e) => Err(e.to_string()),
                        }
                    } else {
                        Err("ToxAV not available".to_string())
                    };
                    let _ = reply.send(result);
                }
                ToxCommand::AvMuteAudio { friend_number, reply } => {
                    let result = if let Some(ref av) = toxav {
                        match av.mute_audio(friend_number) {
                            Ok(()) => {
                                if let Ok(mut mgr) = av_manager.lock() {
                                    mgr.set_audio_muted(friend_number, true);
                                }
                                Ok(())
                            }
                            Err(e) => Err(e.to_string()),
                        }
                    } else {
                        Err("ToxAV not available".to_string())
                    };
                    let _ = reply.send(result);
                }
                ToxCommand::AvUnmuteAudio { friend_number, reply } => {
                    let result = if let Some(ref av) = toxav {
                        match av.unmute_audio(friend_number) {
                            Ok(()) => {
                                if let Ok(mut mgr) = av_manager.lock() {
                                    mgr.set_audio_muted(friend_number, false);
                                }
                                Ok(())
                            }
                            Err(e) => Err(e.to_string()),
                        }
                    } else {
                        Err("ToxAV not available".to_string())
                    };
                    let _ = reply.send(result);
                }
                ToxCommand::AvHideVideo { friend_number, reply } => {
                    let result = if let Some(ref av) = toxav {
                        match av.hide_video(friend_number) {
                            Ok(()) => {
                                // Update av_manager state
                                if let Ok(mut mgr) = av_manager.lock() {
                                    mgr.set_video_muted(friend_number, true);
                                }
                                info!("Video hidden for friend {}", friend_number);
                                Ok(())
                            }
                            Err(e) => Err(e.to_string()),
                        }
                    } else {
                        Err("ToxAV not available".to_string())
                    };
                    let _ = reply.send(result);
                }
                ToxCommand::AvShowVideo { friend_number, reply } => {
                    let result = if let Some(ref av) = toxav {
                        match av.show_video(friend_number) {
                            Ok(()) => {
                                // Update av_manager state
                                if let Ok(mut mgr) = av_manager.lock() {
                                    mgr.set_video_muted(friend_number, false);
                                }
                                info!("Video shown for friend {}", friend_number);
                                Ok(())
                            }
                            Err(e) => Err(e.to_string()),
                        }
                    } else {
                        Err("ToxAV not available".to_string())
                    };
                    let _ = reply.send(result);
                }
                ToxCommand::AvSendAudioFrame {
                    friend_number,
                    pcm,
                    sample_count,
                    channels,
                    sampling_rate,
                } => {
                    if let Some(ref av) = toxav {
                        let frame = AudioFrame {
                            pcm,
                            sample_count,
                            channels,
                            sampling_rate,
                        };
                        if let Err(e) = av.audio_send_frame(friend_number, &frame) {
                            debug!("Failed to send audio frame: {e}");
                        }
                    }
                }
                ToxCommand::AvGetCallState { friend_number, reply } => {
                    // Get call state from the AV manager using blocking access
                    let state = if let Ok(mgr) = av_manager.lock() {
                        mgr.get_call(friend_number).cloned()
                    } else {
                        None
                    };
                    let _ = reply.send(state);
                }
                ToxCommand::SaveProfile(reply) => {
                    save_profile(&tox, &password, &profile_path);
                    let _ = reply.send(Ok(()));
                }
                ToxCommand::Shutdown(reply) => {
                    save_profile(&tox, &password, &profile_path);
                    info!("Tox thread shutting down");
                    let _ = reply.send(());
                    // Clean up handler pointers
                    unsafe {
                        let _ = Box::from_raw(handler_ptr);
                        if let Some(av_ptr) = av_handler {
                            let _ = Box::from_raw(av_ptr);
                        }
                    }
                    // ToxAV will be dropped automatically when toxav goes out of scope
                    return;
                }
            }
        }

        // Run tox_iterate with the handler as user_data
        tox.iterate_with_userdata(handler_ptr as *mut std::ffi::c_void);

        // Run toxav_iterate
        if let Some(ref av) = toxav {
            av.iterate();
        }

        // Check if we have any active calls (in_progress state) to manage audio
        let (has_active_call, call_count) = if let Ok(mgr) = av_manager.lock() {
            let calls = mgr.get_all_calls();
            let count = calls.len();
            let active = calls.iter().any(|c| c.state == CallStatus::InProgress);
            // Log when there are calls but audio hasn't started yet
            if count > 0 && !audio_active {
                for call in calls {
                    debug!("Call state check: friend={} state={:?} audio_active={}",
                           call.friend_number, call.state, audio_active);
                }
            }
            (active, count)
        } else {
            (false, 0)
        };

        // Start audio capture/playback when a call becomes active
        if has_active_call && !audio_active {
            info!("Starting audio for active call");

            // Start audio capture (microphone)
            match AudioCapture::start(audio_tx.clone()) {
                Ok(capture) => {
                    audio_capture = Some(capture);
                    info!("Audio capture started");
                }
                Err(e) => {
                    error!("Failed to start audio capture: {e}");
                }
            }

            // Start audio playback (speakers) with the shared mixer
            match AudioPlayback::start(mixer.clone()) {
                Ok(playback) => {
                    audio_playback = Some(playback);
                    info!("Audio playback started");
                }
                Err(e) => {
                    error!("Failed to start audio playback: {e}");
                }
            }

            audio_active = true;
        }

        // Stop audio when no calls are active
        if !has_active_call && audio_active {
            info!("Stopping audio - no active calls");
            audio_capture = None;
            audio_playback = None;
            if let Ok(mut m) = mixer.lock() {
                m.clear();
            }
            audio_active = false;
        }

        // Check if we have any active video calls
        let has_video_call = if let Ok(mgr) = av_manager.lock() {
            let calls = mgr.get_all_calls();
            if !calls.is_empty() && !video_active {
                // Log call states when checking for video (use info! to be visible)
                for c in &calls {
                    info!(
                        "VIDEO CHECK - friend {}: state={:?}, has_video={}, is_video_muted={}",
                        c.friend_number, c.state, c.has_video, c.is_video_muted
                    );
                }
            }
            calls.iter().any(|c| c.state == CallStatus::InProgress && c.has_video && !c.is_video_muted)
        } else {
            false
        };

        // Start video capture when a video call becomes active (and hasn't already failed)
        if has_video_call && !video_active && !video_capture_failed {
            // Check if screen sharing is active
            let (is_screen_sharing, screen_share_id) = {
                let state = app_handle.state::<AppState>();
                let sharing = state.is_screen_sharing.try_lock().ok().map(|g| *g).unwrap_or(false);
                let screen_id = state.screen_share_id.try_lock().ok().and_then(|g| *g);
                (sharing, screen_id)
            };

            if is_screen_sharing {
                // Start screen capture
                info!("Starting screen capture for active video call (screen_id: {:?})", screen_share_id);
                match ScreenCapture::start(screen_share_id, video_tx.clone(), video_error_tx.clone()) {
                    Ok(capture) => {
                        screen_capture = Some(capture);
                        video_active = true;
                        info!("Screen capture started successfully");
                    }
                    Err(e) => {
                        error!("Failed to start screen capture: {e}");
                        video_capture_failed = true;
                        let error_event = ToxAvEvent::VideoError {
                            error: e.to_string(),
                        };
                        if let Err(emit_err) = app_handle.emit("toxav://local-video", &error_event) {
                            error!("Failed to emit video error event: {emit_err}");
                        }
                    }
                }
            } else {
                // Start camera capture
                let selected_camera_index = {
                    let state = app_handle.state::<AppState>();
                    state.selected_camera_index.try_lock().ok().and_then(|guard| *guard)
                };
                info!("Starting video capture for active video call (device index: {:?})", selected_camera_index);
                match VideoCapture::start_with_device(selected_camera_index, video_tx.clone(), video_error_tx.clone()) {
                    Ok(capture) => {
                        video_capture = Some(capture);
                        video_active = true;
                        info!("Video capture started successfully");
                    }
                    Err(e) => {
                        error!("Failed to start video capture: {e}");
                        video_capture_failed = true;
                        let error_event = ToxAvEvent::VideoError {
                            error: e.to_string(),
                        };
                        if let Err(emit_err) = app_handle.emit("toxav://local-video", &error_event) {
                            error!("Failed to emit video error event: {emit_err}");
                        }
                    }
                }
            }
        }

        // Check if screen sharing state changed (to switch between camera and screen)
        if has_video_call && video_active {
            let (is_screen_sharing_now, _) = {
                let state = app_handle.state::<AppState>();
                let sharing = state.is_screen_sharing.try_lock().ok().map(|g| *g).unwrap_or(false);
                let screen_id = state.screen_share_id.try_lock().ok().and_then(|g| *g);
                (sharing, screen_id)
            };

            // Detect state change: screen_capture is Some means we're screen sharing, None means camera
            let currently_screen_sharing = screen_capture.is_some();
            if is_screen_sharing_now != currently_screen_sharing {
                info!(
                    "Screen sharing state changed: {} -> {}",
                    currently_screen_sharing, is_screen_sharing_now
                );
                // Stop current capture
                video_capture = None;
                screen_capture = None;
                video_active = false;
                // Will restart with new source on next iteration
            }
        }

        // Check for video capture errors (from capture thread)
        while let Ok(err) = video_error_rx.try_recv() {
            error!("Video capture thread error: {}", err.message);
            video_capture_failed = true;
            // Notify frontend about the video capture error
            let error_event = ToxAvEvent::VideoError {
                error: err.message,
            };
            if let Err(emit_err) = app_handle.emit("toxav://local-video", &error_event) {
                error!("Failed to emit video error event: {emit_err}");
            }
            // Stop video/screen capture since it failed
            video_capture = None;
            screen_capture = None;
            video_active = false;
        }

        // Stop video capture when no video calls are active
        if !has_video_call && video_active {
            info!("Stopping video/screen capture - no active video calls");
            video_capture = None;
            screen_capture = None;
            video_active = false;
        }

        // Reset video_capture_failed when video call ends so it can retry on next call
        if !has_video_call && video_capture_failed {
            video_capture_failed = false;
        }

        // Send captured audio frames to all active calls
        if let Some(ref av) = toxav {
            let mut frame_count = 0;
            while let Ok(pcm) = audio_rx.try_recv() {
                frame_count += 1;
                // Get list of friends we're in active calls with
                let active_friends: Vec<u32> = if let Ok(mgr) = av_manager.lock() {
                    mgr.get_all_calls()
                        .iter()
                        .filter(|c| c.state == CallStatus::InProgress && !c.is_audio_muted)
                        .map(|c| c.friend_number)
                        .collect()
                } else {
                    vec![]
                };

                if active_friends.is_empty() && frame_count == 1 {
                    debug!("Captured audio but no active friends to send to");
                }

                // Send audio to each active call
                for friend_number in active_friends {
                    let frame = AudioFrame {
                        pcm: pcm.clone(),
                        sample_count: pcm.len(),
                        channels: 1,
                        sampling_rate: 48000,
                    };
                    match av.audio_send_frame(friend_number, &frame) {
                        Ok(()) => {
                            debug!("Sent {} samples to friend {}", pcm.len(), friend_number);
                        }
                        Err(e) => {
                            debug!("Failed to send audio frame to friend {}: {e}", friend_number);
                        }
                    }
                }
            }
        }

        // Send captured video frames to all active video calls
        if let Some(ref av) = toxav {
            let mut video_frame_count = 0;
            while let Ok(frame) = video_rx.try_recv() {
                video_frame_count += 1;
                if video_frame_count <= 3 {
                    info!("Video frame {}: {}x{}, Y={} U={} V={} bytes",
                           video_frame_count, frame.width, frame.height,
                           frame.y.len(), frame.u.len(), frame.v.len());
                }

                // Get list of friends we're in active video calls with
                let active_video_friends: Vec<u32> = if let Ok(mgr) = av_manager.lock() {
                    mgr.get_all_calls()
                        .iter()
                        .filter(|c| c.state == CallStatus::InProgress && c.has_video && !c.is_video_muted)
                        .map(|c| c.friend_number)
                        .collect()
                } else {
                    vec![]
                };

                // Send video to each active video call
                for friend_number in &active_video_friends {
                    let tox_frame = VideoFrame::new(
                        frame.y.clone(),
                        frame.u.clone(),
                        frame.v.clone(),
                        frame.width,
                        frame.height,
                    );
                    if let Err(e) = tox_frame.validate() {
                        debug!("Invalid video frame: {e}");
                        continue;
                    }
                    if let Err(e) = av.video_send_frame(*friend_number, &tox_frame) {
                        debug!("Failed to send video frame to friend {}: {e}", friend_number);
                    }
                }

                // Emit local preview to frontend (combine YUV into single buffer)
                let mut data = Vec::with_capacity(frame.y.len() + frame.u.len() + frame.v.len());
                data.extend_from_slice(&frame.y);
                data.extend_from_slice(&frame.u);
                data.extend_from_slice(&frame.v);

                let event = ToxAvEvent::VideoFrame {
                    friend_number: 0, // 0 indicates local preview
                    width: frame.width,
                    height: frame.height,
                    data,
                };
                if let Err(e) = app_handle.emit("toxav://local-video", &event) {
                    debug!("Failed to emit local video frame: {e}");
                }
            }
        }

        // Process offline queue flush requests
        while let Ok(friend_number) = offline_flush_rx.try_recv() {
            let queued = store.get_offline_messages_for("friend", &friend_number.to_string());
            if let Ok(messages) = queued {
                for (queue_id, _msg_type, content) in messages {
                    let chunks = toxcord_protocol::codec::split_friend_message(&content);
                    let mut all_sent = true;
                    for chunk in &chunks {
                        if tox.friend_send_message(friend_number, MessageType::Normal, chunk).is_err() {
                            all_sent = false;
                            break;
                        }
                    }
                    if all_sent {
                        if let Err(e) = store.remove_offline_message(queue_id) {
                            error!("Failed to remove offline message {queue_id}: {e}");
                        } else {
                            info!("Flushed offline message {queue_id} to friend {friend_number}");
                        }
                    }
                }
            }
        }

        // Sleep for the recommended interval
        let interval = tox.iteration_interval();
        std::thread::sleep(interval);
    }
}

/// Save the Tox profile to disk (encrypted)
fn save_profile(tox: &ToxInstance, password: &str, path: &PathBuf) {
    let savedata = tox.savedata();

    let data = if !password.is_empty() {
        match encrypt_savedata(&savedata, password) {
            Ok(encrypted) => encrypted,
            Err(e) => {
                error!("Failed to encrypt profile: {e}");
                savedata
            }
        }
    } else {
        savedata
    };

    if let Err(e) = std::fs::write(path, &data) {
        error!("Failed to save profile to {}: {e}", path.display());
    } else {
        debug!("Profile saved to {}", path.display());
    }
}

/// Get the profiles directory
fn get_profiles_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("toxcord")
        .join("profiles")
}
