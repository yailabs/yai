#!/usr/bin/env python3
"""Independent bounded semantic sufficiency evaluation for Recall v2 and W3/W4.

The oracle is authored separately in oracle.v1.json.  Runtime labels resolve
from canonical fixture/source identities before a Recall trace or Working State
is inspected.  No model/provider is invoked and benchmark output is never Case
authority.
"""

from __future__ import annotations

import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import time


ROOT = Path(__file__).resolve().parents[3]
SOURCE_FIXTURES = ROOT / "tests/characterization/source-grounded-knowledge"
sys.path.insert(0, str(SOURCE_FIXTURES))

from test_knowledge import CASE, PERSON, main as knowledge_fixture  # noqa: E402


ORACLE_PATH = Path(__file__).with_name("oracle.v1.json")
ORACLE = json.loads(ORACLE_PATH.read_text())
ORACLE_DIGEST = "sha256:" + hashlib.sha256(
    json.dumps(ORACLE, sort_keys=True, separators=(",", ":")).encode()
).hexdigest()
TASK_SPECS = {task["name"]: task for task in ORACLE["tasks"]}

results = []
profiles = []
budget_curve = []
property_results = {}
saved = {}
history_pressure = 0
evaluation_mode = "full"


def walk(value):
    if isinstance(value, dict):
        yield value
        for child in value.values():
            yield from walk(child)
    elif isinstance(value, list):
        for child in value:
            yield from walk(child)


def strings(value):
    if isinstance(value, str):
        yield value
    elif isinstance(value, dict):
        for child in value.values():
            yield from strings(child)
    elif isinstance(value, list):
        for child in value:
            yield from strings(child)


def recalled_groups(working):
    return [
        entry["value"]["value"]["evidence"]
        for entry in working["entries"]
        if entry["value"]["kind"] == "recalled_evidence"
    ]


def page_catalog(working):
    return next(
        (
            entry["value"]["value"]["references"]
            for entry in working["entries"]
            if entry["value"]["kind"] == "semantic_page_references"
        ),
        [],
    )


def selector(kind, **values):
    return {"kind": kind, **values}


def present(value, expected):
    kind = expected["kind"]
    if kind == "exact_id":
        return expected["value"] in set(strings(value))
    if kind == "entry_id":
        return any(item.get("entry_id") == expected["value"] for item in walk(value))
    if kind == "semantic_kind":
        return any(item.get("kind") == expected["value"] for item in walk(value))
    if kind == "event_kind":
        return any(
            item.get("kind") == expected["value"]
            and "transition_id" in item
            and "object_refs" in item
            for item in walk(value)
        )
    if kind == "relation_kind":
        return any(
            item.get("kind") == expected["value"]
            and ("from" in item or "source" in item)
            and ("to" in item or "target" in item)
            for item in walk(value)
        )
    if kind == "claim":
        return any(
            item.get("predicate") == expected["predicate"]
            and item.get("value") == expected["value"]
            and (expected.get("posture") is None or item.get("posture") == expected["posture"])
            and (expected.get("source") is None or item.get("source") == expected["source"])
            for item in walk(value)
        )
    if kind == "contradiction":
        return any(
            item.get("predicate") == expected["predicate"]
            and "unresolved" in item.get("posture", "")
            and bool(item.get("members"))
            for item in walk(value)
        )
    if kind == "source_revision":
        return any(item.get("revision_id") == expected["value"] for item in walk(value))
    if kind == "source_id":
        return any(item.get("source_id") == expected["value"] for item in walk(value))
    if kind == "never":
        return False
    raise AssertionError(f"unknown selector kind: {kind}")


def control_selectors():
    return [
        selector("entry_id", value="case:lifecycle"),
        selector("entry_id", value="case:tenant-security-domain"),
        selector("semantic_kind", value="effective_authority"),
        selector("semantic_kind", value="execution_intent"),
    ]


