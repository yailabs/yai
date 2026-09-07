//! Ordinary HTTP and MCP protocol adapters for admitted Case Resources.
//! No provider identity, tool authority, runtime loop, or persistent catalog.

#![allow(dead_code)]

use super::provider_transport::resource_http;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};
use yai_core_engine::effect::access::{
    LocalAccessBinding, NetworkResourceAddress, ResourceAddress,
};
use yai_core_engine::effect::digest_bytes;

pub(super) const MCP_VERSION: &str = "2026-07-28";
const MAX_PROTOCOL_BYTES: usize = 65_536;
const MAX_PAGES: usize = 8;
static NEXT_REQUEST: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, Serialize)]
pub(super) struct McpCatalog {
    pub configuration_digest: String,
    pub catalog_digest: String,
    pub protocol_version: String,
    pub server_info: Option<Value>,
    pub capabilities: Value,
    pub tools: BTreeMap<String, Value>,
    pub resources: BTreeMap<String, Value>,
    pub excluded_tools: BTreeMap<String, String>,
    pub cache_posture: &'static str,
}

pub(super) fn fetch_http(
    binding: &LocalAccessBinding,
    name: &str,
    limit: usize,
) -> Result<Value, String> {
    binding.validate()?;
    let ResourceAddress::HttpService { endpoint, paths } = &binding.address else {
        return Err("http_resource_binding_required".into());
    };
    let path = paths
        .get(name)
        .ok_or_else(|| "http_resource_operation_not_bound".to_string())?;
    let response = resource_http(endpoint, "GET", Some(path), &[], &[], limit)?;
    if response.status != 200 {
        return Err(format!("resource_http_status:{}", response.status));
    }
    let body_digest = digest_bytes(&response.body);
    let text =
        String::from_utf8(response.body).map_err(|_| "resource_http_body_not_utf8".to_string())?;
    Ok(
        json!({"operation": name, "status": response.status, "body_digest": body_digest,
        "content_type": response.content_type, "text": text}),
    )
}

fn mcp_address(binding: &LocalAccessBinding) -> Result<&NetworkResourceAddress, String> {
    binding.validate()?;
    match &binding.address {
        ResourceAddress::Mcp { endpoint } => Ok(endpoint),
        _ => Err("mcp_resource_binding_required".into()),
    }
}

pub(super) fn inspect_mcp(binding: &LocalAccessBinding) -> Result<McpCatalog, String> {
    let address = mcp_address(binding)?;
    let discovery = rpc(address, "server/discover", json!({}), &[])?;
    if !discovery["supportedVersions"]
        .as_array()
        .is_some_and(|versions| versions.iter().any(|v| v == MCP_VERSION))
        || !discovery["capabilities"].is_object()
    {
        return Err("mcp_current_protocol_or_capabilities_unavailable".into());
    }
    let capabilities = discovery["capabilities"].clone();
    let mut tools = BTreeMap::new();
    let mut excluded_tools = BTreeMap::new();
    if capabilities.get("tools").is_some_and(Value::is_object) {
        for (name, definition) in list_all(address, "tools/list", "tools", "name")? {
            match validate_tool(&definition) {
                Ok(()) => {
                    tools.insert(name, definition);
                }
                Err(reason) => {
                    excluded_tools.insert(name, reason);
                }
            }
        }
    }
    let resources = if capabilities.get("resources").is_some_and(Value::is_object) {
        list_all(address, "resources/list", "resources", "uri")?
    } else {
        BTreeMap::new()
    };
    let digest = digest_bytes(
        &serde_json::to_vec(&json!({
            "configuration_digest": binding.digest(), "protocol_version": MCP_VERSION,
            "capabilities": capabilities, "tools": tools, "resources": resources,
        }))
        .map_err(|e| e.to_string())?,
    );
    let server_info = discovery["_meta"]["io.modelcontextprotocol/serverInfo"]
        .as_object()
        .filter(|info| {
            info.get("name").is_some_and(Value::is_string)
                && info.get("version").is_some_and(Value::is_string)
        })
        .map(|info| Value::Object(info.clone()));
    Ok(McpCatalog {
        configuration_digest: binding.digest(),
        catalog_digest: digest,
        protocol_version: MCP_VERSION.into(),
        server_info,
        capabilities,
        tools,
        resources,
        excluded_tools,
        cache_posture: "request_local_no_shared_cache_revalidate_before_use",
    })
}

