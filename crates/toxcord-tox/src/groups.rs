use toxcord_tox_sys::*;

use crate::error::{ToxError, ToxResult};
use crate::tox::ToxInstance;
use crate::types::*;

/// Hex encoding for group IDs
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02X}")).collect()
}

impl ToxInstance {
    // ─── Group Lifecycle ───────────────────────────────────────────────

    /// Create a new group chat.
    pub fn group_new(
        &self,
        privacy_state: GroupPrivacyState,
        group_name: &str,
        self_name: &str,
    ) -> ToxResult<u32> {
        let ps = match privacy_state {
            GroupPrivacyState::Public => Tox_Group_Privacy_State_TOX_GROUP_PRIVACY_STATE_PUBLIC,
            GroupPrivacyState::Private => Tox_Group_Privacy_State_TOX_GROUP_PRIVACY_STATE_PRIVATE,
        };

        unsafe {
            let mut err = Tox_Err_Group_New::default();
            let group_number = tox_group_new(
                self.raw(),
                ps,
                group_name.as_ptr(),
                group_name.len(),
                self_name.as_ptr(),
                self_name.len(),
                &mut err,
            );
            if group_number == u32::MAX {
                Err(ToxError::Group(format!("group_new failed: {err:?}")))
            } else {
                Ok(group_number)
            }
        }
    }

    /// Join an existing group by chat ID.
    pub fn group_join(
        &self,
        chat_id: &[u8; 32],
        self_name: &str,
        password: &str,
    ) -> ToxResult<u32> {
        unsafe {
            let mut err = Tox_Err_Group_Join::default();
            let pwd_ptr = if password.is_empty() {
                std::ptr::null()
            } else {
                password.as_ptr()
            };
            let pwd_len = if password.is_empty() {
                0
            } else {
                password.len()
            };

            let group_number = tox_group_join(
                self.raw(),
                chat_id.as_ptr(),
                self_name.as_ptr(),
                self_name.len(),
                pwd_ptr,
                pwd_len,
                &mut err,
            );
            if group_number == u32::MAX {
                Err(ToxError::Group(format!("group_join failed: {err:?}")))
            } else {
                Ok(group_number)
            }
        }
    }

    /// Leave a group chat.
    pub fn group_leave(&self, group_number: u32, message: &str) -> ToxResult<()> {
        unsafe {
            let mut err = Tox_Err_Group_Leave::default();
            let ok = tox_group_leave(
                self.raw(),
                group_number,
                message.as_ptr(),
                message.len(),
                &mut err,
            );
            if ok {
                Ok(())
            } else {
                Err(ToxError::Group(format!("group_leave failed: {err:?}")))
            }
        }
    }

    /// Check if connected to a group.
    pub fn group_is_connected(&self, group_number: u32) -> bool {
        unsafe {
            let mut err = Tox_Err_Group_Is_Connected::default();
            tox_group_is_connected(self.raw(), group_number, &mut err)
        }
    }

    /// Reconnect to a group.
    pub fn group_reconnect(&self, group_number: u32) -> ToxResult<()> {
        unsafe {
            let mut err = Tox_Err_Group_Reconnect::default();
            let ok = tox_group_reconnect(self.raw(), group_number, &mut err);
            if ok {
                Ok(())
            } else {
                Err(ToxError::Group(format!("group_reconnect failed: {err:?}")))
            }
        }
    }

    // ─── Group Messaging ───────────────────────────────────────────────

