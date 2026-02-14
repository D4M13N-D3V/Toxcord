use serde::{Deserialize, Serialize};

/// TOX address (38 bytes = 76 hex chars)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToxAddress(pub String);

impl ToxAddress {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ToxAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// TOX public key (32 bytes = 64 hex chars)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToxPublicKey(pub String);

/// Connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    None,
    Tcp,
    Udp,
}

impl ConnectionStatus {
    pub fn is_connected(&self) -> bool {
        !matches!(self, ConnectionStatus::None)
    }
}

/// User status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    None,
    Away,
    Busy,
}

/// Message type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    Normal,
    Action,
}

/// Friend information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendInfo {
    pub number: u32,
    pub public_key: ToxPublicKey,
    pub name: String,
    pub status_message: String,
    pub status: UserStatus,
    pub connection_status: ConnectionStatus,
}

/// Profile info for the local user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileInfo {
    pub tox_id: ToxAddress,
    pub name: String,
    pub status_message: String,
    pub status: UserStatus,
}

/// Group role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupRole {
    Founder,
    Moderator,
    User,
    Observer,
}

impl GroupRole {
    pub fn from_raw(raw: u32) -> Self {
        match raw {
            0 => GroupRole::Founder,
            1 => GroupRole::Moderator,
            2 => GroupRole::User,
            _ => GroupRole::Observer,
        }
    }

    pub fn to_raw(self) -> u32 {
        match self {
            GroupRole::Founder => 0,
            GroupRole::Moderator => 1,
            GroupRole::User => 2,
            GroupRole::Observer => 3,
        }
    }
}

/// Group privacy state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupPrivacyState {
    Public,
    Private,
}

/// Group information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    pub number: u32,
    pub chat_id: String,
    pub name: String,
    pub topic: String,
    pub privacy_state: GroupPrivacyState,
    pub peer_count: u32,
}

/// Group peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupPeerInfo {
    pub peer_id: u32,
    pub name: String,
    pub public_key: String,
    pub role: GroupRole,
    pub status: UserStatus,
}

/// Bootstrap node
#[derive(Debug, Clone)]
pub struct BootstrapNode {
    pub address: String,
    pub port: u16,
    pub public_key: String,
}
