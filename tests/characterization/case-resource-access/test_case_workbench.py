#!/usr/bin/env python3
"""Real Product workbench / loopback native functions. Not full Golden or human PASS."""
import fcntl
import json
import os
from pathlib import Path
import pty
import re
import select
import subprocess
import struct
import sys
import tempfile
import termios
import time

ROOT = Path(__file__).resolve().parents[3]
CASE, TENANT = "case:workbench", "tenant:workbench"
HUMAN, MODEL = "participant:human", "participant:model"


def main():
    with tempfile.TemporaryDirectory(prefix="yai-workbench-") as directory:
        run = Path(directory)
        env = dict(os.environ, YAI_HOME=str(run / "home"), TERM="xterm-256color")
        records = []
        evidence_path = os.environ.get("YAI_WORKBENCH_TEST_EVIDENCE")
        if evidence_path:
            Path(evidence_path).open("x").close()

        def observe(**value):
            records.append(dict(run_id=run.name, order=len(records) + 1, **value))
            evidence = os.environ.get("YAI_WORKBENCH_TEST_EVIDENCE")
            if evidence:
                Path(evidence).write_text(json.dumps(records, indent=2))

        observe(proof_class="product", provider_mode="loopback_fixture", cwd=str(ROOT),
                pre_state="fresh disposable home; no YVEX; real Product executable and PTY",
                yai_home=env["YAI_HOME"], baseline=subprocess.check_output(
                    ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip())

        def cli(*args, reject=False):
            command = ["./yai", *args]
            result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True,
                                    text=True, timeout=30)
            observe(command=command, exit=result.returncode, stdout=result.stdout, stderr=result.stderr)
            assert (result.returncode != 0) == reject, result
            return result.stdout

        def field(text, name):
            return re.search(r"^" + re.escape(name) + r": (.+)$", text, re.M)[1]

        cli("init", "--tenant", TENANT, "--organization", "organization:workbench")
        cli("case", "create", CASE, "--tenant", TENANT)
        for participant, role in [(HUMAN, "human-reviewer"), (MODEL, "model-executor")]:
            cli("case", "participant", "role", "add", CASE, "--participant", participant, "--role", role)
        cli("case", "participant", "link-principal", CASE, "--principal", "self", "--participant", HUMAN)
        cli("case", "participant", "view", "admit", CASE, "--participant", MODEL, "--consumer", "model", "--view", "model_context")
        workspace = run / "workspace"
        (workspace / "src").mkdir(parents=True)
        (workspace / "src/retry.txt").write_text("real source evidence, not an assistant assertion")
        definition = run / "resource.json"
        definition.write_text(json.dumps({"schema":"yai.resource_definition.v1", "attachment_id":"resource:workspace",
            "policy_owner":HUMAN, "participant_ids":[HUMAN,MODEL], "operations":["filesystem_read"],
            "read_prefixes":["src"],"names":[],"max_output_bytes":8192,"max_items":16,
            "address":{"kind":"filesystem","root":str(workspace)}}))
        policy = run / "policy.json"
        policy.write_text(json.dumps({"schema":"yai.policy_source_input.v4", "policy_key":"workbench-read", "source_version":"1",
            "owner_ref":"organization:workbench", "source_origin":{"source_system":"product-qualification", "source_uri":"test://workbench/policy"},
            "validity":{"mode":"unbounded"}, "rules":[{"kind":"operation_restriction", "rule_id":"read",
            "operation_kind":"filesystem.read", "resource_kind":"filesystem", "effect":"allow", "reason":"exact admitted source"}]}))
        release, log = run / "release", run / "provider.jsonl"
        server = subprocess.Popen([sys.executable,str(ROOT / "tests/fixtures/provider_governance_server.py"),
            "--mode","capabilities","--model","vision-whisper-name-is-not-authority","--requests","64",
            "--release-file",str(release),"--log",str(log)], stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
        terminal = None
        master = slave = None
        try:
            port = server.stdout.readline().strip()
            assert port.isdigit(), port
            cli("case","workbench",CASE,"--participant",HUMAN,"--executor",MODEL,"--json",reject=True)
            master, slave = pty.openpty()
            fcntl.ioctl(slave,termios.TIOCSWINSZ,struct.pack("HHHH",24,100,0,0))
            before = termios.tcgetattr(slave)
            output = bytearray()

            def acquire():
                os.setsid()
                fcntl.ioctl(0,termios.TIOCSCTTY,0)

            command = ["./yai","case","workbench",CASE,"--participant",HUMAN,"--executor",MODEL]
            terminal = subprocess.Popen(command,cwd=ROOT,env=env,stdin=slave,stdout=slave,stderr=slave,preexec_fn=acquire)

            def wait(predicate, timeout=15):
                end = time.monotonic() + timeout
                while time.monotonic() < end:
                    if select.select([master],[],[],0.05)[0]:
                        output.extend(os.read(master,65536))
                    if predicate():
                        return
                    assert terminal.poll() is None, output.decode(errors="replace")
                raise AssertionError(output.decode(errors="replace"))

            def send(value):
                observe(terminal_input=value)
                os.write(master,value.encode())

            wait(lambda: b"\x1b[?2004h" in output)
            send(f"/attach {definition}\r")
            wait(lambda: b'"attachment_is_permission": false' in output)
            send(f"/policy publish {policy} explicit reference publication\r")
            wait(lambda: b'"readiness": "ready"' in output)
            send(f"/connect http://127.0.0.1:{port} vision-whisper-name-is-not-authority --trust approve --attest evidence:deterministic-native-fixture\r")
            wait(lambda: b'"semantic_posture": "operator_attested"' in output)
            target = re.search(rb'"target_id": "([^"]+)"',output)[1].decode()
            send("/work Inspect the admitted source and report the evidence\r")
            wait(lambda: b"conversation_turn:" in output)
            turn_id = re.search(rb"conversation_turn: ([^\r\n]+)",output)[1].decode()
            # The external peer is waiting; an independent process sees SEND.
            turns = cli("case","conversation","turn","list",CASE,"--participant",HUMAN,"--json")
            assert turn_id in turns and '"participant_id":"participant:human"' in turns.replace(" ","")
            assert "provider_result_recorded" not in cli("case","history",CASE,"--json")
            release.touch()
            wait(lambda: b'conversation_execution: "completed"' in output)
            assert b"case-work-complete: exact source observation received" in output
            assert target.encode() in output
            sends = [json.loads(line) for line in log.read_text().splitlines() if not json.loads(line)["synthetic"]]
            assert len(sends) == 2, sends
            offset = len(output)
            send("/retry " + turn_id + "\r")
            wait(lambda: b'conversation_execution: "completed"' in output[offset:])
            assert len([line for line in log.read_text().splitlines() if not json.loads(line)["synthetic"]]) == 2
            for action, expected in [("/case",b'"executor": "participant:model"'),("/history",b"transition_ledger"),("/verify",b'"replay_equal": true')]:
                offset = len(output)
                send(action + "\r")
                wait(lambda: expected in output[offset:])
            send("/exit\r")
            terminal.wait(timeout=10)
            assert terminal.returncode == 0
            assert termios.tcgetattr(slave) == before
            observe(command=command, exit=terminal.returncode, output_hex=output.hex(), turn_id=turn_id,
                    exact_target=target, semantic_dispatches=2, duplicate_retry_dispatches=0,
                    human_author_model_executor_separate=True, termios_restored=True)
            print("workbench_product: PASS provider_mode=loopback_fixture native_REPLAI=true SEND_before_provider=true exact_native_feedback=true retry_no_redispatch=true replay_equal=true HUMAN_GOLDEN_CASE=PENDING_OPERATOR")
        finally:
            if terminal and terminal.poll() is None:
                terminal.terminate()
                terminal.wait(timeout=5)
            if terminal:
                observe(terminal_final_exit=terminal.returncode, output_hex=output.hex())
            for fd in [master,slave]:
                if fd is not None:
                    os.close(fd)
            server.terminate()
            server.wait(timeout=5)


if __name__ == "__main__":
    main()
