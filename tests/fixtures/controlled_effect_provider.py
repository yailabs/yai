#!/usr/bin/env python3
"""Deterministic OpenAI-compatible provider for the controlled-effect vertical.

The fixture only returns model material over HTTP. It never touches the test
filesystem or constructs YAI authority objects.
"""

from http.server import BaseHTTPRequestHandler, HTTPServer
import json
import os
import sys


SCENARIO = sys.argv[1] if len(sys.argv) > 1 else "allow"
EXPECTED_REQUESTS = 2 if SCENARIO in {"allow", "deny", "carrier_failure"} else 1
REQUESTS = []


FIRST_RESPONSES = {
    "allow": {
        "schema": "yai.operation_proposal.filesystem_write.v1",
        "operation": "filesystem.write",
        "resource": "workspace",
        "path": "allowed/hello.txt",
        "content": "hello from controlled YAI\n",
    },
    "deny": {
        "schema": "yai.operation_proposal.filesystem_write.v1",
        "operation": "filesystem.write",
        "resource": "workspace",
        "path": "denied/hello.txt",
        "content": "must not exist\n",
    },
    "traversal": {
        "schema": "yai.operation_proposal.filesystem_write.v1",
        "operation": "filesystem.write",
        "resource": "workspace",
        "path": "../escape.txt",
        "content": "escape",
    },
    "absolute": {
        "schema": "yai.operation_proposal.filesystem_write.v1",
        "operation": "filesystem.write",
        "resource": "workspace",
        "path": "/tmp/escape.txt",
        "content": "escape",
    },
    "symlink_escape": {
        "schema": "yai.operation_proposal.filesystem_write.v1",
        "operation": "filesystem.write",
        "resource": "workspace",
        "path": "allowed/link/escape.txt",
        "content": "escape",
    },
    "wrong_attachment": {
        "schema": "yai.operation_proposal.filesystem_write.v1",
        "operation": "filesystem.write",
        "resource": "unbound-resource",
        "path": "allowed/hello.txt",
        "content": "wrong",
    },
    "oversized": {
        "schema": "yai.operation_proposal.filesystem_write.v1",
        "operation": "filesystem.write",
        "resource": "workspace",
        "path": "allowed/hello.txt",
        "content": "x" * 256,
    },
}


class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0"))
        request = json.loads(self.rfile.read(length))
        if self.path != "/v1/chat/completions" or request.get("model") != "controlled-model":
            self.send_error(400)
            return
        messages = request.get("messages", [])
        if len(messages) != 2:
            self.send_error(400)
            return
        user_content = messages[1].get("content", "")
        REQUESTS.append(user_content)
        turn = len(REQUESTS)
        if turn == 1:
            if "operation_output_contract" not in user_content:
                self.send_error(400)
                return
            if SCENARIO == "malformed":
                content = "{not valid json"
            elif SCENARIO == "claim_only":
                content = "I created allowed/hello.txt successfully."
            else:
                proposal_scenario = (
                    "allow" if SCENARIO in {"allow_once", "carrier_failure"} else SCENARIO
                )
                content = json.dumps(
                    FIRST_RESPONSES[proposal_scenario], separators=(",", ":")
                )
        else:
            if SCENARIO == "allow":
                required = [
                    "real_world_consequence: observed_applied",
                    "truth_source: committed_transition_plus_observation_not_provider_claim",
                ]
                content = "fixture observed the committed filesystem consequence"
            elif SCENARIO == "carrier_failure":
                required = [
                    "real_world_consequence: observed_failed_no_effect",
                    "truth_source: committed_transition_plus_observation_not_provider_claim",
                ]
                content = "fixture observed the committed carrier failure and no effect"
            else:
                required = [
                    "decision: deny",
                    "real_world_consequence: no_effect_authorized",
                ]
                content = "fixture observed the committed denial and no filesystem effect"
            if not all(marker in user_content for marker in required):
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
for _ in range(EXPECTED_REQUESTS):
    server.handle_request()
server.server_close()
if log_path := os.environ.get("YAI_CONTROLLED_PROVIDER_LOG"):
    with open(log_path, "w", encoding="utf-8") as handle:
        json.dump(REQUESTS, handle, indent=2)
sys.exit(0)
