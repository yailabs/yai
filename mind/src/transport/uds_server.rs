use crate::providers::client::{ProviderClient, ProviderRequest, ProviderResponse};
use crate::transport::protocol::{
    deterministic_error_reply, ControlCallReplyV1, ControlCallRequestV1, YaiCommand,
    is_valid_target_plane,
};
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;

#[repr(C, packed)]
pub struct RpcEnvelope {
    pub magic: u32,
    pub version: u32,
    pub command_id: u32,
    pub payload_len: u32,
    pub arming: u32,
    pub session_id: u32,
    pub ws_id: [u8; 36],
    pub trace_id: [u8; 36],
}

pub struct EngineClient {
    stream: UnixStream,
    ws_id: String,
}

impl EngineClient {
    pub fn connect(ws_id: &str) -> Result<Self> {
        let home = std::env::var("HOME")?;
        let socket_path = format!("{}/.yai/run/{}/control.sock", home, ws_id);

        if !Path::new(&socket_path).exists() {
            return Err(anyhow!("Engine socket not found at {}", socket_path));
        }

        let stream = UnixStream::connect(&socket_path)?;
        Ok(Self {
            stream,
            ws_id: ws_id.to_string(),
        })
    }

    /// Helper interno per il protocollo binario LAW
    fn send_raw(&self, command_id: u32, payload_val: &Value) -> Result<Value> {
        let mut stream_mut = &self.stream; // Serve mutability per write_all
        let payload = payload_val.to_string();
        let trace_id = format!("tr-{}", uuid::Uuid::new_v4());

        let mut env = RpcEnvelope {
            magic: 0x59414950,
            version: 1,
            command_id,
            payload_len: payload.len() as u32,
            arming: 1,
            session_id: 0,
            ws_id: [0; 36],
            trace_id: [0; 36],
        };

        copy_to_fixed_buffer(&mut env.ws_id, &self.ws_id);
        copy_to_fixed_buffer(&mut env.trace_id, &trace_id);

        let env_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                (&env as *const RpcEnvelope) as *const u8,
                std::mem::size_of::<RpcEnvelope>(),
            )
        };

        stream_mut.write_all(env_bytes)?;
        stream_mut.write_all(payload.as_bytes())?;
        stream_mut.flush()?;

        let mut buffer = vec![0u8; 16384]; // Buffer un po' più largo per sicurezza
        let n = stream_mut.read(&mut buffer)?;

        Ok(serde_json::from_slice(&buffer[..n])?)
    }

    pub fn control_call(
        &self,
        target_plane: &str,
        command_id: &str,
        argv: &[String],
    ) -> ControlCallReplyV1 {
        if !is_valid_target_plane(target_plane) {
            return deterministic_error_reply(
                command_id,
                target_plane,
                "INVALID_TARGET",
                "INVALID_TARGET_PLANE",
            );
        }

        let request = ControlCallRequestV1 {
            req_type: "yai.control.call.v1".to_string(),
            target_plane: target_plane.to_string(),
            command_id: command_id.to_string(),
            argv: argv.to_vec(),
        };

        let payload = match serde_json::to_value(&request) {
            Ok(v) => v,
            Err(_) => {
                return deterministic_error_reply(
                    command_id,
                    target_plane,
                    "INTERNAL_ERROR",
                    "REQUEST_ENCODE_FAILED",
                );
            }
        };

        let raw = match self.send_raw(YaiCommand::ControlCall as u32, &payload) {
            Ok(v) => v,
            Err(_) => {
                return deterministic_error_reply(
                    command_id,
                    target_plane,
                    "INTERNAL_ERROR",
                    "CONTROL_CALL_TRANSPORT_FAILED",
                );
            }
        };

        match serde_json::from_value::<ControlCallReplyV1>(raw) {
            Ok(reply) => reply,
            Err(_) => deterministic_error_reply(
                command_id,
                target_plane,
                "INTERNAL_ERROR",
                "MALFORMED_CONTROL_REPLY",
            ),
        }
    }
}

/// Implementazione del trait per abilitare l'uso nel BackendRpc
impl ProviderClient for EngineClient {
    fn call(&self, req: ProviderRequest) -> Result<ProviderResponse> {
        // Mappiamo il target/provider dell'RPC a un comando LAW
        let cmd_id = match req.provider.as_str() {
            "E_RPC_STORAGE_GATE" => 0x0401, // Esempio: CMD_STORAGE
            "E_RPC_INFERENCE" => 0x0301,    // CMD_INFERENCE
            _ => 0x0201,                    // Default/Generic
        };

        let result = self.send_raw(cmd_id, &req.payload)?;

        Ok(ProviderResponse { payload: result })
    }
}

fn copy_to_fixed_buffer(dest: &mut [u8; 36], src: &str) {
    let bytes = src.as_bytes();
    let len = bytes.len().min(35);
    dest[..len].copy_from_slice(&bytes[..len]);
}
