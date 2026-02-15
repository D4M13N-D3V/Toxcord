use std::ffi::CString;
use std::ptr;
use std::time::Duration;

use toxcord_tox_sys::*;
use tracing::{debug, info};

use crate::callbacks::*;
use crate::error::{ToxError, ToxResult};
use crate::types::*;

/// Proxy type for Tox connections
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProxyType {
    /// No proxy
    #[default]
    None,
    /// HTTP proxy
    Http,
    /// SOCKS5 proxy (recommended for I2P/Tor)
    Socks5,
}

/// Safe wrapper around a c-toxcore Tox instance.
///
/// SAFETY: ToxInstance is NOT Send/Sync — it must live on a single dedicated thread.
/// All cross-thread access goes through the ToxManager command channel.
pub struct ToxInstance {
    tox: *mut Tox,
    /// Prevent Send/Sync
    _marker: std::marker::PhantomData<*mut ()>,
}

/// Builder for ToxOptions
pub struct ToxOptionsBuilder {
    savedata: Option<Vec<u8>>,
    ipv6_enabled: bool,
    udp_enabled: bool,
    local_discovery_enabled: bool,
    hole_punching_enabled: bool,
    proxy_type: ProxyType,
    proxy_host: Option<String>,
    proxy_port: u16,
    start_port: u16,
    end_port: u16,
}

impl Default for ToxOptionsBuilder {
    fn default() -> Self {
        Self {
            savedata: None,
            ipv6_enabled: true,
            udp_enabled: true,
            local_discovery_enabled: true,
            hole_punching_enabled: true,
            proxy_type: ProxyType::None,
            proxy_host: None,
            proxy_port: 0,
            start_port: 0,
            end_port: 0,
        }
    }
}

impl ToxOptionsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn savedata(mut self, data: Vec<u8>) -> Self {
        self.savedata = Some(data);
        self
    }

    pub fn ipv6_enabled(mut self, enabled: bool) -> Self {
        self.ipv6_enabled = enabled;
        self
    }

    pub fn udp_enabled(mut self, enabled: bool) -> Self {
        self.udp_enabled = enabled;
        self
    }

    pub fn local_discovery_enabled(mut self, enabled: bool) -> Self {
        self.local_discovery_enabled = enabled;
        self
    }

    /// Configure a SOCKS5 proxy (recommended for I2P/Tor)
    ///
    /// When a proxy is set, UDP is automatically disabled since proxies
    /// don't support UDP. Tox will use TCP relay mode instead.
    pub fn proxy_socks5(mut self, host: &str, port: u16) -> Self {
        self.proxy_type = ProxyType::Socks5;
        self.proxy_host = Some(host.to_string());
        self.proxy_port = port;
        self
    }

    /// Configure an HTTP proxy
    ///
    /// When a proxy is set, UDP is automatically disabled since proxies
    /// don't support UDP. Tox will use TCP relay mode instead.
    pub fn proxy_http(mut self, host: &str, port: u16) -> Self {
        self.proxy_type = ProxyType::Http;
        self.proxy_host = Some(host.to_string());
        self.proxy_port = port;
        self
    }

    /// Disable proxy (default)
    pub fn no_proxy(mut self) -> Self {
        self.proxy_type = ProxyType::None;
        self.proxy_host = None;
        self.proxy_port = 0;
        self
    }

    pub fn build(self) -> ToxResult<ToxInstance> {
        ToxInstance::new(self)
    }
}

