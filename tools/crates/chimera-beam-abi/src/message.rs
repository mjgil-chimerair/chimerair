//! Message encoding for BEAM ABI.
//!
//! Defines how messages are encoded for cross-language communication.

use chimera_beam_process::BeamPid;
use serde::{Deserialize, Serialize};

/// Message encoding format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageEncoding {
    /// Term format (internal BEAM representation).
    Term,
    /// JSON format for external systems.
    Json,
    /// Protocol Buffers (future).
    Protobuf,
    /// Raw binary format.
    Binary,
}

impl MessageEncoding {
    /// Get the encoding name.
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageEncoding::Term => "term",
            MessageEncoding::Json => "json",
            MessageEncoding::Protobuf => "protobuf",
            MessageEncoding::Binary => "binary",
        }
    }
}

impl Default for MessageEncoding {
    fn default() -> Self {
        MessageEncoding::Term
    }
}

/// Message header for BEAM messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    /// Source PID (sender).
    pub src_pid: BeamPid,
    /// Destination PID (receiver).
    pub dst_pid: BeamPid,
    /// Message sequence number.
    pub seq: u64,
    /// Message size in bytes.
    pub size: usize,
    /// Encoding format.
    pub encoding: MessageEncoding,
    /// Flags (compressed, encrypted, etc.).
    pub flags: MessageFlags,
}

impl MessageHeader {
    /// Create a new header.
    pub fn new(src_pid: BeamPid, dst_pid: BeamPid) -> Self {
        MessageHeader {
            src_pid,
            dst_pid,
            seq: 0,
            size: 0,
            encoding: MessageEncoding::default(),
            flags: MessageFlags::default(),
        }
    }

    /// Set sequence number.
    pub fn with_seq(mut self, seq: u64) -> Self {
        self.seq = seq;
        self
    }

    /// Set encoding.
    pub fn with_encoding(mut self, encoding: MessageEncoding) -> Self {
        self.encoding = encoding;
        self
    }

    /// Set message size.
    pub fn with_size(mut self, size: usize) -> Self {
        self.size = size;
        self
    }
}

/// Message flags.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct MessageFlags {
    /// Message is compressed.
    pub compressed: bool,
    /// Message is encrypted.
    pub encrypted: bool,
    /// Message is a system message.
    pub system: bool,
    /// Message requires confirmation.
    pub confirm: bool,
}

impl MessageFlags {
    /// Create new flags.
    pub fn new() -> Self {
        MessageFlags::default()
    }

    /// Set compressed flag.
    pub fn compressed(mut self) -> Self {
        self.compressed = true;
        self
    }

    /// Set encrypted flag.
    pub fn encrypted(mut self) -> Self {
        self.encrypted = true;
        self
    }

    /// Set system flag.
    pub fn system(mut self) -> Self {
        self.system = true;
        self
    }
}

/// Message body variants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageBody {
    /// Atom message.
    Atom(String),
    /// Integer message.
    Integer(i64),
    /// Float message.
    Float(f64),
    /// Tuple message.
    Tuple(Vec<MessageBody>),
    /// List message.
    List(Vec<MessageBody>),
    /// Binary message.
    Binary(Vec<u8>),
    /// PID message.
    Pid(BeamPid),
    /// Reference message.
    Reference(u64),
}

impl MessageBody {
    /// Create an atom message.
    pub fn atom(s: impl Into<String>) -> Self {
        MessageBody::Atom(s.into())
    }

    /// Create an integer message.
    pub fn integer(i: i64) -> Self {
        MessageBody::Integer(i)
    }

    /// Create a tuple message.
    pub fn tuple(items: Vec<MessageBody>) -> Self {
        MessageBody::Tuple(items)
    }

    /// Get the approximate encoded size.
    pub fn encoded_size(&self) -> usize {
        match self {
            MessageBody::Atom(s) => 8 + s.len(),
            MessageBody::Integer(_) => 16,
            MessageBody::Float(_) => 16,
            MessageBody::Tuple(items) => {
                8 + items.iter().map(MessageBody::encoded_size).sum::<usize>()
            }
            MessageBody::List(items) => {
                8 + items.iter().map(MessageBody::encoded_size).sum::<usize>()
            }
            MessageBody::Binary(b) => 8 + b.len(),
            MessageBody::Pid(_) => 16,
            MessageBody::Reference(_) => 16,
        }
    }
}

/// Full message structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message header.
    pub header: MessageHeader,
    /// Message body.
    pub body: MessageBody,
}

