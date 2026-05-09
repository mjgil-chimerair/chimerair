//! Encoder for BEAM ABI messages.
//!
//! Encodes terms into BEAM binary format.

use super::message::MessageBody;
use chimera_beam_process::BeamPid;
use serde::{Deserialize, Serialize};

/// Encoder state.
#[derive(Debug, Clone)]
pub struct Encoder {
    /// Output buffer.
    buffer: Vec<u8>,
    /// Current offset.
    offset: usize,
}

impl Encoder {
    /// Create a new encoder.
    pub fn new() -> Self {
        Encoder {
            buffer: Vec::new(),
            offset: 0,
        }
    }

    /// Create with capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Encoder {
            buffer: Vec::with_capacity(capacity),
            offset: 0,
        }
    }

    /// Get the encoded bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.buffer
    }

    /// Get a reference to the buffer.
    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer
    }

    /// Encode an atom.
    pub fn encode_atom(&mut self, atom: &str) {
        self.buffer.push(0x00); // atom tag
        self.buffer
            .extend_from_slice(&(atom.len() as u16).to_be_bytes());
        self.buffer.extend_from_slice(atom.as_bytes());
    }

    /// Encode an integer.
    pub fn encode_integer(&mut self, value: i64) {
        if value >= -0x10000000 && value < 0x10000000 {
            // Small integer (immediate)
            self.buffer.push(0x01); // small integer tag
            self.buffer.extend_from_slice(&(value as u32).to_be_bytes());
        } else {
            // Large integer
            self.buffer.push(0x02); // integer tag
            self.buffer.extend_from_slice(&value.to_be_bytes());
        }
    }

    /// Encode a float.
    pub fn encode_float(&mut self, value: f64) {
        self.buffer.push(0x03); // float tag
        self.buffer.extend_from_slice(&value.to_be_bytes());
    }

    /// Encode a PID.
    pub fn encode_pid(&mut self, pid: BeamPid) {
        self.buffer.push(0x04); // pid tag
        self.buffer.extend_from_slice(&pid.to_u64().to_be_bytes());
    }

    /// Encode a tuple.
    pub fn encode_tuple(&mut self, elements: &[MessageBody]) {
        self.buffer.push(0x05); // tuple tag
        self.buffer
            .extend_from_slice(&(elements.len() as u32).to_be_bytes());
        for elem in elements {
            self.encode_message_body(elem);
        }
    }

    /// Encode a list.
    pub fn encode_list(&mut self, elements: &[MessageBody]) {
        self.buffer.push(0x06); // list tag
        self.buffer
            .extend_from_slice(&(elements.len() as u32).to_be_bytes());
        for elem in elements {
            self.encode_message_body(elem);
        }
        self.buffer.push(0x00); // nil terminator
    }

    /// Encode binary data.
    pub fn encode_binary(&mut self, data: &[u8]) {
        self.buffer.push(0x07); // binary tag
        self.buffer
            .extend_from_slice(&(data.len() as u32).to_be_bytes());
        self.buffer.extend_from_slice(data);
    }

    /// Encode a reference.
    pub fn encode_reference(&mut self, ref_val: u64) {
        self.buffer.push(0x08); // reference tag
        self.buffer.extend_from_slice(&ref_val.to_be_bytes());
    }

    /// Encode a message body.
    pub fn encode_message_body(&mut self, body: &MessageBody) {
        match body {
            MessageBody::Atom(s) => self.encode_atom(s),
            MessageBody::Integer(i) => self.encode_integer(*i),
            MessageBody::Float(f) => self.encode_float(*f),
            MessageBody::Pid(pid) => self.encode_pid(*pid),
            MessageBody::Tuple(elements) => self.encode_tuple(elements),
            MessageBody::List(elements) => self.encode_list(elements),
            MessageBody::Binary(data) => self.encode_binary(data),
            MessageBody::Reference(r) => self.encode_reference(*r),
        }
    }

    /// Encode a message with header.
    pub fn encode_message(&mut self, src_pid: BeamPid, dst_pid: BeamPid, body: &MessageBody) {
        // Encode header
        self.buffer.extend_from_slice(&0xFF_u8.to_be_bytes()); // message start marker
        self.encode_pid(src_pid);
        self.encode_pid(dst_pid);

        // Encode body
        self.encode_message_body(body);
    }

    /// Get current encoded size.
    pub fn encoded_size(&self) -> usize {
        self.buffer.len()
    }

    /// Reset the encoder.
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.offset = 0;
    }

    /// Reserve capacity.
    pub fn reserve(&mut self, additional: usize) {
        self.buffer.reserve(additional);
    }
}

