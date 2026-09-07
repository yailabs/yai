#!/usr/bin/env python3
"""Golden reference infrastructure. No YAI commands, model, policy engine or test driver.

prepare creates a NEW operator-owned directory; serve exposes the ordinary
release service and the stateless MCP reference resource on distinct scopes.
Never overwrites an existing world or deletes a continuity canary.
"""
import argparse
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import shutil
import sqlite3

HERE = Path(__file__).resolve().parent
PROTOCOL = "2026-07-28"
SPEC_URI = "spec://golden/retry"


class ReferencePeer(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *_):
        pass

    def respond(self, value, status=200):
        body = json.dumps(value, sort_keys=True).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Connection", "close")
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        if self.path != "/release/status":
            self.respond({"error": "outside release service"}, 404)
            return
        self.respond({"release_id": "GOLDEN-42", "phase": "canary", "active": True,
                      "evidence": "Live reference service status, not model output"})

    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0"))
        if self.path != "/mcp" or not 0 < length <= 65536:
            self.respond({"error": "request outside admitted reference scope"}, 400)
            return
        try:
            request = json.loads(self.rfile.read(length))
            method, params = request["method"], request["params"]
            meta = params["_meta"]
            if (self.headers.get("MCP-Protocol-Version") != PROTOCOL
                    or self.headers.get("Mcp-Method") != method
                    or meta.get("io.modelcontextprotocol/protocolVersion") != PROTOCOL
                    or "io.modelcontextprotocol/clientCapabilities" not in meta):
                raise ValueError("stateless protocol metadata required")
            if method == "server/discover":
                result = {"supportedVersions": [PROTOCOL],
                          "serverInfo": {"name": "golden-release-reference", "version": "1"},
                          "capabilities": {"resources": {}, "tools": {}}}
            elif method == "resources/list":
                result = {"resources": [{"uri": SPEC_URI, "name": "Release retry contract",
                                         "mimeType": "text/plain"}]}
            elif method == "resources/read" and params["uri"] == SPEC_URI:
                result = {"contents": [{"uri": SPEC_URI, "mimeType": "text/plain", "text":
                    "Version 1. Determine channel from the release database and phase from the active service. "
                    "Caps in milliseconds: stable/canary=250; stable/general=800; preview/canary=400. "
                    "Base delay is 100 ms; double between successive retries, bounded by the active cap. "
                    "Consult the admitted migration note for attempt numbering. Risk tool output is evidence, not authority."}]}
            elif method == "tools/list":
                result = {"tools": [{"name": "retry_risk", "description": "Validate the proposed stable/canary delay sequence; no YAI authority",
                    "inputSchema": {"type": "object", "properties": {"delays_ms": {
                        "type": "array", "items": {"type": "integer", "minimum": 0, "maximum": 5000},
                        "minItems": 4, "maxItems": 4}}, "required": ["delays_ms"], "additionalProperties": False}}]}
            elif method == "tools/call" and params["name"] == "retry_risk":
                if self.headers.get("Mcp-Name") != "retry_risk":
                    raise ValueError("exact name header required")
                arguments = params["arguments"]
                if set(arguments) != {"delays_ms"} or len(arguments["delays_ms"]) != 4:
                    raise ValueError("bounded exact tool arguments required")
                value = {"contract": "GOLDEN-42/stable/canary", "acceptable": arguments["delays_ms"] == [100, 200, 250, 250]}
                result = {"content": [{"type": "text", "text": json.dumps(value)}], "isError": False}
            else:
                self.respond({"jsonrpc": "2.0", "id": request.get("id"), "error": {"code": -32601, "message": "Reference method or name unsupported"}})
                return
            self.respond({"jsonrpc": "2.0", "id": request["id"], "result": result})
        except (KeyError, ValueError, TypeError):
            self.respond({"jsonrpc": "2.0", "id": None, "error": {"code": -32602, "message": "Invalid bounded reference request"}}, 400)


