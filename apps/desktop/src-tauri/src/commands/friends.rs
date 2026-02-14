use tauri::State;
use tokio::sync::oneshot;

use crate::managers::tox_manager::ToxCommand;
use crate::AppState;

#[tauri::command]
pub async fn add_friend(
    state: State<'_, AppState>,
    tox_id: String,
    message: String,
) -> Result<u32, String> {
    let guard = state.tox_manager.lock().await;
    let manager = guard.as_ref().ok_or("Not connected")?;
    let mgr = manager.lock().await;
    let (tx, rx) = oneshot::channel();
    mgr.send_command(ToxCommand::FriendAdd(tox_id, message, tx)).await?;
    rx.await.map_err(|_| "Failed to receive response".to_string())?
}

#[tauri::command]
pub async fn accept_friend_request(
    state: State<'_, AppState>,
    public_key: String,
) -> Result<u32, String> {
    // Parse hex public key to bytes
    let pk_bytes = hex_to_bytes_32(&public_key)?;

    // Accept in Tox
    let friend_number = {
        let guard = state.tox_manager.lock().await;
        let manager = guard.as_ref().ok_or("Not connected")?;
        let mgr = manager.lock().await;
        let (tx, rx) = oneshot::channel();
        mgr.send_command(ToxCommand::FriendAccept(pk_bytes, tx)).await?;
        rx.await.map_err(|_| "Failed to receive response".to_string())??
    };

    // Remove from pending requests in DB
    let store_guard = state.message_store.lock().await;
    if let Some(store) = store_guard.as_ref() {
        store.remove_friend_request(&public_key)?;
        store.upsert_friend(friend_number, &public_key, "", "")?;
    }

    Ok(friend_number)
}

#[tauri::command]
pub async fn deny_friend_request(
    state: State<'_, AppState>,
    public_key: String,
) -> Result<(), String> {
    let store_guard = state.message_store.lock().await;
    let store = store_guard.as_ref().ok_or("Not connected")?;
    store.remove_friend_request(&public_key)?;
    Ok(())
}

#[tauri::command]
pub async fn remove_friend(
    state: State<'_, AppState>,
    friend_number: u32,
) -> Result<(), String> {
    // Remove from Tox
    {
        let guard = state.tox_manager.lock().await;
        let manager = guard.as_ref().ok_or("Not connected")?;
        let mgr = manager.lock().await;
        let (tx, rx) = oneshot::channel();
        mgr.send_command(ToxCommand::FriendDelete(friend_number, tx)).await?;
        rx.await.map_err(|_| "Failed to receive response".to_string())??;
    }

    // Remove from DB
    let store_guard = state.message_store.lock().await;
    if let Some(store) = store_guard.as_ref() {
        store.remove_friend(friend_number)?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_friends(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    // Get live data from Tox
    let tox_friends = {
        let guard = state.tox_manager.lock().await;
        let manager = guard.as_ref().ok_or("Not connected")?;
        let mgr = manager.lock().await;
        let (tx, rx) = oneshot::channel();
        mgr.send_command(ToxCommand::FriendList(tx)).await?;
        rx.await.map_err(|_| "Failed to receive response".to_string())?
    };

    // Merge with DB data (for last_seen, notes)
    let store_guard = state.message_store.lock().await;
    let db_friends = if let Some(store) = store_guard.as_ref() {
        store.get_friends().unwrap_or_default()
    } else {
        vec![]
    };

    let friends: Vec<serde_json::Value> = tox_friends
        .iter()
        .map(|tf| {
            let db_match = db_friends.iter().find(|df| df.friend_number == tf.number as i64);
            serde_json::json!({
                "friend_number": tf.number,
                "public_key": tf.public_key.0,
                "name": tf.name,
                "status_message": tf.status_message,
                "user_status": format!("{:?}", tf.status).to_lowercase(),
                "connection_status": format!("{:?}", tf.connection_status).to_lowercase(),
                "last_seen": db_match.and_then(|d| d.last_seen.clone()),
                "notes": db_match.map(|d| d.notes.clone()).unwrap_or_default(),
            })
        })
        .collect();

    Ok(serde_json::json!(friends))
}

#[tauri::command]
pub async fn get_friend_requests(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let store_guard = state.message_store.lock().await;
    let store = store_guard.as_ref().ok_or("Not connected")?;
    let requests = store.get_friend_requests()?;
    Ok(serde_json::json!(requests))
}

/// Parse a 64-char hex public key into a [u8; 32]
fn hex_to_bytes_32(hex: &str) -> Result<[u8; 32], String> {
    if hex.len() != 64 {
        return Err(format!("Invalid public key length: {} (expected 64)", hex.len()));
    }
    let bytes: Vec<u8> = (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| format!("Invalid hex: {e}"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}
