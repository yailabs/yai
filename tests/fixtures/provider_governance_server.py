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
    choices=("full", "text_only", "reject", "malformed", "drop", "slow", "memory", "memory_w20"),
    default="full",
)
parser.add_argument("--model", default="provider-governance-model")
parser.add_argument("--requests", type=int, default=32)
args = parser.parse_args()
MEMORY_TURNS = 0
W20_REPLACEMENT_EMITTED = False
W20_INITIAL_WRITE_EMITTED = False


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
        global MEMORY_TURNS, W20_REPLACEMENT_EMITTED, W20_INITIAL_WRITE_EMITTED
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
        messages = request.get("messages", [])
        is_case_runtime = any(
            isinstance(message, dict)
            and isinstance(message.get("content"), str)
            and message["content"].startswith("YAI typed ContextFrame:\n")
            for message in messages
        )
        if wants_json and args.mode == "text_only":
            self.reply(400, {"error": {"type": "response_format_unsupported"}})
            return
        consolidation_frame = None
        case_runtime_task = ""
        if args.mode == "memory_w20":
            for message in messages:
                content_value = message.get("content") if isinstance(message, dict) else None
                if isinstance(content_value, str) and content_value.startswith("YAI typed ContextFrame:\n"):
                    try:
                        frame = json.loads(content_value.split("\n", 1)[1])
                    except (json.JSONDecodeError, IndexError):
                        continue
                    case_runtime_task = frame.get("task", "")
                    if frame.get("purpose") == "memory_consolidation":
                        consolidation_frame = frame
                        break
        if consolidation_frame is not None:
            task = consolidation_frame.get("task", "")
            marker = "Exact typed input:\n"
            packet = json.loads(task.split(marker, 1)[1])
            consolidation_input = packet["consolidation_input"]
            support = consolidation_input["allowed_support_refs"][0]
            representations = packet.get("operational_representation_documents", [])

            def support_for(marker_value):
                for document in representations:
                    if marker_value in document.get("canonical_text", ""):
                        return {"family": "operational", "id": document["source_id"]}
                return support

            support_4188 = support_for("4188")
            support_9999 = support_for("9999")
            content = json.dumps(
                {
                    "schema": "yai.memory_consolidation_candidate.v1",
                    "case_id": consolidation_input["case_id"],
                    "consolidation_input_id": consolidation_input["input_id"],
                    "episode_narratives": [],
                    "assertions": [
                        {
                            "subject": {"kind": "named_entity", "id": "project:w20-fixture"},
                            "predicate": "project.codename",
                            "value": {"type": "string", "value": "ORCHID-W20"},
                            "support_refs": [support_4188],
                        },
                        {
                            "subject": {"kind": "named_entity", "id": "project:w20-fixture"},
                            "predicate": "project.numeric_fact",
                            "value": {"type": "integer", "value": 4188},
                            "support_refs": [support_4188],
                        },
                        {
                            "subject": {"kind": "named_entity", "id": "project:w20-fixture"},
                            "predicate": "project.numeric_fact",
                            "value": {"type": "integer", "value": 9999},
                            "support_refs": [support_9999],
                        },
                    ],
                },
                sort_keys=True,
                separators=(",", ":"),
            )
        elif args.mode == "memory" and is_case_runtime and MEMORY_TURNS == 0:
            MEMORY_TURNS += 1
            content = json.dumps(
                {
                    "schema": "yai.operation_proposal.filesystem_write.v1",
                    "operation": "filesystem.write",
                    "resource": "resource:w19-memory",
                    "path": "allowed/codename.txt",
                    "content": "project codename ORCHID-731\n",
                },
                sort_keys=True,
                separators=(",", ":"),
            )
        elif args.mode == "memory" and is_case_runtime:
            MEMORY_TURNS += 1
            content = '{"schema":"yai.case_runtime_turn.v1","outcome":"complete"}'
        elif (
            args.mode == "memory_w20"
            and is_case_runtime
            and "9999 provider-only" in case_runtime_task
        ):
            content = json.dumps(
                {
                    "schema": "yai.provider_claim.v1",
                    "claim": "numeric fact 9999 provider-only",
                },
                sort_keys=True,
                separators=(",", ":"),
            )
        elif (
            args.mode == "memory_w20"
            and is_case_runtime
            and "denied attempt" in case_runtime_task
        ):
            content = json.dumps(
                {
                    "schema": "yai.operation_proposal.filesystem_write.v1",
                    "operation": "filesystem.write",
                    "resource": "resource:w20-memory",
                    "path": "denied/blocked.txt",
                    "content": "DENIED-W20\n",
                },
                sort_keys=True,
                separators=(",", ":"),
            )
        elif (
            args.mode == "memory_w20"
            and is_case_runtime
            and "initial write 4187" in case_runtime_task
            and not W20_INITIAL_WRITE_EMITTED
        ):
            W20_INITIAL_WRITE_EMITTED = True
            content = json.dumps(
                {
                    "schema": "yai.operation_proposal.filesystem_write.v1",
                    "operation": "filesystem.write",
                    "resource": "resource:w20-memory",
                    "path": "allowed/codename.txt",
                    "content": "codename ORCHID-W20 numeric fact 4187\n",
                },
                sort_keys=True,
                separators=(",", ":"),
            )
        elif (
            args.mode == "memory_w20"
            and is_case_runtime
            and "initial write 4187" in case_runtime_task
        ):
            content = '{"schema":"yai.case_runtime_turn.v1","outcome":"complete"}'
        elif (
            args.mode == "memory_w20"
            and is_case_runtime
            and "replacement 4188" in case_runtime_task
            and not W20_REPLACEMENT_EMITTED
        ):
            W20_REPLACEMENT_EMITTED = True
            content = json.dumps(
                {
                    "schema": "yai.operation_proposal.filesystem_write.v1",
                    "operation": "filesystem.write",
                    "resource": "resource:w20-memory",
                    "path": "allowed/codename.txt",
                    "content": "codename ORCHID-W20 numeric fact 4188\n",
                },
                sort_keys=True,
                separators=(",", ":"),
            )
        elif (
            args.mode == "memory_w20"
            and is_case_runtime
            and "replacement 4188" in case_runtime_task
        ):
            content = '{"schema":"yai.case_runtime_turn.v1","outcome":"complete"}'
        elif args.mode == "memory_w20" and is_case_runtime:
            MEMORY_TURNS += 1
            content = '{"schema":"yai.case_runtime_turn.v1","outcome":"complete"}'
        elif wants_json:
            content = '{"ok":true}'
        else:
            content = '{"schema":"yai.case_runtime_turn.v1","outcome":"complete"}'
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
