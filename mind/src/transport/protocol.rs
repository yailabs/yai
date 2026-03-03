// mind/src/transport/protocol.rs

use serde::{Deserialize, Serialize};

pub type TraceId = [u8; 36];
pub type WorkspaceId = [u8; 36];

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YaiCommand {
    None = 0,
    Ping = 0x0101,
    Handshake = 0x0102,
    Control = 0x0104,
    ControlCall = 0x0105,
    StorageRpc = 0x0201,
    Inference = 0x0301,
}

#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub allowed: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ControlCallRequestV1 {
    #[serde(rename = "type")]
    pub req_type: String,
    pub target_plane: String,
    pub command_id: String,
    pub argv: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ControlCallReplyV1 {
    #[serde(rename = "type")]
    pub resp_type: String,
    pub status: String,
    pub code: String,
    pub reason: String,
    pub command_id: String,
    pub target_plane: String,
}

pub fn is_valid_target_plane(target_plane: &str) -> bool {
    matches!(target_plane, "kernel" | "engine" | "root")
}

pub fn deterministic_nyi_reply(command_id: &str, target_plane: &str) -> ControlCallReplyV1 {
    ControlCallReplyV1 {
        resp_type: "yai.control.reply.v1".to_string(),
        status: "nyi".to_string(),
        code: "NOT_IMPLEMENTED".to_string(),
        reason: "NYI_DETERMINISTIC".to_string(),
        command_id: command_id.to_string(),
        target_plane: target_plane.to_string(),
    }
}

pub fn deterministic_error_reply(
    command_id: &str,
    target_plane: &str,
    code: &str,
    reason: &str,
) -> ControlCallReplyV1 {
    ControlCallReplyV1 {
        resp_type: "yai.control.reply.v1".to_string(),
        status: "error".to_string(),
        code: code.to_string(),
        reason: reason.to_string(),
        command_id: command_id.to_string(),
        target_plane: target_plane.to_string(),
    }
}

// Helper per convertire stringhe in buffer fissi da 36 byte (usato da uds_server)
pub fn string_to_fixed_36(s: &str) -> [u8; 36] {
    let mut buffer = [0u8; 36];
    let bytes = s.as_bytes();
    let len = bytes.len().min(35);
    buffer[..len].copy_from_slice(&bytes[..len]);
    buffer
}

#[cfg(test)]
mod tests {
    use super::{deterministic_nyi_reply, is_valid_target_plane};

    #[test]
    fn target_plane_validation_is_strict() {
        assert!(is_valid_target_plane("kernel"));
        assert!(is_valid_target_plane("engine"));
        assert!(is_valid_target_plane("root"));
        assert!(!is_valid_target_plane("mind"));
    }

    #[test]
    fn nyi_reply_is_deterministic() {
        let a = deterministic_nyi_reply("yai.kernel.ws", "kernel");
        let b = deterministic_nyi_reply("yai.kernel.ws", "kernel");
        assert_eq!(a, b);
        assert_eq!(a.status, "nyi");
        assert_eq!(a.code, "NOT_IMPLEMENTED");
    }
}
