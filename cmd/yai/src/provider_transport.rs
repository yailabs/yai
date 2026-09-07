//! Bounded OpenAI-compatible HTTP/TLS transport shared by provider runtime
//! and synthetic qualification. It owns dispatch-time DNS locality checks and
//! application-byte delivery classification, not provider semantics.

use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;
use yai_core_engine::provider_governance::{provider_address_admitted, ProviderLocality};

const MAX_HTTP_HEADERS: usize = 64 * 1024;
const MAX_HTTP_BODY: usize = 2 * 1024 * 1024;
const IO_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ProviderScheme {
    Http,
    Https,
}

#[derive(Clone, Debug)]
pub(super) struct ProviderEndpoint {
    pub scheme: ProviderScheme,
    pub host: String,
    pub port: u16,
    pub path: String,
}

impl ProviderEndpoint {
    pub fn api_path(&self, suffix: &str) -> String {
        let prefix = self.path.trim_matches('/');
        let prefix = if prefix.is_empty() { "v1" } else { prefix };
        format!("/{}/{}", prefix, suffix.trim_matches('/'))
    }

    fn host_header(&self) -> String {
        let host = if self.host.contains(':') {
            format!("[{}]", self.host)
        } else {
            self.host.clone()
        };
        let default = matches!(self.scheme, ProviderScheme::Http) && self.port == 80
            || matches!(self.scheme, ProviderScheme::Https) && self.port == 443;
        if default {
            host
        } else {
            format!("{host}:{}", self.port)
        }
    }
}

pub(super) fn parse_provider_endpoint(value: &str) -> Result<ProviderEndpoint, String> {
    let (scheme, remainder, default_port) = if let Some(rest) = value.strip_prefix("http://") {
        (ProviderScheme::Http, rest, 80)
    } else if let Some(rest) = value.strip_prefix("https://") {
        (ProviderScheme::Https, rest, 443)
    } else {
        return Err("provider_not_dispatched:url_scheme".to_string());
    };
    let (authority, path) = remainder.split_once('/').unwrap_or((remainder, ""));
    let (host, port) = if let Some(rest) = authority.strip_prefix('[') {
        let (host, suffix) = rest
            .split_once(']')
            .ok_or_else(|| "provider_not_dispatched:url_ipv6".to_string())?;
        let port = if suffix.is_empty() {
            default_port
        } else {
            suffix
                .strip_prefix(':')
                .ok_or_else(|| "provider_not_dispatched:url_authority".to_string())?
                .parse::<u16>()
                .map_err(|_| "provider_not_dispatched:url_port".to_string())?
        };
        (host.to_string(), port)
    } else if authority.matches(':').count() == 1 {
        let (host, port) = authority
            .rsplit_once(':')
            .ok_or_else(|| "provider_not_dispatched:url_authority".to_string())?;
        (
            host.to_string(),
            port.parse::<u16>()
                .map_err(|_| "provider_not_dispatched:url_port".to_string())?,
        )
    } else {
        (authority.to_string(), default_port)
    };
    if host.is_empty() || path.contains(['#', '?']) {
        return Err("provider_not_dispatched:url_invalid".to_string());
    }
    Ok(ProviderEndpoint {
        scheme,
        host,
        port,
        path: format!("/{}", path.trim_matches('/')),
    })
}

fn resolve(
    endpoint: &ProviderEndpoint,
    locality: Option<&ProviderLocality>,
) -> Result<Vec<SocketAddr>, String> {
    let mut addresses = (endpoint.host.as_str(), endpoint.port)
        .to_socket_addrs()
        .map_err(|_| "provider_not_dispatched:dns_resolution_failed".to_string())?
        .collect::<Vec<_>>();
    addresses.sort();
    addresses.dedup();
    if addresses.is_empty() {
        return Err("provider_not_dispatched:dns_no_addresses".to_string());
    }
    if let Some(locality) = locality {
        if addresses
            .iter()
            .any(|address| !provider_address_admitted(locality, address.ip()))
        {
            return Err("provider_not_dispatched:dns_locality_violation".to_string());
        }
    }
    Ok(addresses)
}

enum Connection {
    Plain(TcpStream),
    Tls(Box<StreamOwned<ClientConnection, TcpStream>>),
}

impl Connection {
    fn zero_application_write_is_provably_not_dispatched(&self) -> bool {
        matches!(self, Self::Plain(_))
    }
}

impl Read for Connection {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Plain(stream) => stream.read(buffer),
            Self::Tls(stream) => stream.read(buffer),
        }
    }
}

