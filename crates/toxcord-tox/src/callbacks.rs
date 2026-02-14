use crate::types::{ConnectionStatus, MessageType, UserStatus};

/// Trait for handling TOX events. Implement this to receive callbacks.
pub trait ToxEventHandler: Send + 'static {
    fn on_self_connection_status(&self, status: ConnectionStatus);
    fn on_friend_request(&self, public_key: &[u8; 32], message: &str);
    fn on_friend_message(&self, friend_number: u32, message_type: MessageType, message: &str);
    fn on_friend_name(&self, friend_number: u32, name: &str);
    fn on_friend_status_message(&self, friend_number: u32, message: &str);
    fn on_friend_status(&self, friend_number: u32, status: UserStatus);
    fn on_friend_connection_status(&self, friend_number: u32, status: ConnectionStatus);
    fn on_friend_typing(&self, friend_number: u32, is_typing: bool);
    fn on_friend_read_receipt(&self, friend_number: u32, message_id: u32);
    fn on_file_recv_control(&self, friend_number: u32, file_number: u32, control: u32);
    fn on_file_chunk_request(&self, friend_number: u32, file_number: u32, position: u64, length: usize);
    fn on_file_recv(&self, friend_number: u32, file_number: u32, kind: u32, file_size: u64, filename: &str);
    fn on_file_recv_chunk(&self, friend_number: u32, file_number: u32, position: u64, data: &[u8]);
    fn on_group_invite(&self, friend_number: u32, invite_data: &[u8], group_name: &str);
    fn on_group_peer_join(&self, group_number: u32, peer_id: u32);
    fn on_group_peer_exit(&self, group_number: u32, peer_id: u32, exit_type: u32, name: &str, message: &str);
    fn on_group_peer_name(&self, group_number: u32, peer_id: u32, name: &str);
    fn on_group_message(&self, group_number: u32, peer_id: u32, message_type: MessageType, message: &str, message_id: u32);
    fn on_group_custom_packet(&self, group_number: u32, peer_id: u32, data: &[u8]);
    fn on_group_custom_private_packet(&self, group_number: u32, peer_id: u32, data: &[u8]);
    fn on_group_self_join(&self, group_number: u32);
    fn on_group_join_fail(&self, group_number: u32, fail_type: u32);
    fn on_group_topic(&self, group_number: u32, peer_id: u32, topic: &str);
    fn on_group_peer_status(&self, group_number: u32, peer_id: u32, status: UserStatus);
}

/// Convert raw C connection status to our enum
pub fn connection_status_from_raw(raw: u32) -> ConnectionStatus {
    match raw {
        1 => ConnectionStatus::Tcp,
        2 => ConnectionStatus::Udp,
        _ => ConnectionStatus::None,
    }
}

/// Convert raw C user status to our enum
pub fn user_status_from_raw(raw: u32) -> UserStatus {
    match raw {
        1 => UserStatus::Away,
        2 => UserStatus::Busy,
        _ => UserStatus::None,
    }
}

/// Convert raw C message type to our enum
pub fn message_type_from_raw(raw: u32) -> MessageType {
    match raw {
        1 => MessageType::Action,
        _ => MessageType::Normal,
    }
}

// ─── extern "C" callback trampolines ───────────────────────────────────────

/// The user_data pointer passed to all callbacks is a raw pointer to a
/// `Box<dyn ToxEventHandler>`. These trampolines extract it and dispatch.

macro_rules! extract_handler {
    ($user_data:expr) => {{
        let handler = &*($user_data as *const Box<dyn ToxEventHandler>);
        handler.as_ref()
    }};
}

pub unsafe extern "C" fn self_connection_status_cb(
    _tox: *mut toxcord_tox_sys::Tox,
    connection_status: toxcord_tox_sys::Tox_Connection,
    user_data: *mut std::ffi::c_void,
) {
    let handler = extract_handler!(user_data);
    handler.on_self_connection_status(connection_status_from_raw(connection_status as u32));
}

pub unsafe extern "C" fn friend_request_cb(
    _tox: *mut toxcord_tox_sys::Tox,
    public_key: *const u8,
    message: *const u8,
    length: usize,
    user_data: *mut std::ffi::c_void,
) {
    let handler = extract_handler!(user_data);
    let pk = &*(public_key as *const [u8; 32]);
    let msg = std::str::from_utf8(std::slice::from_raw_parts(message, length)).unwrap_or("");
    handler.on_friend_request(pk, msg);
}

pub unsafe extern "C" fn friend_message_cb(
    _tox: *mut toxcord_tox_sys::Tox,
    friend_number: u32,
    message_type: toxcord_tox_sys::Tox_Message_Type,
    message: *const u8,
    length: usize,
    user_data: *mut std::ffi::c_void,
) {
    let handler = extract_handler!(user_data);
    let msg = std::str::from_utf8(std::slice::from_raw_parts(message, length)).unwrap_or("");
    handler.on_friend_message(friend_number, message_type_from_raw(message_type as u32), msg);
}

