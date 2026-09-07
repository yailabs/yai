#!/usr/bin/env python3
"""Real YAI process + Linux PTY + canonical CLI/LMDB observations.

Requires tests/requirements-terminal.txt. No REPLAI checkout or test API is used.
"""
import codecs
import fcntl
import hashlib
import json
import os
from pathlib import Path
import pty
import re
import select
import signal
import struct
import subprocess
import sys
import tempfile
import termios
import time

import pyte

ROOT = Path(__file__).resolve().parents[3]
WORK = Path(tempfile.mkdtemp(prefix="yai-r4-"))
BIN = ROOT / "yai"
ARTIFACT = ROOT / "target/debug/yai"
CASE, PARTICIPANT, TENANT = "case:r4", "participant:r4", "tenant:r4"
PROMPT = f"yai({CASE})> "
RECORDS = []
LIVE = []
ENV = dict(os.environ, YAI_HOME=str(WORK / "home"), TERM="xterm-256color")
for key in ("NO_COLOR", "YAI_NO_COLOR"):
    ENV.pop(key, None)


def observe(kind, **values):
    RECORDS.append(dict(order=len(RECORDS) + 1, kind=kind, **values))
    (WORK / "evidence.json").write_text(json.dumps(RECORDS, indent=2, ensure_ascii=False, default=lambda value: value.hex()))


def cli(*args, structured=False):
    command = [str(ROOT / "yai"), *args] + (["--json"] if structured else [])
    p = subprocess.run(command, cwd=ROOT, env=ENV, text=True, capture_output=True, timeout=30)
    observe("cli", command=command, exit=p.returncode, stdout=p.stdout, stderr=p.stderr)
    assert p.returncode == 0, (command, p.stdout, p.stderr)
    return json.loads(p.stdout)["data"] if structured else p.stdout


def content_snapshot():
    root = Path(ENV["YAI_HOME"]) / "conversation-content-v1"
    return {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in root.rglob("*") if p.is_file()}


def state():
    return cli("case", "show", CASE, structured=True)["case"]


def turns():
    return cli("case", "conversation", "turn", "list", CASE, "--participant", PARTICIPANT, structured=True)["value"]["multipart_turns"]


def acquire_controlling_terminal():
    os.setsid()
    fcntl.ioctl(0, termios.TIOCSCTTY, 0)


class Terminal:
    def __init__(self, **environment):
        self.master, self.slave = pty.openpty()
        self.resize(100, 24, initialize=True)
        self.before = termios.tcgetattr(self.slave)
        self.screen = pyte.Screen(100, 24)
        self.stream = pyte.Stream(self.screen)
        self.decoder = codecs.getincrementaldecoder("utf-8")("strict")
        self.output = b""
        self.proc = subprocess.Popen([str(BIN), "prompt", "--case", CASE, "--subject", PARTICIPANT],
                                     stdin=self.slave, stdout=self.slave, stderr=self.slave,
                                     env=dict(ENV, **environment), cwd=ROOT, preexec_fn=acquire_controlling_terminal)
        LIVE.append(self)
        self.wait(lambda: b"\x1b[?2004h" in self.output and PROMPT in "\n".join(self.screen.display))
        self.fd_initial = len(os.listdir(f"/proc/{self.proc.pid}/fd"))
        self.mode = termios.tcgetattr(self.slave)
        assert self.mode != self.before

    def drain(self, timeout=0.06):
        if select.select([self.master], [], [], timeout)[0]:
            data = os.read(self.master, 65536)
            self.output += data
            self.stream.feed(self.decoder.decode(data))

    def wait(self, predicate, timeout=10):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            self.drain()
            if predicate():
                return
            assert self.proc.poll() is None, self.output.decode(errors="replace")
        raise AssertionError(("PTY timeout", self.output.decode(errors="replace"), self.screen.display))

    def send(self, data):
        if isinstance(data, str):
            data = data.encode()
        os.write(self.master, data)
        observe("input", hex=data.hex())
        for _ in range(4):
            self.drain()

    def draft(self, text, cells):
        self.wait(lambda: self.screen.display[self.screen.cursor.y].rstrip() == (PROMPT + text).rstrip())
        assert self.screen.cursor.x == len(PROMPT) + cells, (self.screen.cursor.x, text)
        assert all(c.bg == "default" for row in self.screen.buffer.values() for c in row.values())
        assert not any(marker in self.output for marker in (b"\x1b[?1049h", b"\x1b[?1047h", b"\x1b[?47h"))
        observe("draft", text=text, cursor=[self.screen.cursor.y, self.screen.cursor.x], fd_count=len(os.listdir(f"/proc/{self.proc.pid}/fd")))

    def resize(self, columns, rows, initialize=False):
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", rows, columns, 0, 0))
        if not initialize:
            self.screen.resize(lines=rows, columns=columns)
            self.drain(0.2)
            observe("resize", columns=columns, rows=rows)

    def exit(self, key=b"\x04"):
        self.send(key)
        self.proc.wait(timeout=10)
        while select.select([self.master], [], [], 0.05)[0]:
            self.drain()
        assert self.proc.returncode == 0, self.output
        after = termios.tcgetattr(self.slave)
        assert after == self.before
        assert self.output.count(b"\x1b[?2004h") == self.output.count(b"\x1b[?2004l")
        observe("terminal_close", exit=self.proc.returncode, termios_before=self.before,
                termios_after=after, paste_balanced=True, output_hex=self.output.hex())
        os.close(self.master)
        os.close(self.slave)
        LIVE.remove(self)