impl ToxInstance {
    /// Create a new Tox instance with the given options.
    pub fn new(options: ToxOptionsBuilder) -> ToxResult<Self> {
        unsafe {
            let mut err = Tox_Err_Options_New::default();
            let opts = tox_options_new(&mut err);
            if opts.is_null() {
                return Err(ToxError::New(format!("tox_options_new failed: {err:?}")));
            }

            tox_options_set_ipv6_enabled(opts, options.ipv6_enabled);
            tox_options_set_local_discovery_enabled(opts, options.local_discovery_enabled);
            tox_options_set_hole_punching_enabled(opts, options.hole_punching_enabled);
            tox_options_set_start_port(opts, options.start_port);
            tox_options_set_end_port(opts, options.end_port);

            // Configure proxy settings
            let proxy_type_raw = match options.proxy_type {
                ProxyType::None => Tox_Proxy_Type_TOX_PROXY_TYPE_NONE,
                ProxyType::Http => Tox_Proxy_Type_TOX_PROXY_TYPE_HTTP,
                ProxyType::Socks5 => Tox_Proxy_Type_TOX_PROXY_TYPE_SOCKS5,
            };
            tox_options_set_proxy_type(opts, proxy_type_raw);

            if let Some(ref host) = options.proxy_host {
                let c_host = CString::new(host.as_str())
                    .map_err(|e| ToxError::New(format!("Invalid proxy host: {e}")))?;
                tox_options_set_proxy_host(opts, c_host.as_ptr());
                tox_options_set_proxy_port(opts, options.proxy_port);
                info!("Proxy configured: {:?} {}:{}", options.proxy_type, host, options.proxy_port);
            }

            // UDP must be disabled when using a proxy (proxies only support TCP)
            let udp_enabled = if options.proxy_type != ProxyType::None {
                debug!("UDP disabled due to proxy configuration");
                false
            } else {
                options.udp_enabled
            };
            tox_options_set_udp_enabled(opts, udp_enabled);

            if let Some(ref savedata) = options.savedata {
                tox_options_set_savedata_type(
                    opts,
                    Tox_Savedata_Type_TOX_SAVEDATA_TYPE_TOX_SAVE,
                );
                tox_options_set_savedata_data(opts, savedata.as_ptr(), savedata.len());
            }

            let mut new_err = Tox_Err_New::default();
            let tox = tox_new(opts, &mut new_err);
            tox_options_free(opts);

            if tox.is_null() {
                return Err(ToxError::New(format!("tox_new failed: {new_err:?}")));
            }

            info!("Tox instance created successfully");
            Ok(Self {
                tox,
                _marker: std::marker::PhantomData,
            })
        }
    }

    /// Create from existing savedata bytes
    pub fn from_savedata(savedata: Vec<u8>) -> ToxResult<Self> {
        ToxOptionsBuilder::new().savedata(savedata).build()
    }

    /// Get the raw pointer (for FFI calls within the same thread)
    pub fn raw(&self) -> *mut Tox {
        self.tox
    }

    /// Get the savedata for this instance
    pub fn savedata(&self) -> Vec<u8> {
        unsafe {
            let size = tox_get_savedata_size(self.tox);
            let mut data = vec![0u8; size];
            tox_get_savedata(self.tox, data.as_mut_ptr());
            data
        }
    }

    /// Get the Tox address (76 hex chars)
    pub fn self_address(&self) -> ToxAddress {
        unsafe {
            let mut address = [0u8; TOX_ADDRESS_SIZE as usize];
            tox_self_get_address(self.tox, address.as_mut_ptr());
            ToxAddress(hex::encode(address))
        }
    }

    /// Get the Tox public key (64 hex chars)
    pub fn self_public_key(&self) -> ToxPublicKey {
        unsafe {
            let mut pk = [0u8; TOX_PUBLIC_KEY_SIZE as usize];
            tox_self_get_public_key(self.tox, pk.as_mut_ptr());
            ToxPublicKey(hex::encode(pk))
        }
    }

    /// Get current connection status
    pub fn self_connection_status(&self) -> ConnectionStatus {
        unsafe {
            let status = tox_self_get_connection_status(self.tox);
            connection_status_from_raw(status as u32)
        }
    }

    /// Set the user's display name
    pub fn set_name(&self, name: &str) -> ToxResult<()> {
        unsafe {
            let mut err = Tox_Err_Set_Info::default();
            let ok = tox_self_set_name(self.tox, name.as_ptr(), name.len(), &mut err);
            if ok {
                Ok(())
            } else {
                Err(ToxError::SetName(format!("{err:?}")))
            }
        }
    }