impl Write for Connection {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Plain(stream) => stream.write(buffer),
            Self::Tls(stream) => stream.write(buffer),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Plain(stream) => stream.flush(),
            Self::Tls(stream) => stream.flush(),
        }
    }
}

fn connect(
    endpoint: &ProviderEndpoint,
    locality: Option<&ProviderLocality>,
    test_roots: Option<RootCertStore>,
    exact_addresses: Option<&[String]>,
) -> Result<Connection, String> {
    let addresses = resolve(endpoint, locality)?;
    if let Some(admitted) = exact_addresses {
        if addresses.iter().any(|address| {
            !admitted
                .iter()
                .any(|ip| ip.parse::<std::net::IpAddr>().ok() == Some(address.ip()))
        }) {
            return Err("provider_not_dispatched:resource_address_not_admitted".into());
        }
    }
    let mut last_error = None;
    let mut socket = None;
    for address in addresses {
        match TcpStream::connect_timeout(&address, IO_TIMEOUT) {
            Ok(stream) => {
                socket = Some(stream);
                break;
            }
            Err(error) => last_error = Some(error),
        }
    }
    let socket = socket.ok_or_else(|| {
        format!(
            "provider_not_dispatched:connect:{}",
            last_error.map_or_else(|| "unavailable".to_string(), |error| error.to_string())
        )
    })?;
    socket
        .set_read_timeout(Some(IO_TIMEOUT))
        .map_err(|error| format!("provider_not_dispatched:timeout_config:{error}"))?;
    socket
        .set_write_timeout(Some(IO_TIMEOUT))
        .map_err(|error| format!("provider_not_dispatched:timeout_config:{error}"))?;
    if endpoint.scheme == ProviderScheme::Http {
        return Ok(Connection::Plain(socket));
    }
    let mut roots = test_roots.unwrap_or_else(RootCertStore::empty);
    if roots.is_empty() {
        roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    }
    let config = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let name = ServerName::try_from(endpoint.host.clone())
        .map_err(|_| "provider_not_dispatched:tls_server_name_invalid".to_string())?;
    let connection = ClientConnection::new(Arc::new(config), name)
        .map_err(|error| format!("provider_not_dispatched:tls_setup:{error}"))?;
    let mut stream = StreamOwned::new(connection, socket);
    while stream.conn.is_handshaking() {
        stream
            .conn
            .complete_io(&mut stream.sock)
            .map_err(|error| format!("provider_not_dispatched:tls_handshake:{error}"))?;
    }
    Ok(Connection::Tls(Box::new(stream)))
}

#[derive(Debug)]
pub(super) struct ProviderHttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
    pub request_bytes_written: usize,
    pub content_type: Option<String>,
}

/// Shared HTTP mechanics only. An ordinary service/MCP Resource is never a
/// ProviderTarget. Its caller must establish independent Case authorization.
pub(super) fn resource_http(
    address: &yai_core_engine::effect::access::NetworkResourceAddress,
    method: &str,
    relative_path: Option<&str>,
    body: &[u8],
    headers: &[(String, String)],
    max_body: usize,
) -> Result<ProviderHttpResponse, String> {
    address.validate()?;
    if !matches!(method, "GET" | "POST")
        || max_body == 0
        || max_body > MAX_HTTP_BODY
        || body.len() > MAX_HTTP_BODY
    {
        return Err("resource_http_request_bounds_invalid".into());
    }
    for (name, value) in headers {
        if name.is_empty()
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b))
            || value
                .bytes()
                .any(|b| !(32..=126).contains(&b) && b != b'\t')
            || !matches!(
                name.to_ascii_lowercase().as_str(),
                "accept" | "mcp-protocol-version" | "mcp-method" | "mcp-name"
            ) && !name.to_ascii_lowercase().starts_with("mcp-param-")
        {
            return Err("resource_http_header_not_admitted".into());
        }
    }
    let endpoint = parse_provider_endpoint(&address.endpoint)?;
    let path = if let Some(path) = relative_path {
        let path = yai_core_engine::effect::normalize_relative_path(path)?;
        if path.contains(['%', '?', '#', '\\']) || path.chars().any(char::is_whitespace) {
            return Err("resource_http_path_invalid".into());
        }
        format!("{}/{}", endpoint.path.trim_end_matches('/'), path)
    } else {
        endpoint.path.clone()
    };
    let credential = address
        .credential_ref
        .as_ref()
        .map(|key| std::env::var(key).map_err(|_| "resource_credential_unavailable".to_string()))
        .transpose()?;
    if credential
        .as_ref()
        .is_some_and(|value| value.is_empty() || value.bytes().any(|b| !(33..=126).contains(&b)))
    {
        return Err("resource_credential_invalid".into());
    }
    http_with_options(
        &endpoint,
        None,
        method,
        &path,
        body,
        credential.as_deref(),
        None,
        Some(&address.allowed_ip_addresses),
        headers,
        max_body,
        true,
    )
}