fn list_all(
    address: &NetworkResourceAddress,
    method: &str,
    field: &str,
    key: &str,
) -> Result<BTreeMap<String, Value>, String> {
    let mut cursor: Option<String> = None;
    let mut seen = BTreeSet::new();
    let mut items = BTreeMap::new();
    let mut cache_scope = None;
    for _ in 0..MAX_PAGES {
        let parameters = match &cursor {
            Some(value) => json!({"cursor": value}),
            None => json!({}),
        };
        let page = rpc(address, method, parameters, &[])?;
        // No response is shared or reused; hints cannot authorize stale calls.
        let scope = page
            .get("cacheScope")
            .and_then(Value::as_str)
            .unwrap_or("private");
        if !matches!(scope, "private" | "public")
            || cache_scope.as_deref().is_some_and(|old| old != scope)
        {
            return Err("mcp_pagination_cache_scope_mismatch".into());
        }
        cache_scope = Some(scope.to_string());
        let values = page[field]
            .as_array()
            .ok_or_else(|| "mcp_list_shape_invalid".to_string())?;
        for item in values {
            let name = item[key]
                .as_str()
                .filter(|name| {
                    !name.is_empty() && name.len() <= 1024 && !name.chars().any(char::is_control)
                })
                .ok_or_else(|| "mcp_list_identity_invalid".to_string())?;
            if items.insert(name.to_string(), item.clone()).is_some() {
                return Err("mcp_catalog_duplicate_or_drift".into());
            }
            if items.len() > 128
                || serde_json::to_vec(&items).map_err(|e| e.to_string())?.len() > MAX_PROTOCOL_BYTES
            {
                return Err("mcp_catalog_bound_exceeded".into());
            }
        }
        cursor = match page.get("nextCursor") {
            None | Some(Value::Null) => return Ok(items),
            Some(Value::String(value)) if value.len() <= 2048 => Some(value.clone()),
            _ => return Err("mcp_cursor_shape_invalid".into()),
        };
        // Empty is a valid opaque cursor, not an end marker.
        if !seen.insert(cursor.clone()) {
            return Err("mcp_cursor_cycle".into());
        }
    }
    Err("mcp_pagination_bound_exceeded".into())
}

pub(super) fn read_mcp_resource(
    binding: &LocalAccessBinding,
    expected_catalog: &str,
    uri: &str,
) -> Result<Value, String> {
    let catalog = inspect_mcp(binding)?;
    if catalog.catalog_digest != expected_catalog {
        return Err("mcp_catalog_stale_before_read".into());
    }
    if !catalog.resources.contains_key(uri) {
        return Err("mcp_resource_not_catalogued".into());
    }
    let result = rpc(
        mcp_address(binding)?,
        "resources/read",
        json!({"uri": uri}),
        &[],
    )?;
    let contents = result["contents"]
        .as_array()
        .filter(|values| !values.is_empty() && values.len() <= 16)
        .ok_or_else(|| "mcp_resource_content_shape_invalid".to_string())?;
    if contents.iter().any(|content| {
        content["uri"] != uri || !content["text"].is_string() || content.get("blob").is_some()
    }) {
        return Err("mcp_resource_requires_exact_bounded_text".into());
    }
    Ok(result)
}

