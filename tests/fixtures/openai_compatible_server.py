#!/usr/bin/env python3
"""One-request OpenAI-compatible HTTP fixture for provider characterization."""

from http.server import BaseHTTPRequestHandler, HTTPServer
import json
import sys


class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0"))
        request = json.loads(self.rfile.read(length))
        if self.path != "/v1/chat/completions":
            self.send_error(404)
            return
        if request.get("model") != "characterization-model":
            self.send_error(400)
            return
        messages = request.get("messages", [])
        if len(messages) != 2 or "characterize provider continuity" not in messages[1].get(
            "content", ""
        ):
            self.send_error(400)
            return
        body = json.dumps(
            {
                "choices": [
                    {
                        "message": {
                            "role": "assistant",
                            "content": "fixture provider result",
                        }
                    }
                ]
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
sys.exit(0)
