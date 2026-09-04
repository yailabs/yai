#!/usr/bin/env python3
"""One-shot generic provider fixture for a test-owned process signal."""

from http.server import BaseHTTPRequestHandler, HTTPServer
import json


class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0"))
        request = json.loads(self.rfile.read(length))
        messages = request.get("messages", [])
        marker = "YAI typed ContextFrame:\n"
        if self.path != "/v1/chat/completions" or len(messages) != 2:
            self.send_error(400)
            return
        content = messages[1].get("content", "")
        if not content.startswith(marker):
            self.send_error(400)
            return
        frame = json.loads(content[len(marker) :])
        contract = frame.get("output_contract", {})
        if (
            frame.get("schema") not in {"yai.context_frame.v6", "yai.context_frame.v7"}
            or frame.get("purpose") != "process_signal_proposal"
            or contract.get("kind") != "process_signal_proposal"
        ):
            self.send_error(409)
            return
        proposal = json.dumps(
            {
                "schema": "yai.operation_proposal.process_signal.v1",
                "operation": "process.signal",
                "resource": "process-fixture",
                "action": "suspend",
            },
            separators=(",", ":"),
        )
        body = json.dumps(
            {
                "model": request.get("model"),
                "choices": [{"message": {"role": "assistant", "content": proposal}}],
            }
        ).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, _format, *_args):
        return


server = HTTPServer(("127.0.0.1", 0), Handler)
print(server.server_port, flush=True)
server.handle_request()
server.server_close()