/// The caller owns Decision/Grant/PREPARE. This adapter never retries a call,
/// including MRTR or HeaderMismatch. A new catalog cannot silently change the
/// semantics of an already governed operation.
pub(super) fn call_mcp_tool(
    binding: &LocalAccessBinding,
    expected_catalog: &str,
    name: &str,
    arguments: &Value,
    authorize_dispatch: impl FnOnce() -> Result<(), String>,
) -> Result<Value, yai_core_engine::effect::access::ResourceCarrierFailure> {
    use yai_core_engine::effect::access::ResourceCarrierFailure as Failure;
    let catalog = inspect_mcp(binding).map_err(Failure::before_dispatch)?;
    if catalog.catalog_digest != expected_catalog {
        return Err(Failure::before_dispatch("mcp_catalog_stale_before_call"));
    }
    let definition = catalog
        .tools
        .get(name)
        .ok_or_else(|| Failure::before_dispatch("mcp_tool_not_catalogued"))?;
    let validator =
        schema_validator(&definition["inputSchema"]).map_err(Failure::before_dispatch)?;
    if !validator.is_valid(arguments) {
        return Err(Failure::before_dispatch("mcp_tool_arguments_invalid"));
    }
    let headers = parameter_headers(&definition["inputSchema"], arguments)
        .map_err(Failure::before_dispatch)?;
    let endpoint = mcp_address(binding).map_err(Failure::before_dispatch)?;
    // A catalog round trip must not hide a changed Case/Grant/fence.
    authorize_dispatch().map_err(Failure::before_dispatch)?;
    let result = rpc(
        endpoint,
        "tools/call",
        json!({"name": name, "arguments": arguments}),
        &headers,
    )
    .map_err(Failure::indeterminate)?;
    if !result["content"].as_array().is_some_and(|blocks| {
        blocks.len() <= 32
            && blocks
                .iter()
                .all(|block| block["type"] == "text" && block["text"].is_string())
    }) || result
        .get("isError")
        .is_some_and(|value| !value.is_boolean())
    {
        return Err(Failure::indeterminate(
            "mcp_tool_result_shape_invalid_after_delivery",
        ));
    }
    if result.get("isError") != Some(&Value::Bool(true)) {
        if let Some(schema) = definition.get("outputSchema") {
            if !schema_validator(schema)
                .map_err(Failure::indeterminate)?
                .is_valid(&result["structuredContent"])
            {
                return Err(Failure::indeterminate(
                    "mcp_tool_result_schema_invalid_after_delivery",
                ));
            }
        }
    }
    Ok(result)
}

fn validate_tool(definition: &Value) -> Result<(), String> {
    if !definition["name"].is_string() {
        return Err("mcp_tool_name_invalid".into());
    }
    schema_validator(&definition["inputSchema"])?;
    if let Some(schema) = definition.get("outputSchema") {
        schema_validator(schema)?;
    }
    header_paths(&definition["inputSchema"])?;
    Ok(())
}

pub(super) fn schema_validator(schema: &Value) -> Result<jsonschema::Validator, String> {
    if !schema.is_object() || serde_json::to_vec(schema).map_err(|e| e.to_string())?.len() > 8192 {
        return Err("mcp_schema_not_bounded_object".into());
    }
    if schema
        .get("$schema")
        .is_some_and(|dialect| dialect != "https://json-schema.org/draft/2020-12/schema")
    {
        return Err("mcp_schema_dialect_not_supported".into());
    }
    // No ambient remote/file resolution. Only document-local references are
    // admitted; full 2020-12 validation is delegated to the qualified library.
    fn check(value: &Value, depth: usize) -> Result<(), String> {
        if depth > 24 {
            return Err("mcp_schema_depth_bound".into());
        }
        if let Some(object) = value.as_object() {
            for (key, child) in object {
                if matches!(key.as_str(), "$ref" | "$dynamicRef")
                    && child.as_str().is_none_or(|r| !r.starts_with('#'))
                {
                    return Err("mcp_schema_external_reference_refused".into());
                }
                if key == "$id" {
                    return Err("mcp_schema_external_base_not_admitted".into());
                }
                check(child, depth + 1)?;
            }
        } else if let Some(array) = value.as_array() {
            for child in array {
                check(child, depth + 1)?;
            }
        }
        Ok(())
    }
    check(schema, 0)?;
    jsonschema::draft202012::options()
        .build(schema)
        .map_err(|_| "mcp_schema_invalid".into())
}