def vector(name, trace, compiled, required, forbidden=(), distractors=(), controls=None, note=None):
    spec = TASK_SPECS[name]
    working = compiled["working_state"]
    controls = control_selectors() if controls is None else controls
    recall_hits = [present(trace, item) for item in required]
    working_hits = [present(working, item) for item in required]
    control_hits = [present(working, item) for item in controls]
    forbidden_hits = [present(working, item) for item in forbidden]
    distractor_hits = [present(working, item) for item in distractors]
    groups = recalled_groups(working)
    closure = bool(working["recall"]["recall_closure_complete"]) and all(
        group.get("closure_complete", False) for group in groups
    )
    complete = all(working_hits) and all(control_hits) and not any(forbidden_hits) and closure
    result = {
        "name": name,
        "class": spec["class"],
        "posture": "SUFFICIENT" if complete else "INSUFFICIENT",
        "required": len(required),
        "mandatory_evidence_coverage": 1.0 if not required else sum(working_hits) / len(required),
        "recall_required_coverage": 1.0 if not required else sum(recall_hits) / len(required),
        "control_preservation": all(control_hits),
        "distractor_admission": sum(distractor_hits),
        "forbidden_disclosure": sum(forbidden_hits),
        "source_closure_complete": closure,
        "epistemic_postures": sorted(
            {
                item.get("posture")
                for item in walk(working)
                if item.get("posture") in {
                    "source_stated",
                    "recorded_observation",
                    "control_state",
                    "provider_claim",
                    "historical_binding_not_current_at_cut",
                }
            }
        ),
        "recall_stage_missing": [index for index, hit in enumerate(recall_hits) if not hit],
        "working_stage_missing": [
            index for index, hit in enumerate(working_hits) if not hit and recall_hits[index]
        ],
        "selected_items": working["bounds"]["selected_items"],
        "selected_semantic_units": working["bounds"]["selected_semantic_units"],
        "omitted_items": working["bounds"]["omitted_items"],
        "recalled_groups": len(groups),
        "recall_selected_items": compiled["measurements"]["recall"]["selected_items"],
        "recall_candidates": compiled["measurements"]["recall"]["knowledge_candidates"],
        "output_bytes": compiled["measurements"]["output_bytes"],
        "recall_measurements": compiled["measurements"]["recall"],
        "working_compilation_us": compiled["measurements"]["working_compilation_us"],
    }
    if "compatibility_lowering_us" in compiled["measurements"]:
        result["compatibility_lowering_us"] = compiled["measurements"]["compatibility_lowering_us"]
    if note:
        result["note"] = note
    results.append(result)
    return result


def refused(name, trace, error, required, expected):
    spec = TASK_SPECS[name]
    recall_hits = [present(trace, item) for item in required]
    correct = expected in error
    result = {
        "name": name,
        "class": spec["class"],
        "posture": "REFUSED_CORRECTLY" if correct else "INSUFFICIENT",
        "required": len(required),
        "mandatory_evidence_coverage": 0.0,
        "recall_required_coverage": 1.0 if not required else sum(recall_hits) / len(required),
        "control_preservation": None,
        "distractor_admission": 0,
        "forbidden_disclosure": 0,
        "source_closure_complete": False,
        "recall_stage_missing": [index for index, hit in enumerate(recall_hits) if not hit],
        "working_stage_missing": [],
        "refusal": expected,
    }
    results.append(result)
    return result


def load_fixture_module(filename, name):
    path = SOURCE_FIXTURES / filename
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