def prepare():
    cli("init", "--tenant", TENANT, "--organization", "organization:r4")
    cli("case", "create", CASE, "--tenant", TENANT)
    cli("case", "participant", "role", "add", CASE, "--participant", PARTICIPANT, "--role", "model-executor")
    cli("case", "participant", "link-principal", CASE, "--principal", "self", "--participant", PARTICIPANT)
    cli("case", "participant", "view", "admit", CASE, "--participant", PARTICIPANT, "--consumer", "model", "--view", "model_context")


def terminal_contract():
    initial = state()
    assert not turns()
    content_before = content_snapshot()
    t = Terminal()
    assert b"\x1b[38;5;81m" + PROMPT[:-1].encode() in t.output
    t.send("cafe\u0301界")
    t.send(b"\x1b[D\x7fX")
    t.draft("cafX界", 4)
    assert state() == initial and not turns() and content_snapshot() == content_before
    t.send(b"\x03")
    t.draft("", 0)
    assert state() == initial and not turns() and content_snapshot() == content_before
    t.send(b"\x04")
    t.proc.wait(timeout=5)
    # Complete exit assertions without sending another editor key.
    t.exit(b"")
    for env in ({"NO_COLOR": ""}, {"TERM": "dumb"}):
        t = Terminal(**env)
        assert not re.search(rb"\x1b\[[0-9;]*m", t.output)
        t.exit()
    t = Terminal()
    t.send("earlier\r")
    t.wait(lambda: b'"provider_unavailable"' in t.output)
    t.draft("", 0)
    committed = turns()
    assert len(committed) == 1
    assert state()["generation"] == initial["generation"] + 2
    t.send("draft")
    t.send(b"\x1b[D\x1b[A\x1b[B")
    t.draft("draft", 4)
    t.send(b"\x0c")
    t.draft("draft", 4)
    t.resize(80, 20)
    t.draft("draft", 4)
    assert turns() == committed
    t.send(b"\x03/thread st\t")
    t.draft("/thread status", len("/thread status"))
    t.send(b"\r")
    t.draft("", 0)
    assert turns() == committed
    # Ambiguous completion uses REPLAI external output and retains a non-end cursor.
    t.send(b"/thread \x1b[D\t")
    assert t.screen.cursor.x == len(PROMPT) + 7
    assert "/thread " in t.screen.display[t.screen.cursor.y]
    t.send(b"\x03")
    before_paste = state()
    paste_content_before = content_snapshot()
    t.send(b"\x1b[200~" + "é\r\n界".encode() + b"\x1b[201~")
    t.wait(lambda: "... 界" in t.screen.display[t.screen.cursor.y])
    assert t.screen.cursor.x == 6
    assert state() == before_paste and turns() == committed and content_snapshot() == paste_content_before
    t.send(b"\r")
    t.draft("", 0)
    after = turns()
    assert len(after) == 2
    assert state()["generation"] == before_paste["generation"] + 2
    # Confirm actual immutable content rather than the terminal's echo.
    latest = cli("case", "conversation", "turn", "show", CASE, "latest", "--participant", PARTICIPANT, structured=True)
    turn = latest["value"]["turn"]
    assert turn["participant_id"] == PARTICIPANT and turn["case_id"] == CASE
    assert turn["base_generation"] == before_paste["generation"]
    assert len(turn["ordered_parts"]) == 1
    part = turn["ordered_parts"][0]
    assert part["ordinal"] == 0
    assert part["object"]["inline_text"] == "é\n界"
    payload = "é\n界".encode()
    assert part["object"]["byte_length"] == len(payload)
    assert part["object"]["content_digest"] == "sha256:" + hashlib.sha256(payload).hexdigest()
    assert part["provenance"] == {"provenance_kind": "original", "imported_by_principal_id": turn["submitted_by_principal_id"]}
    observe("multipart_submission", latest=latest, submitted_hex=payload.hex())
    t.send(b"abc\x04\x01\x04")
    t.draft("bc", 0)
    t.send(b"\x03")
    counts = []
    for _ in range(24):
        t.send(b"ab\x03")
        t.draft("", 0)
        counts.append(len(os.listdir(f"/proc/{t.proc.pid}/fd")))
        assert termios.tcgetattr(t.slave) == t.mode
    assert min(counts) == max(counts) == t.fd_initial
    assert turns() == after
    observe("repeated_interactions", count=len(counts), fd_counts=counts)
    for _ in range(12):
        t.send("repeat\r")
        t.draft("", 0)
        assert len(os.listdir(f"/proc/{t.proc.pid}/fd")) == t.fd_initial
    assert len(turns()) == len(after) + 12
    observe("repeated_submissions", count=12, fd_count=t.fd_initial, state=state())
    t.exit()