fn rpc(
    address: &NetworkResourceAddress,
    method: &str,
    mut params: Value,
    custom_headers: &[(String, String)],
) -> Result<Value, String> {
    let id = format!(
        "yai-resource-{}",
        NEXT_REQUEST.fetch_add(1, Ordering::Relaxed)
    );
    let object = params
        .as_object_mut()
        .ok_or_else(|| "mcp_parameters_invalid".to_string())?;
    object.insert("_meta".into(), json!({
        "io.modelcontextprotocol/protocolVersion": MCP_VERSION,
        "io.modelcontextprotocol/clientInfo": {"name": "yai", "version": env!("CARGO_PKG_VERSION")},
        "io.modelcontextprotocol/clientCapabilities": {},
    }));
    let mut headers = vec![
        (
            "Accept".into(),
            "application/json, text/event-stream".into(),
        ),
        ("MCP-Protocol-Version".into(), MCP_VERSION.into()),
        ("Mcp-Method".into(), method.into()),
    ];
    if let Some(name) = params
        .get("name")
        .or_else(|| params.get("uri"))
        .and_then(Value::as_str)
    {
        headers.push(("Mcp-Name".into(), encode_header_value(name)));
    }
    headers.extend_from_slice(custom_headers);
    let body =
        serde_json::to_vec(&json!({"jsonrpc":"2.0", "id":id, "method":method, "params":params}))
            .map_err(|e| e.to_string())?;
    let response = resource_http(address, "POST", None, &body, &headers, MAX_PROTOCOL_BYTES)?;
    let content_type = response
        .content_type
        .as_deref()
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("")
        .trim();
    let message = match content_type {
        "application/json" => serde_json::from_slice::<Value>(&response.body)
            .map_err(|_| "mcp_response_json_invalid_after_delivery".to_string())?,
        "text/event-stream" => parse_sse(&response.body, &id)?,
        _ => return Err("mcp_response_content_type_not_supported_after_delivery".into()),
    };
    validate_rpc_response(message, &id, response.status)
}

fn validate_rpc_response(message: Value, id: &str, status: u16) -> Result<Value, String> {
    if message["jsonrpc"] != "2.0"
        || message.get("method").is_some()
        || message.get("result").is_some() == message.get("error").is_some()
    {
        return Err("mcp_response_envelope_invalid_after_delivery".into());
    }
    if message["id"] != id {
        return Err("mcp_response_id_mismatch_after_delivery".into());
    }
    if message.get("error").is_some() {
        let code = message["error"]["code"]
            .as_i64()
            .ok_or_else(|| "mcp_error_shape_invalid".to_string())?;
        return Err(format!(
            "mcp_remote_error:status={status}:code={code}:no_automatic_retry"
        ));
    }
    if status != 200 {
        return Err(format!("mcp_http_status:{status}"));
    }
    let result = message["result"]
        .as_object()
        .ok_or_else(|| "mcp_result_shape_invalid".to_string())?;
    match result.get("resultType").and_then(Value::as_str) {
        Some("complete") | None => Ok(Value::Object(result.clone())),
        Some("input_required") => Err("mcp_input_required_not_supported_no_automatic_retry".into()),
        _ => Err("mcp_unknown_result_type_after_delivery".into()),
    }
}

fn parse_sse(bytes: &[u8], id: &str) -> Result<Value, String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "mcp_sse_utf8_invalid".to_string())?;
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut final_response = None;
    let mut events = 0usize;
    for event in normalized.split("\n\n") {
        let data = event
            .lines()
            .filter_map(|line| {
                line.strip_prefix("data:")
                    .map(|data| data.strip_prefix(' ').unwrap_or(data))
            })
            .collect::<Vec<_>>()
            .join("\n");
        if data.is_empty() {
            continue;
        }
        events += 1;
        if events > 128 {
            return Err("mcp_sse_event_bound".into());
        }
        let message: Value =
            serde_json::from_str(&data).map_err(|_| "mcp_sse_message_invalid".to_string())?;
        if message.get("id").is_some() {
            if message["id"] != id || message.get("method").is_some() || final_response.is_some() {
                return Err("mcp_sse_unrelated_or_duplicate_response".into());
            }
            final_response = Some(message);
        } else if message["jsonrpc"] != "2.0"
            || !message["method"]
                .as_str()
                .is_some_and(|m| matches!(m, "notifications/progress" | "notifications/message"))
        {
            return Err("mcp_sse_unrequested_message".into());
        }
    }
    final_response.ok_or_else(|| "mcp_delivery_indeterminate:missing_final_response".into())
}