impl Default for Encoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Encoded term representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodedTerm {
    /// The encoded bytes.
    pub bytes: Vec<u8>,
    /// Original term type.
    pub term_type: String,
}

impl EncodedTerm {
    /// Create from encoder.
    pub fn from_encoder(encoder: &Encoder, term_type: &str) -> Self {
        EncodedTerm {
            bytes: encoder.as_bytes().to_vec(),
            term_type: term_type.to_string(),
        }
    }

    /// Get the encoded bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Get the encoded size.
    pub fn size(&self) -> usize {
        self.bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoder_new() {
        let encoder = Encoder::new();
        assert!(encoder.as_bytes().is_empty());
    }

    #[test]
    fn test_encoder_encode_atom() {
        let mut encoder = Encoder::new();
        encoder.encode_atom("test");
        assert!(!encoder.as_bytes().is_empty());
    }

    #[test]
    fn test_encoder_encode_integer() {
        let mut encoder = Encoder::new();
        encoder.encode_integer(42);
        assert!(encoder.encoded_size() > 0);
    }

    #[test]
    fn test_encoder_encode_float() {
        let mut encoder = Encoder::new();
        encoder.encode_float(3.14);
        assert_eq!(encoder.encoded_size(), 9); // 1 byte tag + 8 bytes
    }

    #[test]
    fn test_encoder_encode_pid() {
        let mut encoder = Encoder::new();
        encoder.encode_pid(BeamPid::new(1, 1, 0));
        assert_eq!(encoder.encoded_size(), 9); // 1 byte tag + 8 bytes
    }

    #[test]
    fn test_encoder_encode_tuple() {
        let mut encoder = Encoder::new();
        encoder.encode_tuple(&[MessageBody::atom("a"), MessageBody::integer(1)]);
        assert!(encoder.encoded_size() > 0);
    }

    #[test]
    fn test_encoder_encode_list() {
        let mut encoder = Encoder::new();
        encoder.encode_list(&[MessageBody::integer(1), MessageBody::integer(2)]);
        assert!(encoder.encoded_size() > 0);
    }

    #[test]
    fn test_encoder_encode_binary() {
        let mut encoder = Encoder::new();
        encoder.encode_binary(b"hello");
        assert!(encoder.encoded_size() > 5);
    }

    #[test]
    fn test_encoder_message() {
        let mut encoder = Encoder::new();
        encoder.encode_message(
            BeamPid::new(1, 1, 0),
            BeamPid::new(2, 1, 0),
            &MessageBody::integer(42),
        );
        assert!(encoder.encoded_size() > 18); // marker + 2 PIDs + integer
    }

    #[test]
    fn test_encoder_reset() {
        let mut encoder = Encoder::new();
        encoder.encode_atom("test");
        encoder.reset();
        assert!(encoder.as_bytes().is_empty());
    }

    #[test]
    fn test_encoder_with_capacity() {
        let encoder = Encoder::with_capacity(1024);
        assert!(encoder.as_bytes().is_empty());
    }

    #[test]
    fn test_encoded_term() {
        let mut encoder = Encoder::new();
        encoder.encode_atom("test");
        let term = EncodedTerm::from_encoder(&encoder, "atom");
        assert_eq!(term.term_type, "atom");
        assert!(!term.bytes.is_empty());
    }
}