    /// Get the user's display name
    pub fn self_name(&self) -> String {
        unsafe {
            let size = tox_self_get_name_size(self.tox);
            let mut name = vec![0u8; size];
            tox_self_get_name(self.tox, name.as_mut_ptr());
            String::from_utf8_lossy(&name).to_string()
        }
    }

    /// Set the user's status message
    pub fn set_status_message(&self, message: &str) -> ToxResult<()> {
        unsafe {
            let mut err = Tox_Err_Set_Info::default();
            let ok = tox_self_set_status_message(
                self.tox,
                message.as_ptr(),
                message.len(),
                &mut err,
            );
            if ok {
                Ok(())
            } else {
                Err(ToxError::SetStatusMessage(format!("{err:?}")))
            }
        }
    }

    /// Bootstrap to a DHT node
    pub fn bootstrap(&self, address: &str, port: u16, public_key_hex: &str) -> ToxResult<()> {
        let pk_bytes = hex_to_bytes(public_key_hex)
            .ok_or_else(|| ToxError::Bootstrap("Invalid public key hex".into()))?;

        let c_address =
            CString::new(address).map_err(|e| ToxError::Bootstrap(format!("Invalid address: {e}")))?;

        unsafe {
            let mut err = Tox_Err_Bootstrap::default();
            let ok = tox_bootstrap(
                self.tox,
                c_address.as_ptr(),
                port,
                pk_bytes.as_ptr(),
                &mut err,
            );
            if ok {
                debug!("Bootstrapped to {address}:{port}");
                Ok(())
            } else {
                Err(ToxError::Bootstrap(format!(
                    "tox_bootstrap to {address}:{port} failed: {err:?}"
                )))
            }
        }
    }

    /// Add a TCP relay node for NAT traversal fallback.
    ///
    /// TCP relays are essential for communication when direct UDP connection fails
    /// (common behind symmetric NATs and strict firewalls). Call this in addition
    /// to bootstrap() for nodes that support TCP.
    pub fn add_tcp_relay(&self, address: &str, port: u16, public_key_hex: &str) -> ToxResult<()> {
        let pk_bytes = hex_to_bytes(public_key_hex)
            .ok_or_else(|| ToxError::Bootstrap("Invalid public key hex".into()))?;

        let c_address =
            CString::new(address).map_err(|e| ToxError::Bootstrap(format!("Invalid address: {e}")))?;

        unsafe {
            let mut err = Tox_Err_Bootstrap::default();
            let ok = tox_add_tcp_relay(
                self.tox,
                c_address.as_ptr(),
                port,
                pk_bytes.as_ptr(),
                &mut err,
            );
            if ok {
                debug!("Added TCP relay {address}:{port}");
                Ok(())
            } else {
                Err(ToxError::Bootstrap(format!(
                    "tox_add_tcp_relay to {address}:{port} failed: {err:?}"
                )))
            }
        }
    }

    /// Run one iteration of the tox event loop
    pub fn iterate(&self) {
        unsafe {
            tox_iterate(self.tox, ptr::null_mut());
        }
    }

    /// Run one iteration with a user_data pointer for callbacks
    pub fn iterate_with_userdata(&self, user_data: *mut std::ffi::c_void) {
        unsafe {
            tox_iterate(self.tox, user_data);
        }
    }

    /// Get the recommended iteration interval
    pub fn iteration_interval(&self) -> Duration {
        unsafe {
            let ms = tox_iteration_interval(self.tox);
            Duration::from_millis(ms as u64)
        }
    }