impl Message {
    /// Create a new message.
    pub fn new(src_pid: BeamPid, dst_pid: BeamPid, body: MessageBody) -> Self {
        Message {
            header: MessageHeader::new(src_pid, dst_pid).with_encoding(MessageEncoding::Term),
            body,
        }
    }

    /// Get total message size.
    pub fn total_size(&self) -> usize {
        32 + self.body.encoded_size() // header + body
    }

    /// Encode the message to bytes.
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.total_size());

        // Encode header
        bytes.extend_from_slice(&self.header.src_pid.to_u64().to_le_bytes());
        bytes.extend_from_slice(&self.header.dst_pid.to_u64().to_le_bytes());
        bytes.extend_from_slice(&self.header.seq.to_le_bytes());

        // Encode body based on type
        match &self.body {
            MessageBody::Atom(s) => {
                bytes.push(1); // tag
                bytes.extend_from_slice(&(s.len() as u32).to_le_bytes());
                bytes.extend_from_slice(s.as_bytes());
            }
            MessageBody::Integer(i) => {
                bytes.push(2);
                bytes.extend_from_slice(&i.to_le_bytes());
            }
            MessageBody::Float(f) => {
                bytes.push(3);
                bytes.extend_from_slice(&f.to_le_bytes());
            }
            MessageBody::Binary(b) => {
                bytes.push(4);
                bytes.extend_from_slice(&(b.len() as u32).to_le_bytes());
                bytes.extend_from_slice(b);
            }
            _ => {
                bytes.push(0); // unsupported type
            }
        }

        bytes
    }

    /// Decode a message from bytes.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 24 {
            return None;
        }

        let src_pid_val = u64::from_le_bytes(bytes[0..8].try_into().ok()?);
        let dst_pid_val = u64::from_le_bytes(bytes[8..16].try_into().ok()?);
        let seq = u64::from_le_bytes(bytes[16..24].try_into().ok()?);

        let body = if bytes.len() > 24 {
            match bytes[24] {
                1 => {
                    // Atom
                    if bytes.len() >= 28 {
                        let len = u32::from_le_bytes(bytes[25..29].try_into().ok()?) as usize;
                        if bytes.len() >= 29 + len {
                            let s = String::from_utf8_lossy(&bytes[29..29 + len]).to_string();
                            MessageBody::Atom(s)
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                }
                2 => {
                    // Integer
                    if bytes.len() >= 33 {
                        let i = i64::from_le_bytes(bytes[25..33].try_into().ok()?);
                        MessageBody::Integer(i)
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        } else {
            return None;
        };

        Some(Message {
            header: MessageHeader::new(
                BeamPid::from_u64(src_pid_val),
                BeamPid::from_u64(dst_pid_val),
            )
            .with_seq(seq),
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_encoding_as_str() {
        assert_eq!(MessageEncoding::Term.as_str(), "term");
        assert_eq!(MessageEncoding::Json.as_str(), "json");
    }

    #[test]
    fn test_message_flags() {
        let flags = MessageFlags::new().compressed().encrypted();
        assert!(flags.compressed);
        assert!(flags.encrypted);
        assert!(!flags.system);
    }

    #[test]
    fn test_message_body_atom() {
        let body = MessageBody::atom("test");
        assert_eq!(body.encoded_size(), 12); // 8 + 4
    }

    #[test]
    fn test_message_body_integer() {
        let body = MessageBody::integer(42);
        assert_eq!(body.encoded_size(), 16);
    }

    #[test]
    fn test_message_body_tuple() {
        let body = MessageBody::tuple(vec![MessageBody::atom("a"), MessageBody::integer(1)]);
        // Tuple: 8 (tuple header) + atom (8 + 1) + integer (16) = 33
        assert_eq!(body.encoded_size(), 33);
    }

    #[test]
    fn test_message_new() {
        let msg = Message::new(
            BeamPid::new(1, 1, 0),
            BeamPid::new(2, 1, 0),
            MessageBody::atom("hello"),
        );
        assert_eq!(msg.header.src_pid.index(), 1);
        assert_eq!(msg.header.dst_pid.index(), 2);
    }

    #[test]
    fn test_message_encode_decode() {
        let msg = Message::new(
            BeamPid::new(1, 1, 0),
            BeamPid::new(2, 1, 0),
            MessageBody::integer(42),
        );
        let encoded = msg.encode();
        let decoded = Message::decode(&encoded);
        assert!(decoded.is_some());
        if let MessageBody::Integer(i) = decoded.unwrap().body {
            assert_eq!(i, 42);
        } else {
            panic!("expected integer");
        }
    }
}
