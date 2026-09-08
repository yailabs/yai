#!/usr/bin/env python3
"""Real PTY onboarding and cognitive text dispatch; explicit external opt-in."""
import argparse
import fcntl
import json
import os
from pathlib import Path
import pty
import select
import struct
import subprocess
import sys
import tempfile
import termios
import time

ROOT = Path(__file__).resolve().parents[3]

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--external", action="store_true")
    parser.add_argument("--qualification-only", action="store_true", help="External synthetic connection qualification only; no SEND/effects; verify tools and JSON explicitly")
    parser.add_argument("--scenario", choices=("single", "all_shapes", "no_text", "multiple", "auth", "empty", "malformed", "duplicate", "drift", "stale", "replace", "cancel"))
    args = parser.parse_args()
    if args.qualification_only and not args.external:
        parser.error("--qualification-only requires --external")
    if args.external and args.scenario:
        parser.error("local scenarios cannot replace an external provider")
    if not args.external and not args.scenario:
        for scenario in ("single", "all_shapes", "no_text", "multiple", "auth", "empty", "malformed", "duplicate", "drift", "stale", "replace", "cancel"):
            result = subprocess.run([sys.executable, __file__, "--scenario", scenario])
            if result.returncode:
                return result.returncode
        return 0
    mode = "external_yvex" if args.external else "loopback_fixture"
    endpoint = os.environ.get("YAI_EXTERNAL_PROVIDER_BASE_URL", "")
    model = os.environ.get("YAI_EXTERNAL_PROVIDER_MODEL", "")
    if args.external and not (endpoint and model):
        print(json.dumps(dict(provider_mode=mode, posture="DEPLOYMENT_LIMITATION", external_request_executed=False)))
        return 3
    with tempfile.TemporaryDirectory(prefix="yai-connect-product-") as directory:
        run = Path(directory)
        env = dict(os.environ, YAI_HOME=str(run / "home"), TERM="xterm-256color")
        if args.scenario == "auth":
            env["YAI_TEST_CATALOG_KEY"] = "fixture-catalog-only"
        order = 0
        def record(**value):
            nonlocal order
            order += 1
            data = dict(run_id=run.name, order=order, scenario=args.scenario, provider_mode=mode, cwd=str(ROOT), yai_home=env["YAI_HOME"], **value)
            print(json.dumps(data), flush=True)
            if os.environ.get("YAI_CONNECTION_EVIDENCE"):
                with Path(os.environ["YAI_CONNECTION_EVIDENCE"]).open("a") as stream:
                    stream.write(json.dumps(data) + "\n")
        def cli(*items):
            proc = subprocess.run(["./yai", *items], cwd=ROOT, env=env, capture_output=True, text=True, timeout=30)
            record(command=proc.args, exit=proc.returncode, stdout=proc.stdout[:6000], stdout_tail=proc.stdout[-4000:] if len(proc.stdout)>6000 else "", stdout_truncated=len(proc.stdout)>6000, stderr=proc.stderr)
            assert proc.returncode == 0, proc
            return proc.stdout
        provider = None
        try:
            if not args.external:
                model = "vision-whisper-only-a-name"
                fixture_mode = "capabilities" if args.scenario == "all_shapes" else "malformed" if args.scenario == "no_text" else "string_text_only"
                fixture_args = [sys.executable, str(ROOT / "tests/fixtures/provider_governance_server.py"), "--mode", fixture_mode, "--model", model, "--requests", "64", "--log", str(run / "requests.jsonl")]
                if args.scenario == "multiple":
                    fixture_args += ["--catalog-model", "a-decoy-name-is-not-ranking"]
                if args.scenario == "auth":
                    fixture_args += ["--catalog-auth"]
                if args.scenario in ("empty", "malformed", "duplicate", "drift"):
                    fixture_args += ["--catalog-mode", args.scenario]
                provider = subprocess.Popen(fixture_args, stdout=subprocess.PIPE, text=True)
                endpoint = "http://127.0.0.1:" + provider.stdout.readline().strip()
            record(pre_state="fresh disposable Case home; no operator state; no Case effects", endpoint=endpoint, model=model,
                   yai_sha=subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip())
            cli("init", "--tenant", "tenant:connect", "--organization", "organization:connect")
            cli("case", "create", "case:connect", "--tenant", "tenant:connect")
            master, slave = pty.openpty()
            fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 32, 160, 0, 0))
            def acquire():
                os.setsid()
                fcntl.ioctl(0, termios.TIOCSCTTY, 0)
            proc = subprocess.Popen(["./yai", "open", "connect"], cwd=ROOT, env=env, stdin=slave, stdout=slave, stderr=slave, preexec_fn=acquire)
            record(command=["./yai", "open", "connect"], pre_state="Case exists without Participants/provider")
            output = bytearray()
            offset = 0
            def expect(needle, answer=None, timeout=None):
                nonlocal offset
                if timeout is None:
                    timeout = int(env.get("YAI_PROVIDER_RESPONSE_TIMEOUT_SECS", "300")) + 60 if args.external else 20
                deadline = time.monotonic() + timeout
                while needle.encode() not in output[offset:]:
                    if select.select([master], [], [], .05)[0]:
                        output.extend(os.read(master, 65536))
                    assert proc.poll() is None and time.monotonic() < deadline, output[offset:].decode(errors="replace")
                    pending = bytes(output[offset:])
                    if needle == 'conversation_execution: "completed"' and b'conversation_execution: "' in pending:
                        raise AssertionError(pending.decode(errors="replace"))
                    if needle == '"connection": "connected"' and b"case_connect_mechanical_contract_unqualified" in pending:
                        raise AssertionError(pending.decode(errors="replace"))
                # Wait for the actual editor, not just the preceding printed
                # label, before submitting the next answer.
                time.sleep(.08)
                if select.select([master], [], [], .01)[0]:
                    output.extend(os.read(master, 65536))
                end = output.index(needle.encode(), offset) + len(needle.encode())
                record(expected=needle, response=answer, output_hex=bytes(output[offset:end]).hex())
                offset = end
                if answer is not None:
                    os.write(master, answer.encode() + b"\r")
            try:
                expect("Operator Participant", "")
                expect("Model Participant", "")
                expect("Type admit", "admit")
                expect("case_prompt: entered", "/help all")
                expect("/thread status")
                assert b"/connect workbench" not in output, "help/completion must expose one connection action"
                if args.scenario == "single":
                    os.write(master, b"/connect workbench\r")
                    expect("connect_syntax:")
                    assert not (run / "requests.jsonl").exists()
                    record(claim="removed suffix is neither completion nor executable alias; malformed command does not probe")
                os.write(master, b"/connect\r")
                expect("Public provider endpoint", endpoint)
                if args.scenario in ("empty", "malformed", "duplicate"):
                    expect("provider_catalog_empty" if args.scenario == "empty" else "provider_catalog_invalid")
                    assert "cognitive_binding" not in cli("case", "history", "case:connect", "--json")
                    assert not (run / "requests.jsonl").exists(), "catalog failure must not invoke inference"
                    os.write(master, b"/exit\r")
                    proc.wait(timeout=10)
                    record(result="PASS", claim="invalid/empty catalog fails before inference, target registration or Case binding")
                    return 0
                if args.scenario == "multiple":
                    expect("Choose an exact name or number", "2")
                if args.scenario == "auth":
                    expect("Credential reference env:NAME only", "env:YAI_TEST_CATALOG_KEY")
                expect("Type approve")
                assert ("Selected model: " + model).encode() in output, "exact operator-supplied external model must match discovery"
                assert b"Your semantic suitability attestation reference" not in output
                assert b"Locality: loopback /" not in output, "literal loopback requires no locality question"
                if args.scenario == "cancel":
                    before = cli("case", "history", "case:connect", "--json")
                    targets = cli("provider", "list", "--tenant", "tenant:connect", "--json")
                    os.write(master, b"no\r")
                    expect("setup_cancelled_no_approval", "/exit")
                    proc.wait(timeout=10)
                    assert cli("case", "history", "case:connect", "--json") == before
                    assert cli("provider", "list", "--tenant", "tenant:connect", "--json") == targets
                    assert not (run / "requests.jsonl").exists()
                    record(result="PASS", claim="catalog GET and cancelled combined consent grant no trust/attestation/target or Case binding; zero inference")
                    return 0
                if args.scenario == "stale":
                    cli("case", "participant", "role", "add", "case:connect", "--participant", "participant:operator", "--role", "discovery-test-generation")
                os.write(master, b"approve\r")
                if args.scenario in ("drift", "stale"):
                    expect("exact_model_not_in_current_catalog" if args.scenario == "drift" else "case_connect_approval_stale")
                    assert "cognitive_binding" not in cli("case", "history", "case:connect", "--json")
                    assert not (run / "requests.jsonl").exists(), "stale catalog or approval must not dispatch inference"
                    os.write(master, b"/exit\r")
                    proc.wait(timeout=10)
                    record(result="PASS", claim="stale catalog/Case approval refuses before inference and binding")
                    return 0
                if args.scenario == "no_text":
                    expect("no trust or Case binding added", "/exit")
                    proc.wait(timeout=10)
                    assert "cognitive_binding" not in cli("case", "history", "case:connect", "--json")
                    assert all(row["synthetic"] for row in map(json.loads, (run / "requests.jsonl").read_text().splitlines()))
                    record(result="PASS", claim="unqualified text refuses connection; no trust, Case binding or semantic dispatch")
                    return 0
                expect('"connection": "connected"')
                assert b'"connection_profile"' not in output
                if not args.external:
                    available = b"true" if args.scenario == "all_shapes" else b"false"
                    assert b'"native_functions": ' + available in output
                    assert b'"json_object": ' + available in output
                    assert b'"text": true' in output
                    record(claim="one connect reports only independently qualified capabilities", functions_and_json=args.scenario == "all_shapes")
                if args.scenario == "replace":
                    before = cli("case", "history", "case:connect", "--json")
                    os.write(master, b"/connect\r")
                    expect("Public provider endpoint", endpoint)
                    expect("Type replace", "no")
                    expect("setup_cancelled_no_approval")
                    assert cli("case", "history", "case:connect", "--json") == before
                    os.write(master, b"/connect\r")
                    expect("Public provider endpoint", endpoint)
                    expect("Type replace", "replace")
                    expect('"connection": "connected"')
                    assert cli("case", "history", "case:connect", "--json") != before
                if args.qualification_only:
                    assert b'"native_functions": true' in output and b'"json_object": true' in output
                    os.write(master, b"/exit\r")
                    proc.wait(timeout=10)
                    record(exit=proc.returncode, result="PASS", claim="single connect synthetic text/functions/JSON qualification only; no user SEND or Golden execution")
                    assert proc.returncode == 0
                    return 0
                history = cli("case", "history", "case:connect", "--json")
                assert "case_cognitive" in history or "cognitive_binding" in history
                os.write(master, b"Reply briefly in Italian: ciao.\r")
                expect("conversation_turn:")
                # Independent canonical read while inference is in flight.
                history = cli("case", "history", "case:connect", "--json")
                assert "conversation_turn_committed" in history
                expect('conversation_execution: "completed"', "/exit")
                proc.wait(timeout=10)
                record(exit=proc.returncode, output_hex=bytes(output[offset:]).hex())
                assert proc.returncode == 0
                history = cli("case", "history", "case:connect", "--json")
                assert "provider_invocation_started" in history and "provider_result_recorded" in history
                cli("case", "verify", "case:connect", "--json")
                record(result="PASS", claim="real PTY connect -> canonical SEND -> exact cognitive realization -> ProviderResult; text only, not Golden tools or human acceptance",
                       human_golden_case="PENDING_OPERATOR")
            finally:
                if proc.poll() is None:
                    record(failure_output_hex=bytes(output[offset:]).hex())
                    proc.terminate()
                    proc.wait(timeout=5)
                os.close(master)
                os.close(slave)
        finally:
            if provider:
                provider.terminate()
                provider.wait(timeout=5)
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