pub(super) fn provider_http(
    endpoint: &ProviderEndpoint,
    locality: Option<&ProviderLocality>,
    method: &str,
    path: &str,
    body: &[u8],
    api_key: Option<&str>,
) -> Result<ProviderHttpResponse, String> {
    provider_http_with_roots(endpoint, locality, method, path, body, api_key, None)
}

fn provider_http_with_roots(
    endpoint: &ProviderEndpoint,
    locality: Option<&ProviderLocality>,
    method: &str,
    path: &str,
    body: &[u8],
    api_key: Option<&str>,
    test_roots: Option<RootCertStore>,
) -> Result<ProviderHttpResponse, String> {
    http_with_options(
        endpoint,
        locality,
        method,
        path,
        body,
        api_key,
        test_roots,
        None,
        &[],
        MAX_HTTP_BODY,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
fn http_with_options(
    endpoint: &ProviderEndpoint,
    locality: Option<&ProviderLocality>,
    method: &str,
    path: &str,
    body: &[u8],
    api_key: Option<&str>,
    test_roots: Option<RootCertStore>,
    exact_addresses: Option<&[String]>,
    extra_headers: &[(String, String)],
    max_body: usize,
    allow_chunked: bool,
) -> Result<ProviderHttpResponse, String> {
    let mut stream = connect(endpoint, locality, test_roots, exact_addresses)?;
    let mut request = format!(
        "{method} {path} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n",
        endpoint.host_header()
    );
    if !extra_headers
        .iter()
        .any(|(name, _)| name.eq_ignore_ascii_case("accept"))
    {
        request.push_str("Accept: application/json\r\n");
    }
    for (name, value) in extra_headers {
        request.push_str(&format!("{name}: {value}\r\n"));
    }
    if let Some(key) = api_key.filter(|key| !key.is_empty()) {
        request.push_str(&format!("Authorization: Bearer {key}\r\n"));
    }
    if !body.is_empty() {
        request.push_str("Content-Type: application/json\r\n");
        request.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    request.push_str("\r\n");
    let mut request = request.into_bytes();
    request.extend_from_slice(body);
    let mut written = 0usize;
    while written < request.len() {
        match stream.write(&request[written..]) {
            Ok(0) if written == 0 && stream.zero_application_write_is_provably_not_dispatched() => {
                return Err("provider_not_dispatched:zero_write".to_string())
            }
            Ok(0) => {
                return Err(format!(
                    "provider_delivery_indeterminate:partial_write:{written}"
                ))
            }
            Ok(count) => written = written.saturating_add(count),
            Err(error)
                if written == 0 && stream.zero_application_write_is_provably_not_dispatched() =>
            {
                return Err(format!("provider_not_dispatched:write:{error}"))
            }
            Err(error) => {
                return Err(format!(
                    "provider_delivery_indeterminate:partial_write:{written}:{error}"
                ))
            }
        }
    }
    stream.flush().map_err(|error| {
        format!("provider_delivery_indeterminate:write_flush:bytes={written}:{error}")
    })?;
    let mut response = Vec::new();
    let mut buffer = [0u8; 8192];
    let started = std::time::Instant::now();
    loop {
        if allow_chunked && started.elapsed() > IO_TIMEOUT {
            return Err(format!(
                "provider_delivery_indeterminate:resource_response_deadline:bytes={written}"
            ));
        }
        let count = stream.read(&mut buffer).map_err(|error| {
            format!("provider_delivery_indeterminate:response_read:bytes={written}:{error}")
        })?;
        if count == 0 {
            break;
        }
        if response.len().saturating_add(count) > MAX_HTTP_HEADERS + max_body {
            return Err(format!(
                "provider_response_invalid:response_too_large:bytes={written}"
            ));
        }
        response.extend_from_slice(&buffer[..count]);
    }
    let split = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| {
            format!("provider_delivery_indeterminate:partial_headers:bytes={written}")
        })?;
    if split > MAX_HTTP_HEADERS {
        return Err(format!(
            "provider_response_invalid:headers_too_large:bytes={written}"
        ));
    }
    let headers = std::str::from_utf8(&response[..split])
        .map_err(|_| format!("provider_response_invalid:header_utf8:bytes={written}"))?;
    let mut lines = headers.split("\r\n");
    let status_line = lines.next().unwrap_or_default();
    if !status_line.starts_with("HTTP/1.1 ") && !status_line.starts_with("HTTP/1.0 ") {
        return Err(format!(
            "provider_response_invalid:http_version:bytes={written}"
        ));
    }
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| format!("provider_response_invalid:status:bytes={written}"))?;
    if (100..200).contains(&status) {
        return Err(format!(
            "provider_response_invalid:informational_status:bytes={written}"
        ));
    }
    let mut content_lengths = Vec::new();
    let mut transfer_encodings = Vec::new();
    let mut content_type = None;
    for line in lines {
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| format!("provider_response_invalid:header_syntax:bytes={written}"))?;
        if name.eq_ignore_ascii_case("content-length") {
            content_lengths.push(value.trim().parse::<usize>().map_err(|_| {
                format!("provider_response_invalid:content_length:bytes={written}")
            })?);
        }
        if name.eq_ignore_ascii_case("transfer-encoding") {
            transfer_encodings.push(value.trim().to_ascii_lowercase());
        }
        if name.eq_ignore_ascii_case("content-type") {
            if content_type.is_some() {
                return Err("provider_response_invalid:duplicate_content_type".into());
            }
            content_type = Some(value.trim().to_string());
        }
    }
    if content_lengths.len() > 1 {
        return Err(format!(
            "provider_response_invalid:duplicate_content_length:status={status}:bytes={written}"
        ));
    }
    if !transfer_encodings.is_empty()
        && (!allow_chunked || transfer_encodings != ["chunked"] || !content_lengths.is_empty())
    {
        return Err(format!(
            "provider_response_invalid:transfer_encoding_unsupported:status={status}:bytes={written}"
        ));
    }
    let body = if transfer_encodings.is_empty() {
        response[split + 4..].to_vec()
    } else {
        decode_resource_chunks(&response[split + 4..], max_body)?
    };
    if body.len() > max_body {
        return Err("provider_response_invalid:resource_body_bound".into());
    }
    if let Some(expected) = content_lengths.first().copied() {
        if body.len() < expected {
            return Err(format!(
                "provider_delivery_indeterminate:truncated_body:bytes={written}"
            ));
        }
        if body.len() > expected {
            return Err(format!(
                "provider_response_invalid:trailing_body_bytes:status={status}:bytes={written}"
            ));
        }
    }
    if (300..400).contains(&status) {
        return Err(format!(
            "provider_remote_response:{status}:bytes={written}:redirect_refused"
        ));
    }
    Ok(ProviderHttpResponse {
        status,
        body,
        request_bytes_written: written,
        content_type,
    })
}