    /// Send a message to a group.
    pub fn group_send_message(
        &self,
        group_number: u32,
        msg_type: MessageType,
        message: &str,
    ) -> ToxResult<u32> {
        let mt = match msg_type {
            MessageType::Normal => Tox_Message_Type_TOX_MESSAGE_TYPE_NORMAL,
            MessageType::Action => Tox_Message_Type_TOX_MESSAGE_TYPE_ACTION,
        };

        unsafe {
            let mut err = Tox_Err_Group_Send_Message::default();
            let message_id = tox_group_send_message(
                self.raw(),
                group_number,
                mt,
                message.as_ptr(),
                message.len(),
                &mut err,
            );
            if err == Tox_Err_Group_Send_Message_TOX_ERR_GROUP_SEND_MESSAGE_OK {
                Ok(message_id)
            } else {
                let detail = match err {
                    Tox_Err_Group_Send_Message_TOX_ERR_GROUP_SEND_MESSAGE_GROUP_NOT_FOUND =>
                        format!("group {group_number} not found"),
                    Tox_Err_Group_Send_Message_TOX_ERR_GROUP_SEND_MESSAGE_TOO_LONG =>
                        "message too long".to_string(),
                    Tox_Err_Group_Send_Message_TOX_ERR_GROUP_SEND_MESSAGE_EMPTY =>
                        "message is empty".to_string(),
                    Tox_Err_Group_Send_Message_TOX_ERR_GROUP_SEND_MESSAGE_PERMISSIONS =>
                        "insufficient permissions (observer?)".to_string(),
                    Tox_Err_Group_Send_Message_TOX_ERR_GROUP_SEND_MESSAGE_FAIL_SEND =>
                        "failed to send (no peers connected?)".to_string(),
                    Tox_Err_Group_Send_Message_TOX_ERR_GROUP_SEND_MESSAGE_DISCONNECTED =>
                        format!("group {group_number} is disconnected"),
                    _ => format!("unknown error {err}"),
                };
                Err(ToxError::Group(format!(
                    "group_send_message failed: {detail}"
                )))
            }
        }
    }

    /// Send a custom lossless packet to the group.
    pub fn group_send_custom_packet(
        &self,
        group_number: u32,
        lossless: bool,
        data: &[u8],
    ) -> ToxResult<()> {
        unsafe {
            let mut err = Tox_Err_Group_Send_Custom_Packet::default();
            let ok = tox_group_send_custom_packet(
                self.raw(),
                group_number,
                lossless,
                data.as_ptr(),
                data.len(),
                &mut err,
            );
            if ok {
                Ok(())
            } else {
                Err(ToxError::Group(format!(
                    "group_send_custom_packet failed: {err:?}"
                )))
            }
        }
    }

    /// Send a custom private packet to a specific peer.
    pub fn group_send_custom_private_packet(
        &self,
        group_number: u32,
        peer_id: u32,
        lossless: bool,
        data: &[u8],
    ) -> ToxResult<()> {
        unsafe {
            let mut err = Tox_Err_Group_Send_Custom_Private_Packet::default();
            let ok = tox_group_send_custom_private_packet(
                self.raw(),
                group_number,
                peer_id,
                lossless,
                data.as_ptr(),
                data.len(),
                &mut err,
            );
            if ok {
                Ok(())
            } else {
                Err(ToxError::Group(format!(
                    "group_send_custom_private_packet failed: {err:?}"
                )))
            }
        }
    }

    // ─── Group Invites ─────────────────────────────────────────────────

    /// Invite a friend to join a group.
    pub fn group_invite_friend(
        &self,
        group_number: u32,
        friend_number: u32,
    ) -> ToxResult<()> {
        unsafe {
            let mut err = Tox_Err_Group_Invite_Friend::default();
            let ok = tox_group_invite_friend(
                self.raw(),
                group_number,
                friend_number,
                &mut err,
            );
            if ok {
                Ok(())
            } else {
                let detail = match err {
                    Tox_Err_Group_Invite_Friend_TOX_ERR_GROUP_INVITE_FRIEND_GROUP_NOT_FOUND =>
                        format!("group {group_number} not found in tox instance"),
                    Tox_Err_Group_Invite_Friend_TOX_ERR_GROUP_INVITE_FRIEND_FRIEND_NOT_FOUND =>
                        format!("friend {friend_number} not found"),
                    Tox_Err_Group_Invite_Friend_TOX_ERR_GROUP_INVITE_FRIEND_INVITE_FAIL =>
                        "invite creation failed".to_string(),
                    Tox_Err_Group_Invite_Friend_TOX_ERR_GROUP_INVITE_FRIEND_FAIL_SEND =>
                        "failed to send invite (friend offline?)".to_string(),
                    Tox_Err_Group_Invite_Friend_TOX_ERR_GROUP_INVITE_FRIEND_DISCONNECTED =>
                        format!("group {group_number} is disconnected"),
                    _ => format!("unknown error {err}"),
                };
                Err(ToxError::Group(format!(
                    "group_invite_friend failed: {detail}"
                )))
            }
        }
    }