def provider_contract(mode, cancel=False):
    label = mode + ("-cancel" if cancel else "")
    release = WORK / f"{label}-release"
    log = WORK / f"{label}-requests.jsonl"
    server = subprocess.Popen([sys.executable, str(ROOT / "tests/fixtures/provider_governance_server.py"),
        "--mode", mode, "--model", "r4-fixture", "--release-file", str(release), "--log", str(log)],
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    decoy_log = WORK / f"{label}-decoy.jsonl"
    decoy = subprocess.Popen([sys.executable, str(ROOT / "tests/fixtures/provider_governance_server.py"),
        "--mode", "full", "--model", "whisper-vision-best-looking-name", "--log", str(decoy_log)],
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    try:
        decoy_port = decoy.stdout.readline().strip()
        port = server.stdout.readline().strip()
        assert port.isdigit(), port
        added = cli("provider", "add", "--tenant", TENANT, "--provider-key", label,
                    "--endpoint", f"http://127.0.0.1:{port}", "--model", "r4-fixture", "--locality", "loopback")
        target = re.search(r"provider-target:[\w:.-]+", added)[0]
        decoy_added = cli("provider", "add", "--tenant", TENANT, "--provider-key", label + "-decoy",
                    "--endpoint", f"http://127.0.0.1:{decoy_port}", "--model", "whisper-vision-best-looking-name", "--locality", "loopback")
        decoy_target = re.search(r"provider-target:[\w:.-]+", decoy_added)[0]
        cli("provider", "qualify", "--target", decoy_target, "--realization-shape", "text_to_text")
        cli("provider", "trust", "approve", "--target", decoy_target)
        cli("provider", "qualify", "--target", target, "--realization-shape", "text_to_text")
        cli("provider", "trust", "approve", "--target", target)
        cli("case", "provider", "bind", CASE, "--participant", PARTICIPANT, "--target", decoy_target, "--target", target,
            "--failover", "safe_only", "--max-attempts", "2")
        def suitability(identity):
            out = cli("provider", "suitability", "record", identity, "--capability", "primary_conversation",
                "--suite", "fixture:i06-pty", "--run", label, "--evidence-ref", "evidence:i06-pty")
            return re.search(r"^evidence_id: (.+)$", out, re.M)[1]
        evidence, decoy_evidence = suitability(target), suitability(decoy_target)
        bind_args = ["case", "cognitive", "bind", CASE, "--participant", PARTICIPANT,
            "--role", "primary", "--capability", "primary_conversation", "--target", target, "--evidence", evidence]
        if cli("case", "cognitive", "show", CASE, "--participant", PARTICIPANT, structured=True)["value"]["bindings"]:
            bind_args += ["--replace"]
        if mode == "full" and not cancel:
            bind_args += ["--alternative", decoy_target + "=" + decoy_evidence]
        cli(*bind_args)
        before, inventory = state(), turns()
        t = Terminal()
        t.send("provider boundary")
        assert state() == before and turns() == inventory
        t.send(b"\r")
        t.wait(lambda: b"conversation_turn:" in t.output)
        # The fixture cannot finish until this separate process observes the Turn.
        committed = turns()
        assert len(committed) == len(inventory) + 1
        during = state()
        assert during["generation"] > before["generation"]
        assert termios.tcgetattr(t.slave) == t.before
        if cancel:
            # Existing cancellation is conservative: no forced abort of an already
            # dispatched buffered request; its result must still be accounted for.
            t.wait(lambda: log.exists() and '"synthetic":false' in log.read_text())
            t.send(b"\x03")
        assert committed[-1]["base_generation"] == before["generation"]
        intent = cli("case", "conversation", "turn", "show", CASE, committed[-1]["turn_id"],
            "--participant", PARTICIPANT, structured=True)["value"]["execution_intent"]
        assert intent["goal"] == "primary_conversation" and "prerequisite" not in intent
        observe("provider_pending", mode=label, before=before, during=during, committed=committed, intent=intent)
        release.touch()
        expected = b'"completed"' if mode == "full" else b'"delivery_indeterminate"'
        if cancel:
            t.wait(lambda: b'"completed"' in t.output or b'"delivery_indeterminate"' in t.output)
        else:
            t.wait(lambda: expected in t.output)
        t.draft("", 0)
        if mode == "full" and not cancel:
            assert b"fixture native conversation ORCHID-I03" in t.output
        assert turns() == committed
        assert len(os.listdir(f"/proc/{t.proc.pid}/fd")) == t.fd_initial
        requests = [json.loads(line) for line in log.read_text().splitlines()]
        assert sum(not request["synthetic"] for request in requests) == 1
        provider_posture = cli("case", "provider", "show", CASE)
        assert "last_selected_target: " + target in provider_posture
        assert "last_selected_model: r4-fixture" in provider_posture
        if mode == "full" and not cancel:
            lineage = json.loads(re.search(rb"conversation_cognition: (\{[^\r\n]+\})", t.output)[1])
            assert lineage["target_id"] == target and lineage["intent_id"] == intent["request_id"]
            assert lineage["execution_plan_id"].startswith("cognitive-plan:")
            assert lineage["lane_id"].startswith("cognitive-lane:")
            assert "last_selection_id: " + lineage["selection_id"] in provider_posture
            observe("i06_exact_lineage", lineage=lineage, provider_posture=provider_posture)
        assert not any(not json.loads(line)["synthetic"] for line in decoy_log.read_text().splitlines())
        if not cancel:
            if mode != "full":
                # New exact binding cannot erase uncertainty on the prior Turn.
                cli("case", "cognitive", "bind", CASE, "--participant", PARTICIPANT, "--role", "primary",
                    "--capability", "primary_conversation", "--target", decoy_target, "--evidence", decoy_evidence, "--replace")
            retry_start = len(t.output)
            t.send("/retry " + committed[-1]["turn_id"] + "\r")
            t.wait(lambda: expected in t.output[retry_start:])
            t.draft("", 0)
            assert turns() == committed
            assert sum(not json.loads(line)["synthetic"] for line in log.read_text().splitlines()) == 1
            assert not any(not json.loads(line)["synthetic"] for line in decoy_log.read_text().splitlines())
            if mode == "full":
                assert b'"recovered":true' in t.output[retry_start:]
            observe("i06_retry", mode=label, output=t.output, same_turn=True, redispatches=0)
        observe("provider_finished", mode=label, state=state(), turns=turns(), requests=requests,
                selected_target=target, historical_first_target=decoy_target, cognitive_selection=True)
        t.exit()
    finally:
        server.terminate()
        server.wait(timeout=5)
        decoy.terminate()
        decoy.wait(timeout=5)


def rejected_acquisition():
    initial, content = state(), content_snapshot()
    for extra in ([], ["--model", "not-a-governed-override"]):
        master, slave = pty.openpty()
        before = termios.tcgetattr(slave)
        try:
            command = [str(BIN), "prompt", "--case", CASE, "--subject", PARTICIPANT, *extra]
            process = subprocess.run(command, stdin=slave, capture_output=True, env=ENV, timeout=10)
            assert process.returncode != 0
            assert b"\x1b[?2004h" not in process.stdout
            assert before == termios.tcgetattr(slave)
            assert state() == initial and content_snapshot() == content
            if extra:
                assert b"interactive_prompt_uses_governed_provider_binding" in process.stderr
            observe("rejected_open", command=command, exit=process.returncode,
                    stdout=process.stdout.decode(), stderr=process.stderr.decode(),
                    termios_unchanged=True, canonical_unchanged=True)
        finally:
            os.close(master)
            os.close(slave)


def artifact_contract():
    meta = json.loads(subprocess.check_output(["cargo", "metadata", "--locked", "--offline", "--format-version=1", "--manifest-path", "cmd/yai/Cargo.toml"], cwd=ROOT))
    dependency = next(p for p in meta["packages"] if p["name"] == "replai")
    assert dependency["source"].endswith("#df5538c718b8d068432032e7fb116fb8bfab158e")
    assert not any(p["name"] == "replai-c" for p in meta["packages"])
    symbols = subprocess.check_output(["nm", "-C", str(ARTIFACT)], text=True)
    assert "replai::terminal::Interaction" in symbols
    assert "linenoise" not in symbols.lower()
    assert not (ROOT / "cmd/yai/build.rs").exists()
    observe("artifact", replai_source=dependency["source"], native_rust=True, linenoise_symbols=False)


try:
    observe("run", run_id=WORK.name, cwd=str(ROOT), home=ENV["YAI_HOME"], baseline=subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip())
    prepare()
    artifact_contract()
    rejected_acquisition()
    terminal_contract()
    provider_contract("full")
    provider_contract("drop_realization")
    provider_contract("full", cancel=True)
    print("r4_terminal: canonical_editing_unchanged=true commit_before_provider=true success_failure_turn_retained=true exact_termios=true bounded_fds=true native_replai=true")
    print("i06_pty: cognitive_arbitration=true provider_order_bypassed=true pinned=true durable_intent=true retry_no_redispatch=true indeterminate_no_cross_target=true")
    print("evidence:", WORK)
except BaseException:
    print("FAILED evidence:", WORK, file=sys.stderr)
    raise

finally:
    for terminal in LIVE:
        if terminal.proc.poll() is None:
            terminal.proc.terminate()
            terminal.proc.wait(timeout=5)
        os.close(terminal.master)
        os.close(terminal.slave)
