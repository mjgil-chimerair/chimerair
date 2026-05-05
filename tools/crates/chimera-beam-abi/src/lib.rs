//! BEAM ABI for ChimeraIR.
//!
//! Defines the calling convention and message encoding for
//! cross-language BEAM interoperability.

pub mod calling_conv;
pub mod decode;
pub mod encode;
pub mod message;

pub use calling_conv::{Argument, CallingConvention, ReturnValue, StackSlot};
pub use decode::Decoder;
pub use encode::Encoder;
pub use message::{MessageBody, MessageEncoding, MessageHeader};

/// BEAM ABI version.
pub const BEAM_ABI_VERSION: u32 = 1;

/// Maximum arguments per function call.
pub const MAX_ARGS: usize = 256;

/// Maximum message size in bytes.
pub const MAX_MESSAGE_SIZE: usize = 64 * 1024;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abi_version() {
        assert_eq!(BEAM_ABI_VERSION, 1);
    }

    #[test]
    fn test_max_args() {
        assert!(MAX_ARGS > 0);
    }

    #[test]
    fn test_max_message_size() {
        assert!(MAX_MESSAGE_SIZE > 0);
    }
}
