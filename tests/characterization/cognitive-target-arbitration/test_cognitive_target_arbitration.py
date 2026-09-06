#!/usr/bin/env python3
"""Product/recovery qualification via ./yai and real loopback HTTP, not YVEX."""
import base64
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[3]


def main():
    with tempfile.TemporaryDirectory(prefix="yai-i05-") as directory:
        run = Path(directory)
        env = dict(os.environ, YAI_HOME=str(run / "home"), NO_COLOR="1")
        processes, handles, peers = [], [], {}
        order = 0

        def command(*args, reject=None):
            nonlocal order
            order += 1
            argv = ["./yai", *args]
            result = subprocess.run(argv, cwd=ROOT, env=env, text=True,
                                    capture_output=True, timeout=90)
            print(json.dumps({"run_id": run.name, "order": order, "cwd": str(ROOT),
                              "yai_home": env["YAI_HOME"], "command": argv,
                              "exit": result.returncode, "stdout": result.stdout,
                              "stderr": result.stderr}, separators=(",", ":")), flush=True)
            if reject is not None:
                assert result.returncode != 0 and reject in result.stdout + result.stderr, result
                return result
            assert result.returncode == 0, result
            if "--json" in args:
                root = json.loads(result.stdout)
                return root.get("data", {}).get("value", root)
            return result.stdout

        def field(output, key):
            return next(line.split(": ", 1)[1] for line in output.splitlines() if line.startswith(key + ": "))

        def count(peer):
            path = run / (peer + ".log")
            return sum(not row["synthetic"] for row in map(json.loads, path.read_text().splitlines())) if path.exists() else 0

        def binding(case, role, capability, names, replace=False):
            name = names[0]
            args = ["case", "cognitive", "bind", case, "--participant", "participant:model",
                    "--role", role, "--capability", capability, "--target", peers[name][0],
                    "--evidence", peers[name][1][capability]]
            for name in names[1:]:
                args += ["--alternative", peers[name][0] + "=" + peers[name][1][capability]]
            if replace:
                args += ["--replace"]
            return command(*args, "--json")

        def prepare(case):
            command("case", "create", case, "--tenant", "tenant:i05-cli")
            command("case", "participant", "role", "add", case, "--participant", "participant:model", "--role", "model-executor")
            command("case", "participant", "link-principal", case, "--participant", "participant:model", "--principal", "self")
            command("case", "participant", "view", "admit", case, "--participant", "participant:model", "--consumer", "model", "--view", "model_context")
            args = ["case", "provider", "bind", case, "--participant", "participant:model", "--failover", "safe_only", "--max-attempts", "3"]
            for target, _ in peers.values():
                args += ["--target", target]
            command(*args)

        def turn(case, draft):
            command("case", "conversation", "draft", "create", case, draft, "--participant", "participant:model")
            command("case", "conversation", "draft", "add-text", case, draft, "--text", "before I05 audio")
            command("case", "conversation", "draft", "import", case, draft, str(run / "source.wav"), "--type", "audio", "--mime", "audio/wav")
            command("case", "conversation", "draft", "add-text", case, draft, "--text", "after I05 audio")
            identity = field(command("case", "conversation", "draft", "send", case, draft), "turn_id")
            value = command("case", "conversation", "turn", "show", case, identity, "--participant", "participant:model", "--json")["turn"]
            audio = next(p["part_id"] for p in value["ordered_parts"] if p["object"]["modality"] == "audio")
            return identity, audio, value

        def compose(case, identity, part, **kwargs):
            return command("case", "cognitive", "compose", case, "--participant", "participant:model",
                           "--goal", "primary_conversation", "--turn", identity,
                           "--prerequisite", "speech_to_text", "--prerequisite-part", part, "--json", **kwargs)

        try:
            command("init", "--tenant", "tenant:i05-cli", "--organization", "organization:i05")
            (run / "source.wav").write_bytes(base64.b64decode((ROOT / "tests/fixtures/conversation/i03-audio.wav.base64").read_bytes()))
            definitions = [
                ("preferred", "full", "whisper-vision-best-name", ["text_to_text"], ["primary_conversation"]),
                ("native", "full", "plain-native", ["text_to_text", "audio_wav_to_text"], ["primary_conversation"]),
                ("auxiliary", "full", "not-whisper", ["audio_wav_to_text"], ["speech_to_text"]),
                ("drop", "drop_realization", "deepseek-most-intelligent", ["audio_wav_to_text"], ["speech_to_text"]),
                ("unsuitable", "full", "whisper-vision", ["audio_wav_to_text"], []),
            ]
            for name, mode, model, shapes, capabilities in definitions:
                port_file = run / (name + ".port")
                out = port_file.open("w")
                err = (run / (name + ".err")).open("w")
                handles += [out, err]
                processes.append(subprocess.Popen(["python3", str(ROOT / "tests/fixtures/provider_governance_server.py"),
                    "--mode", mode, "--model", model, "--requests", "128", "--log", str(run / (name + ".log"))],
                    cwd=ROOT, stdout=out, stderr=err))
                deadline = time.monotonic() + 10
                while not port_file.stat().st_size and time.monotonic() < deadline:
                    time.sleep(0.02)
                port = int(port_file.read_text().splitlines()[0])
                target = field(command("provider", "add", "--tenant", "tenant:i05-cli", "--provider-key", name,
                                       "--endpoint", f"http://127.0.0.1:{port}", "--model", model, "--locality", "loopback"), "target_id")
                args = ["provider", "qualify", target]
                for shape in shapes:
                    args += ["--realization-shape", shape]
                command(*args)
                command("provider", "trust", "approve", target)
                evidence = {}
                for capability in capabilities:
                    evidence[capability] = field(command("provider", "suitability", "record", target,
                        "--capability", capability, "--suite", "fixture:i05", "--run", name,
                        "--evidence-ref", "fixture:i05:" + name), "evidence_id")
                peers[name] = target, evidence

            case = "case:i05-direct"
            prepare(case)
            binding(case, "primary", "primary_conversation", ["preferred", "native"])
            binding(case, "auxiliary", "speech_to_text", ["auxiliary", "drop"])
            identity, audio, original = turn(case, "native-source")
            plan_args = ["case", "cognitive", "plan", case, "--participant", "participant:model",
                         "--capability", "primary_conversation", "--source", identity, "--shape", "audio_wav_to_text", "--json"]
            first_plan = command(*plan_args)["plan"]
            assert first_plan == command(*plan_args)["plan"]  # new process, same snapshot
            assert first_plan["selected_target_id"] == peers["native"][0]
            assert "mechanical_shape_unsupported" in first_plan["arbitration"]["candidates"][0]["exclusions"]
            direct = compose(case, identity, audio)
            assert direct["composition_route"] == "direct"
            assert count("native") == 1 and count("auxiliary") == count("drop") == count("unsuitable") == 0
            # Exact semantic evidence for another target cannot onboard this one.
            command("case", "cognitive", "bind", case, "--participant", "participant:model", "--role", "auxiliary",
                    "--capability", "speech_to_text", "--target", peers["unsuitable"][0], "--evidence", peers["auxiliary"][1]["speech_to_text"],
                    "--replace", reject="semantic_suitability_evidence_binding_mismatch")

            # No native audio target is trusted: explicit auxiliary, then primary.
            command("provider", "trust", "deny", peers["native"][0])
            case = "case:i05-composed"
            prepare(case)
            # Bind while trusted; eligibility can change afterwards.
            command("provider", "trust", "approve", peers["native"][0])
            binding(case, "primary", "primary_conversation", ["preferred", "native"])
            binding(case, "auxiliary", "speech_to_text", ["drop", "auxiliary"])
            command("provider", "trust", "deny", peers["native"][0])
            command("provider", "trust", "deny", peers["drop"][0])
            identity, audio, original = turn(case, "composed-source")
            command("case", "cognitive", "compose", case, "--participant", "participant:model", "--goal", "primary_conversation",
                    "--turn", identity, "--prerequisite", "speech_to_text", "--prerequisite-part", audio,
                    "--failpoint", "after-prerequisite", "--json", reject="cognitive_composition_failpoint_after_prerequisite")
            assert count("auxiliary") == 1 and count("preferred") == 0
            composed = compose(case, identity, audio)
            assert composed["composition_route"] == "composed"
            assert count("auxiliary") == 1 and count("preferred") == 1
            repeated = compose(case, identity, audio)
            assert repeated["source_closure"] == composed["source_closure"]
            assert count("auxiliary") == 1 and count("preferred") == 1
            assert original == command("case", "conversation", "turn", "show", case, identity, "--participant", "participant:model", "--json")["turn"]

            # A new explicit cycle may select B, but cannot repeat uncertain work.
            command("provider", "trust", "approve", peers["drop"][0])
            identity, audio, _ = turn(case, "indeterminate-source")
            compose(case, identity, audio, reject="DeliveryIndeterminate")
            auxiliary_before, primary_before = count("auxiliary"), count("preferred")
            command("provider", "trust", "deny", peers["drop"][0])
            compose(case, identity, audio, reject="prior_delivery_indeterminate_requires_resolution")
            assert count("auxiliary") == auxiliary_before and count("preferred") == primary_before
            print(json.dumps({"i05_qualification": "PASS", "proof_class": "product/recovery", "provider_mode": "loopback_fixture",
                              "native_bypass": True, "ordered_shape_exclusions": True,
                              "composed_arbitrated_targets": True, "restart_no_redispatch": True,
                              "indeterminate_no_cross_target_retry": True,
                              "actual_requests": {name: count(name) for name in peers}}), flush=True)
        finally:
            for process in processes:
                process.terminate()
            for process in processes:
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
            for handle in handles:
                handle.close()


if __name__ == "__main__":
    main()