type HeaderPath = (String, Vec<String>);
fn header_paths(schema: &Value) -> Result<Vec<HeaderPath>, String> {
    fn visit(
        schema: &Value,
        path: Vec<String>,
        reachable: bool,
        output: &mut Vec<HeaderPath>,
    ) -> Result<(), String> {
        if let Some(object) = schema.as_object() {
            if let Some(annotation) = object.get("x-mcp-header") {
                let name = annotation
                    .as_str()
                    .ok_or_else(|| "mcp_header_annotation_not_string".to_string())?;
                if !reachable
                    || path.is_empty()
                    || name.is_empty()
                    || name.len() > 128
                    || !name
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b))
                    || !matches!(
                        object.get("type").and_then(Value::as_str),
                        Some("string" | "integer" | "boolean")
                    )
                    || output
                        .iter()
                        .any(|(existing, _)| existing.eq_ignore_ascii_case(name))
                {
                    return Err("mcp_header_annotation_invalid".into());
                }
                output.push((name.into(), path.clone()));
            }
            for (key, child) in object {
                if key == "properties" {
                    if let Some(properties) = child.as_object() {
                        for (name, property) in properties {
                            let mut nested = path.clone();
                            nested.push(name.clone());
                            visit(property, nested, reachable, output)?;
                        }
                    }
                } else if key != "x-mcp-header" {
                    visit(child, path.clone(), false, output)?;
                }
            }
        } else if let Some(items) = schema.as_array() {
            for child in items {
                visit(child, path.clone(), false, output)?;
            }
        }
        Ok(())
    }
    let mut result = Vec::new();
    visit(schema, Vec::new(), true, &mut result)?;
    Ok(result)
}

fn parameter_headers(schema: &Value, arguments: &Value) -> Result<Vec<(String, String)>, String> {
    let mut output = Vec::new();
    for (name, path) in header_paths(schema)? {
        let mut value = Some(arguments);
        for component in path {
            value = value.and_then(|v| v.get(component));
        }
        let text = match value {
            None | Some(Value::Null) => continue,
            Some(Value::String(value)) => value.clone(),
            Some(Value::Bool(value)) => value.to_string(),
            Some(Value::Number(value))
                if value.as_i64().is_some_and(|n| {
                    (-9_007_199_254_740_991..=9_007_199_254_740_991).contains(&n)
                }) =>
            {
                value.to_string()
            }
            _ => return Err("mcp_header_parameter_invalid".into()),
        };
        output.push((format!("Mcp-Param-{name}"), encode_header_value(&text)));
    }
    Ok(output)
}

