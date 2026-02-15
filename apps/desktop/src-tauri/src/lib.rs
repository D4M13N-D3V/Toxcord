mod audio;
mod commands;
mod db;
mod managers;
mod video;

use std::sync::Arc;
use tokio::sync::Mutex;

use db::MessageStore;
use managers::tox_manager::ToxManager;

/// Global application state shared across Tauri commands
pub struct AppState {
    pub tox_manager: Mutex<Option<Arc<Mutex<ToxManager>>>>,
    pub message_store: Mutex<Option<Arc<MessageStore>>>,
    /// Selected audio input device index (None = default)
    pub selected_mic_index: Mutex<Option<u32>>,
    /// Selected audio output device index (None = default)
    pub selected_speaker_index: Mutex<Option<u32>>,
    /// Selected video device index (None = default)
    pub selected_camera_index: Mutex<Option<u32>>,
    /// Whether screen sharing is active (replaces camera)
    pub is_screen_sharing: Mutex<bool>,
    /// Selected screen ID for sharing (None = primary)
    pub screen_share_id: Mutex<Option<u32>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "toxcord=debug,toxcord_tox=debug".into()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            tox_manager: Mutex::new(None),
            message_store: Mutex::new(None),
            selected_mic_index: Mutex::new(None),
            selected_speaker_index: Mutex::new(None),
            selected_camera_index: Mutex::new(None),
            is_screen_sharing: Mutex::new(false),
            screen_share_id: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth::list_profiles,
            commands::auth::create_profile,
            commands::auth::load_profile,
            commands::auth::delete_profile,
            commands::auth::get_tox_id,
            commands::auth::get_connection_status,
            commands::auth::get_profile_info,
            commands::auth::logout,
            commands::auth::set_display_name,
            commands::auth::set_status_message,
            commands::friends::add_friend,
            commands::friends::accept_friend_request,
            commands::friends::deny_friend_request,
            commands::friends::remove_friend,
            commands::friends::get_friends,
            commands::friends::get_friend_requests,
            commands::messaging::send_direct_message,
            commands::messaging::get_direct_messages,
            commands::messaging::set_typing,
            commands::messaging::mark_messages_read,
            commands::guilds::create_guild,
            commands::guilds::get_guilds,
            commands::guilds::get_guild_channels,
            commands::guilds::create_channel,
            commands::guilds::delete_channel,
            commands::guilds::send_channel_message,
            commands::guilds::get_channel_messages,
            commands::guilds::invite_to_guild,
            commands::guilds::accept_guild_invite,
            commands::guilds::get_guild_members,
            commands::guilds::set_channel_topic,
            commands::guilds::kick_member,
            commands::guilds::set_member_role,
            commands::guilds::rename_guild,
            commands::guilds::rename_channel,
            commands::guilds::leave_guild,
            commands::guilds::create_dm_group,
            commands::guilds::send_dm_group_message,
            commands::guilds::get_dm_groups,
            // Call commands
            commands::calls::call_friend,
            commands::calls::answer_call,
            commands::calls::hangup_call,
            commands::calls::toggle_mute,
            commands::calls::toggle_video,
            commands::calls::get_call_state,
            commands::calls::list_audio_input_devices,
            commands::calls::list_audio_output_devices,
            commands::calls::list_video_devices,
            commands::calls::set_audio_input_device,
            commands::calls::set_audio_output_device,
            commands::calls::set_video_device,
            commands::calls::check_camera_status,
            commands::calls::load_camera_driver,
            // Screen sharing
            commands::calls::list_screens,
            commands::calls::start_screen_share,
            commands::calls::stop_screen_share,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
