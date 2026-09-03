#!/usr/bin/env python3
"""Test-only deterministic OpenAI-compatible embedding fixture for W19.

The controlled four-dimensional vectors prove request/response, indexing,
dimension, normalization, and ranking contracts. They make no semantic-quality
claim and are never reachable from product code.
"""

from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import argparse
import json


parser = argparse.ArgumentParser()
parser.add_argument("--port", type=int, default=0)
parser.add_argument("--model", default="memory-fixture-encoder")
args = parser.parse_args()


def controlled_vector(text):
    lowered = text.lower()
    vector = [
        1.0 if any(term in lowered for term in ("provider", "claim", "model")) else 0.0,
        1.0 if any(term in lowered for term in ("file", "filesystem", "resource")) else 0.0,
        1.0 if any(term in lowered for term in ("deny", "denied", "failed")) else 0.0,
        1.0 if any(term in lowered for term in ("complete", "success", "final")) else 0.0,
    ]
    if not any(vector):
        vector[0] = 1.0
    return vector


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
        try:
            request = json.loads(self.rfile.read(length))
        except json.JSONDecodeError:
            self.reply(400, {"error": {"type": "invalid_json"}})
            return
        if self.path != "/v1/embeddings":
            self.reply(404, {"error": {"type": "not_found"}})
            return
        if request.get("model") != args.model:
            self.reply(404, {"error": {"type": "model_not_found"}})
            return
        inputs = request.get("input")
        if isinstance(inputs, str):
            inputs = [inputs]
        if not isinstance(inputs, list) or not all(isinstance(item, str) for item in inputs):
            self.reply(400, {"error": {"type": "invalid_input"}})
            return
        self.reply(
            200,
            {
                "object": "list",
                "model": args.model,
                "data": [
                    {
                        "object": "embedding",
                        "index": index,
                        "embedding": controlled_vector(text),
                    }
                    for index, text in enumerate(inputs)
                ],
                "usage": {"prompt_tokens": len(inputs), "total_tokens": len(inputs)},
            },
        )

    def log_message(self, _format, *_args):
        return


server = ThreadingHTTPServer(("127.0.0.1", args.port), Handler)
print(server.server_port, flush=True)
server.serve_forever()