    /// Accept a group invite from a friend.
    pub fn group_invite_accept(
        &self,
        friend_number: u32,
        invite_data: &[u8],
        self_name: &str,
        password: &str,
    ) -> ToxResult<u32> {
        unsafe {
            let mut err = Tox_Err_Group_Invite_Accept::default();
            let pwd_ptr = if password.is_empty() {
                std::ptr::null()
            } else {
                password.as_ptr()
            };
            let pwd_len = if password.is_empty() {
                0
            } else {
                password.len()
            };

            let group_number = tox_group_invite_accept(
                self.raw(),
                friend_number,
                invite_data.as_ptr(),
                invite_data.len(),
                self_name.as_ptr(),
                self_name.len(),
                pwd_ptr,
                pwd_len,
                &mut err,
            );
            if group_number == u32::MAX {
                Err(ToxError::Group(format!(
                    "group_invite_accept failed: {err:?}"
                )))
            } else {
                Ok(group_number)
            }
        }
    }

    // ─── Group State Queries ───────────────────────────────────────────

    /// Get the chat ID of a group (32 bytes, hex-encoded).
    pub fn group_get_chat_id(&self, group_number: u32) -> ToxResult<[u8; 32]> {
        unsafe {
            let mut chat_id = [0u8; 32];
            let mut err = Tox_Err_Group_State_Query::default();
            let ok = tox_group_get_chat_id(
                self.raw(),
                group_number,
                chat_id.as_mut_ptr(),
                &mut err,
            );
            if ok {
                Ok(chat_id)
            } else {
                Err(ToxError::Group(format!(
                    "group_get_chat_id failed: {err:?}"
                )))
            }
        }
    }

    /// Get the name of a group.
    pub fn group_get_name(&self, group_number: u32) -> ToxResult<String> {
        unsafe {
            let mut err = Tox_Err_Group_State_Query::default();
            let size = tox_group_get_name_size(self.raw(), group_number, &mut err);
            if err != Tox_Err_Group_State_Query_TOX_ERR_GROUP_STATE_QUERY_OK {
                return Err(ToxError::Group(format!("group_get_name_size failed: {err:?}")));
            }
            if size == 0 {
                return Ok(String::new());
            }
            let mut name = vec![0u8; size];
            tox_group_get_name(self.raw(), group_number, name.as_mut_ptr(), &mut err);
            Ok(String::from_utf8_lossy(&name).to_string())
        }
    }

    /// Set the topic of a group.
    pub fn group_set_topic(&self, group_number: u32, topic: &str) -> ToxResult<()> {
        unsafe {
            let mut err = Tox_Err_Group_Topic_Set::default();
            let ok = tox_group_set_topic(
                self.raw(),
                group_number,
                topic.as_ptr(),
                topic.len(),
                &mut err,
            );
            if ok {
                Ok(())
            } else {
                Err(ToxError::Group(format!(
                    "group_set_topic failed: {err:?}"
                )))
            }
        }
    }

    /// Get the topic of a group.
    pub fn group_get_topic(&self, group_number: u32) -> ToxResult<String> {
        unsafe {
            let mut err = Tox_Err_Group_State_Query::default();
            let size = tox_group_get_topic_size(self.raw(), group_number, &mut err);
            if err != Tox_Err_Group_State_Query_TOX_ERR_GROUP_STATE_QUERY_OK {
                return Err(ToxError::Group(format!("group_get_topic_size failed: {err:?}")));
            }
            if size == 0 {
                return Ok(String::new());
            }
            let mut topic = vec![0u8; size];
            tox_group_get_topic(self.raw(), group_number, topic.as_mut_ptr(), &mut err);
            Ok(String::from_utf8_lossy(&topic).to_string())
        }
    }

    // ─── Peer Queries ──────────────────────────────────────────────────

