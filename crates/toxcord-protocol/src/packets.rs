use serde::{Deserialize, Serialize};

/// Custom protocol packet types sent over NGC custom lossless packets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PacketType {
    /// Automerge sync message for guild state
    GuildMetaSync = 0x01,
    /// Request full metadata sync from peers
    GuildMetaRequest = 0x02,

    /// Add/remove emoji reaction
    MessageReaction = 0x10,
    /// Edit message content
    MessageEdit = 0x11,
    /// Delete message
    MessageDelete = 0x12,
    /// Pin/unpin message
    MessagePin = 0x13,
    /// Create thread from message
    ThreadCreate = 0x14,
    /// Message within a thread
    ThreadMessage = 0x15,

    /// Typing indicator start
    TypingStart = 0x20,
    /// Typing indicator stop
    TypingStop = 0x21,

    /// Announce joining voice channel
    VoiceJoin = 0x30,
    /// Announce leaving voice channel
    VoiceLeave = 0x31,
    /// Mute/deafen state update
    VoiceState = 0x32,

    /// Broadcast invite availability
    InviteCreate = 0x40,
    /// Request invite to guild
    InviteRequest = 0x41,

    /// Custom status/activity update
    PresenceUpdate = 0x50,
}

impl PacketType {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(Self::GuildMetaSync),
            0x02 => Some(Self::GuildMetaRequest),
            0x10 => Some(Self::MessageReaction),
            0x11 => Some(Self::MessageEdit),
            0x12 => Some(Self::MessageDelete),
            0x13 => Some(Self::MessagePin),
            0x14 => Some(Self::ThreadCreate),
            0x15 => Some(Self::ThreadMessage),
            0x20 => Some(Self::TypingStart),
            0x21 => Some(Self::TypingStop),
            0x30 => Some(Self::VoiceJoin),
            0x31 => Some(Self::VoiceLeave),
            0x32 => Some(Self::VoiceState),
            0x40 => Some(Self::InviteCreate),
            0x41 => Some(Self::InviteRequest),
            0x50 => Some(Self::PresenceUpdate),
            _ => None,
        }
    }
}

/// A reaction on a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReactionPayload {
    pub message_id: String,
    pub emoji: String,
    pub add: bool,
}

/// An edit to a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEditPayload {
    pub message_id: String,
    pub new_content: String,
}

/// A message deletion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeletePayload {
    pub message_id: String,
}

/// Pin/unpin a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePinPayload {
    pub message_id: String,
    pub pinned: bool,
}

/// Voice state update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceStatePayload {
    pub muted: bool,
    pub deafened: bool,
    pub video_enabled: bool,
    pub screen_sharing: bool,
}

/// Typing indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingPayload {
    pub channel_id: String,
}

/// Presence/status update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUpdatePayload {
    pub status: String,
    pub custom_status: Option<String>,
}