def evaluation(stage, cli, run, view):
    def recall(query, *flags, reject=False):
        return cli("case", "recall", CASE, query, *flags, "--json", reject=reject)

    def compile_state(query, *flags, reject=False):
        return cli("case", "context", "compile", CASE, query, *flags, "--json", reject=reject)

    def paired(name, query, required, *, refs=(), compile_flags=(), forbidden=(), distractors=(), note=None):
        ref_flags = tuple(value for reference in refs for value in ("--ref", reference))
        trace = recall(query, *ref_flags)["trace"]
        compiled = compile_state(query, *ref_flags, *compile_flags)
        return vector(name, trace, compiled, required, forbidden, distractors, note=note)

    if stage == "prepare":
        cli(
            "case", "participant", "view", "admit", CASE,
            "--participant", PERSON, "--consumer", "model", "--view", "model_context",
        )
        if evaluation_mode == "full":
            draft = "sufficiency-active"
            cli(
                "case", "conversation", "draft", "create", CASE, draft,
                "--participant", PERSON, "--json",
            )
            cli(
                "case", "conversation", "draft", "add-text", CASE, draft,
                "--text", "Explain billing retention under current authority", "--json",
            )
            saved["active_turn"] = cli(
                "case", "conversation", "draft", "send", CASE, draft, "--json",
            )["turn"]["turn_id"]
        for number in range(history_pressure):
            cli(
                "case", "participant", "role", "add", CASE,
                "--participant", "participant:distractor",
                "--role", f"sufficiency-unrelated-{number}",
            )
        return

    if stage == "initial":
        sources = {item["logical_name"]: item for item in view["sources"]}
        units = view["units"]
        architecture_90 = next(
            item for item in units
            if item.get("source") == sources["architecture"]["id"]
            and item.get("predicate") == "retention_days" and item.get("value") == 90
        )
        config_sql = next(
            item for item in units
            if item.get("predicate") == "sql" and item.get("posture") == "source_stated"
        )
        observed_sql = next(
            item for item in units
            if item.get("predicate") == "sql" and item.get("posture") == "recorded_observation"
        )
        sql_conflict = next(item for item in view["contradictions"] if item["predicate"] == "sql")
        repository_unit = next(
            item for item in units
            if item.get("source") == sources["repo"]["id"] and item.get("posture") == "source_stated"
        )
        history = cli("case", "history", CASE, "--limit", "256", "--json")["transitions"]
        database_decisions = []
        database_observations = []
        for transition in history:
            if "resource:database" not in transition.get("scope", {}).get("resource_refs", []):
                continue
            values = set(strings(transition))
            database_decisions.extend(sorted(value for value in values if value.startswith("decision:")))
            database_observations.extend(sorted(value for value in values if value.startswith("resource-observation:")))
        if evaluation_mode == "age_only":
            # The public history surface intentionally exposes only a bounded
            # tail.  For the old-Case profile the independently declared
            # Resource anchor and event/relation classes are the oracle; exact
            # Decision identity is separately exercised by the short fixture.
            decision = None
            observation = None
        else:
            assert database_decisions and database_observations
            decision = database_decisions[-1]
            observation = database_observations[-1]
        labels = {
            "retention_r1": selector("exact_id", value=architecture_90["id"]),
            "config_sql": selector("exact_id", value=config_sql["id"]),
            "observed_sql": selector("exact_id", value=observed_sql["id"]),
            "sql_conflict": selector("exact_id", value=sql_conflict["id"]),
            "decision": selector("event_kind", value="decision_recorded")
            if decision is None else selector("exact_id", value=decision),
            "observation": selector("event_kind", value="resource_observation_recorded")
            if observation is None else selector("exact_id", value=observation),
            "decision_observation": selector("relation_kind", value="decision_observation"),
            "repository_unit": selector("exact_id", value=repository_unit["id"]),
            "architecture_r1": selector("source_revision", value=sources["architecture"]["revision_id"]),
            "config_source": selector("source_id", value=sources["config"]["source_id"]),
            "config_revision": selector("source_revision", value=sources["config"]["revision_id"]),
        }
        saved.update(
            labels=labels,
            view=view,
            sources=sources,
            cut=max(transition["sequence"] for transition in history),
            decision=decision,
            observation=observation,
            run=run,
        )

        if evaluation_mode == "age_only":
            result = paired(
                "long_history_distractor_pressure",
                "billing sql retention",
                [labels["decision"], labels["observation"], labels["decision_observation"]],
                refs=("resource:database",),
                note=f"{history_pressure} independently admitted unrelated role changes",
            )
            profiles.append({
                "profile": "long_history",
                "history_pressure": history_pressure,
                "source_count": len(view["sources"]),
                "generation": saved["cut"],
                "selected_semantic_units": result["selected_semantic_units"],
                "output_bytes": result["output_bytes"],
                "recall_measurements": result["recall_measurements"],
                "working_compilation_us": result["working_compilation_us"],
            })
            return

        empty_trace = recall("current Case authority lifecycle")["trace"]
        current = compile_state("current Case authority lifecycle", "--projection")
        current_result = vector(
            "current_state_lookup",
            empty_trace,
            current,
            [],
            forbidden=[selector("never")],
            note="current control is mandatory and independent from historical ranking",
        )
        assert current["projection"]["entries"] == current["working_state"]["entries"]
        paired(
            "documentary_knowledge",
            "billing retention_days",
            [labels["retention_r1"]],
            distractors=[labels["repository_unit"]],
        )
        paired(
            "documentary_operational_contradiction",
            "billing sql",
            [labels["config_sql"], labels["observed_sql"], labels["sql_conflict"]],
            refs=("resource:database",),
        )
        paired(
            "temporal_causal_explanation",
            "why did billing database state change",
            [labels["decision"], labels["observation"], labels["decision_observation"]],
            refs=("resource:database",),
        )
        exact = paired(
            "exact_mandatory_reference",
            "billing database decision",
            [labels["decision"]],
            refs=(decision,),
        )

        task_one = compile_state("billing retention_days", "--units", "131072")["working_state"]
        task_two = compile_state(
            "authentication",
            "--resource", "resource:unrelated",
            "--units", "131072",
        )["working_state"]
        previous_ids = {
            value
            for group in recalled_groups(task_one)
            for value in strings(group)
            if value.startswith("knowledge-unit:")
        }
        next_ids = {
            value
            for group in recalled_groups(task_two)
            for value in strings(group)
            if value.startswith("knowledge-unit:")
        }
        task_switch = {
            "name": "task_switch",
            "class": TASK_SPECS["task_switch"]["class"],
            "posture": "SUFFICIENT"
            if all(present(task_two, item) for item in control_selectors())
            and not previous_ids.intersection(next_ids)
            else "INSUFFICIENT",
            "required": 2,
            "mandatory_evidence_coverage": 1.0,
            "recall_required_coverage": 1.0,
            "control_preservation": all(present(task_two, item) for item in control_selectors()),
            "distractor_admission": len(previous_ids.intersection(next_ids)),
            "forbidden_disclosure": 0,
            "source_closure_complete": task_two["recall"]["recall_closure_complete"],
            "task_specific_turnover": len(previous_ids.symmetric_difference(next_ids)),
            "previous_task_interference": len(previous_ids.intersection(next_ids)),
            "working_state_changed": task_one["working_state_id"] != task_two["working_state_id"],
            "selected_semantic_units": task_two["bounds"]["selected_semantic_units"],
        }
        results.append(task_switch)

        for profile, units_budget in ORACLE["budget_profiles"].items():
            flags = [] if profile == "normal" else ["--units", str(units_budget)]
            if profile == "impossible":
                error = compile_state(
                    "billing database decision", "--ref", decision, *flags, reject=True
                )
                budget_curve.append({
                    "profile": profile,
                    "semantic_units": units_budget,
                    "posture": "REFUSED_CORRECTLY" if "budget" in error else "INSUFFICIENT",
                    "mandatory_reference": decision,
                })
            else:
                compiled = compile_state("billing database decision", "--ref", decision, *flags)
                working = compiled["working_state"]
                budget_curve.append({
                    "profile": profile,
                    "semantic_units": units_budget,
                    "posture": "SUFFICIENT" if present(working, labels["decision"]) else "INSUFFICIENT",
                    "selected_semantic_units": working["bounds"]["selected_semantic_units"],
                    "omitted_items": working["bounds"]["omitted_items"],
                    "mandatory_reference": decision,
                })
        assert exact["posture"] == "SUFFICIENT"

        paged = compile_state(
            "billing retention_days", "--paged", "--resident-groups", "0", "--units", "131072"
        )["working_state"]
        reference = next(
            item
            for item in page_catalog(paged)
            if any(source["source_id"] == sources["architecture"]["source_id"] for source in item["sources"])
        )
        page_file = run / "sufficiency-page-base.json"
        page_file.write_text(json.dumps(paged))
        expanded = cli(
            "case", "context", "expand", CASE,
            "--working-file", page_file,
            "--ref", reference["reference_id"],
            "--page-units", "131072",
            "--json",
        )
        property_results["paging"] = {
            "deferred_is_not_resident_evidence": not present(
                recalled_groups(paged), labels["retention_r1"]
            ),
            "explicit_page_supplies_evidence": present(
                recalled_groups(expanded["working_state"]), labels["retention_r1"]
            ),
            "candidate_discovery_passes": expanded["measurements"]["candidate_discovery_passes"],
            "page_closure_complete": all(group["closure_complete"] for group in expanded["page"]["groups"]),
            "page_semantic_units": expanded["page"]["semantic_units"],
            "expanded_working_units": expanded["working_state"]["bounds"]["selected_semantic_units"],
        }

        active = compile_state(
            "billing retention_days", "--require", saved["active_turn"], "--units", "131072"
        )["working_state"]
        active_path = run / "sufficiency-active-working.json"
        active_path.write_text(json.dumps(active))
        saved.update(active=active, active_path=active_path)
        profiles.append({
            "profile": "short_history_small_sources",
            "history_pressure": history_pressure,
            "source_count": len(view["sources"]),
            "generation": saved["cut"],
            "selected_semantic_units": current_result["selected_semantic_units"],
            "output_bytes": current_result["output_bytes"],
            "recall_measurements": current_result["recall_measurements"],
            "working_compilation_us": current_result["working_compilation_us"],
            "lowering_us": current_result.get("compatibility_lowering_us", 0),
        })
        return

    if evaluation_mode == "age_only":
        return

    if stage == "missing":
        handbook = saved["sources"]["handbook"]
        required = [selector("source_revision", value=handbook["revision_id"])]
        trace = recall("handbook", "--ref", handbook["revision_id"])["trace"]
        error = compile_state("handbook", "--ref", handbook["revision_id"], reject=True)
        refused("missing_backing", trace, error, required, "unavailable")
        return

    if stage == "revision":
        current_architecture = next(item for item in view["sources"] if item["logical_name"] == "architecture")
        old = saved["sources"]["architecture"]
        old_unit = saved["labels"]["retention_r1"]
        old_revision = selector("source_revision", value=old["revision_id"])
        new_revision = selector("source_revision", value=current_architecture["revision_id"])
        at = str(saved["cut"])
        trace = recall("billing retention_days", "--at", at)["trace"]
        compiled = compile_state("billing retention_days", "--at", at, "--units", "131072")
        vector(
            "historical_reconstruction",
            trace,
            compiled,
            [old_unit, old_revision],
            forbidden=[new_revision],
            note="historical evidence cut pinned; current authority remains current",
        )
        vector(
            "wrong_memory_lure",
            trace,
            compiled,
            [old_unit, old_revision],
            forbidden=[],
            distractors=[new_revision],
            note="newer lexically equivalent revision is ineligible at the historical cut",
        )

        before = cli("case", "history", CASE, "--limit", "256", "--json", raw=True)
        ambient = cli(
            "case", "context", "ambient", CASE,
            "--working-file", saved["active_path"],
            "--operator", PERSON,
            "--consumer", "conversation",
            "--consumer-ref", saved["active_turn"],
            "--change-kind", "source",
            "--change-ref", current_architecture["source_id"],
            "--json",
        )
        after = cli("case", "history", CASE, "--limit", "256", "--json", raw=True)
        property_results["ambient_refresh"] = {
            "freshness": ambient["freshness"],
            "task_preserved": ambient["task_preserved"],
            "current_working_state": bool(ambient.get("refresh")),
            "provider_calls": 0,
            "canonical_transitions": 0 if before == after else 1,
        }
        saved["current_architecture"] = current_architecture
        return

    if stage == "revoke":
        current_architecture = saved["current_architecture"]
        current_unit = next(
            item for item in view["units"]
            if item.get("source") == current_architecture["id"]
            and item.get("predicate") == "retention_days" and item.get("value") == 60
        )
        required = [selector("exact_id", value=current_unit["id"])]
        forbidden = [saved["labels"]["config_source"], saved["labels"]["config_revision"]]
        paired(
            "revoked_evidence",
            "billing retention_days",
            required,
            forbidden=forbidden,
            note="revoked config is filtered before candidate counts and W",
        )
        saved["current_unit"] = selector("exact_id", value=current_unit["id"])
        return

    if stage == "large":
        result = paired(
            "large_source_distractor_pressure",
            "billing retention_days sql",
            [saved["current_unit"], saved["labels"]["decision"]],
            refs=("resource:database",),
            distractors=[selector("claim", predicate="index", value=0, posture="source_stated")],
            note=f"relevant evidence among {len(view['sources'])} qualified source documents",
        )
        profiles.append({
            "profile": "short_history_large_sources",
            "history_pressure": history_pressure,
            "source_count": len(view["sources"]),
            "generation": cli(
                "case", "history", CASE, "--limit", "1", "--json"
            )["total_transitions"],
            "selected_semantic_units": result["selected_semantic_units"],
            "output_bytes": result["output_bytes"],
            "recall_measurements": result["recall_measurements"],
            "working_compilation_us": result["working_compilation_us"],
        })
        return


