#!/usr/bin/env python3
"""Short Product entrypoints, real REPLAI PTY and canonical stores; no provider."""
import fcntl
import json
import os
from pathlib import Path
import pty
import select
import struct
import subprocess
import tempfile
import termios
import time

ROOT = Path(__file__).resolve().parents[3]


def main():
    with tempfile.TemporaryDirectory(prefix="yai-guided-product-") as directory:
        run = Path(directory)
        env = dict(os.environ, YAI_HOME=str(run / "home"), TERM="xterm-256color")
        order = 0
        evidence = os.environ.get("YAI_GUIDED_EVIDENCE")
        if evidence:
            Path(evidence).open("x").close()

        def record(**value):
            nonlocal order
            order += 1
            data = dict(run_id=run.name, order=order, cwd=str(ROOT), yai_home=env["YAI_HOME"], **value)
            print(json.dumps(data), flush=True)
            if evidence:
                with Path(evidence).open("a") as stream:
                    stream.write(json.dumps(data) + "\n")

        def cli(*args, reject=False):
            result = subprocess.run(["./yai", *args], cwd=ROOT, env=env, capture_output=True, text=True, timeout=30)
            record(command=result.args, exit=result.returncode, stdout=result.stdout, stderr=result.stderr)
            assert (result.returncode != 0) == reject, result
            return result.stdout + result.stderr if reject else result.stdout

        def terminal(args, steps, reject=False):
            master, slave = pty.openpty()
            fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 32, 120, 0, 0))
            before = termios.tcgetattr(slave)
            def acquire():
                os.setsid()
                fcntl.ioctl(0, termios.TIOCSCTTY, 0)
            process = subprocess.Popen(["./yai", *args], cwd=ROOT, env=env, stdin=slave, stdout=slave, stderr=slave, preexec_fn=acquire)
            record(command=["./yai", *args], pre_state="existing home retained; new PTY process")
            output = bytearray()
            offset = 0
            try:
                for needle, response in steps:
                    deadline = time.monotonic() + 15
                    while needle.encode() not in output[offset:]:
                        if select.select([master], [], [], 0.03)[0]:
                            output.extend(os.read(master, 65536))
                        assert process.poll() is None, output.decode(errors="replace")
                        assert time.monotonic() < deadline, output.decode(errors="replace")
                    # Each answer is sent only after the preceding application
                    # question, not a pasted shell script or editor shortcut.
                    record(expected=needle, response=response, output_hex=bytes(output[offset:]).hex())
                    offset = len(output)
                    os.write(master, response.encode())
                deadline = time.monotonic() + 15
                while process.poll() is None:
                    if select.select([master], [], [], 0.03)[0]:
                        output.extend(os.read(master, 65536))
                    assert time.monotonic() < deadline, output.decode(errors="replace")
                record(exit=process.returncode, output_hex=bytes(output[offset:]).hex(), terminal_restored=termios.tcgetattr(slave) == before)
                assert (process.returncode != 0) == reject, output.decode(errors="replace")
                assert termios.tcgetattr(slave) == before
                return output.decode(errors="replace")
            finally:
                if process.poll() is None:
                    process.terminate()
                    process.wait(timeout=5)
                os.close(master)
                os.close(slave)

        record(proof_class="product", provider_mode="no_provider", pre_state="fresh disposable home; no external provider; no operator home")
        cli("init", reject=True)
        error = cli("init", "--json", reject=True)
        assert '"schema":"yai.cli.error.v1"' in error and "\x1b" not in error
        cli("open", "golden:free", reject=True)
        assert not (run / "home").exists(), "noninteractive refusal must precede bootstrap"
        terminal(["init"], [("Tenant name", "\x03")], reject=True)
        assert not (run / "home").exists(), "cancelled input is not bootstrap consent"
        terminal(["init"], [("Tenant name", "golden\r"), ("Organization name", "golden\r"), ("Type create", "create\r")])
        terminal(["open", "discard"], [("Type create", "no\r")], reject=True)
        assert "case:discard" not in cli("case", "list", "--json")
        # Exactly the user's material pre-state: Case exists, no Participants.
        cli("case", "create", "case:golden:free", "--tenant", "tenant:golden")
        before = cli("case", "history", "case:golden:free", "--json")
        terminal(["open", "golden:free"], [("Operator Participant name", "\r"), ("Model Participant name", "\r"), ("Type admit", "cancel\r")], reject=True)
        assert cli("case", "history", "case:golden:free", "--json") == before
        terminal(["open", "golden:free"], [("Operator Participant name", "\r"), ("Model Participant name", "\r"), ("Type admit", "admit\r"), ("case_prompt: entered", "/exit\r")])
        history = cli("case", "history", "case:golden:free", "--json")
        assert "participant_principal_linked" in history
        assert "execution_grant_issued" not in history and "provider_invocation_started" not in history
        assert "provider_trust" not in history
        terminal(["open", "golden:free"], [("case_prompt: entered", "/retry\r"), ("no_committed_turn", "/exit\r")])
        assert cli("case", "history", "case:golden:free", "--json") == history
        terminal(["case", "open", "case:golden:free"], [("case_prompt: entered", "/connect\r"), ("Public provider endpoint", "\x03"), ("setup_cancelled_no_approval", "/exit\r")])
        assert cli("case", "history", "case:golden:free", "--json") == history
        providers = cli("provider", "list", "--tenant", "tenant:golden", "--json")
        terminal(["open", "golden:free"], [("case_prompt: entered", "/connect\r"),
            ("Public provider endpoint", "http://user:forbidden@127.0.0.1:9\r"),
            ("provider_endpoint_credentials_query_or_fragment_forbidden", "/exit\r")])
        assert cli("provider", "list", "--tenant", "tenant:golden", "--json") == providers
        terminal(["open", "golden:free"], [("case_prompt: entered", "/setup\r"), ("Model Participant", "operator\r"), ("Type admit", "admit\r"), ("setup_identity_conflict", "/exit\r")])
        assert cli("case", "history", "case:golden:free", "--json") == history
        assert "equivalent_to_replay" in cli("case", "verify", "case:golden:free", "--json")
        # One visible Case can be reopened without a name. A new tenant never
        # becomes ambient authority: creation then needs an explicit choice.
        terminal(["open"], [("case_prompt: entered", "/exit\r")])
        terminal(["open", "fresh"], [("Type create", "create\r"), ("Operator Participant name", "\r"),
            ("Model Participant name", "\r"), ("Type admit", "admit\r"), ("case_prompt: entered", "/exit\r")])
        assert "equivalent_to_replay" in cli("case", "verify", "case:fresh", "--json")
        terminal(["open"], [("Choose an exact name or number", "case:golden:free\r"), ("case_prompt: entered", "/exit\r")])
        cli("init", "--tenant", "tenant:other", "--organization", "organization:other")
        terminal(["open", "new"], [("Choose an exact name or number", "\x03")], reject=True)
        assert "case:new" not in cli("case", "list", "--json")
        record(result="PASS", invariant="short guided setup preserves existing Case, cancellation, exact scope, replay and explicit consent", human_golden_case="PENDING_OPERATOR")


if __name__ == "__main__":
    main()