pub unsafe extern "C" fn friend_name_cb(
    _tox: *mut toxcord_tox_sys::Tox,
    friend_number: u32,
    name: *const u8,
    length: usize,
    user_data: *mut std::ffi::c_void,
) {
    let handler = extract_handler!(user_data);
    let n = std::str::from_utf8(std::slice::from_raw_parts(name, length)).unwrap_or("");
    handler.on_friend_name(friend_number, n);
}

pub unsafe extern "C" fn friend_status_message_cb(
    _tox: *mut toxcord_tox_sys::Tox,
    friend_number: u32,
    message: *const u8,
    length: usize,
    user_data: *mut std::ffi::c_void,
) {
    let handler = extract_handler!(user_data);
    let msg = std::str::from_utf8(std::slice::from_raw_parts(message, length)).unwrap_or("");
    handler.on_friend_status_message(friend_number, msg);
}

pub unsafe extern "C" fn friend_status_cb(
    _tox: *mut toxcord_tox_sys::Tox,
    friend_number: u32,
    status: toxcord_tox_sys::Tox_User_Status,
    user_data: *mut std::ffi::c_void,
) {
    let handler = extract_handler!(user_data);
    handler.on_friend_status(friend_number, user_status_from_raw(status as u32));
}

pub unsafe extern "C" fn friend_connection_status_cb(
    _tox: *mut toxcord_tox_sys::Tox,
    friend_number: u32,
    connection_status: toxcord_tox_sys::Tox_Connection,
    user_data: *mut std::ffi::c_void,
) {
    let handler = extract_handler!(user_data);
    handler.on_friend_connection_status(
        friend_number,
        connection_status_from_raw(connection_status as u32),
    );
}

pub unsafe extern "C" fn friend_typing_cb(
    _tox: *mut toxcord_tox_sys::Tox,
    friend_number: u32,
    is_typing: bool,
    user_data: *mut std::ffi::c_void,
) {
    let handler = extract_handler!(user_data);
    handler.on_friend_typing(friend_number, is_typing);
}

pub unsafe extern "C" fn friend_read_receipt_cb(
    _tox: *mut toxcord_tox_sys::Tox,
    friend_number: u32,
    message_id: u32,
    user_data: *mut std::ffi::c_void,
) {
    let handler = extract_handler!(user_data);
    handler.on_friend_read_receipt(friend_number, message_id);
}

pub unsafe extern "C" fn file_recv_control_cb(
    _tox: *mut toxcord_tox_sys::Tox,
    friend_number: u32,
    file_number: u32,
    control: toxcord_tox_sys::Tox_File_Control,
    user_data: *mut std::ffi::c_void,
) {
    let handler = extract_handler!(user_data);
    handler.on_file_recv_control(friend_number, file_number, control as u32);
}

pub unsafe extern "C" fn file_chunk_request_cb(
    _tox: *mut toxcord_tox_sys::Tox,
    friend_number: u32,
    file_number: u32,
    position: u64,
    length: usize,
    user_data: *mut std::ffi::c_void,
) {
    let handler = extract_handler!(user_data);
    handler.on_file_chunk_request(friend_number, file_number, position, length);
}

pub unsafe extern "C" fn file_recv_cb(
    _tox: *mut toxcord_tox_sys::Tox,
    friend_number: u32,
    file_number: u32,
    kind: u32,
    file_size: u64,
    filename: *const u8,
    filename_length: usize,
    user_data: *mut std::ffi::c_void,
) {
    let handler = extract_handler!(user_data);
    let name =
        std::str::from_utf8(std::slice::from_raw_parts(filename, filename_length)).unwrap_or("");
    handler.on_file_recv(friend_number, file_number, kind, file_size, name);
}

pub unsafe extern "C" fn file_recv_chunk_cb(
    _tox: *mut toxcord_tox_sys::Tox,
    friend_number: u32,
    file_number: u32,
    position: u64,
    data: *const u8,
    length: usize,
    user_data: *mut std::ffi::c_void,
) {
    let handler = extract_handler!(user_data);
    let d = if length > 0 {
        std::slice::from_raw_parts(data, length)
    } else {
        &[]
    };
    handler.on_file_recv_chunk(friend_number, file_number, position, d);
}

pub unsafe extern "C" fn group_invite_cb(
    _tox: *mut toxcord_tox_sys::Tox,
    friend_number: u32,
    invite_data: *const u8,
    length: usize,
    group_name: *const u8,
    group_name_length: usize,
    user_data: *mut std::ffi::c_void,
) {
    let handler = extract_handler!(user_data);
    let data = std::slice::from_raw_parts(invite_data, length);
    let name =
        std::str::from_utf8(std::slice::from_raw_parts(group_name, group_name_length))
            .unwrap_or("");
    handler.on_group_invite(friend_number, data, name);
}

