use serde::{Deserialize, Serialize};

/// Maximum payload size for a single TOX message/packet.
/// The actual limit is 1373 bytes for custom lossless packets,
/// but we reserve some for our header.
pub const TOX_MAX_CUSTOM_PACKET_SIZE: usize = 1373;

/// Header size for chunked messages:
/// - 1 byte: packet type
/// - 4 bytes: message ID
/// - 2 bytes: sequence number
/// - 2 bytes: total chunks
pub const CHUNK_HEADER_SIZE: usize = 9;

/// Maximum payload per chunk
pub const MAX_CHUNK_PAYLOAD: usize = TOX_MAX_CUSTOM_PACKET_SIZE - CHUNK_HEADER_SIZE;

/// Maximum size for a friend message (tox_friend_send_message limit)
pub const TOX_MAX_MESSAGE_LENGTH: usize = 1372;

/// A chunk of a larger message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageChunk {
    pub packet_type: u8,
    pub message_id: u32,
    pub sequence: u16,
    pub total: u16,
    pub payload: Vec<u8>,
}

impl MessageChunk {
    /// Serialize chunk to bytes for transmission
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(CHUNK_HEADER_SIZE + self.payload.len());
        buf.push(self.packet_type);
        buf.extend_from_slice(&self.message_id.to_be_bytes());
        buf.extend_from_slice(&self.sequence.to_be_bytes());
        buf.extend_from_slice(&self.total.to_be_bytes());
        buf.extend_from_slice(&self.payload);
        buf
    }

    /// Deserialize chunk from bytes
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < CHUNK_HEADER_SIZE {
            return None;
        }

        let packet_type = data[0];
        let message_id = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);
        let sequence = u16::from_be_bytes([data[5], data[6]]);
        let total = u16::from_be_bytes([data[7], data[8]]);
        let payload = data[CHUNK_HEADER_SIZE..].to_vec();

        Some(Self {
            packet_type,
            message_id,
            sequence,
            total,
            payload,
        })
    }
}

/// Split a payload into chunks for transmission over TOX
pub fn split_payload(packet_type: u8, message_id: u32, data: &[u8]) -> Vec<MessageChunk> {
    if data.len() <= MAX_CHUNK_PAYLOAD {
        return vec![MessageChunk {
            packet_type,
            message_id,
            sequence: 0,
            total: 1,
            payload: data.to_vec(),
        }];
    }

    let chunks: Vec<&[u8]> = data.chunks(MAX_CHUNK_PAYLOAD).collect();
    let total = chunks.len() as u16;

    chunks
        .into_iter()
        .enumerate()
        .map(|(i, chunk)| MessageChunk {
            packet_type,
            message_id,
            sequence: i as u16,
            total,
            payload: chunk.to_vec(),
        })
        .collect()
}

/// Reassembly buffer for collecting chunks into complete messages
pub struct ReassemblyBuffer {
    chunks: std::collections::HashMap<u32, Vec<Option<Vec<u8>>>>,
    received_counts: std::collections::HashMap<u32, usize>,
    timestamps: std::collections::HashMap<u32, std::time::Instant>,
    timeout: std::time::Duration,
}

impl ReassemblyBuffer {
    pub fn new(timeout: std::time::Duration) -> Self {
        Self {
            chunks: std::collections::HashMap::new(),
            received_counts: std::collections::HashMap::new(),
            timestamps: std::collections::HashMap::new(),
            timeout,
        }
    }

    /// Add a chunk to the buffer. Returns the complete payload if all chunks received.
    pub fn add_chunk(&mut self, chunk: MessageChunk) -> Option<Vec<u8>> {
        let msg_id = chunk.message_id;
        let total = chunk.total as usize;
        let seq = chunk.sequence as usize;

        // Single-chunk message
        if total == 1 {
            return Some(chunk.payload);
        }

        // Initialize storage if first chunk for this message
        let slots = self
            .chunks
            .entry(msg_id)
            .or_insert_with(|| vec![None; total]);

        // Store the chunk
        if seq < slots.len() && slots[seq].is_none() {
            slots[seq] = Some(chunk.payload);
            *self.received_counts.entry(msg_id).or_insert(0) += 1;
            self.timestamps.entry(msg_id).or_insert_with(std::time::Instant::now);
        }

        // Check if complete
        if self.received_counts.get(&msg_id) == Some(&total) {
            let slots = self.chunks.remove(&msg_id).unwrap();
            self.received_counts.remove(&msg_id);
            self.timestamps.remove(&msg_id);

            let mut payload = Vec::new();
            for slot in slots {
                payload.extend_from_slice(&slot.unwrap());
            }
            return Some(payload);
        }

        None
    }

    /// Clean up timed-out incomplete messages
    pub fn cleanup(&mut self) {
        let now = std::time::Instant::now();
        let expired: Vec<u32> = self
            .timestamps
            .iter()
            .filter(|(_, ts)| now.duration_since(**ts) > self.timeout)
            .map(|(id, _)| *id)
            .collect();

        for id in expired {
            self.chunks.remove(&id);
            self.received_counts.remove(&id);
            self.timestamps.remove(&id);
        }
    }
}

/// Split a text message for friend_send_message (1372 byte limit)
pub fn split_friend_message(message: &str) -> Vec<String> {
    if message.len() <= TOX_MAX_MESSAGE_LENGTH {
        return vec![message.to_string()];
    }

    let mut parts = Vec::new();
    let mut remaining = message;

    while !remaining.is_empty() {
        if remaining.len() <= TOX_MAX_MESSAGE_LENGTH {
            parts.push(remaining.to_string());
            break;
        }

        // Find a good split point (at a char boundary, prefer whitespace)
        let mut split_at = TOX_MAX_MESSAGE_LENGTH;
        while split_at > 0 && !remaining.is_char_boundary(split_at) {
            split_at -= 1;
        }

        // Try to split at whitespace
        if let Some(ws_pos) = remaining[..split_at].rfind(char::is_whitespace) {
            split_at = ws_pos + 1;
        }

        parts.push(remaining[..split_at].to_string());
        remaining = &remaining[split_at..];
    }

    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_chunk() {
        let data = b"Hello, world!";
        let chunks = split_payload(0x01, 1, data);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].payload, data);
        assert_eq!(chunks[0].total, 1);
    }

    #[test]
    fn test_multi_chunk() {
        let data = vec![0xABu8; MAX_CHUNK_PAYLOAD * 3 + 100];
        let chunks = split_payload(0x01, 1, &data);
        assert_eq!(chunks.len(), 4);

        let mut buffer = ReassemblyBuffer::new(std::time::Duration::from_secs(30));
        let mut result = None;
        for chunk in chunks {
            result = buffer.add_chunk(chunk);
        }

        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_chunk_serialization() {
        let chunk = MessageChunk {
            packet_type: 0x10,
            message_id: 42,
            sequence: 0,
            total: 3,
            payload: vec![1, 2, 3],
        };

        let bytes = chunk.to_bytes();
        let decoded = MessageChunk::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.packet_type, 0x10);
        assert_eq!(decoded.message_id, 42);
        assert_eq!(decoded.sequence, 0);
        assert_eq!(decoded.total, 3);
        assert_eq!(decoded.payload, vec![1, 2, 3]);
    }

    #[test]
    fn test_split_friend_message() {
        let short = "Hello!";
        assert_eq!(split_friend_message(short), vec!["Hello!"]);

        let long = "x".repeat(3000);
        let parts = split_friend_message(&long);
        assert!(parts.len() > 1);
        let reassembled: String = parts.join("");
        assert_eq!(reassembled, long);
    }
}
