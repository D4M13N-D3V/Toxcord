use std::sync::Arc;

use tauri::State;
use tokio::sync::oneshot;

use crate::db::MessageStore;
use crate::managers::tox_manager::{ToxCommand, ToxManager};
use crate::AppState;

/// Get the database directory for a profile
fn get_db_path(profile_name: &str) -> std::path::PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("toxcord")
        .join("profiles")
        .join(format!("{profile_name}.db"))
}

#[tauri::command]
pub async fn list_profiles() -> Result<Vec<String>, String> {
    Ok(ToxManager::list_profiles())
}

#[tauri::command]
pub async fn delete_profile(
    state: State<'_, AppState>,
    profile_name: String,
) -> Result<(), String> {
    // Make sure we're not deleting a currently loaded profile
    {
        let guard = state.tox_manager.lock().await;
        if guard.is_some() {
            return Err("Cannot delete profile while logged in. Please logout first.".to_string());
        }
    }

    // Delete the .tox profile file
    let profile_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("toxcord")
        .join("profiles");

    let tox_path = profile_dir.join(format!("{profile_name}.tox"));
    let db_path = profile_dir.join(format!("{profile_name}.db"));

    // Check if profile exists
    if !tox_path.exists() {
        return Err(format!("Profile '{profile_name}' not found"));
    }

    // Delete the .tox file
    if let Err(e) = std::fs::remove_file(&tox_path) {
        return Err(format!("Failed to delete profile: {e}"));
    }

    // Delete the database file if it exists
    if db_path.exists() {
        if let Err(e) = std::fs::remove_file(&db_path) {
            tracing::warn!("Failed to delete profile database: {e}");
            // Don't fail the whole operation if DB deletion fails
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn create_profile(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    profile_name: String,
    password: String,
    display_name: String,
) -> Result<serde_json::Value, String> {
    {
        let guard = state.tox_manager.lock().await;
        if guard.is_some() {
            return Err("Already connected to a profile".to_string());
        }
    }

    // Initialize database
    let db_path = get_db_path(&profile_name);
    let store = Arc::new(MessageStore::open(&db_path, &password)?);

    let manager = ToxManager::create_profile(
        app_handle,
        &profile_name,
        &password,
        &display_name,
        store.clone(),
    )?;

    let address = {
        let mgr = manager.lock().await;
        mgr.get_address().await?
    };

    let profile_info = {
        let mgr = manager.lock().await;
        mgr.get_profile_info().await?
    };

    // Save profile in DB
    store.upsert_profile(address.as_str(), &profile_info.name, &profile_info.status_message)?;

    {
        let mut guard = state.tox_manager.lock().await;
        *guard = Some(manager);
    }
    {
        let mut guard = state.message_store.lock().await;
        *guard = Some(store);
    }

    Ok(serde_json::json!({
        "tox_id": address.as_str(),
        "name": profile_info.name,
        "status_message": profile_info.status_message,
    }))
}

#[tauri::command]
pub async fn load_profile(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    profile_name: String,
    password: String,
) -> Result<serde_json::Value, String> {
    {
        let guard = state.tox_manager.lock().await;
        if guard.is_some() {
            return Err("Already connected to a profile".to_string());
        }
    }

    // Initialize database
    let db_path = get_db_path(&profile_name);
    let store = Arc::new(MessageStore::open(&db_path, &password)?);

    let manager = ToxManager::load_profile(app_handle, &profile_name, &password, store.clone())?;

    let address = {
        let mgr = manager.lock().await;
        mgr.get_address().await?
    };

    let profile_info = {
        let mgr = manager.lock().await;
        mgr.get_profile_info().await?
    };

    store.upsert_profile(address.as_str(), &profile_info.name, &profile_info.status_message)?;

    {
        let mut guard = state.tox_manager.lock().await;
        *guard = Some(manager);
    }
    {
        let mut guard = state.message_store.lock().await;
        *guard = Some(store);
    }

    Ok(serde_json::json!({
        "tox_id": address.as_str(),
        "name": profile_info.name,
        "status_message": profile_info.status_message,
    }))
}

#[tauri::command]
pub async fn get_tox_id(state: State<'_, AppState>) -> Result<String, String> {
    let guard = state.tox_manager.lock().await;
    let manager = guard.as_ref().ok_or("Not connected")?;
    let mgr = manager.lock().await;
    let address = mgr.get_address().await?;
    Ok(address.to_string())
}

#[tauri::command]
pub async fn get_connection_status(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let guard = state.tox_manager.lock().await;
    let manager = guard.as_ref().ok_or("Not connected")?;
    let mgr = manager.lock().await;
    let status = mgr.get_connection_status().await?;
    Ok(serde_json::json!({
        "connected": status.is_connected(),
        "status": match status {
            toxcord_tox::ConnectionStatus::None => "none",
            toxcord_tox::ConnectionStatus::Tcp => "tcp",
            toxcord_tox::ConnectionStatus::Udp => "udp",
        }
    }))
}

#[tauri::command]
pub async fn get_profile_info(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let guard = state.tox_manager.lock().await;
    let manager = guard.as_ref().ok_or("Not connected")?;
    let mgr = manager.lock().await;
    let info = mgr.get_profile_info().await?;
    Ok(serde_json::json!({
        "tox_id": info.tox_id.as_str(),
        "name": info.name,
        "status_message": info.status_message,
    }))
}

#[tauri::command]
pub async fn set_display_name(
    state: State<'_, AppState>,
    name: String,
) -> Result<(), String> {
    let guard = state.tox_manager.lock().await;
    let manager = guard.as_ref().ok_or("Not connected")?;
    let mgr = manager.lock().await;
    let (tx, rx) = oneshot::channel();
    mgr.send_command(ToxCommand::SetName(name, tx)).await?;
    rx.await.map_err(|_| "Failed to receive response".to_string())?
}

#[tauri::command]
pub async fn set_status_message(
    state: State<'_, AppState>,
    message: String,
) -> Result<(), String> {
    let guard = state.tox_manager.lock().await;
    let manager = guard.as_ref().ok_or("Not connected")?;
    let mgr = manager.lock().await;
    let (tx, rx) = oneshot::channel();
    mgr.send_command(ToxCommand::SetStatusMessage(message, tx)).await?;
    rx.await.map_err(|_| "Failed to receive response".to_string())?
}

#[tauri::command]
pub async fn logout(state: State<'_, AppState>) -> Result<(), String> {
    {
        let mut guard = state.tox_manager.lock().await;
        if let Some(manager) = guard.take() {
            let mgr = manager.lock().await;
            mgr.shutdown().await?;
        }
    }
    {
        let mut guard = state.message_store.lock().await;
        *guard = None;
    }
    Ok(())
}