fn decode_resource_chunks(mut input: &[u8], limit: usize) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    loop {
        let end = input
            .windows(2)
            .position(|w| w == b"\r\n")
            .ok_or_else(|| "provider_delivery_indeterminate:partial_chunk_header".to_string())?;
        if end > 128 {
            return Err("provider_response_invalid:chunk_header_bound".into());
        }
        let size = std::str::from_utf8(&input[..end])
            .ok()
            .and_then(|value| value.split(';').next())
            .filter(|value| !value.is_empty() && value.bytes().all(|b| b.is_ascii_hexdigit()))
            .and_then(|value| usize::from_str_radix(value, 16).ok())
            .ok_or_else(|| "provider_response_invalid:chunk_size".to_string())?;
        input = &input[end + 2..];
        if size == 0 {
            if input != b"\r\n" {
                return Err("provider_response_invalid:chunk_trailers_not_admitted".into());
            }
            return Ok(output);
        }
        if size > limit.saturating_sub(output.len()) {
            return Err("provider_response_invalid:resource_body_bound".into());
        }
        if input.len() < size + 2 || &input[size..size + 2] != b"\r\n" {
            return Err("provider_delivery_indeterminate:partial_chunk_body".into());
        }
        output.extend_from_slice(&input[..size]);
        input = &input[size + 2..];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rcgen::generate_simple_self_signed;
    use rustls::pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
    use rustls::{ServerConfig, ServerConnection};
    use std::net::TcpListener;
    use std::thread;

    fn plain_fixture(response: Vec<u8>) -> (u16, thread::JoinHandle<Vec<u8>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut request = vec![0u8; 8192];
            let count = stream.read(&mut request).unwrap();
            stream.write_all(&response).unwrap();
            request.truncate(count);
            request
        });
        (port, handle)
    }

    fn tls_fixture(
        certificate_name: &str,
        response: Vec<u8>,
    ) -> (u16, RootCertStore, thread::JoinHandle<()>) {
        let certified = generate_simple_self_signed(vec![certificate_name.to_string()]).unwrap();
        let certificate = certified.cert.der().clone();
        let key =
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(certified.key_pair.serialize_der()));
        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![certificate.clone()], key)
            .unwrap();
        let mut roots = RootCertStore::empty();
        roots.add(certificate).unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            let (socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            socket
                .set_write_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let connection = ServerConnection::new(Arc::new(config)).unwrap();
            let mut stream = StreamOwned::new(connection, socket);
            let mut request = [0u8; 8192];
            if stream.read(&mut request).is_ok() {
                let _ = stream.write_all(&response);
                let _ = stream.flush();
                stream.conn.send_close_notify();
                let _ = stream.conn.complete_io(&mut stream.sock);
            }
        });
        (port, roots, handle)
    }

    #[test]
    fn dispatch_dns_locality_rejects_rebinding_classes() {
        let endpoint = parse_provider_endpoint("https://localhost:443/v1").unwrap();
        let error = provider_http(
            &endpoint,
            Some(&ProviderLocality::Remote),
            "GET",
            "/",
            &[],
            None,
        )
        .unwrap_err();
        assert_eq!(error, "provider_not_dispatched:dns_locality_violation");
        println!("h18_dns_rebinding: host=localhost declared=remote result=not_dispatched request_bytes=0");
    }

    #[test]
    fn redirects_and_ambiguous_http_framing_fail_closed() {
        let (port, redirected) = plain_fixture(
            b"HTTP/1.1 302 Found\r\nLocation: http://127.0.0.1:9/steal\r\nContent-Length: 0\r\n\r\n"
                .to_vec(),
        );
        let endpoint = parse_provider_endpoint(&format!("http://localhost:{port}/v1")).unwrap();
        let error = provider_http(
            &endpoint,
            Some(&ProviderLocality::Loopback),
            "GET",
            "/v1/models",
            &[],
            Some("h18-test-secret"),
        )
        .unwrap_err();
        assert!(error.contains("redirect_refused"));
        let request = String::from_utf8_lossy(&redirected.join().unwrap()).to_string();
        assert!(request.contains("Authorization: Bearer h18-test-secret"));

        let (port, duplicate) = plain_fixture(
            b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Length: 2\r\n\r\n{}".to_vec(),
        );
        let endpoint = parse_provider_endpoint(&format!("http://localhost:{port}/v1")).unwrap();
        let error = provider_http(
            &endpoint,
            Some(&ProviderLocality::Loopback),
            "GET",
            "/v1/models",
            &[],
            None,
        )
        .unwrap_err();
        assert!(error.contains("duplicate_content_length"));
        duplicate.join().unwrap();
        println!("h18_http_boundary: redirect_followed=false credential_forwarded=false duplicate_content_length=response_invalid");
    }

    #[test]
    fn tls_validates_chain_and_hostname_without_downgrade() {
        let response = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}".to_vec();
        let (port, roots, server) = tls_fixture("localhost", response.clone());
        let endpoint = parse_provider_endpoint(&format!("https://localhost:{port}/v1")).unwrap();
        let valid = provider_http_with_roots(
            &endpoint,
            Some(&ProviderLocality::Loopback),
            "GET",
            "/v1/models",
            &[],
            None,
            Some(roots),
        )
        .unwrap();
        assert_eq!(valid.status, 200);
        server.join().unwrap();

        let (port, roots, server) = tls_fixture("localhost", response.clone());
        let endpoint = parse_provider_endpoint(&format!("https://127.0.0.1:{port}/v1")).unwrap();
        let mismatch = provider_http_with_roots(
            &endpoint,
            Some(&ProviderLocality::Loopback),
            "GET",
            "/v1/models",
            &[],
            None,
            Some(roots),
        )
        .unwrap_err();
        assert!(mismatch.starts_with("provider_not_dispatched:tls_handshake:"));
        server.join().unwrap();

        let (port, _roots, server) = tls_fixture("localhost", response);
        let endpoint = parse_provider_endpoint(&format!("https://localhost:{port}/v1")).unwrap();
        let unknown_ca = provider_http(
            &endpoint,
            Some(&ProviderLocality::Loopback),
            "GET",
            "/v1/models",
            &[],
            None,
        )
        .unwrap_err();
        assert!(unknown_ca.starts_with("provider_not_dispatched:tls_handshake:"));
        server.join().unwrap();
        println!("h18_tls: valid_ca_hostname=accepted wrong_hostname=not_dispatched unknown_ca=not_dispatched insecure_downgrade=false");
    }
}
