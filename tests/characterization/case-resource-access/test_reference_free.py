#!/usr/bin/env python3
"""Golden free/Workflow Product proof; not external or human acceptance."""
import fcntl
import hashlib
import json
import os
from pathlib import Path
import pty
import re
import select
import struct
import subprocess
import sys
import tempfile
import termios
import time
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parents[3]
REFERENCE = ROOT / "tests/cases/04-golden"
CASE = "case:golden:free"
# One authenticated Principal has one Participant per Case. The human operator
# holds the reviewer role; the model is a different Participant with no link.
OPERATOR, MODEL, REVIEWER = "participant:operator", "participant:model", "participant:operator"


def main():
    options = sys.argv[1:]
    if len(set(options)) != len(options) or set(options) - {"--workflow", "--external"}:
        raise SystemExit("usage: test_reference_free.py [--workflow] [--external]")
    workflow, external = "--workflow" in options, "--external" in options
    endpoint = os.environ.get("YAI_EXTERNAL_PROVIDER_BASE_URL", "") if external else ""
    model = os.environ.get("YAI_EXTERNAL_PROVIDER_MODEL", "") if external else "whisper-vision-name-is-not-authority"
    if external and (not endpoint or not model):
        print(json.dumps({"provider_mode":"external_yvex","posture":"DEPLOYMENT_LIMITATION",
            "external_request_attempted":False,"reason":"operator endpoint and exact model required; no fixture fallback"}))
        raise SystemExit(3)
    if external and (any(c.isspace() or ord(c)<32 for c in endpoint + model)):
        raise SystemExit("endpoint/model must be exact single-token identities")
    if external:
        address = urlsplit(endpoint)
        if address.scheme not in ("http", "https") or not address.hostname or address.username or address.password or address.query or address.fragment:
            raise SystemExit("external endpoint must contain no credentials query or fragment")
    case_id = "case:golden:workflow" if workflow else CASE
    with tempfile.TemporaryDirectory(prefix="yai-golden-free-") as directory:
        run = Path(directory)
        env = dict(os.environ, YAI_HOME=str(run / "home"), TERM="xterm-256color")
        evidence = os.environ.get("YAI_GOLDEN_FREE_EVIDENCE")
        if not evidence and os.environ.get("YAI_GOLDEN_EVIDENCE_PREFIX"):
            evidence = os.environ["YAI_GOLDEN_EVIDENCE_PREFIX"] + ("-workflow.jsonl" if workflow else "-free.jsonl")
        if evidence:
            Path(evidence).open("x").close()
        order = 0

        def record(**value):
            nonlocal order
            order += 1
            value = dict(run_id=run.name, order=order, cwd=str(ROOT), yai_home=env["YAI_HOME"], **value)
            if evidence:
                with Path(evidence).open("a") as stream:
                    stream.write(json.dumps(value) + "\n")
            print(json.dumps(value), flush=True)

        record(proof_class="external" if external else "product", provider_mode="external_yvex" if external else "loopback_fixture",
               pre_state="fresh home and independent reference world; no human acceptance",
               endpoint=endpoint if external else "loopback fixture",model=model,
               provider_ref=os.environ.get("YAI_EXTERNAL_PROVIDER_REF") if external else None,
               ref_provenance="operator_supplied_not_source_inspected" if external else None)

        def cli(*args, reject=False):
            result = subprocess.run(["./yai", *args], cwd=ROOT, env=env, capture_output=True, text=True, timeout=45)
            record(command=["./yai", *args], exit=result.returncode, stdout=result.stdout[:8192], stderr=result.stderr[:4096])
            assert (result.returncode != 0) == reject, result
            return result.stdout

        services = subprocess.Popen([sys.executable, str(REFERENCE / "world.py"), "serve", "--port", "0"], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        provider = encoder = terminal = None
        master = slave = None
        output = bytearray()
        try:
            service = json.loads(services.stdout.readline())
            preparation = subprocess.run([sys.executable, str(REFERENCE / "world.py"), "prepare", "--root", str(run / "world"), "--endpoint", f'http://127.0.0.1:{service["port"]}'], capture_output=True, text=True, timeout=20)
            record(command=preparation.args, exit=preparation.returncode, stdout=preparation.stdout, stderr=preparation.stderr)
            assert preparation.returncode == 0
            world = run / ("world/workflow" if workflow else "world/free")
            cli("init", "--tenant", "tenant:golden", "--organization", "organization:golden")
            for case in [CASE, "case:golden:workflow", "case:golden:isolation"]:
                cli("case", "create", case, "--tenant", "tenant:golden")
            log = run / "provider.jsonl"
            if not external:
                provider = subprocess.Popen([sys.executable,str(ROOT / "tests/fixtures/provider_governance_server.py"),"--mode","golden",
                    "--model",model,"--requests","128","--log",str(log)], stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
                port = provider.stdout.readline().strip()
                assert port.isdigit()
                endpoint = f"http://127.0.0.1:{port}"

            def dispatch_count():
                if not external:
                    return len([v for v in log.read_text().splitlines() if not json.loads(v)["synthetic"]])
                # No access to provider internals. Canonical YAI invocation
                # lineage, not endpoint presence, proves execution attempts.
                data = json.loads(cli("case","history",case_id,"--limit","256","--json"))
                history = data["data"]["value"]
                assert history["total_transitions"] <= 256, "external proof exceeds retained inspection bound"
                return sum(t["payload"]["kind"] == "provider_invocation_started" for t in history["transitions"])

            locality, credential = "loopback", "none"
            if external:
                locality = os.environ.get("YAI_EXTERNAL_PROVIDER_LOCALITY", "private_network")
                if locality not in ("loopback", "private_network", "remote"):
                    raise AssertionError("invalid explicit provider locality")
                if os.environ.get("YAI_EXTERNAL_PROVIDER_API_KEY"):
                    credential = "env:YAI_EXTERNAL_PROVIDER_API_KEY"

            def open_workbench(selected_case=None, first=False):
                nonlocal terminal, master, slave, output
                output = bytearray()
                master, slave = pty.openpty()
                fcntl.ioctl(slave,termios.TIOCSWINSZ,struct.pack("HHHH",32,120,0,0))
                def acquire():
                    os.setsid()
                    fcntl.ioctl(0,termios.TIOCSCTTY,0)
                command = ["./yai","open",selected_case or case_id]
                terminal = subprocess.Popen(command,cwd=ROOT,env=env,stdin=slave,stdout=slave,stderr=slave,preexec_fn=acquire)
                record(command=command, pre_state="canonical Case retained; new terminal process")
                if first:
                    wait(b"Operator Participant name")
                    action("",b"Model Participant name")
                    action("",b"Type admit")
                    action("admit",b"Case opened:")
                else:
                    wait(b"Case opened:")

            def wait(needle, offset=0, timeout=30):
                needles = (needle,) if isinstance(needle, bytes) else needle
                if external:
                    timeout = max(timeout, 300)
                end = time.monotonic() + timeout
                while time.monotonic() < end:
                    if select.select([master],[],[],0.03)[0]:
                        output.extend(os.read(master,65536))
                    if any(item in output[offset:] for item in needles):
                        return bytes(output[offset:])
                    execution_wait = needle in (b"Response received.", b"Human review required.")
                    if execution_wait and any(marker in output[offset:] for marker in (b"Execution unresolved:", b"Execution unavailable", b"Execution failed:")):
                        record(failure="different execution posture",output_tail_hex=bytes(output[-4096:]).hex())
                        raise AssertionError(output[-4096:].decode(errors="replace"))
                    if execution_wait and b"\x1b[?2004h" in output[offset:] and b"Message saved." in output[offset:]:
                        # An application refusal returned to the editor without
                        # an execution record; don't burn a full timeout.
                        suffix = bytes(output[offset:])
                        if suffix.rfind(b"\x1b[?2004h") > suffix.rfind(b"Message saved."):
                            record(failure="application refused execution",output_tail_hex=bytes(output[-4096:]).hex())
                            raise AssertionError(output[-4096:].decode(errors="replace"))
                    assert terminal.poll() is None, output.decode(errors="replace")
                record(failure="expected terminal outcome absent",expected=[item.decode(errors="replace") for item in needles],output_tail_hex=bytes(output[-16384:]).hex())
                raise AssertionError(output[-16384:].decode(errors="replace"))

            def action(value, expected, timeout=30):
                offset = len(output)
                record(terminal_action=value)
                os.write(master,(value + "\r").encode())
                data = wait(expected,offset,timeout)
                record(output_hex=data.hex())
                return data

            def refusal(value, code):
                action(value, b"The action could not be completed.")
                detail_bytes = action("/details", b"[/YAI details]")
                assert code.encode() in detail_bytes

            def close_workbench():
                nonlocal terminal, master, slave
                os.write(master,b"/exit\r")
                # A real terminal continues consuming output. Stopping reads
                # while a large inspection is printing can fill the PTY and
                # prevent the host from ever reaching the queued /exit.
                deadline = time.monotonic() + 10
                while terminal.poll() is None and time.monotonic() < deadline:
                    if select.select([master],[],[],0.03)[0]:
                        try:
                            output.extend(os.read(master,65536))
                        except OSError:
                            break
                terminal.wait(timeout=1)
                record(terminal_exit=terminal.returncode, output_tail_hex=bytes(output[-8192:]).hex())
                assert terminal.returncode == 0
                os.close(master)
                os.close(slave)
                master = slave = None
                terminal = None

            def connect():
                action("/connect",b"Public provider endpoint")
                questions = (b"Type approve", b"Locality: loopback /", b"Credential reference env:NAME only", b"Choose an exact name or number", b"provider_catalog_")
                data = action(endpoint,questions)
                for _ in range(4):
                    if b"Type approve" in data:
                        assert ("Selected model: " + model).encode() in data
                        connected = action("approve",b'Connected to ')
                        assert b'native functions: qualified' in connected and b'JSON: qualified' in connected, "Golden requires independently qualified functions and JSON, not merely a connected target"
                        evidence = action("/details",b"[/YAI details]")
                        assert b'"native_functions": true' in evidence and b'"json_object": true' in evidence
                        return
                    if b"Locality: loopback /" in data:
                        data = action(locality,questions)
                    elif b"Credential reference env:NAME only" in data:
                        assert credential != "none", "external authentication unavailable; no fixture fallback"
                        data = action(credential,questions)
                    elif b"Choose an exact name or number" in data:
                        data = action(model,questions)
                    else:
                        raise AssertionError(data.decode(errors="replace"))
                raise AssertionError("unbounded connection dialogue")

            open_workbench(first=True)
            for definition in sorted((world / "attachments").glob("*.json")):
                action("/attach",b"Resource definition file")
                action(str(definition),b'"attachment_is_permission": false')
            action("/policy publish",b"Policy source file")
            action(str(REFERENCE / "policy.json"),b"Publication reason")
            action("publish exact Golden reference deck", b'"readiness": "ready"')
            discovery = action("/discover discovery issue", b'"reused": false')
            digest = re.search(rb'"digest": "(sha256:[a-f0-9]+)"',discovery)[1].decode()
            action("/admit",b"Discovery resource")
            action("discovery",b"Exact discovered candidate digest")
            action(digest,b"Exact discovered relative path")
            action("issue/issue.md",b'"reused": false')
            # Provenance remains the admitted immutable bytes, even after source drift.
            (world / "material/issue/issue.md").write_text("Changed external source must not replace admitted issue.")
            connect()
            initial = hashlib.sha256((world / "workspace/src/retry.py").read_bytes()).hexdigest()
            if workflow:
                action("/workflow bind",b"Workflow definition file")
                action(str(REFERENCE / "workflow.json"),b'"workflow_binding": {')
                action("/workflow input understand GOLDEN-42 reviewed; investigate with admitted capabilities",b'"workflow_input": "committed"')
                action("/workflow run investigate",b'Response received.',timeout=60)
                assert hashlib.sha256((world / "workspace/src/retry.py").read_bytes()).hexdigest() == initial
                close_workbench()
                investigation_calls = dispatch_count()
                open_workbench()
                action("/workflow run investigate",b'Response received.',timeout=60)
                assert dispatch_count() == investigation_calls
                assert external or investigation_calls == 11
                action("/workflow run risk-plan",b'Response received.',timeout=60)
                data = action("/workflow",b'"schema": "yai.workflow_resolution.v2"')
                patch = re.search(rb'"patch_id": "([^"]+)"',data)[1].decode()
                assert b'"amendments": []' in data, "Model proposal must not adopt itself"
                action("/workflow patch validate " + patch,b'"valid": true')
                action("/workflow patch adopt " + patch,b'"adopted": true')
                data = action("/workflow run repair",b'Human review required.',timeout=60)
            else:
                data = action("/work Investigate GOLDEN-42 from owned issue and independent source database HTTP MCP and discovered migration evidence. Use the risk tool. Demonstrate protected-path denial and the initial failing test. Request human review for the source repair and run the exact tests after approval.",b'Human review required.',timeout=60)
            detail_bytes = action("/details", b"[/YAI details]")
            execution = json.loads(detail_bytes.split(b"[YAI details]\r\n",1)[1].split(b"\r\n[/YAI details]",1)[0])
            turn = execution["turn_id"]
            review = execution["work"]["steps"][-1]["outcome"]["review_id"]
            assert execution["posture"] == "awaiting_review"
            assert hashlib.sha256((world / "workspace/src/retry.py").read_bytes()).hexdigest() == initial
            calls_before = dispatch_count()
            assert external or calls_before == (15 if workflow else 13), calls_before
            refusal(f"/review approve {review} {MODEL} self approval is forbidden", "authenticated_principal_participant_link_required")
            close_workbench()
            open_workbench()
            pending = action("/reviews", b'"status": "pending"')
            operation = re.search(rb'"operation_id": "([^"]+)"', pending)[1].decode()
            inspected = action("/operation " + operation, b'"causal_transitions": [')
            assert b"src/retry.py" in inspected, "Reviewer must inspect the exact proposed source mutation"
            reviewed = action("/review",b"Review this exact operation:")
            assert b"src/retry.py" in reviewed
            action("approve",b"Review reason")
            action("reviewed source against independent release evidence",b"review_action: committed")
            assert hashlib.sha256((world / "workspace/src/retry.py").read_bytes()).hexdigest() == initial, "Review itself must not execute effect"
            action("/retry",b'Response received.',timeout=60)
            action("/verify",b'"replay_equal": true')
            if not external:
                assert "min(250" in (world / "workspace/src/retry.py").read_text()
            assert "Protected reference" in (world / "workspace/protected/release.conf").read_text()
            calls_after = dispatch_count()
            assert external or calls_after == (18 if workflow else 15), calls_after
            if external:
                verified = action("/test resource:runner tests",b'"reused": false')
                assert b'"exit_code": 0' in verified, "real external-model repair must pass the governed software oracle"
            action("/retry " + turn,b'Response received.',timeout=60)
            assert calls_after == dispatch_count()
            if workflow:
                action("/workflow advance",b'"schema": "yai.workflow_resolution.v2"')
                action("/workflow input verify-history inspected exact review effect and process receipts",b'"workflow_input": "committed"')
                action("/workflow advance",b'"schema": "yai.workflow_resolution.v2"')
                action("/workflow input verify actual passing test and replay verified",b'"workflow_input": "committed"')
                action("/workflow advance",b'"completed": true')
                action("/verify",b'"replay_equal": true')
            action("/rebuild", b'"canonical_history_unchanged": true')
            action("/memory", b'"authority": "derived"')
            action("/graph", b'"authority": "derived"')
            if not workflow:
                # A legal Handoff conveys explicit bounded material, never the
                # source resource/provider authority. The target starts empty.
                isolation = "case:golden:isolation"
                cli("init","--tenant","tenant:golden-other","--organization","organization:golden-other")
                cli("case","create","case:golden:other-tenant","--tenant","tenant:golden-other")
                refusal("/handoff offer case:golden:other-tenant operation-proposer This payload must not cross the Tenant boundary.", "handoff_offer_rederivation_mismatch")
                for participant, role in [(OPERATOR,"operation-proposer"),(MODEL,"model-executor")]:
                    cli("case","participant","role","add",isolation,"--participant",participant,"--role",role)
                cli("case","participant","link-principal",isolation,"--principal","self","--participant",OPERATOR)
                cli("case","participant","view","admit",isolation,"--participant",MODEL,"--consumer","model","--view","model_context")
                offered = action(f"/handoff offer {isolation} operation-proposer Independently acknowledge the exact GOLDEN-42 release finding; no workspace access is transferred.",b'"authority_transferred": false')
                handoff = re.search(rb'"handoff_id": "([^"]+)"',offered)[1].decode()
                close_workbench()
                open_workbench(isolation)
                refusal("/read workspace src/retry.py", "resource_not_attached")
                empty = action("/resources",b"[]")
                assert b"resource:workspace" not in empty
                accepted = action(f"/handoff accept {case_id} {handoff}",b'"authority_transferred": false')
                acceptance = re.search(rb'"acceptance_id": "([^"]+)"',accepted)[1].decode()
                action(f"/handoff result {handoff} succeeded {acceptance} Acknowledged bounded finding only; source resources remain unavailable.",b'"authority_transferred": false')
                # Same physical workspace and same provider do not imply model
                # visibility: this Case admits only human resource inspection.
                definition = json.loads((world / "attachments/workspace.json").read_text())
                definition["participant_ids"] = [OPERATOR]
                definition.pop("write_prefix")
                definition.pop("max_write_bytes")
                isolated_definition = run / "isolation-workspace.json"
                isolated_definition.write_text(json.dumps(definition))
                action("/attach " + str(isolated_definition),b'"attachment_is_permission": false')
                action("/policy publish " + str(REFERENCE / "policy.json") + " isolated human-only resource envelope",b'"readiness": "ready"')
                connect()
                view = action("/capabilities",b'"entries": []')
                assert b'"resource_participant_not_disclosed"' in view or b'"exclusions"' in view
                action("/verify",b'"replay_equal": true')
                close_workbench()
                open_workbench()
                action("/handoff reconcile " + handoff,b'"authority_transferred": false')
                action("/rebuild",b'"canonical_history_unchanged": true')
                action("/verify",b'"replay_equal": true')
                assert calls_after == dispatch_count()
            close_workbench()
            # Qualify the actual same-Case lifecycle independently of scripted
            # provider answers. This also prevents a real-model external run
            # from passing merely by guessing a repair and printing success.
            inspected_history = json.loads(cli("case","history",case_id,"--limit","256","--json"))["data"]["value"]
            assert inspected_history["total_transitions"] <= 256
            payloads = [t["payload"] for t in inspected_history["transitions"]]
            observations = [p["data"]["observation"] for p in payloads
                            if p["kind"] in ("resource_observation_recorded","resource_effect_finalized")]
            model_observations = [o for o in observations if o["participant_id"] == MODEL]
            observed_kinds = {o["kind"] for o in model_observations}
            assert {"content_read","filesystem_read","database_query","http_fetch","mcp_catalog",
                    "mcp_resource_read","mcp_tool_call","discover","process_run"} <= observed_kinds, observed_kinds
            admissions = [p["data"]["admission"] for p in payloads if p["kind"] == "case_content_admitted"]
            assert {"issue/issue.md","notes/migration.md"} <= {a["source_path"] for a in admissions}
            decisions = [p["data"]["decision"] for p in payloads if p["kind"] == "decision_recorded"]
            assert {"allow","deny","require_review"} <= {d["outcome"] for d in decisions}
            denied = {d["operation_id"] for d in decisions if d["outcome"] == "deny"}
            grants = [p["data"]["grant"] for p in payloads if p["kind"] == "execution_grant_issued"]
            assert not denied & {g["operation_id"] for g in grants}
            process_exits = [o["result"]["exit_code"] for o in model_observations if o["kind"] == "process_run"]
            record(process_results=[o["result"] for o in model_observations if o["kind"] == "process_run"])
            assert 0 in process_exits and any(code is not None and code != 0 for code in process_exits), process_exits
            assert any(p["kind"] == "effect_finalized" for p in payloads)
            record(canonical_lifecycle="PASS",observed_kinds=sorted(observed_kinds),
                   authority=["allow","deny","require_review"],process_exits=process_exits,
                   admission_ids=[a["admission_id"] for a in admissions],
                   no_grant_for_denied_operations=True)
            if not external:
                # Existing W19/W20 encoder administration, not another cognitive
                # owner. This extra fixture is explicitly local-only: external
                # Golden never quietly substitutes an encoder model fixture.
                encoder = subprocess.Popen([sys.executable,str(ROOT / "tests/fixtures/memory_embedding_server.py"),
                    "--model","golden-reference-encoder"],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
                encoder_port = encoder.stdout.readline().strip()
                assert encoder_port.isdigit()
                added = cli("provider","add","--tenant","tenant:golden","--provider-key","golden-reference-encoder",
                    "--endpoint",f"http://127.0.0.1:{encoder_port}","--model","golden-reference-encoder","--locality","loopback")
                encoder_target = re.search(r"^target_id: (.+)$",added,re.M)[1]
                cli("provider","qualify",encoder_target,"--embedding")
                cli("provider","trust","approve",encoder_target)
                before_index = cli("case","history",case_id,"--limit","256","--json")
                build_args = [case_id,"--encoder-target",encoder_target,"--encoder-revision","reference-fixture-v1","--dimension","4"]
                built = cli("case","memory","index","build",*build_args)
                profile = re.search(r"^representation_profile_id: (.+)$",built,re.M)[1]
                index = re.search(r"^index_manifest_id: (.+)$",built,re.M)[1]
                verified = cli("case","memory","index","verify",case_id,"--profile",profile)
                assert "posture: current" in verified
                cli("case","memory","index","drop",case_id,"--profile",profile)
                rebuilt = cli("case","memory","index","rebuild",*build_args)
                assert re.search(r"^index_manifest_id: (.+)$",rebuilt,re.M)[1] == index
                searched = cli("case","memory","search",case_id,"--participant",MODEL,"--query","release retry process",
                    "--profile",profile,"--limit","8")
                assert "plane: lexical_bm25 available:true" in searched
                assert "plane: vector_exact_cosine available:true" in searched
                assert cli("case","history",case_id,"--limit","256","--json") == before_index
                record(index_rebuild="PASS",profile_id=profile,index_id=index,canonical_history_unchanged=True,
                       provider_mode="loopback_fixture",scope="existing W19/W20 encoder and canonical Golden Case")
            record(result="PASS", property="reference work through real Product; review/restart/replay; no retry duplicates",
                   workflow=workflow, model_dispatches=calls_after,provider_mode="external_yvex" if external else "loopback_fixture",
                   turn_id=turn,review_id=review,qualification_scope="golden_workflow" if workflow else "golden_free",
                   external_request_attempted=external,human_golden_case="PENDING_OPERATOR")
        finally:
            if terminal and terminal.poll() is None:
                terminal.terminate()
                terminal.wait(timeout=5)
            for fd in [master,slave]:
                if fd is not None:
                    os.close(fd)
            for process in [provider,encoder,services]:
                if process:
                    process.terminate()
                    process.wait(timeout=5)
                    record(peer_exit=process.returncode,stderr=process.stderr.read()[:8192])


if __name__ == "__main__":
    main()
