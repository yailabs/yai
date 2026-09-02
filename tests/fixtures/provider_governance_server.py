#!/usr/bin/env python3
"""Deterministic OpenAI-compatible provider-governance fixture."""

from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import argparse
import json
import socket
import time


parser = argparse.ArgumentParser()
parser.add_argument("--port", type=int, default=0)
parser.add_argument(
    "--mode",
    choices=("full", "text_only", "reject", "malformed", "drop", "slow"),
    default="full",
)
parser.add_argument("--model", default="provider-governance-model")
parser.add_argument("--requests", type=int, default=32)
args = parser.parse_args()


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def reply(self, status, value):
        body = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Connection", "close")
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        if self.path == "/health":
            self.reply(200, {"status": "ok", "model": args.model})
            return
        if self.path == "/v1/models":
            self.reply(200, {"object": "list", "data": [{"id": args.model}]})
            return
        self.reply(404, {"error": {"type": "not_found"}})

    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0"))
        raw = self.rfile.read(length)
        if args.mode == "drop":
            self.connection.shutdown(socket.SHUT_RDWR)
            self.connection.close()
            return
        if args.mode == "slow":
            time.sleep(12)
        if self.path != "/v1/chat/completions":
            self.reply(404, {"error": {"type": "not_found"}})
            return
        try:
            request = json.loads(raw)
        except json.JSONDecodeError:
            self.reply(400, {"error": {"type": "invalid_json"}})
            return
        if request.get("model") != args.model:
            self.reply(404, {"error": {"type": "model_not_found"}})
            return
        if args.mode == "reject":
            self.reply(503, {"error": {"type": "fixture_unavailable"}})
            return
        if args.mode == "malformed":
            body = b'{"choices":['
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.send_header("Connection", "close")
            self.end_headers()
            self.wfile.write(body)
            return
        wants_json = request.get("response_format") == {"type": "json_object"}
        if wants_json and args.mode == "text_only":
            self.reply(400, {"error": {"type": "response_format_unsupported"}})
            return
        content = (
            '{"ok":true}'
            if wants_json
            else '{"schema":"yai.case_runtime_turn.v1","outcome":"complete"}'
        )
        self.reply(
            200,
            {
                "id": "completion:fixture",
                "object": "chat.completion",
                "model": args.model,
                "choices": [
                    {
                        "index": 0,
                        "message": {"role": "assistant", "content": content},
                        "finish_reason": "stop",
                    }
                ],
                "usage": {
                    "prompt_tokens": 8,
                    "completion_tokens": 4,
                    "total_tokens": 12,
                },
                "yvex_completion_metrics": {
                    "ttft_ms": 1,
                    "generation_ms": 2,
                    "stop_class": "stop",
                },
            },
        )

    def log_message(self, _format, *_args):
        return


server = ThreadingHTTPServer(("127.0.0.1", args.port), Handler)
print(server.server_port, flush=True)
for _ in range(args.requests):
    server.handle_request()
server.server_close()
