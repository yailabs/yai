#!/usr/bin/env python3
"""Deterministic HTTP provider for typed semantic-continuity proofs.

The fixture validates provider-visible ContextFrame JSON. It never reads or
writes the controlled filesystem and never constructs YAI authority objects.
"""

from http.server import BaseHTTPRequestHandler, HTTPServer
import json
import os
import sys


MODE = sys.argv[1]
EXPECTED = 2 if MODE in {"model-switch", "invalid-continuation"} else 1
REQUESTS = []


def frame_from(request):
    messages = request.get("messages", [])
    if len(messages) != 2:
        return None
    content = messages[1].get("content", "")
    marker = "YAI typed ContextFrame:\n"
    if not content.startswith(marker):
        return None
    try:
        return json.loads(content[len(marker) :])
    except json.JSONDecodeError:
        return None


def frame_has(frame, posture=None, kind=None, **values):
    for entry in frame.get("entries", []):
        if posture is not None and entry.get("posture") != posture:
            continue
        value = entry.get("value", {})
        if kind is not None and value.get("kind") != kind:
            continue
        material = value.get("value", {})
        if all(material.get(key) == expected for key, expected in values.items()):
            return True
    return False


def proposal():
    return json.dumps(
        {
            "schema": "yai.operation_proposal.filesystem_write.v1",
            "operation": "filesystem.write",
            "resource": "workspace",
            "path": "allowed/provider-switch.txt",
            "content": "provider-independent continuity\n",
        },
        separators=(",", ":"),
    )


class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0"))
        request = json.loads(self.rfile.read(length))
        frame = frame_from(request)
        if self.path != "/v1/chat/completions" or frame is None:
            self.send_error(400)
            return
        REQUESTS.append({"request": request, "frame": frame})
        turn = len(REQUESTS)

        if MODE == "proposal-a":
            valid = (
                request.get("model") == "model-a"
                and frame.get("purpose") == "filesystem_write_proposal"
                and frame_has(
                    frame,
                    posture="committed_operational_fact",
                    kind="provider_binding",
                    provider_id="provider:a",
                    model_id="model-a",
                )
            )
            content = proposal()
        elif MODE == "consequence-b":
            valid = (
                request.get("model") == "model-b"
                and frame.get("purpose") == "effect_consequence"
                and frame_has(
                    frame,
                    posture="committed_operational_fact",
                    kind="provider_binding",
                    provider_id="provider:b",
                    model_id="model-b",
                )
                and frame_has(
                    frame,
                    posture="observed_resource_state",
                    kind="resource_consequence",
                    lifecycle="finalized",
                    outcome="applied",
                    relative_path="allowed/provider-switch.txt",
                )
            )
            content = "Provider B observed the finalized filesystem consequence."
        elif MODE == "model-switch":
            if turn == 1:
                valid = (
                    request.get("model") == "model-a"
                    and frame_has(
                        frame,
                        kind="provider_binding",
                        provider_id="provider:stable",
                        model_id="model-a",
                    )
                )
                content = proposal()
            else:
                valid = (
                    request.get("model") == "model-b"
                    and frame_has(
                        frame,
                        kind="provider_binding",
                        provider_id="provider:stable",
                        model_id="model-b",
                    )
                    and frame_has(
                        frame,
                        posture="observed_resource_state",
                        kind="resource_consequence",
                        lifecycle="finalized",
                        outcome="applied",
                    )
                )
                content = "The replacement model observed canonical state."
        elif MODE == "invalid-continuation":
            continuation = request.get("yai_provider_continuation")
            if turn == 1:
                valid = continuation is not None
                if not valid:
                    self.send_error(400)
                    return
                body = json.dumps({"error": "invalid_continuation"}).encode()
                self.send_response(409)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            valid = continuation is None and (
                REQUESTS[0]["frame"]["frame_id"] == frame["frame_id"]
            )
            content = "Fresh full-frame rendering recovered from continuation loss."
        elif MODE == "fresh-restart":
            valid = (
                request.get("model") == "continuation-model"
                and "yai_provider_continuation" not in request
                and frame_has(frame, kind="interaction_turn")
                and frame_has(frame, posture="provider_claim", kind="provider_claim")
            )
            content = "Provider restart preserved semantic continuity."
        else:
            valid = False
            content = "unsupported mode"

        if not valid:
            self.send_error(409)
            return
        body = json.dumps(
            {"choices": [{"message": {"role": "assistant", "content": content}}]}
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
for _ in range(EXPECTED):
    server.handle_request()
server.server_close()
if log_path := os.environ.get("YAI_SEMANTIC_PROVIDER_LOG"):
    with open(log_path, "w", encoding="utf-8") as handle:
        json.dump(REQUESTS, handle, indent=2)