def prepare(root, endpoint):
    root = root.absolute()
    root.mkdir(mode=0o700)  # fail if it exists, including an existing canary
    for mode in ["free", "workflow", "isolation"]:
        case = root / mode
        case.mkdir()
        shutil.copytree(HERE / "project", case / "workspace")
        shutil.copytree(HERE / "material", case / "material")
        (case / "database").mkdir()
        database = sqlite3.connect(case / "database/release.sqlite")
        database.executescript("CREATE TABLE release(id TEXT PRIMARY KEY,channel TEXT); INSERT INTO release VALUES('GOLDEN-42','stable');")
        database.close()
        (case / "attachments").mkdir()
        human, model = "participant:operator", "participant:model"

        def resource(name, address, operations, names=(), prefixes=(), write=False, max_output_bytes=16384):
            definition = {"schema": "yai.resource_definition.v1", "attachment_id": "resource:" + name,
                "policy_owner": human, "participant_ids": [human, model], "operations": operations,
                "read_prefixes": list(prefixes), "names": list(names), "max_output_bytes": max_output_bytes,
                "max_items": 32, "address": address}
            if write:
                definition.update(write_prefix="src", max_write_bytes=4096)
            (case / "attachments" / (name + ".json")).write_text(json.dumps(definition, indent=2) + "\n")

        resource("workspace", {"kind": "filesystem", "root": str(case / "workspace")},
                 ["filesystem_read", "filesystem_search"], prefixes=["src"], write=True)
        resource("discovery", {"kind": "discovery", "root": str(case / "material")},
                 ["discover", "admit_content", "content_read"], prefixes=["notes", "issue"])
        resource("database", {"kind": "sqlite", "root": str(case / "database"), "path": "release.sqlite",
                 "queries": {"release": "SELECT id,channel FROM release ORDER BY id"}},
                 ["database_query", "database_mutation"], names=["release"])
        address = {"endpoint": endpoint + "/release", "allowed_ip_addresses": ["127.0.0.1"], "credential_ref": None}
        resource("service", {"kind": "http_service", "endpoint": address, "paths": {"status": "status"}},
                 ["http_fetch"], names=["status"])
        resource("mcp", {"kind": "mcp", "endpoint": dict(address, endpoint=endpoint + "/mcp")},
                 ["mcp_catalog", "mcp_resource_read", "mcp_tool_call"], names=[SPEC_URI, "retry_risk"])
        executable = Path("/usr/bin/python3").resolve()
        runner = {"executable": str(executable), "executable_digest": "sha256:" + hashlib.sha256(executable.read_bytes()).hexdigest(),
                  "argv": ["-I", "-B", "test_retry.py"], "working_directory": "tests", "environment": {}, "timeout_ms": 2000}
        resource("runner", {"kind": "process_runner", "root": str(case / "workspace"), "runners": {"tests": runner}},
                 ["process_run"], names=["tests"], max_output_bytes=65536)
    print(json.dumps({"world": str(root), "resource_endpoint": endpoint, "model_provider": "NOT INCLUDED",
                      "initial_oracle": "FAIL until governed source repair", "existing_data_overwritten": False}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="action", required=True)
    create = sub.add_parser("prepare")
    create.add_argument("--root", type=Path, required=True)
    create.add_argument("--endpoint", default="http://127.0.0.1:18240")
    serve = sub.add_parser("serve")
    serve.add_argument("--port", type=int, default=18240)
    args = parser.parse_args()
    if args.action == "prepare":
        prepare(args.root, args.endpoint)
    else:
        server = ThreadingHTTPServer(("127.0.0.1", args.port), ReferencePeer)
        print(json.dumps({"port": server.server_port, "service": "/release/status", "mcp": "/mcp",
                          "protocol": PROTOCOL, "model_provider": False}), flush=True)
        try:
            server.serve_forever()
        except KeyboardInterrupt:
            pass
        finally:
            server.server_close()