fn encode_header_value(value: &str) -> String {
    if value.trim() == value
        && value.bytes().all(|b| (32..=126).contains(&b) || b == b'\t')
        && !(value.starts_with("=?base64?") && value.ends_with("?="))
    {
        value.into()
    } else {
        format!(
            "=?base64?{}?=",
            super::provider::encode_base64(value.as_bytes())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    fn binding(endpoint: String) -> LocalAccessBinding {
        LocalAccessBinding {
            schema: yai_core_engine::effect::access::LOCAL_ACCESS_BINDING_SCHEMA.into(),
            case_id: "case:protocol".into(),
            attachment_id: "resource:mcp".into(),
            address: ResourceAddress::Mcp {
                endpoint: NetworkResourceAddress {
                    endpoint,
                    allowed_ip_addresses: vec!["127.0.0.1".into()],
                    credential_ref: None,
                },
            },
        }
    }

    fn read_http(stream: &mut std::net::TcpStream) -> (String, Value) {
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(3)))
            .unwrap();
        let mut bytes = Vec::new();
        let (header_end, length) = loop {
            let mut buffer = [0; 4096];
            let count = stream.read(&mut buffer).unwrap();
            assert_ne!(count, 0);
            bytes.extend_from_slice(&buffer[..count]);
            if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                let header = std::str::from_utf8(&bytes[..end]).unwrap();
                let length = header
                    .lines()
                    .find_map(|line| line.strip_prefix("Content-Length: "))
                    .unwrap()
                    .parse::<usize>()
                    .unwrap();
                break (end + 4, length);
            }
        };
        while bytes.len() < header_end + length {
            let mut buffer = [0; 4096];
            let count = stream.read(&mut buffer).unwrap();
            assert_ne!(count, 0);
            bytes.extend_from_slice(&buffer[..count]);
        }
        (
            String::from_utf8(bytes[..header_end].to_vec()).unwrap(),
            serde_json::from_slice(&bytes[header_end..]).unwrap(),
        )
    }

    fn peer(
        request_count: usize,
        drift_on_call: bool,
    ) -> (LocalAccessBinding, thread::JoinHandle<Vec<String>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let endpoint = format!(
            "http://127.0.0.1:{}/mcp",
            listener.local_addr().unwrap().port()
        );
        let handle = thread::spawn(move || {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(8);
            let mut methods = Vec::new();
            while methods.len() < request_count {
                let (mut stream, _) = match listener.accept() {
                    Ok(pair) => pair,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(
                            std::time::Instant::now() < deadline,
                            "MCP peer request deadline"
                        );
                        thread::sleep(std::time::Duration::from_millis(5));
                        continue;
                    }
                    Err(error) => panic!("{error}"),
                };
                let (headers, request) = read_http(&mut stream);
                let method = request["method"].as_str().unwrap().to_string();
                assert!(headers.starts_with("POST /mcp HTTP/1.1\r\n"));
                assert!(headers.contains("Accept: application/json, text/event-stream\r\n"));
                assert!(headers.contains(&format!("MCP-Protocol-Version: {MCP_VERSION}\r\n")));
                assert!(headers.contains(&format!("Mcp-Method: {method}\r\n")));
                assert_eq!(
                    request["params"]["_meta"]["io.modelcontextprotocol/protocolVersion"],
                    MCP_VERSION
                );
                assert_eq!(
                    request["params"]["_meta"]["io.modelcontextprotocol/clientCapabilities"],
                    json!({})
                );
                assert!(!headers.to_lowercase().contains("mcp-session-id"));
                let changed = drift_on_call && methods.len() >= 3;
                let mut result = match method.as_str() {
                    "server/discover" => {
                        json!({"supportedVersions":[MCP_VERSION],"capabilities":{"tools":{},"resources":{}},
                        "_meta":{"io.modelcontextprotocol/serverInfo":{"name":"not-a-trust-anchor","version":"1"}}})
                    }
                    "tools/list" => {
                        json!({"tools":[{"name":"risk","description":if changed {"changed"} else {"initial"},
                        "inputSchema":{"type":"object","properties":{"release":{"type":"string","x-mcp-header":"Release"}},"required":["release"],"additionalProperties":false}}]})
                    }
                    "resources/list" => {
                        json!({"resources":[{"name":"spec","uri":"release://spec"}]})
                    }
                    "resources/read" => {
                        assert!(headers.contains("Mcp-Name: release://spec\r\n"));
                        json!({"contents":[{"uri":"release://spec","text":"Bounded release evidence"}]})
                    }
                    "tools/call" => {
                        assert!(headers.contains("Mcp-Name: risk\r\n"));
                        assert!(headers.contains("Mcp-Param-Release: stable\r\n"));
                        json!({"content":[{"type":"text","text":"Risk assessment only; no YAI authority"}],"isError":false})
                    }
                    other => panic!("unexpected MCP method {other}"),
                };
                result["resultType"] = json!("complete");
                result["ttlMs"] = json!(86_400_000);
                result["cacheScope"] = json!("private");
                let body = json!({"jsonrpc":"2.0","id":request["id"],"result":result}).to_string();
                // Actual request-scoped SSE and chunked HTTP, not an in-process
                // adapter stub. JSON is exercised by discovery/listing calls.
                if method == "tools/call" {
                    let event = format!(": heartbeat\r\n\r\ndata: {body}\r\n\r\n");
                    write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n{:x}\r\n{}\r\n0\r\n\r\n", event.len(), event).unwrap();
                } else {
                    write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
                }
                methods.push(method);
            }
            methods
        });
        (binding(endpoint), handle)
    }

    #[test]
    fn mcp_current_protocol_real_transport_catalog_read_and_call() {
        let (binding, server) = peer(11, false);
        let catalog = inspect_mcp(&binding).unwrap();
        assert!(catalog.tools.contains_key("risk"));
        assert!(catalog.resources.contains_key("release://spec"));
        assert_eq!(
            read_mcp_resource(&binding, &catalog.catalog_digest, "release://spec").unwrap()
                ["contents"][0]["text"],
            "Bounded release evidence"
        );
        assert_eq!(
            call_mcp_tool(
                &binding,
                &catalog.catalog_digest,
                "risk",
                &json!({"release":"stable"}),
                || Ok(())
            )
            .unwrap()["isError"],
            false
        );
        let methods = server.join().unwrap();
        assert_eq!(methods.iter().filter(|m| *m == "tools/call").count(), 1);
        assert!(!methods.iter().any(|m| m == "initialize"));
    }

    #[test]
    fn mcp_catalog_drift_refuses_call_despite_long_ttl() {
        let (binding, server) = peer(6, true);
        let catalog = inspect_mcp(&binding).unwrap();
        assert!(call_mcp_tool(
            &binding,
            &catalog.catalog_digest,
            "risk",
            &json!({"release":"stable"}),
            || Ok(())
        )
        .unwrap_err()
        .reason
        .contains("stale"));
        assert!(!server.join().unwrap().iter().any(|m| m == "tools/call"));
    }

    #[test]
    fn mcp_headers_schema_and_delivery_shapes_fail_closed() {
        assert_eq!(encode_header_value(" padded "), "=?base64?IHBhZGRlZCA=?=");
        assert!(encode_header_value("=?base64?literal?=").starts_with("=?base64?PT9"));
        let valid = json!({"type":"object","properties":{"outer":{"type":"object","properties":{"release":{"type":"string","x-mcp-header":"Release"}}}}});
        assert_eq!(
            parameter_headers(&valid, &json!({"outer":{"release":"stable"}})).unwrap(),
            vec![("Mcp-Param-Release".into(), "stable".into())]
        );
        assert!(header_paths(&json!({"type":"object","oneOf":[{"properties":{"x":{"type":"string","x-mcp-header":"x"}}}]})).is_err());
        assert!(header_paths(&json!({"type":"object","properties":{"a":{"type":"string","x-mcp-header":"X"},"b":{"type":"string","x-mcp-header":"x"}}})).is_err());
        assert!(schema_validator(&json!({"$ref":"file:///etc/passwd"})).is_err());
        assert!(schema_validator(&json!({"$ref":"https://unadmitted.test/schema"})).is_err());
        assert!(validate_rpc_response(
            json!({"jsonrpc":"2.0","id":"a","result":{"resultType":"input_required"}}),
            "a",
            200
        )
        .unwrap_err()
        .contains("no_automatic_retry"));
        assert!(parse_sse(
            b"data: {\"jsonrpc\":\"2.0\",\"id\":\"wrong\",\"result\":{}}\n\n",
            "a"
        )
        .is_err());
        assert!(parse_sse(b": heartbeat\n\n", "a")
            .unwrap_err()
            .contains("indeterminate"));
    }
}