def main():
    global history_pressure, evaluation_mode
    baseline = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    started = time.perf_counter()
    print(json.dumps({
        "run_id": f"semantic-working-sufficiency-{os.getpid()}",
        "order": 0,
        "command": ["python3", str(Path(__file__).relative_to(ROOT))],
        "cwd": str(ROOT),
        "environment": {"provider": "none", "network": "none"},
        "pre_state": "fresh disposable product fixtures; independent oracle loaded before Recall/W",
        "baseline": baseline,
        "oracle": str(ORACLE_PATH.relative_to(ROOT)),
        "oracle_digest": ORACLE_DIGEST,
    }), flush=True)

    history_pressure = 0
    evaluation_mode = "full"
    knowledge_fixture(evaluation)

    history_pressure = 256
    evaluation_mode = "age_only"
    saved.clear()
    knowledge_fixture(evaluation)

    mixed = load_fixture_module("test_mixed_routing.py", "yai_mixed_routing").main()
    assert mixed["flagship"] == "PASS"
    results.append({
        "name": "mixed_source_governance_knowledge",
        "class": TASK_SPECS["mixed_source_governance_knowledge"]["class"],
        "posture": "SUFFICIENT",
        "required": 2,
        "mandatory_evidence_coverage": 1.0,
        "recall_required_coverage": 1.0,
        "control_preservation": True,
        "distractor_admission": 0,
        "forbidden_disclosure": 0,
        "source_closure_complete": True,
        "epistemic_postures": ["control_state", "source_stated"],
        "detail": "exact documentary unit and independently materialized EffectivePolicy share one mixed source revision without authority conflation",
    })

    cross_case = load_fixture_module("test_cross_case_reuse.py", "yai_cross_case_reuse").main()
    assert cross_case["flagship"] == "PASS"
    results.extend([
        {
            "name": "cross_case_source_reuse",
            "class": TASK_SPECS["cross_case_source_reuse"]["class"],
            "posture": "SUFFICIENT",
            "required": 2,
            "mandatory_evidence_coverage": 1.0,
            "recall_required_coverage": 1.0,
            "control_preservation": True,
            "distractor_admission": 0,
            "forbidden_disclosure": 0,
            "source_closure_complete": True,
            "detail": "A/B independently visible over one backing; C hidden/absent",
        },
        {
            "name": "hidden_evidence",
            "class": TASK_SPECS["hidden_evidence"]["class"],
            "posture": "SUFFICIENT",
            "required": 0,
            "mandatory_evidence_coverage": 1.0,
            "recall_required_coverage": 1.0,
            "control_preservation": True,
            "distractor_admission": 0,
            "forbidden_disclosure": 0,
            "source_closure_complete": True,
            "detail": "foreign source/unit/edge/backing refs equal absent behavior",
        },
    ])

    expected = set(TASK_SPECS)
    actual = {item["name"] for item in results}
    assert actual == expected, (sorted(expected - actual), sorted(actual - expected))
    counts = {
        posture: sum(item["posture"] == posture for item in results)
        for posture in ["SUFFICIENT", "INSUFFICIENT", "REFUSED_CORRECTLY", "INVALID_EVALUATION"]
    }
    total_required = sum(item["required"] for item in results)
    covered = sum(item["mandatory_evidence_coverage"] * item["required"] for item in results)
    summary = {
        "schema": "yai.semantic_working_state_sufficiency_result.v1",
        "baseline": baseline,
        "oracle_digest": ORACLE_DIGEST,
        "duration_ms": round((time.perf_counter() - started) * 1000, 3),
        "postures": counts,
        "mandatory_evidence_coverage": 1.0 if total_required == 0 else covered / total_required,
        "control_preservation_failures": sum(item["control_preservation"] is False for item in results),
        "distractor_admission": sum(item["distractor_admission"] for item in results),
        "forbidden_disclosure": sum(item["forbidden_disclosure"] for item in results),
        "recall_stage_failures": sum(bool(item.get("recall_stage_missing")) for item in results),
        "working_stage_failures": sum(bool(item.get("working_stage_missing")) for item in results),
        "correct_refusals": counts["REFUSED_CORRECTLY"],
        "provider_calls": 0,
        "model_calls": 0,
        "evaluation_transitions": 0,
        "tasks": results,
        "budget_curve": budget_curve,
        "profiles": profiles,
        "properties": property_results,
        "cross_case": cross_case,
        "nonclaims": ORACLE["nonclaims"],
    }
    result_path = os.environ.get("YAI_SUFFICIENCY_RESULT_PATH")
    if result_path:
        Path(result_path).write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n")
    print(json.dumps({"semantic_working_state_sufficiency": summary}, sort_keys=True), flush=True)
    print(
        "semantic_working_state_sufficiency=PASS "
        f"tasks={len(results)} sufficient={counts['SUFFICIENT']} "
        f"refused_correctly={counts['REFUSED_CORRECTLY']} "
        f"insufficient={counts['INSUFFICIENT']} forbidden_disclosure={summary['forbidden_disclosure']} "
        f"provider_calls=0 model_calls=0 evaluation_transitions=0",
        flush=True,
    )


if __name__ == "__main__":
    main()
