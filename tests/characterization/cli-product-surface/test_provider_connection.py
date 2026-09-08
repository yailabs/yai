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
    parser.add_argument("--workbench-qualification", action="store_true", help="External synthetic full-workbench qualification only; no SEND/effects")
    args = parser.parse_args()
    if args.workbench_qualification and not args.external:
        parser.error("--workbench-qualification requires --external")
    mode = "external_yvex" if args.external else "loopback_fixture"
    endpoint = os.environ.get("YAI_EXTERNAL_PROVIDER_BASE_URL", "")
    model = os.environ.get("YAI_EXTERNAL_PROVIDER_MODEL", "")
    if args.external and not (endpoint and model):
        print(json.dumps(dict(provider_mode=mode, posture="DEPLOYMENT_LIMITATION", external_request_executed=False)))
        return 3
    with tempfile.TemporaryDirectory(prefix="yai-connect-product-") as directory:
        run = Path(directory)
        env = dict(os.environ, YAI_HOME=str(run / "home"), TERM="xterm-256color")
        order = 0
        def record(**value):
            nonlocal order
            order += 1
            data = dict(run_id=run.name, order=order, provider_mode=mode, cwd=str(ROOT), yai_home=env["YAI_HOME"], **value)
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
                provider = subprocess.Popen([sys.executable, str(ROOT / "tests/fixtures/provider_governance_server.py"), "--mode", "string_text_only", "--model", model, "--requests", "64"], stdout=subprocess.PIPE, text=True)
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
                    if needle.startswith('"connection_profile":') and b"case_connect_mechanical_contract_unqualified" in pending:
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
                expect("case_prompt: entered", "/connect" if args.external and not args.workbench_qualification else "/connect workbench")
                if not args.external:
                    expect("Public provider endpoint", endpoint)
                    expect("Exact provider-exposed model", model)
                    expect("Locality:", "loopback")
                    expect("Credential reference only", "none")
                    expect("Your semantic suitability", "fixture-not-semantic-certification")
                    expect("Type approve", "approve")
                    expect("Replace an existing primary", "no")
                    expect("no trust or Case binding added")
                    history = cli("case", "history", "case:connect", "--json")
                    assert "cognitive_binding" not in history and "provider_invocation_started" not in history
                    assert b"http_400:string_text_only" in output
                    record(claim="full workbench refuses missing functions/JSON; no silent downgrade, no Case binding or dispatch")
                    os.write(master, b"/connect\r")
                expect("Public provider endpoint", endpoint)
                expect("Exact provider-exposed model", model)
                expect("Locality:", "loopback")
                expect("Credential reference only", "none")
                expect("Your semantic suitability", "qualification-run-only")
                expect("Type approve", "approve")
                expect("Replace an existing primary", "no")
                expect('"connection_profile": "workbench"' if args.workbench_qualification else '"connection_profile": "conversation"')
                if args.workbench_qualification:
                    os.write(master, b"/exit\r")
                    proc.wait(timeout=10)
                    record(exit=proc.returncode, result="PASS", claim="synthetic workbench mechanical qualification only; no user SEND or Golden execution")
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
