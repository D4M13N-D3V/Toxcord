use tauri::State;
use tokio::sync::oneshot;

use crate::db::message_store::DirectMessageRecord;
use crate::managers::tox_manager::ToxCommand;
use crate::AppState;

#[tauri::command]
pub async fn send_direct_message(
    state: State<'_, AppState>,
    friend_number: u32,
    message: String,
) -> Result<serde_json::Value, String> {
    if message.trim().is_empty() {
        return Err("Message cannot be empty".to_string());
    }

    let msg_id = uuid::Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();

    // Split long messages using the protocol codec
    let chunks = toxcord_protocol::codec::split_friend_message(&message);

    // Send each chunk via Tox
    let guard = state.tox_manager.lock().await;
    let manager = guard.as_ref().ok_or("Not connected")?;
    let mgr = manager.lock().await;

    for chunk in &chunks {
        let (tx, rx) = oneshot::channel();
        mgr.send_command(ToxCommand::FriendSendMessage(friend_number, chunk.clone(), tx))
            .await?;
        // If sending fails (e.g., friend offline), queue for later
        match rx.await.map_err(|_| "Failed to receive response".to_string())? {
            Ok(_tox_msg_id) => {}
            Err(e) => {
                // Queue for offline delivery
                drop(mgr);
                drop(guard);

                let store_guard = state.message_store.lock().await;
                if let Some(store) = store_guard.as_ref() {
                    // Save as outgoing message anyway (for UI display)
                    let record = DirectMessageRecord {
                        id: msg_id.clone(),
                        friend_number: friend_number as i64,
                        sender: "self".to_string(),
                        content: message.clone(),
                        message_type: "normal".to_string(),
                        timestamp: timestamp.clone(),
                        is_outgoing: true,
                        delivered: false,
                        read: false,
                    };
                    store.insert_direct_message(&record).ok();

                    // Queue for offline delivery
                    store.queue_offline_message(
                        "friend",
                        &friend_number.to_string(),
                        "text",
                        &message,
                    ).ok();
                }

                return Ok(serde_json::json!({
                    "id": msg_id,
                    "timestamp": timestamp,
                    "delivered": false,
                    "queued": true,
                    "error": e,
                }));
            }
        }
    }

    // All chunks sent successfully — persist to DB
    drop(mgr);
    drop(guard);

    let store_guard = state.message_store.lock().await;
    if let Some(store) = store_guard.as_ref() {
        let record = DirectMessageRecord {
            id: msg_id.clone(),
            friend_number: friend_number as i64,
            sender: "self".to_string(),
            content: message,
            message_type: "normal".to_string(),
            timestamp: timestamp.clone(),
            is_outgoing: true,
            delivered: true,
            read: false,
        };
        store.insert_direct_message(&record)?;
    }

    Ok(serde_json::json!({
        "id": msg_id,
        "timestamp": timestamp,
        "delivered": true,
        "queued": false,
    }))
}

#[tauri::command]
pub async fn get_direct_messages(
    state: State<'_, AppState>,
    friend_number: u32,
    limit: Option<i64>,
    before_timestamp: Option<String>,
) -> Result<Vec<DirectMessageRecord>, String> {
    let store_guard = state.message_store.lock().await;
    let store = store_guard.as_ref().ok_or("Not connected")?;

    let limit = limit.unwrap_or(50);
    let messages = store.get_direct_messages(
        friend_number,
        limit,
        before_timestamp.as_deref(),
    )?;

    Ok(messages)
}

#[tauri::command]
pub async fn set_typing(
    state: State<'_, AppState>,
    friend_number: u32,
    is_typing: bool,
) -> Result<(), String> {
    let guard = state.tox_manager.lock().await;
    let manager = guard.as_ref().ok_or("Not connected")?;
    let mgr = manager.lock().await;
    let (tx, rx) = oneshot::channel();
    mgr.send_command(ToxCommand::SetTyping(friend_number, is_typing, tx))
        .await?;
    rx.await.map_err(|_| "Failed to receive response".to_string())?
}

#[tauri::command]
pub async fn mark_messages_read(
    state: State<'_, AppState>,
    friend_number: u32,
) -> Result<(), String> {
    let store_guard = state.message_store.lock().await;
    let store = store_guard.as_ref().ok_or("Not connected")?;
    store.mark_messages_read(friend_number)
}
