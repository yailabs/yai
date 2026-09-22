#!/usr/bin/env python3
"""Explicit, convergent enrichment of an existing operator qualification Case.

Only ordinary CLI product commands mutate YAI. The SQLite database and loopback
peer are clearly authored qualification inputs, not production business facts.
The HTTP/MCP peer lives only for `advance`, or explicitly while `serve` runs.
Retained observations survive its shutdown; no continuing health is asserted.
"""
import argparse
import hashlib
import http.server
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import threading
import time


class Peer(http.server.BaseHTTPRequestHandler):
    revision = 1
    def log_message(self, *_):
        pass

    def reply(self, value, status=200):
        body = json.dumps(value).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        self.reply({"purpose": "Studio controlled qualification", "revision": self.revision,
                    "document": "Retained HTTP observation, not endpoint health"},
                   200 if self.path == "/qualification/document" else 404)

    def do_POST(self):
        if self.path != "/mcp":
            self.reply({"error": "unknown endpoint"}, 404)
            return
        request = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        method = request.get("method")
        if method == "server/discover":
            result = {"supportedVersions": ["2026-07-28"], "capabilities": {"resources": {}}}
        elif method == "tools/list":
            result = {"tools": []}
        elif method == "resources/list":
            result = {"resources": [{"uri": "qualification://scope", "name": "Qualification scope"}]}
        elif method == "resources/read" and request.get("params", {}).get("uri") == "qualification://scope":
            result = {"contents": [{"uri": "qualification://scope", "text": "A read-only controlled MCP qualification resource."}]}
        else:
            self.reply({"jsonrpc": "2.0", "id": request.get("id"), "error": {"code": -32601, "message": "Not offered"}})
            return
        self.reply({"jsonrpc": "2.0", "id": request.get("id"), "result": result})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=["inspect", "advance", "serve"])
    parser.add_argument("--yai", required=True)
    parser.add_argument("--case", default="case:studio-live-qualification")
    parser.add_argument("--root", type=Path, required=True, help="Persistent, dedicated qualification input directory")
    parser.add_argument("--port", type=int, default=18765)
    parser.add_argument("--http-revision", type=int, default=1)
    parser.add_argument("--refresh-http", action="store_true", help="Explicitly observe changed controlled HTTP content")
    parser.add_argument("--evidence", type=Path, required=True)
    args = parser.parse_args()
    if not os.environ.get("YAI_HOME"):
        parser.error("Select the operator YAI_HOME explicitly; no implicit profile")
    run = f"studio-world-{time.time_ns()}"
    order = 0
    args.evidence.parent.mkdir(parents=True, exist_ok=True)

    def emit(record):
        record.update(run_id=run, order=order)
        with args.evidence.open("a") as stream:
            stream.write(json.dumps(record) + "\n")
        print(json.dumps(record), flush=True)

    def cli(*parts):
        nonlocal order
        order += 1
        argv = [args.yai, *map(str, parts), "--json"]
        result = subprocess.run(argv, capture_output=True, text=True, timeout=90)
        emit(dict(command=argv, cwd=os.getcwd(), yai_home=os.environ["YAI_HOME"],
                  exit=result.returncode, stdout=result.stdout[:24000], stderr=result.stderr[:4000],
                  stdout_truncated=len(result.stdout) > 24000))
        if result.returncode:
            raise RuntimeError(f"Product command failed: {parts}")
        data = json.loads(result.stdout)["data"]
        return data.get("value", data.get("case", data))

    before = cli("case", "show", args.case)
    emit(dict(material_pre_state=before, invariant="Existing Case retained; no create/reset/delete"))
    inventory = cli("case", "sources", "inventory", args.case)
    cli("case", "verify", args.case)
    if args.mode == "inspect":
        return
    Peer.revision = args.http_revision
    server = http.server.ThreadingHTTPServer(("127.0.0.1", args.port), Peer)
    if args.mode == "serve":
        emit(dict(peer="controlled qualification HTTP/MCP", endpoint=f"http://127.0.0.1:{args.port}", lifecycle="foreground; Ctrl+C stops peer, retained Case observations remain"))
        try:
            server.serve_forever()
        finally:
            server.server_close()
        return
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        root = args.root.resolve()
        root.mkdir(parents=True, exist_ok=True)
        (root / "work").mkdir(exist_ok=True)

        def write(name, value):
            target = root / name
            body = json.dumps(value, indent=2) + "\n"
            if target.exists() and target.read_text() != body:
                raise RuntimeError(f"Existing qualification input differs; reconcile explicitly: {target}")
            if not target.exists():
                target.write_text(body)
            return target

        database = root / "qualification.sqlite"
        if not database.exists():
            with sqlite3.connect(database) as db:
                db.execute("CREATE TABLE qualification_scope(family TEXT PRIMARY KEY, purpose TEXT NOT NULL)")
                db.executemany("INSERT INTO qualification_scope VALUES (?,?)", [
                    ("database", "Read actual SQLite schema through YAI"),
                    ("http", "Retain a bounded loopback observation"),
                    ("process", "Observe one digest-pinned harmless command")])
        with sqlite3.connect(f"file:{database}?mode=ro", uri=True) as db:
            assert db.execute("SELECT count(*) FROM qualification_scope").fetchone()[0] == 3
        operations = [("database.query", "database"), ("http.fetch", "http_service"),
                      ("filesystem.read", "filesystem"), ("mcp.catalog", "mcp"),
                      ("mcp.resource.read", "mcp"), ("process.run", "process_runner")]
        policy = {"schema": "yai.policy_source_input.v4", "policy_key": "studio-operational-world",
                  "source_version": "1", "owner_ref": "organization:yailabs",
                  "source_origin": {"source_system": "yai-product-qualification", "source_uri": "qualification://studio/operational-world/v1"},
                  "validity": {"mode": "unbounded"},
                  "rules": [{"kind": "operation_restriction", "rule_id": action, "operation_kind": action,
                             "resource_kind": family, "effect": "allow", "reason": "Only exact participant-scoped bounded Resource envelopes; unspecified operations remain denied"} for action, family in operations]}
        policy_path = write("policy.json", policy)
        write("scope.json", {"purpose": "Controlled Studio product qualification", "families": [family for _, family in operations]})
        participant = "participant:operator"

        def resource(name, address, operations, prefixes=(), names=()):
            return dict(schema="yai.resource_definition.v1", attachment_id="resource:studio-" + name,
                        policy_owner=participant, participant_ids=[participant], operations=operations,
                        read_prefixes=list(prefixes), names=list(names), max_output_bytes=8192, max_items=16, address=address)

        endpoint = f"http://127.0.0.1:{args.port}"
        peer = lambda url: dict(endpoint=url, allowed_ip_addresses=["127.0.0.1"], credential_ref=None)
        resources = [
            resource("policy-input", dict(kind="discovery", root=str(root)), ["discover", "admit_content", "content_read"], ["policy.json"]),
            resource("database", dict(kind="sqlite", root=str(root), path=database.name, queries={"schema": "SELECT name, sql FROM sqlite_master WHERE type='table' ORDER BY name"}), ["database_query"], names=["schema"]),
            resource("http", dict(kind="http_service", endpoint=peer(endpoint + "/qualification"), paths={"document": "document"}), ["http_fetch"], names=["document"]),
            resource("filesystem", dict(kind="filesystem", root=str(root)), ["filesystem_read"], ["scope.json"]),
            resource("mcp", dict(kind="mcp", endpoint=peer(endpoint + "/mcp")), ["mcp_catalog", "mcp_resource_read"], names=["qualification://scope"]),
        ]
        executable = Path("/usr/bin/python3").resolve()
        resources.append(resource("process", dict(kind="process_runner", root=str(root), runners={"inspect": {
            "executable": str(executable), "executable_digest": "sha256:" + hashlib.sha256(executable.read_bytes()).hexdigest(),
            "argv": ["-I", "-B", "-c", "print('Studio governed process qualification')"],
            "working_directory": "work", "environment": {}, "timeout_ms": 1000}}), ["process_run"], names=["inspect"]))
        sources = [dict(name="operational-policy", resource="resource:studio-policy-input", roles=["policy"], action=dict(action="discover", path="policy.json"), media_type="application/json", bootstrap_policy=False),
                   dict(name="qualification-database", resource="resource:studio-database", roles=["knowledge", "operational"], action=dict(action="database_query", name="schema"), media_type="application/json", bootstrap_policy=False),
                   dict(name="qualification-http", resource="resource:studio-http", roles=["knowledge", "operational"], action=dict(action="http_fetch", name="document"), media_type="application/json", bootstrap_policy=False)]
        manifest = write("perimeter.json", dict(schema="yai.source_perimeter.v1", name="studio-operational-world", participant=participant, resources=resources, sources=sources))
        # Existing governed Cases do not re-enter bootstrap. Publication/binding
        # uses the ordinary explicit Tenant policy lifecycle, independent of files.
        candidate = cli("policy", "ingest", policy_path, "--tenant", before["tenant_id"], "--validate", "--publish", "--reason", "Reviewed bounded operational qualification rules")
        artifact = next(item["value"] for item in candidate["fields"] if item["name"] == "artifact id")
        status = cli("case", "policy", "status", "--case", args.case)
        if not any("policy_key=studio-operational-world " in item["value"] for item in status["fields"] if item["name"] == "policy binding"):
            generation = cli("case", "show", args.case)["generation"]
            cli("case", "policy", "bind", "--case", args.case, "--artifact", artifact, "--expected-generation", generation, "--reason", "Add exact bounded operational qualification authority")
        declared = cli("case", "sources", "declare", args.case, "--file", manifest)
        own = lambda inv, name: next(item for item in inv["sources"] if item["name"] == name)
        for name in ["operational-policy", "qualification-database", "qualification-http"]:
            if own(cli("case", "sources", "inventory", args.case), name)["phase"] != "acquired":
                cli("case", "sources", "resume", args.case, "--source", name)
        if args.refresh_http:
            cli("case", "sources", "acquire", args.case, "--source", "qualification-http", "--refresh")
        bindings = cli("case", "show", args.case)
        assert len(bindings["resources"]) >= len(before["resources"])
        # Stable request identities: repeat observes durable results, never redispatches.
        imported = cli("case", "sources", "inventory", args.case)
        for name, action in [("filesystem", {"action": "filesystem_read", "path": "scope.json"}),
                             ("process", {"action": "process_run", "name": "inspect"}),
                             ("mcp", {"action": "mcp_catalog"})]:
            definition = next(item for item in resources if item["attachment_id"] == "resource:studio-" + name)
            attachment = cli("case", "resource", "import", args.case, "--file", write(name + "-resource.json", definition))
            exact = next(item for item in attachment["resources"] if item["attachment_id"] == definition["attachment_id"])
            request = write(name + "-request.json", dict(schema="yai.resource_request.v1", configuration_digest=exact["access"]["configuration_digest"], action=action))
            result = cli("case", "resource", "request", args.case, "--participant", participant, "--resource", definition["attachment_id"], "--request-id", "studio-world-v1-" + name, "--file", request)
            assert result["posture"] in ("observed", "effect"), result
            if name == "process":
                assert result["observation"]["result"]["exit_code"] == 0
        cli("case", "knowledge", "build", args.case)
        cli("case", "verify", args.case)
        after = cli("case", "show", args.case)
        emit(dict(result="PASS", before_generation=before["generation"], after_generation=after["generation"],
                  case_ref=args.case, sources=[item["name"] for item in sources], resource_families=[item["address"]["kind"] for item in resources],
                  peer_lifecycle="stopped after acquisition; exact retained observations remain; use serve before a future refresh"))
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=3)


if __name__ == "__main__":
    main()
