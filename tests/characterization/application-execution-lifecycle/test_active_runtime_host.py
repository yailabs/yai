#!/usr/bin/env python3
"""Real supervised work + exact Application stop, using a gated local fixture."""
import json
import argparse
import hashlib
import os
from pathlib import Path
import shutil
import socket
import signal
import subprocess
import tempfile
import threading
import time
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--crash-in-flight", action="store_true")
    parser.add_argument("--source-crash-in-flight", action="store_true")
    parser.add_argument("--source-review", action="store_true")
    options = parser.parse_args()
    crash_in_flight = options.crash_in_flight or options.source_crash_in_flight
    root = Path(tempfile.mkdtemp(prefix="yai-active-host-"))
    home = root / "home"
    env = dict(os.environ, YAI_HOME=str(home))
    binary = Path("./yai").resolve()
    entered, release = threading.Event(), threading.Event()
    dispatches = []
    order = 0

    class Provider(BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass

        def do_GET(self):
            dispatches.append(self.path)
            entered.set()
            assert release.wait(15), "source request was not released"
            data = b'{"exact":"source response not durably admitted"}'
            self.send_response(200)
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            try:
                self.wfile.write(data)
            except (BrokenPipeError, ConnectionResetError):
                assert options.source_crash_in_flight

        def do_POST(self):
            payload = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
            messages = payload.get("messages", [])
            active = any("YAI typed ContextFrame:" in str(m.get("content", "")) for m in messages)
            if active:
                dispatches.append(payload)
                entered.set()
                assert release.wait(15), "test did not release the real provider request"
            content = json.dumps({"schema":"yai.case_runtime_turn.v1", "outcome":"complete",
                                  "summary":"bounded deterministic completion"}) if active else '{"probe":true}'
            data = json.dumps({"id":"fixture", "object":"chat.completion", "model":payload["model"],
                "choices":[{"index":0,"message":{"role":"assistant","content":content},"finish_reason":"stop"}],
                "usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}}).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            try:
                self.wfile.write(data)
            except (BrokenPipeError, ConnectionResetError):
                assert crash_in_flight, "unexpected provider connection loss"

    server = ThreadingHTTPServer(("127.0.0.1", 0), Provider)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    endpoint = f"http://127.0.0.1:{server.server_port}/v1/chat/completions"

    def cli(*args):
        result = subprocess.run([str(binary), *args], env=env, capture_output=True, text=True, timeout=30)
        print(json.dumps({"argv":args,"exit":result.returncode,"stdout":result.stdout,"stderr":result.stderr}), flush=True)
        assert result.returncode == 0, result.stderr
        return result.stdout

    def app(operation, data, *, expected="success", lose_response=False):
        nonlocal order
        discovery = json.loads((home / "run/host/discovery.json").read_text())
        with socket.socket(socket.AF_UNIX) as connection:
            connection.settimeout(20)
            connection.connect(discovery["endpoint"])
            reader = connection.makefile("rb")
            connection.sendall((json.dumps({"kind":"handshake", "protocol":discovery["protocol"],
                "client_id":f"qualification:{os.getpid()}:{order}","client_kind":"qualification",
                "pid":os.getpid(),"yai_home_identity":discovery["yai_home_identity"]})+"\n").encode())
            assert json.loads(reader.readline())["kind"] == "handshake"
            request = {"protocol":"yai.studio.application.v1", "operation_ref":operation,
                       "correlation_ref":f"test:{order}", "input":data}
            connection.sendall((json.dumps({"kind":"application_request","request":request})+"\n").encode())
            if lose_response:
                print(json.dumps({"run_id":root.name,"order":order,"request":request,
                    "action":"close_without_reading_response"}), flush=True)
                order += 1
                return None
            response = json.loads(reader.readline())
            order += 1
            print(json.dumps({"run_id":root.name,"order":order,"request":request,"response":response}), flush=True)
            assert response["kind"] == "application_response", response
            result = response["result"]
            if expected is None:
                return result
            assert result["result_state"] == expected, result
            return result["data"]

    success = False
    try:
        cli("host", "start", "--json")
        app("identity.bootstrap", {"tenant_id":"tenant:active", "organization_ref":"organization:cli-product"})
        app("case.create", {"tenant_id":"tenant:active", "case_ref":"case:active"})
        base = {"case_ref":"case:active", "participant_ref":"participant:operator"}
        app("participant.role.add", dict(base, role="operation-proposer"))
        app("participant.principal.link", dict(base, principal_ref="self"))
        app("participant.view.admit", dict(base, consumer="model", view_kind="model_context"))
        if options.source_crash_in_flight:
            binding = {"schema":"yai.local_resource_access_binding.v1", "case_id":"case:active",
                "attachment_id":"source-http", "address":{"kind":"http_service",
                    "endpoint":{"endpoint":f"http://127.0.0.1:{server.server_port}",
                        "allowed_ip_addresses":["127.0.0.1"], "credential_ref":None},
                    "paths":{"document":"document"}}}
            digest = "sha256:" + hashlib.sha256(json.dumps(binding, separators=(",", ":")).encode()).hexdigest()
            app("resource.attach", {"binding":binding, "access":{
                "schema":"yai.resource_access.v1", "configuration_digest":digest,
                "participant_ids":[base["participant_ref"]], "operations":["http_fetch"],
                "read_prefixes":[], "names":["document"], "max_output_bytes":4096, "max_items":8},
                "policy_owner_participant_ref":base["participant_ref"], "review_requirement":"automatic"})
            policy = {"schema":"yai.policy_source_input.v4", "policy_key":"source-crash",
                "source_version":"1", "owner_ref":"organization:cli-product",
                "source_origin":{"source_system":"qualification", "source_uri":"test://source-crash"},
                "validity":{"mode":"unbounded"}, "rules":[{"kind":"operation_restriction",
                    "rule_id":"fetch", "operation_kind":"http.fetch", "resource_kind":"http_service",
                    "effect":"allow", "reason":"one bounded local source"}]}
            if options.source_review:
                app("participant.role.add", dict(base, role="source-reviewer"))
                policy["rules"].extend([
                    {"kind":"review_requirement", "rule_id":"review", "operation_kind":"http.fetch",
                     "resource_kind":"http_service", "required":True, "reason":"review before acquisition"},
                    {"kind":"authority_requirement", "rule_id":"reviewer", "operation_kind":"http.fetch",
                     "resource_kind":"http_service", "subject":"reviewer", "required_role":"source-reviewer",
                     "reason":"exact eligible reviewer"}])
            artifact = app("policy.ingest", {"tenant_id":"tenant:active",
                "source_bytes":list(json.dumps(policy).encode())})["view"]["artifact"]["artifact_id"]
            for operation in ["policy.validate", "policy.publish"]:
                app(operation, {"artifact_ref":artifact,"reason":"source crash qualification"})
            generation = app("case.summary", {"case_ref":"case:active"})["case"]["generation"]
            app("policy.case.bind", {"case_ref":"case:active", "artifact_ref":artifact,
                "expected_generation":generation, "reason":"source crash qualification"})
            declared = app("source.declare", dict(base, perimeter="qualification", logical_name="document",
                resource_ref="source-http", roles=["knowledge"], action={"action":"http_fetch", "name":"document"},
                bootstrap_policy=False, media_type="application/json"))
            source = declared["state"]["sources"][0]["declaration"]["source_id"]
            submit = dict(base, source_ref=source, attempt=1, expected_generation=declared["state"]["generation"])
            observe = dict(base, execution={"domain":"source_acquisition", "source_ref":source, "attempt":1})
            if options.source_review:
                pending = app("source.acquire", submit)
                assert pending["execution"]["posture"] == "waiting_for_review"
                assert not dispatches
                cli("host", "restart", "--json")
                assert app("source.acquire", submit)["execution"] == pending["execution"]
                review = app("case.summary", {"case_ref":"case:active"})["authority"]["reviews"][0]["id"]
                wrong = app("review.approve", dict(base, participant_ref="participant:hidden",
                    review_ref=review, reason="must refuse"), expected=None)
                assert wrong["result_state"] != "success" and not dispatches
                approved = app("review.approve", dict(base, review_ref=review, reason="qualified source read"))
                assert approved["external_effect"] is False and not dispatches
                generation = app("case.summary", {"case_ref":"case:active"})["case"]["generation"]
                resume_submission = dict(submit, expected_generation=generation,
                    previous_progress_ref=pending["execution"]["progress_ref"])
                app("source.resume", resume_submission, lose_response=True)
            else:
                app("source.acquire", submit, lose_response=True)
            assert entered.wait(10), app("execution.get", observe, expected=None)
            active = app("execution.get", observe)
            assert active["phase"] == "acquiring"
            assert active["posture"] == "unresolved", "durable admission is not proof of a live carrier"
            discovery = json.loads((home / "run/host/discovery.json").read_text())
            assert Path(discovery["yai_home"]).resolve() == home.resolve()
            print(json.dumps({"run_id":root.name,"action":"kill_source_host_in_flight",
                "pid":discovery["pid"],"process_identity":discovery["process_identity"],
                "attempt":active}), flush=True)
            os.kill(discovery["pid"], signal.SIGKILL)
            release.set()
            cli("host", "start", "--json")
            recovered = app("execution.get", observe)
            assert recovered == active, "restart invented terminal source evidence"
            retry = app("source.acquire", submit)
            assert retry["created"] is False and retry["execution"] == active
            if options.source_review:
                retry = app("source.resume", resume_submission)
                assert retry["created"] is False and retry["execution"] == active
            resume = app("source.resume", dict(submit, expected_generation=active["observed_generation"],
                previous_progress_ref=active["progress_ref"]), expected=None)
            assert resume["result_state"] != "success", resume
            replacement = app("source.acquire", dict(submit, attempt=2,
                expected_generation=active["observed_generation"]), expected=None)
            assert replacement["result_state"] != "success", replacement
            assert len(dispatches) == 1, "source restart/retry duplicated remote acquisition"
            assert app("execution.get", observe) == active
            hidden = app("execution.get", dict(observe, participant_ref="participant:hidden"), expected=None)
            assert hidden["result_state"] == "unauthorized" and hidden.get("data") is None
            success = True
            print("PASS: interrupted source attempt remains unresolved; restart/retry/resume do not redispatch")
            return
        workspace = root / "workspace"
        (workspace / "allowed").mkdir(parents=True)
        # Existing CLI is fixture setup only; Application never invokes it.
        cli("case", "attach-filesystem", "--case", "case:active", "--attachment", "workspace",
            "--root", str(workspace), "--allow-prefix", "allowed", "--policy-owner", "participant:operator", "--max-bytes", "1024")
        source = list(Path("tests/fixtures/cli-product-policy.json").read_bytes())
        artifact = app("policy.ingest", {"tenant_id":"tenant:active","source_bytes":source})["view"]["artifact"]["artifact_id"]
        for operation in ["policy.validate", "policy.publish"]:
            app(operation, {"artifact_ref":artifact,"reason":"real Host work qualification"})
        generation = app("case.summary", {"case_ref":"case:active"})["case"]["generation"]
        app("policy.case.bind", {"case_ref":"case:active","artifact_ref":artifact,
            "expected_generation":generation,"reason":"real Host work qualification"})
        target = app("provider.register", {"tenant_id":"tenant:active", "provider_key":"host-fixture",
            "adapter":"open_ai_compatible", "endpoint":endpoint,"model_id":"local-fixture",
            "credential_ref":"none","locality":"loopback"})["target_id"]
        # Actual bounded probe, not fabricated runtime capability evidence.
        started = int(time.time()*1000)
        probe = urllib.request.Request(endpoint, data=json.dumps({"model":"local-fixture",
            "messages":[{"role":"user","content":"probe"}]}).encode(), headers={"Content-Type":"application/json"})
        with urllib.request.urlopen(probe, timeout=5) as response:
            proof = json.load(response)
        assert json.loads(proof["choices"][0]["message"]["content"])["probe"] is True
        app("provider.qualify", {"target_ref":target,"suite_ref":"test:host-active",
            "evidence":{"run_id":root.name,"target_id":target,"started_at_unix_ms":started,
                "completed_at_unix_ms":int(time.time()*1000),"transport_connected":True,
                "exact_model_addressed":proof["model"] == "local-fixture","chat_text_envelope_valid":True,
                "structured_json_object_valid":True,"usage_accounting_observed":True,
                "health_endpoint_observed":False,"extension_telemetry_observed":False,"failure_codes":[]}})
        app("provider.trust.set", {"target_ref":target,"posture":"approved"})
        app("provider.case.bind", dict(base, ordered_target_refs=[target], failover_policy="none", max_attempts_per_turn=1))
        submission = dict(base, resource_ref="workspace", submission_ref="request:active",
            task="complete one bounded task", budgets={"max_invocations":2,"max_operations":1,
            "max_semantic_units":65536,"max_resident_items":64,"max_estimated_input_units":131072,
            "max_provider_retries":0,"max_runtime_ms":10000,"stop_on_deny":True,"continue_after_malformed":False})
        admitted = app("case.run", submission)
        assert admitted["created"] is True
        observe = dict(base, execution={"domain":"runtime_work","submission_ref":"request:active"})
        assert entered.wait(10), app("execution.get", observe)
        active = app("execution.get", observe)
        assert active["state"] == "running"
        if crash_in_flight:
            discovery = json.loads((home / "run/host/discovery.json").read_text())
            assert Path(discovery["yai_home"]).resolve() == home.resolve()
            print(json.dumps({"run_id":root.name,"action":"kill_test_host_in_flight",
                "pid":discovery["pid"],"process_identity":discovery["process_identity"],
                "execution_ref":active["execution_ref"]}), flush=True)
            os.kill(discovery["pid"], signal.SIGKILL)
            release.set()
            cli("host", "start", "--json")
            deadline = time.monotonic()+10
            while True:
                recovered = app("execution.get", observe)
                assert len(dispatches) == 1, "recovery blindly redispatched the provider request"
                if recovered["state"] == "delivery_indeterminate":
                    break
                assert time.monotonic() < deadline, recovered
                time.sleep(.05)
            assert recovered["execution_ref"] == active["execution_ref"]
            assert app("case.run", submission)["created"] is False
            assert len(dispatches) == 1
            success = True
            print("PASS: abrupt in-flight Host loss remains delivery_indeterminate with one provider dispatch")
            return
        stop = dict(base, submission_ref="request:active", run_ref=active["runner"]["run_ref"])
        stopped = app("case.stop", stop)
        assert stopped["runner"]["stop_requested"] is True
        release.set()
        deadline = time.monotonic()+10
        while True:
            final = app("execution.get", observe)
            if final["state"] in {"completed", "cancelled"}:
                break
            assert time.monotonic() < deadline, final
            time.sleep(.05)
        assert len(dispatches) == 1
        cli("host", "restart", "--json")
        recovered = app("execution.get", observe)
        assert recovered["execution_ref"] == final["execution_ref"]
        assert recovered["state"] == final["state"]
        assert app("case.run", submission)["created"] is False
        assert len(dispatches) == 1
        success = True
        print("PASS: real supervised provider work, exact cooperative stop, Host restart, no duplicate dispatch")
    finally:
        release.set()
        subprocess.run([str(binary), "host", "stop", "--json"], env=env, capture_output=True, timeout=30)
        server.shutdown()
        server.server_close()
        if success:
            shutil.rmtree(root)
        else:
            print(f"Failure profile retained: {root}", flush=True)


if __name__ == "__main__":
    main()