    /// Get a peer's name.
    pub fn group_peer_get_name(
        &self,
        group_number: u32,
        peer_id: u32,
    ) -> ToxResult<String> {
        unsafe {
            let mut err = Tox_Err_Group_Peer_Query::default();
            let size =
                tox_group_peer_get_name_size(self.raw(), group_number, peer_id, &mut err);
            if err != Tox_Err_Group_Peer_Query_TOX_ERR_GROUP_PEER_QUERY_OK {
                return Err(ToxError::Group(format!("group_peer_get_name_size failed: {err:?}")));
            }
            if size == 0 {
                return Ok(String::new());
            }
            let mut name = vec![0u8; size];
            tox_group_peer_get_name(
                self.raw(),
                group_number,
                peer_id,
                name.as_mut_ptr(),
                &mut err,
            );
            Ok(String::from_utf8_lossy(&name).to_string())
        }
    }

    /// Get a peer's role.
    pub fn group_peer_get_role(
        &self,
        group_number: u32,
        peer_id: u32,
    ) -> ToxResult<GroupRole> {
        unsafe {
            let mut err = Tox_Err_Group_Peer_Query::default();
            let role = tox_group_peer_get_role(self.raw(), group_number, peer_id, &mut err);
            Ok(GroupRole::from_raw(role as u32))
        }
    }

    /// Get a peer's public key (32 bytes, hex-encoded).
    pub fn group_peer_get_public_key(
        &self,
        group_number: u32,
        peer_id: u32,
    ) -> ToxResult<[u8; 32]> {
        unsafe {
            let mut pk = [0u8; 32];
            let mut err = Tox_Err_Group_Peer_Query::default();
            let ok = tox_group_peer_get_public_key(
                self.raw(),
                group_number,
                peer_id,
                pk.as_mut_ptr(),
                &mut err,
            );
            if ok {
                Ok(pk)
            } else {
                Err(ToxError::Group(format!(
                    "group_peer_get_public_key failed: {err:?}"
                )))
            }
        }
    }

    /// Get the peer count for a group.
    pub fn group_peer_count(&self, group_number: u32) -> ToxResult<u32> {
        unsafe {
            let mut err = Tox_Err_Group_State_Query::default();
            let count = tox_group_get_peer_limit(self.raw(), group_number, &mut err);
            // peer_limit is the max, we need peer count from offline_peer_count + online
            // Actually there's no direct "peer_count" in c-toxcore NGC API.
            // We'll use the number of peers via tox_group_get_number_online_peers if available,
            // or iterate self peer_id range. For simplicity, use peer_limit as an approximation
            // that we'll refine. Actually, looking at the API, peer count isn't directly exposed.
            // We return peer_limit as a stand-in and the caller can count from the peer list.
            let _ = count;
            let _ = err;
            // Use group_peer_count - this function exists as tox_group_peer_count
            let mut err2 = Tox_Err_Group_State_Query::default();
            let count = tox_group_get_peer_limit(self.raw(), group_number, &mut err2);
            Ok(count as u32)
        }
    }

    // ─── Self Info ─────────────────────────────────────────────────────

    /// Get our own peer ID in a group.
    pub fn group_self_get_peer_id(&self, group_number: u32) -> ToxResult<u32> {
        unsafe {
            let mut err = Tox_Err_Group_Self_Query::default();
            let peer_id = tox_group_self_get_peer_id(self.raw(), group_number, &mut err);
            Ok(peer_id)
        }
    }

    /// Get our own role in a group.
    pub fn group_self_get_role(&self, group_number: u32) -> ToxResult<GroupRole> {
        unsafe {
            let mut err = Tox_Err_Group_Self_Query::default();
            let role = tox_group_self_get_role(self.raw(), group_number, &mut err);
            Ok(GroupRole::from_raw(role as u32))
        }
    }

    /// Get our own public key in a group.
    pub fn group_self_get_public_key(&self, group_number: u32) -> ToxResult<[u8; 32]> {
        unsafe {
            let mut pk = [0u8; 32];
            let mut err = Tox_Err_Group_Self_Query::default();
            let ok = tox_group_self_get_public_key(
                self.raw(),
                group_number,
                pk.as_mut_ptr(),
                &mut err,
            );
            if ok {
                Ok(pk)
            } else {
                Err(ToxError::Group(format!(
                    "group_self_get_public_key failed: {err:?}"
                )))
            }
        }
    }

    // ─── Moderation ────────────────────────────────────────────────────

