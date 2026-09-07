#!/usr/bin/env python3
"""Resource product qualification via ./yai, real SQLite and HTTP/MCP peers.

This is automated product/contract evidence, not the Golden lifecycle and not
human acceptance. No model provider or YVEX is involved.
"""
import http.server
import hashlib
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import tempfile
import threading

ROOT = Path(__file__).resolve().parents[3]
CASE = "case:resources-product"
TENANT = "tenant:resources-product"
HUMAN = "participant:human"


class Peer(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"
    requests = []
    errors = []
    revision = "1"
    allow_tools = False
    tool_mode = "normal"

    def log_message(self, *_):
        pass

    def reply(self, value, status=200):
        data = json.dumps(value).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.send_header("Connection", "close")
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        self.requests.append(("GET", self.path))
        self.reply({"active_release": "stable", "status": "qualification_required"},
                   200 if self.path == "/release/status" else 404)

    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        method = body.get("method")
        self.requests.append((method, self.path))
        try:
            assert self.path == "/mcp"
            assert self.headers["MCP-Protocol-Version"] == "2026-07-28"
            assert self.headers["Mcp-Method"] == method
            assert body["params"]["_meta"]["io.modelcontextprotocol/protocolVersion"] == "2026-07-28"
            assert "io.modelcontextprotocol/clientCapabilities" in body["params"]["_meta"]
            if method == "server/discover":
                result = {"supportedVersions": ["2026-07-28"],
                          "capabilities": {"tools": {}, "resources": {}}}
            elif method == "tools/list":
                result = {"tools": [{"name": "risk_check", "description": "revision " + self.revision,
                                     "inputSchema": {"type": "object", "properties": {},
                                                     "additionalProperties": False}}]}
            elif method == "resources/list":
                result = {"resources": [{"uri": "spec://retry", "name": "Retry specification"}]}
            elif method == "resources/read":
                assert body["params"]["uri"] == "spec://retry"
                assert self.headers["Mcp-Name"] == "spec://retry"
                result = {"contents": [{"uri": "spec://retry", "text": "Use the active channel constraint."}]}
            elif method == "tools/call" and self.allow_tools:
                assert body["params"]["name"] == "risk_check"
                assert self.headers["Mcp-Name"] == "risk_check"
                if self.tool_mode == "drop":
                    self.close_connection = True
                    return
                result = {"content": [{"type": "text", "text": "risk evidence observed"}], "isError": False}
            else:
                raise AssertionError("No tool effect is authorized in this read-only qualification")
            self.reply({"jsonrpc": "2.0", "id": body["id"], "result": result})
        except Exception as error:
            self.errors.append(repr(error))
            self.reply({"jsonrpc": "2.0", "id": body.get("id"),
                        "error": {"code": -32600, "message": "contract violation"}}, 400)


def main(effects=False):
    Peer.allow_tools = effects
    with tempfile.TemporaryDirectory(prefix="yai-resource-product-") as directory:
        run = Path(directory)
        env = dict(os.environ, YAI_HOME=str(run / "home"), NO_COLOR="1")
        order = 0

        evidence = os.environ.get("YAI_RESOURCE_TEST_EVIDENCE")
        if evidence:
            Path(evidence).open("x").close()

        def emit(value):
            line = json.dumps(value, separators=(",", ":"))
            if evidence:
                with Path(evidence).open("a") as stream:
                    stream.write(line + "\n")
            print(line, flush=True)

        emit({"run_id": run.name, "order": 0, "material_pre_state": "fresh empty disposable YAI_HOME",
              "proof_class": "product", "provider_mode": "no_provider", "network": "loopback",
              "peers": "ordinary HTTP and MCP resources; no model provider"})

        def command(*args, reject=None):
            nonlocal order
            order += 1
            argv = ["./yai", *args]
            result = subprocess.run(argv, cwd=ROOT, env=env, capture_output=True,
                                    text=True, timeout=45)
            emit({"run_id": run.name, "order": order, "cwd": str(ROOT),
                              "yai_home": env["YAI_HOME"], "command": argv,
                              "exit": result.returncode, "stdout": result.stdout[:8192],
                              "stdout_excerpt": len(result.stdout) > 8192,
                              "stderr": result.stderr[:4096]})
            assert "\x1b" not in result.stdout
            if reject:
                assert result.returncode != 0 and reject in result.stdout + result.stderr, result
                return result
            assert result.returncode == 0, result
            if "--json" in args:
                value = json.loads(result.stdout)
                return value.get("data", {}).get("value", value)
            return result.stdout

        def write(name, value):
            path = run / name
            path.write_text(json.dumps(value))
            return str(path)

        def field(text, key):
            return next(line.split(": ", 1)[1] for line in text.splitlines()
                        if line.startswith(key + ": "))

        command("init", "--tenant", TENANT, "--organization", "organization:resources-product")
        command("case", "create", CASE, "--tenant", TENANT)
        command("case", "participant", "role", "add", CASE, "--participant", HUMAN,
                "--role", "operation-proposer")
        command("case", "participant", "link-principal", CASE, "--participant", HUMAN, "--principal", "self")
        workspace = run / "workspace"
        (workspace / "src").mkdir(parents=True)
        (workspace / "protected").mkdir()
        (workspace / "src/retry.txt").write_text("retry must follow release evidence\n")
        (workspace / "protected/secret.txt").write_text("not disclosed\n")
        discovery_root = run / "discovery"
        (discovery_root / "notes").mkdir(parents=True)
        candidate_path = discovery_root / "notes/migration.txt"
        candidate_path.write_text("Use release metadata, not an obsolete hardcoded delay.\n")
        connection = sqlite3.connect(workspace / "release.sqlite")
        connection.executescript("CREATE TABLE release(channel TEXT, max_ms INTEGER); INSERT INTO release VALUES('stable',800);")
        connection.close()
        server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Peer)
        worker = threading.Thread(target=server.serve_forever, daemon=True)
        worker.start()
        endpoint = "http://127.0.0.1:" + str(server.server_port)
        digests = {}

        def attach(name, address, operations, names=(), prefixes=()):
            value = {"schema": "yai.resource_definition.v1", "attachment_id": "resource:" + name,
                     "policy_owner": HUMAN, "participant_ids": [HUMAN], "operations": operations,
                     "read_prefixes": list(prefixes), "names": list(names),
                     "max_output_bytes": 8192, "max_items": 32, "address": address}
            result = command("case", "resource", "import", CASE,
                             "--file", write(name + ".json", value), "--json")
            resource = next(item for item in result["resources"] if item["attachment_id"] == "resource:" + name)
            digests[name] = resource["access"]["configuration_digest"]

        def request(name, request_id, action, reject=None):
            value = {"schema": "yai.resource_request.v1", "configuration_digest": digests[name],
                     "action": action}
            return command("case", "resource", "request", CASE, "--participant", HUMAN,
                           "--resource", "resource:" + name, "--request-id", request_id,
                           "--file", write(request_id + ".json", value), "--json", reject=reject)

        try:
            attach("workspace", {"kind": "filesystem", "root": str(workspace)},
                   ["filesystem_read", "filesystem_search"], prefixes=["src"])
            attach("discovery", {"kind": "discovery", "root": str(discovery_root)},
                   ["discover", "admit_content"], prefixes=["notes"])
            attach("database", {"kind": "sqlite", "root": str(workspace), "path": "release.sqlite",
                                "queries": {"release": "SELECT channel,max_ms FROM release"}},
                   ["database_query", "database_mutation"], names=["release"])
            attach("service", {"kind": "http_service", "endpoint": {
                "endpoint": endpoint + "/release", "allowed_ip_addresses": ["127.0.0.1"],
                "credential_ref": None}, "paths": {"status": "status"}}, ["http_fetch"], names=["status"])
            attach("mcp", {"kind": "mcp", "endpoint": {
                "endpoint": endpoint + "/mcp", "allowed_ip_addresses": ["127.0.0.1"],
                "credential_ref": None}}, ["mcp_catalog", "mcp_resource_read", "mcp_tool_call"],
                   names=["spec://retry", "risk_check"])
            if effects:
                executable = Path("/usr/bin/python3").resolve()
                runner = {"executable": str(executable),
                          "executable_digest": "sha256:" + hashlib.sha256(executable.read_bytes()).hexdigest(),
                          "argv": ["-I", "-B", "-c", "import sys; print('real qualification failure'); sys.exit(7)"],
                          "working_directory": "src", "environment": {}, "timeout_ms": 1000}
                timeout_runner = dict(runner, argv=["-I", "-B", "-c", "while True: pass"], timeout_ms=100)
                attach("runner", {"kind": "process_runner", "root": str(workspace),
                                  "runners": {"test": runner, "timeout": timeout_runner}},
                       ["process_run"], names=["test", "timeout"])

            # Attachment alone has no policy authority, and no peer has executed.
            request("workspace", "unconfigured", {"action": "filesystem_read", "path": "src/retry.txt"},
                    reject="policy_authority_requires_ready_and_valid")
            assert Peer.requests == []
            rules = []
            for action, resource, effect in [
                ("filesystem.read", "filesystem", "allow"), ("filesystem.search", "filesystem", "allow"),
                ("database.query", "database", "allow"), ("database.mutate", "database", "deny"),
                ("http.fetch", "http_service", "allow"), ("mcp.catalog", "mcp", "allow"),
                ("mcp.resource.read", "mcp", "allow"), ("mcp.tool.call", "mcp", "allow" if effects else "deny"),
                ("process.run", "process_runner", "allow")]:
                rules += [{"kind": "operation_restriction", "rule_id": action,
                           "operation_kind": action, "resource_kind": resource, "effect": effect,
                           "reason": "resource qualification policy"}]
            for action in ["discovery.enumerate", "content.admit"]:
                rules += [{"kind": "operation_restriction", "rule_id": action,
                           "operation_kind": action, "resource_kind": "discovery", "effect": "allow",
                           "reason": "discovery and admission are separately authorized"}]
            policy = write("policy.json", {"schema": "yai.policy_source_input.v4", "policy_key": "resources",
                "source_version": "1", "owner_ref": "organization:resources-product",
                "source_origin": {"source_system": "resource-product-test", "source_uri": "test://resources/policy"},
                "validity": {"mode": "unbounded"}, "rules": rules})
            artifact = field(command("policy", "ingest", policy, "--tenant", TENANT), "artifact_id")
            command("policy", "validate", artifact, "--reason", "validate reference rules")
            command("policy", "publish", artifact, "--reason", "publish reference rules")
            generation = field(command("case", "policy", "status", "--case", CASE), "case_generation")
            command("case", "policy", "bind", "--case", CASE, "--artifact", artifact,
                    "--expected-generation", generation, "--reason", "admit resource contract")
            capability_view = command("case", "capabilities", CASE, "--participant", HUMAN, "--json")
            assert capability_view["participant_id"] == HUMAN
            assert all(entry["requires_current_decision"] for entry in capability_view["entries"])
            assert any(exclusion["resource_id"] == "resource:database" and exclusion["reason"] == "explicit_policy_deny"
                       for exclusion in capability_view["exclusions"])
            read = request("workspace", "read", {"action": "filesystem_read", "path": "src/retry.txt"})
            assert read["posture"] == "observed" and "retry" in read["observation"]["result"]["text"]
            # Each CLI command is a fresh process; retry observes canonical material without redispatch.
            repeat = request("workspace", "read", {"action": "filesystem_read", "path": "src/retry.txt"})
            assert repeat["reused"] and repeat["observation"] == read["observation"]
            denied = request("workspace", "protected", {"action": "filesystem_read", "path": "protected/secret.txt"})
            assert denied["posture"] == "denied"
            searched = request("workspace", "search", {"action": "filesystem_search", "path": "src", "needle": "release"})
            assert searched["observation"]["result"]["entries"][0]["path"] == "src/retry.txt"
            discovered = request("discovery", "discover", {"action": "discover", "path": "notes"})
            candidate = discovered["observation"]["result"]["entries"][0]
            assert candidate["admitted"] is False
            original_bytes = candidate_path.read_bytes()
            candidate_path.write_text("changed after discovery; must not import this as the candidate")
            admission_action = {"action": "admit_content", "path": candidate["path"], "candidate_digest": candidate["digest"]}
            request("discovery", "admit-drift", admission_action, reject="discovery_candidate_drift")
            candidate_path.write_bytes(original_bytes)
            admitted = request("discovery", "admit", admission_action)
            assert admitted["posture"] == "content_admitted", admitted
            assert admitted["admission"]["object"]["content_digest"] == candidate["digest"]
            assert admitted["admission"]["discovery_observation_id"] == discovered["observation"]["observation_id"]
            candidate_path.write_text("external source subsequently changes; owned content remains exact")
            repeated_admission = request("discovery", "admit", admission_action)
            assert repeated_admission["reused"] and repeated_admission["admission"] == admitted["admission"]
            queried = request("database", "query", {"action": "database_query", "name": "release"})
            assert queried["observation"]["result"]["rows"] == [["stable", 800]]
            assert request("database", "mutate", {"action": "database_mutation", "name": "release"})["posture"] == "denied"
            fetched = request("service", "fetch", {"action": "http_fetch", "name": "status"})
            assert fetched["observation"]["result"]["status"] == 200
            count = len(Peer.requests)
            assert request("service", "unknown-path", {"action": "http_fetch", "name": "outside"})["posture"] == "denied"
            assert len(Peer.requests) == count
            catalog = request("mcp", "catalog", {"action": "mcp_catalog"})["observation"]["result"]
            resource = request("mcp", "mcp-read", {"action": "mcp_resource_read", "uri": "spec://retry",
                                                       "catalog_digest": catalog["catalog_digest"]})
            assert resource["posture"] == "observed"
            count = len(Peer.requests)
            assert request("mcp", "tool-denied", {"action": "mcp_tool_call", "name": "unknown" if effects else "risk_check",
                       "arguments": {}, "catalog_digest": catalog["catalog_digest"]})["posture"] == "denied"
            assert len(Peer.requests) == count
            if effects:
                invoked = request("mcp", "tool", {"action": "mcp_tool_call", "name": "risk_check",
                    "arguments": {}, "catalog_digest": catalog["catalog_digest"]})
                assert invoked["posture"] == "effect" and invoked["receipt"]["outcome"] == "applied", invoked
                assert invoked["observation"]["result"]["isError"] is False
                count = len(Peer.requests)
                repeated = request("mcp", "tool", {"action": "mcp_tool_call", "name": "risk_check",
                    "arguments": {}, "catalog_digest": catalog["catalog_digest"]})
                assert repeated["reused"] and repeated["receipt"] == invoked["receipt"]
                assert len(Peer.requests) == count
                executed = request("runner", "test-run", {"action": "process_run", "name": "test"})
                assert executed["posture"] == "effect" and executed["receipt"]["outcome"] == "applied", executed
                assert executed["observation"]["result"]["exit_code"] == 7
                assert executed["observation"]["result"]["stdout"] == "real qualification failure\n"
                repeated = request("runner", "test-run", {"action": "process_run", "name": "test"})
                assert repeated["reused"] and repeated["receipt"] == executed["receipt"]
                timeout = request("runner", "timeout", {"action": "process_run", "name": "timeout"})
                assert timeout["observation"]["result"]["timed_out"] is True
                assert request("runner", "arbitrary", {"action": "process_run", "name": "shell"})["posture"] == "denied"
            Peer.revision = "2"
            request("mcp", "stale-read", {"action": "mcp_resource_read", "uri": "spec://retry",
                     "catalog_digest": catalog["catalog_digest"]}, reject="mcp_catalog_stale_before_read")
            if effects:
                tool_count = sum(method == "tools/call" for method, _ in Peer.requests)
                stale = request("mcp", "tool-stale", {"action": "mcp_tool_call", "name": "risk_check",
                    "arguments": {}, "catalog_digest": catalog["catalog_digest"]})
                assert stale["receipt"]["outcome"] == "failed_no_effect", stale
                assert sum(method == "tools/call" for method, _ in Peer.requests) == tool_count
                catalog = request("mcp", "catalog-current", {"action": "mcp_catalog"})["observation"]["result"]
                Peer.tool_mode = "drop"
                uncertain = request("mcp", "tool-drop", {"action": "mcp_tool_call", "name": "risk_check",
                    "arguments": {}, "catalog_digest": catalog["catalog_digest"]})
                assert uncertain["posture"] == "indeterminate", uncertain
                count = len(Peer.requests)
                again = request("mcp", "tool-drop", {"action": "mcp_tool_call", "name": "risk_check",
                    "arguments": {}, "catalog_digest": catalog["catalog_digest"]})
                assert again["posture"] == "indeterminate" and again["effect_id"] == uncertain["effect_id"]
                assert len(Peer.requests) == count
            history = command("case", "history", CASE, "--limit", "256", "--json")
            kinds = [transition["payload"]["kind"] for transition in history["transitions"]]
            assert "resource_observation_recorded" in kinds
            assert kinds.count("case_content_admitted") == 1
            assert "conversation_turn_committed" not in kinds
            if effects:
                assert kinds.count("resource_effect_prepared") == 5
                assert kinds.count("resource_effect_finalized") == 4
                assert kinds.count("resource_effect_indeterminate") == 1
            else:
                assert "execution_grant_issued" not in kinds and "effect_prepared" not in kinds
            assert command("case", "verify", CASE, "--json")["materialization"] == "equivalent_to_replay"
            assert not Peer.errors, Peer.errors
            assert sum(method == "tools/call" for method, _ in Peer.requests) == (2 if effects else 0)
            # Real external loss, not a mocked adapter result. No successful
            # Observation may be fabricated for an unreadable database or
            # stopped HTTP/MCP peer; prior history remains replayable.
            observations_before = kinds.count("resource_observation_recorded")
            (workspace / "release.sqlite").write_bytes(b"corrupt disposable reference database")
            request("database", "database-unavailable", {"action": "database_query", "name": "release"},
                    reject="database_requires_bounded_standalone_rollback_image")
            server.shutdown()
            server.server_close()
            worker.join(timeout=5)
            request("service", "http-unavailable", {"action": "http_fetch", "name": "status"}, reject="connect")
            request("mcp", "mcp-unavailable", {"action": "mcp_catalog"}, reject="connect")
            after_failure = command("case", "history", CASE, "--limit", "256", "--json")
            assert sum(t["payload"]["kind"] == "resource_observation_recorded" for t in after_failure["transitions"]) == observations_before
            assert command("case", "verify", CASE, "--json")["materialization"] == "equivalent_to_replay"
            emit({"run_id": run.name, "resource_product": "PASS", "proof_class": "product",
                              "provider_mode": "no_provider", "network": "loopback",
                              "resources": ["filesystem", "sqlite", "http", "mcp"],
                              "governed_process_mcp_effects": effects,
                              "peer_requests": Peer.requests, "golden_case": "NOT_QUALIFIED_BY_THIS_TEST"})
        finally:
            if Peer.errors:
                emit({"run_id": run.name, "peer_contract_errors": Peer.errors})
            server.shutdown()
            server.server_close()
            worker.join(timeout=5)


if __name__ == "__main__":
    main()