    /// Register all callbacks using the trampoline functions
    pub fn register_callbacks(&self) {
        unsafe {
            tox_callback_self_connection_status(self.tox, Some(self_connection_status_cb));
            tox_callback_friend_request(self.tox, Some(friend_request_cb));
            tox_callback_friend_message(self.tox, Some(friend_message_cb));
            tox_callback_friend_name(self.tox, Some(friend_name_cb));
            tox_callback_friend_status_message(self.tox, Some(friend_status_message_cb));
            tox_callback_friend_status(self.tox, Some(friend_status_cb));
            tox_callback_friend_connection_status(self.tox, Some(friend_connection_status_cb));
            tox_callback_friend_typing(self.tox, Some(friend_typing_cb));
            tox_callback_friend_read_receipt(self.tox, Some(friend_read_receipt_cb));
            tox_callback_file_recv_control(self.tox, Some(file_recv_control_cb));
            tox_callback_file_chunk_request(self.tox, Some(file_chunk_request_cb));
            tox_callback_file_recv(self.tox, Some(file_recv_cb));
            tox_callback_file_recv_chunk(self.tox, Some(file_recv_chunk_cb));
            tox_callback_group_invite(self.tox, Some(group_invite_cb));
            tox_callback_group_peer_join(self.tox, Some(group_peer_join_cb));
            tox_callback_group_peer_exit(self.tox, Some(group_peer_exit_cb));
            tox_callback_group_peer_name(self.tox, Some(group_peer_name_cb));
            tox_callback_group_message(self.tox, Some(group_message_cb));
            tox_callback_group_custom_packet(self.tox, Some(group_custom_packet_cb));
            tox_callback_group_custom_private_packet(self.tox, Some(group_custom_private_packet_cb));
            tox_callback_group_self_join(self.tox, Some(group_self_join_cb));
            tox_callback_group_join_fail(self.tox, Some(group_join_fail_cb));
            tox_callback_group_topic(self.tox, Some(group_topic_cb));
            tox_callback_group_peer_status(self.tox, Some(group_peer_status_cb));
        }
    }

    /// Add a friend by Tox address
    pub fn friend_add(&self, address_hex: &str, message: &str) -> ToxResult<u32> {
        let addr_bytes = hex_to_bytes(address_hex)
            .ok_or_else(|| ToxError::FriendAdd("Invalid address hex".into()))?;

        unsafe {
            let mut err = Tox_Err_Friend_Add::default();
            let friend_number = tox_friend_add(
                self.tox,
                addr_bytes.as_ptr(),
                message.as_ptr(),
                message.len(),
                &mut err,
            );
            if friend_number == u32::MAX {
                Err(ToxError::FriendAdd(format!("{err:?}")))
            } else {
                Ok(friend_number)
            }
        }
    }

    /// Accept a friend request by public key
    pub fn friend_add_norequest(&self, public_key: &[u8; 32]) -> ToxResult<u32> {
        unsafe {
            let mut err = Tox_Err_Friend_Add::default();
            let friend_number =
                tox_friend_add_norequest(self.tox, public_key.as_ptr(), &mut err);
            if friend_number == u32::MAX {
                Err(ToxError::FriendAdd(format!("{err:?}")))
            } else {
                Ok(friend_number)
            }
        }
    }

    /// Send a message to a friend
    pub fn friend_send_message(
        &self,
        friend_number: u32,
        message_type: MessageType,
        message: &str,
    ) -> ToxResult<u32> {
        let msg_type = match message_type {
            MessageType::Normal => Tox_Message_Type_TOX_MESSAGE_TYPE_NORMAL,
            MessageType::Action => Tox_Message_Type_TOX_MESSAGE_TYPE_ACTION,
        };

        unsafe {
            let mut err = Tox_Err_Friend_Send_Message::default();
            let message_id = tox_friend_send_message(
                self.tox,
                friend_number,
                msg_type,
                message.as_ptr(),
                message.len(),
                &mut err,
            );
            if message_id == u32::MAX {
                Err(ToxError::SendMessage(format!("{err:?}")))
            } else {
                Ok(message_id)
            }
        }
    }

    /// Get friend's name
    pub fn friend_name(&self, friend_number: u32) -> Option<String> {
        unsafe {
            let mut err = Tox_Err_Friend_Query::default();
            let size = tox_friend_get_name_size(self.tox, friend_number, &mut err);
            if size == 0 {
                return None;
            }
            let mut name = vec![0u8; size];
            tox_friend_get_name(self.tox, friend_number, name.as_mut_ptr(), &mut err);
            Some(String::from_utf8_lossy(&name).to_string())
        }
    }