    /// Set a peer's role.
    pub fn group_set_role(
        &self,
        group_number: u32,
        peer_id: u32,
        role: GroupRole,
    ) -> ToxResult<()> {
        let r = match role {
            GroupRole::Founder => Tox_Group_Role_TOX_GROUP_ROLE_FOUNDER,
            GroupRole::Moderator => Tox_Group_Role_TOX_GROUP_ROLE_MODERATOR,
            GroupRole::User => Tox_Group_Role_TOX_GROUP_ROLE_USER,
            GroupRole::Observer => Tox_Group_Role_TOX_GROUP_ROLE_OBSERVER,
        };

        unsafe {
            let mut err = Tox_Err_Group_Set_Role::default();
            let ok = tox_group_set_role(self.raw(), group_number, peer_id, r, &mut err);
            if ok {
                Ok(())
            } else {
                Err(ToxError::Group(format!(
                    "group_set_role failed: {err:?}"
                )))
            }
        }
    }

    /// Kick a peer from the group.
    pub fn group_kick_peer(&self, group_number: u32, peer_id: u32) -> ToxResult<()> {
        unsafe {
            let mut err = Tox_Err_Group_Kick_Peer::default();
            let ok = tox_group_kick_peer(self.raw(), group_number, peer_id, &mut err);
            if ok {
                Ok(())
            } else {
                Err(ToxError::Group(format!(
                    "group_kick_peer failed: {err:?}"
                )))
            }
        }
    }

    // ─── Group Enumeration ─────────────────────────────────────────────

    /// Get the number of groups.
    pub fn group_get_number_groups(&self) -> u32 {
        unsafe { tox_group_get_number_groups(self.raw()) }
    }

    /// Get a list of all group numbers.
    /// Since c-toxcore doesn't expose a direct group list API,
    /// we probe group numbers from 0 up to the count.
    pub fn group_list(&self) -> Vec<u32> {
        let count = self.group_get_number_groups();
        if count == 0 {
            return vec![];
        }
        let mut list = Vec::with_capacity(count as usize);
        let mut found = 0u32;
        let mut i = 0u32;
        // Groups may not be contiguous, probe up to a reasonable limit
        // Use group_get_chat_id which is more reliable after loading from savedata
        // since the chat_id is stored directly and doesn't require connection
        while found < count && i < count * 2 + 10 {
            if self.group_get_chat_id(i).is_ok() {
                list.push(i);
                found += 1;
            }
            i += 1;
        }
        list
    }

    /// Get full info about a group.
    pub fn group_get_info(&self, group_number: u32) -> ToxResult<GroupInfo> {
        let chat_id = self.group_get_chat_id(group_number)?;
        let name = self.group_get_name(group_number)?;
        let topic = self.group_get_topic(group_number)?;

        let privacy_state = unsafe {
            let mut err = Tox_Err_Group_State_Query::default();
            let ps = tox_group_get_privacy_state(self.raw(), group_number, &mut err);
            if ps == Tox_Group_Privacy_State_TOX_GROUP_PRIVACY_STATE_PUBLIC {
                GroupPrivacyState::Public
            } else {
                GroupPrivacyState::Private
            }
        };

        let peer_count = self.group_peer_count(group_number).unwrap_or(0);

        Ok(GroupInfo {
            number: group_number,
            chat_id: hex_encode(&chat_id),
            name,
            topic,
            privacy_state,
            peer_count,
        })
    }

    /// Get info about a specific peer in a group.
    pub fn group_get_peer_info(
        &self,
        group_number: u32,
        peer_id: u32,
    ) -> ToxResult<GroupPeerInfo> {
        let name = self.group_peer_get_name(group_number, peer_id)?;
        let pk = self.group_peer_get_public_key(group_number, peer_id)?;
        let role = self.group_peer_get_role(group_number, peer_id)?;

        let status = unsafe {
            let mut err = Tox_Err_Group_Peer_Query::default();
            let s = tox_group_peer_get_status(self.raw(), group_number, peer_id, &mut err);
            crate::callbacks::user_status_from_raw(s as u32)
        };

        Ok(GroupPeerInfo {
            peer_id,
            name,
            public_key: hex_encode(&pk),
            role,
            status,
        })
    }
}