pub unsafe extern "C" fn group_peer_join_cb(
    _tox: *mut toxcord_tox_sys::Tox,
    group_number: u32,
    peer_id: u32,
    user_data: *mut std::ffi::c_void,
) {
    let handler = extract_handler!(user_data);
    handler.on_group_peer_join(group_number, peer_id);
}

pub unsafe extern "C" fn group_peer_exit_cb(
    _tox: *mut toxcord_tox_sys::Tox,
    group_number: u32,
    peer_id: u32,
    exit_type: toxcord_tox_sys::Tox_Group_Exit_Type,
    name: *const u8,
    name_length: usize,
    message: *const u8,
    message_length: usize,
    user_data: *mut std::ffi::c_void,
) {
    let handler = extract_handler!(user_data);
    let n = std::str::from_utf8(std::slice::from_raw_parts(name, name_length)).unwrap_or("");
    let msg =
        std::str::from_utf8(std::slice::from_raw_parts(message, message_length)).unwrap_or("");
    handler.on_group_peer_exit(group_number, peer_id, exit_type as u32, n, msg);
}

pub unsafe extern "C" fn group_peer_name_cb(
    _tox: *mut toxcord_tox_sys::Tox,
    group_number: u32,
    peer_id: u32,
    name: *const u8,
    length: usize,
    user_data: *mut std::ffi::c_void,
) {
    let handler = extract_handler!(user_data);
    let n = std::str::from_utf8(std::slice::from_raw_parts(name, length)).unwrap_or("");
    handler.on_group_peer_name(group_number, peer_id, n);
}

pub unsafe extern "C" fn group_message_cb(
    _tox: *mut toxcord_tox_sys::Tox,
    group_number: u32,
    peer_id: u32,
    message_type: toxcord_tox_sys::Tox_Message_Type,
    message: *const u8,
    length: usize,
    message_id: u32,
    user_data: *mut std::ffi::c_void,
) {
    let handler = extract_handler!(user_data);
    let msg = std::str::from_utf8(std::slice::from_raw_parts(message, length)).unwrap_or("");
    handler.on_group_message(
        group_number,
        peer_id,
        message_type_from_raw(message_type as u32),
        msg,
        message_id,
    );
}

pub unsafe extern "C" fn group_custom_packet_cb(
    _tox: *mut toxcord_tox_sys::Tox,
    group_number: u32,
    peer_id: u32,
    data: *const u8,
    length: usize,
    user_data: *mut std::ffi::c_void,
) {
    let handler = extract_handler!(user_data);
    let d = std::slice::from_raw_parts(data, length);
    handler.on_group_custom_packet(group_number, peer_id, d);
}

pub unsafe extern "C" fn group_custom_private_packet_cb(
    _tox: *mut toxcord_tox_sys::Tox,
    group_number: u32,
    peer_id: u32,
    data: *const u8,
    length: usize,
    user_data: *mut std::ffi::c_void,
) {
    let handler = extract_handler!(user_data);
    let d = std::slice::from_raw_parts(data, length);
    handler.on_group_custom_private_packet(group_number, peer_id, d);
}

pub unsafe extern "C" fn group_self_join_cb(
    _tox: *mut toxcord_tox_sys::Tox,
    group_number: u32,
    user_data: *mut std::ffi::c_void,
) {
    let handler = extract_handler!(user_data);
    handler.on_group_self_join(group_number);
}

pub unsafe extern "C" fn group_join_fail_cb(
    _tox: *mut toxcord_tox_sys::Tox,
    group_number: u32,
    fail_type: toxcord_tox_sys::Tox_Group_Join_Fail,
    user_data: *mut std::ffi::c_void,
) {
    let handler = extract_handler!(user_data);
    handler.on_group_join_fail(group_number, fail_type as u32);
}

pub unsafe extern "C" fn group_topic_cb(
    _tox: *mut toxcord_tox_sys::Tox,
    group_number: u32,
    peer_id: u32,
    topic: *const u8,
    length: usize,
    user_data: *mut std::ffi::c_void,
) {
    let handler = extract_handler!(user_data);
    let t = std::str::from_utf8(std::slice::from_raw_parts(topic, length)).unwrap_or("");
    handler.on_group_topic(group_number, peer_id, t);
}

pub unsafe extern "C" fn group_peer_status_cb(
    _tox: *mut toxcord_tox_sys::Tox,
    group_number: u32,
    peer_id: u32,
    status: toxcord_tox_sys::Tox_User_Status,
    user_data: *mut std::ffi::c_void,
) {
    let handler = extract_handler!(user_data);
    handler.on_group_peer_status(group_number, peer_id, user_status_from_raw(status as u32));
}