    /// Get friend's public key
    pub fn friend_public_key(&self, friend_number: u32) -> Option<ToxPublicKey> {
        unsafe {
            let mut pk = [0u8; TOX_PUBLIC_KEY_SIZE as usize];
            let mut err = Tox_Err_Friend_Get_Public_Key::default();
            let ok = tox_friend_get_public_key(self.tox, friend_number, pk.as_mut_ptr(), &mut err);
            if ok {
                Some(ToxPublicKey(hex::encode(pk)))
            } else {
                None
            }
        }
    }

    /// Get the list of friend numbers
    pub fn friend_list(&self) -> Vec<u32> {
        unsafe {
            let count = tox_self_get_friend_list_size(self.tox);
            let mut list = vec![0u32; count];
            tox_self_get_friend_list(self.tox, list.as_mut_ptr());
            list
        }
    }

    /// Delete a friend
    pub fn friend_delete(&self, friend_number: u32) -> ToxResult<()> {
        unsafe {
            let mut err = Tox_Err_Friend_Delete::default();
            let ok = tox_friend_delete(self.tox, friend_number, &mut err);
            if ok {
                Ok(())
            } else {
                Err(ToxError::FriendAdd(format!("Delete failed: {err:?}")))
            }
        }
    }

    /// Get friend connection status
    pub fn friend_connection_status(&self, friend_number: u32) -> ConnectionStatus {
        unsafe {
            let mut err = Tox_Err_Friend_Query::default();
            let status = tox_friend_get_connection_status(self.tox, friend_number, &mut err);
            connection_status_from_raw(status as u32)
        }
    }

    /// Set typing status for a friend
    pub fn self_set_typing(&self, friend_number: u32, typing: bool) -> ToxResult<()> {
        unsafe {
            let mut err = Tox_Err_Set_Typing::default();
            let ok = tox_self_set_typing(self.tox, friend_number, typing, &mut err);
            if ok {
                Ok(())
            } else {
                Err(ToxError::SendMessage(format!(
                    "set_typing failed: {err:?}"
                )))
            }
        }
    }

    /// Get profile info
    pub fn profile_info(&self) -> ProfileInfo {
        ProfileInfo {
            tox_id: self.self_address(),
            name: self.self_name(),
            status_message: self.self_status_message(),
            status: self.self_status(),
        }
    }

    /// Get self status message
    pub fn self_status_message(&self) -> String {
        unsafe {
            let size = tox_self_get_status_message_size(self.tox);
            let mut msg = vec![0u8; size];
            tox_self_get_status_message(self.tox, msg.as_mut_ptr());
            String::from_utf8_lossy(&msg).to_string()
        }
    }

    /// Get self status
    pub fn self_status(&self) -> UserStatus {
        unsafe {
            let status = tox_self_get_status(self.tox);
            crate::callbacks::user_status_from_raw(status as u32)
        }
    }
}

impl Drop for ToxInstance {
    fn drop(&mut self) {
        if !self.tox.is_null() {
            unsafe {
                tox_kill(self.tox);
            }
            info!("Tox instance destroyed");
        }
    }
}

/// Encrypt savedata with a passphrase using tox_pass_encrypt
pub fn encrypt_savedata(data: &[u8], passphrase: &str) -> ToxResult<Vec<u8>> {
    unsafe {
        let out_len = data.len() + TOX_PASS_ENCRYPTION_EXTRA_LENGTH as usize;
        let mut out = vec![0u8; out_len];
        let mut err = Tox_Err_Encryption::default();
        let ok = tox_pass_encrypt(
            data.as_ptr(),
            data.len(),
            passphrase.as_ptr(),
            passphrase.len(),
            out.as_mut_ptr(),
            &mut err,
        );
        if ok {
            Ok(out)
        } else {
            Err(ToxError::Encryption(format!("{err:?}")))
        }
    }
}

