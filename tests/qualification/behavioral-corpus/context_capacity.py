#!/usr/bin/env python3
"""Exercise exact context inspection and preflight through the real Host dispatcher.

The producer is controlled local HTTP, not external YVEX or model-quality evidence.
Only a newly allocated profile is mutated and removed after Host shutdown.
"""
import argparse
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import threading
import time
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools/validation"))
from behavioral_corpus import Host


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--yai", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--studio-url", help="Optional running isolated Studio Vite for the actual Host UI read")
    args = parser.parse_args()
    binary = str(args.yai.resolve())
    run = f"context-capacity-{time.time_ns()}"
    home = Path(tempfile.mkdtemp(prefix="yai-context-capacity-"))
    evidence = args.output.open("x")
    order = 0
    lock = threading.Lock()

    def emit(**record):
        nonlocal order
        with lock:
            order += 1
            evidence.write(json.dumps(dict(run_id=run, order=order, **record)) + "\n")
            evidence.flush()

    def cli(*parts, structured=True):
        command = [binary, *parts, *(["--json"] if structured else [])]
        result = subprocess.run(command, env={**os.environ, "YAI_HOME": str(home)},
                                cwd=ROOT, capture_output=True, text=True, timeout=45)
        emit(command=command, cwd=str(ROOT), yai_home=str(home), exit=result.returncode,
             stdout=result.stdout, stderr=result.stderr)
        assert result.returncode == 0, result.stderr
        return json.loads(result.stdout) if structured else result.stdout

    model = "controlled-context-model"
    row = dict(id=model, yvex_profile="yvex.openai.compat.v3", engine_generation=1,
               runtime_binding_identity="binding:controlled", runtime_model_identity="model:controlled",
               capacity_plan_identity="capacity:controlled", yvex_capacity=dict(
                   schema="yvex.execution.capacity.v1", input_accounting="exact_tokenizer_including_template_and_tools",
                   resource_reservation=False, http_body_bytes=1048576, runtime_input_tokens=65536,
                   runtime_sequence_tokens=65536, preflight="/v1/chat/completions/preflight"))
    dispatches, preflights = [], []
    compatible = True

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass

        def do_GET(self):
            self.respond(dict(data=[row]), 200 if self.path == "/v1/models" else 404)

        def respond(self, value, status=200):
            body = json.dumps(value).encode()
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def do_POST(self):
            body = self.rfile.read(int(self.headers.get("Content-Length", 0)))
            emit(provider_path=self.path, request_body=body.decode())
            if self.path == "/v1/chat/completions/preflight":
                preflights.append(body)
                self.respond(dict(row, object="yvex.execution.preflight", model=model,
                    scope="complete_stateless_chat_request", execution_or_resources_qualified=False,
                    tokenizer_identity="tokenizer:controlled", prompt_identity="prompt:controlled",
                    provider_request_identity="request:controlled", input_tokens=100 if compatible else 70000,
                    requested_output_tokens=0, effective_output_tokens=128 if compatible else 0,
                    full_requested_output_fits=False, token_capacity_compatible=compatible,
                    input_capacity_exceeded=not compatible, output_capacity_exceeded=False))
            elif self.path == "/v1/chat/completions":
                dispatches.append(body)
                self.respond(dict(model=model, choices=[dict(message=dict(role="assistant", content="Controlled context result"))]))
            else:
                self.respond({}, 404)

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    endpoint = f"http://127.0.0.1:{server.server_port}"
    started = False
    try:
        emit(prestate="fresh isolated profile", home=str(home), evidence_class="deterministic_local_product",
             binary_sha256=hashlib.sha256(Path(binary).read_bytes()).hexdigest())
        started = True
        cli("host", "start")
        host = Host(home)

        def call(operation, inputs, expected="success"):
            response = host.call(operation, inputs, f"{run}:{time.time_ns()}")
            emit(operation=operation, input=inputs, result=response)
            assert response["result_state"] == expected, response
            return response.get("data")

        tenant, case, participant = "tenant:context-capacity", "case:context-capacity", "participant:operator"
        scope = dict(case_ref=case, participant_ref=participant)
        call("identity.bootstrap", dict(tenant_id=tenant, organization_ref="organization:qualification"))
        call("case.create", dict(case_ref=case, tenant_id=tenant))
        call("participant.role.add", dict(scope, role="operator"))
        call("participant.principal.link", dict(scope, principal_ref="self"))
        target = call("provider.register", dict(tenant_id=tenant, provider_key="controlled-context",
            adapter="open_ai_compatible", endpoint=endpoint, model_id=model, credential_ref="none",
            locality="loopback", extension_adapter_id="yvex.http.v1"))["target_id"]
        stamp = time.time_ns() // 1000000
        with urlopen(endpoint + "/v1/models", timeout=5) as response:
            assert json.load(response)["data"][0]["id"] == model
        with urlopen(Request(endpoint + "/v1/chat/completions", data=json.dumps(dict(model=model,
            messages=[dict(role="user", content="Controlled probe")])).encode(),
            headers={"Content-Type": "application/json"}), timeout=5) as response:
            assert json.load(response)["choices"][0]["message"]["content"] == "Controlled context result"
        call("provider.qualify", dict(target_ref=target, suite_ref="suite:controlled-context", evidence=dict(
            run_id=run, target_id=target, started_at_unix_ms=stamp, completed_at_unix_ms=time.time_ns() // 1000000,
            transport_connected=True, exact_model_addressed=True, chat_text_envelope_valid=True,
            realization_shapes=["text_to_text"], structured_json_object_valid=False, usage_accounting_observed=False,
            health_endpoint_observed=False, extension_telemetry_observed=False, failure_codes=[])))
        call("provider.trust.set", dict(target_ref=target, posture="approved"))
        call("provider.case.bind", dict(scope, ordered_target_refs=[target], failover_policy="none", max_attempts_per_turn=1))
        call("participant.view.admit", dict(scope, consumer="model", view_kind="model_context"))
        suitability = call("provider.suitability.record", dict(target_ref=target, capability="primary_conversation",
            suite_ref="studio.operator.conversation.v1", run_ref=run, evidence_refs=["evidence:controlled-probe"]))
        call("cognitive.binding.set", dict(scope, role="primary", capability="primary_conversation",
            candidates=[dict(target_ref=target, semantic_evidence_ref=suitability["evidence_id"])], replace=False))

        def send(identity):
            generation = call("case.summary", dict(case_ref=case))["case"]["generation"]
            inputs = dict(scope, thread_ref="thread:context", submission_ref=identity, expected_generation=generation,
                parts=[dict(modality="text", media_type="text/plain", bytes=list(b"EXACT_CONTEXT_SENTINEL"))])
            call("conversation.send", inputs)
            query = dict(scope, execution=dict(domain="conversation", submission_ref=identity))
            deadline = time.monotonic() + 30
            while True:
                observed = call("execution.get", query)
                if observed["posture"] not in ("admitted", "running", "unresolved"):
                    break
                assert time.monotonic() < deadline, observed
                time.sleep(.1)
            return inputs, query, observed

        inputs, query, observed = send("submission:fits")
        assert observed["primary_result"]["output"] == "Controlled context result", observed
        assert len(dispatches) == 2 and len(preflights) == 1
        assert dispatches[-1] == preflights[-1]
        assert call("conversation.send", inputs)["created"] is False
        inspected = call("execution.get", dict(query, include_context=True))
        entry = inspected["prepared_context"]["invocations"][0]
        assert entry["unavailable_reason"] is None
        assert entry["working_state"] and entry["projection"] and entry["frame"]
        wire = entry["input_observation"]
        assert wire["serialized_request_digest"] == "sha256:" + hashlib.sha256(dispatches[-1]).hexdigest()
        assert wire["serialized_request_bytes"] == len(dispatches[-1])
        assert wire["capacity"]["input_tokens"] == 100
        cli_observation = cli("context", "inspect", "--id", wire["observation_id"], structured=False)
        kind, payload = cli_observation.split("\n", 1)
        assert kind == "artifact_kind: provider_input_observation"
        assert json.loads(payload) == wire, "CLI and Application disagree on exact input observation"
        if args.studio_url:
            profile = args.output.with_suffix(".ui-profile.json")
            profile.write_text(json.dumps(dict(home=str(home), execution=inspected, digest=wire["serialized_request_digest"])))
            command = ["node", str(ROOT / "tests/studio/execution-context.mjs")]
            result = subprocess.run(command, cwd=ROOT, env={**os.environ, "STUDIO_CONTEXT_PROFILE":str(profile),
                "STUDIO_TEST_URL":args.studio_url}, capture_output=True, text=True, timeout=90)
            emit(command=command, cwd=str(ROOT), exit=result.returncode, stdout=result.stdout, stderr=result.stderr)
            assert result.returncode == 0, result.stderr
        assert len(dispatches) == 2 and len(preflights) == 1, "Observation/retry redispatched"
        call("execution.get", dict(query, include_context=True, participant_ref="participant:hidden"), "unauthorized")
        compatible = False
        refused_inputs, refused_query, refused = send("submission:does-not-fit")
        assert refused["primary_result"] is None and refused["posture"] == "refused", refused
        assert len(dispatches) == 2 and len(preflights) == 2
        refusal = call("execution.get", dict(refused_query, include_context=True))["prepared_context"]["invocations"][0]
        assert refusal["input_observation"]["refusal"] == "token_capacity_exceeded"
        # no_execution_proven is reserved for a producer's definitive rejection
        # after delivery. Local preflight instead proves zero inference bytes.
        assert any(item["delivery"] == "not_dispatched" and item["request_bytes_written"] == 0
                   and item["response_status"] is None and item["failure_class"] == "request_capacity_refused"
                   and item["stage"] == "request_serialized" for item in refused["attempt_outcomes"])
        assert call("conversation.send", refused_inputs)["created"] is False
        assert len(dispatches) == 2 and len(preflights) == 2
        instance = host.discovery["instance_id"]
        cli("host", "restart")
        host = Host(home)
        assert host.discovery["instance_id"] != instance
        reopened = call("execution.get", dict(query, include_context=True))
        assert reopened["prepared_context"]["invocations"][0]["input_observation"] == wire
        assert call("conversation.send", inputs)["created"] is False
        assert len(dispatches) == 2 and len(preflights) == 2, "Restart duplicated inference"
        cli("case", "verify", case)
        emit(result="PASS", inference_dispatches=1, capacity_refusals=1, observation_retry_dispatches=0)
        print(json.dumps(dict(result="PASS", run_id=run, evidence=str(args.output))))
    finally:
        server.shutdown()
        server.server_close()
        if started:
            cli("host", "stop")
        shutil.rmtree(home)
        evidence.close()


if __name__ == "__main__":
    main()