/// Decrypt savedata with a passphrase using tox_pass_decrypt
pub fn decrypt_savedata(data: &[u8], passphrase: &str) -> ToxResult<Vec<u8>> {
    unsafe {
        let out_len = data.len() - TOX_PASS_ENCRYPTION_EXTRA_LENGTH as usize;
        let mut out = vec![0u8; out_len];
        let mut err = Tox_Err_Decryption::default();
        let ok = tox_pass_decrypt(
            data.as_ptr(),
            data.len(),
            passphrase.as_ptr(),
            passphrase.len(),
            out.as_mut_ptr(),
            &mut err,
        );
        if ok {
            Ok(out)
        } else {
            Err(ToxError::Decryption(format!("{err:?}")))
        }
    }
}

/// Check if data is encrypted
pub fn is_data_encrypted(data: &[u8]) -> bool {
    unsafe { tox_is_data_encrypted(data.as_ptr()) }
}

/// Convert hex string to bytes
fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

/// Default bootstrap nodes from nodes.tox.chat
/// Updated with active nodes that support both UDP bootstrap and TCP relay
pub fn default_bootstrap_nodes() -> Vec<BootstrapNode> {
    vec![
        // Canada - highly reliable, multiple TCP ports
        BootstrapNode {
            address: "144.217.167.73".into(),
            port: 33445,
            public_key: "7E5668E0EE09E19F320AD47902419331FFEE147BB3606769CFBE921A2A2FD34C".into(),
            tcp_ports: vec![33445, 3389],
        },
        // US - multiple TCP ports including common firewall-friendly ones (port 53/443)
        BootstrapNode {
            address: "205.185.115.131".into(),
            port: 53,
            public_key: "3091C6BEB2A993F1C6300C16549FABA67098FF3D62C6D253828B531470B53D68".into(),
            tcp_ports: vec![53, 443, 33445, 3389],
        },
        // Germany - mf-net.eu node
        BootstrapNode {
            address: "tox1.mf-net.eu".into(),
            port: 33445,
            public_key: "B3E5FA80DC8EBD1149AD2AB35ED8B85BD546DEDE261CA593234C619249419506".into(),
            tcp_ports: vec![33445, 3389],
        },
        // Russia
        BootstrapNode {
            address: "188.225.9.167".into(),
            port: 33445,
            public_key: "1911341A83E02503AB1FD6561BD64AF3A9D6C3F12B5FBB656976B2E678644A67".into(),
            tcp_ports: vec![33445, 3389],
        },
        // Singapore - AWS
        BootstrapNode {
            address: "3.0.24.15".into(),
            port: 33445,
            public_key: "E20ABCF38CDBFFD7D04B29C956B33F7B27A3BB7AF0618101617B036E4AEA402D".into(),
            tcp_ports: vec![33445],
        },
        // US - additional node
        BootstrapNode {
            address: "104.225.141.59".into(),
            port: 43334,
            public_key: "933BA20B2E258B4C0D475B6DECE90C7E827FE83EFA9655414E7841251B19A72C".into(),
            tcp_ports: vec![33445, 3389],
        },
        // Singapore - Linode
        BootstrapNode {
            address: "139.162.110.188".into(),
            port: 33445,
            public_key: "F76A11284547163889DDC89A7738CF271797BF5E5E220643E97AD3C7E7903D55".into(),
            tcp_ports: vec![33445, 3389, 443],
        },
        // Germany - mf-net.eu secondary
        BootstrapNode {
            address: "tox2.mf-net.eu".into(),
            port: 33445,
            public_key: "70EA214FDE161E7432530605213F18F7427DC773E276B3E317A07531F548545F".into(),
            tcp_ports: vec![33445, 3389],
        },
        // Canada - abilinski (original)
        BootstrapNode {
            address: "tox.abilinski.com".into(),
            port: 33445,
            public_key: "10C00EB250C3233E343E2AEBA07115A5C28920E9C8D29492F6D00B29049EDC7E".into(),
            tcp_ports: vec![], // No TCP relay support
        },
        // Netherlands
        BootstrapNode {
            address: "tox.kurnevsky.net".into(),
            port: 33445,
            public_key: "82EF82BA33445A1F91A7DB27189ECFC0C013E06E3DA71F588ED692BED625EC23".into(),
            tcp_ports: vec![], // No TCP relay support
        },
    ]
}

/// Hex encoding utilities
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect()
    }
}
