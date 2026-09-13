#!/usr/bin/env python3
"""Same stored task -> current qualification/Recall/W through the actual CLI.

Real source owners and persistence; no model, refresh-specific retriever or
watcher. Each command's run/order/pre-state/exit is retained by the shared fixture.
"""
import json
import sqlite3
import sys
import time
from test_knowledge import main, CASE, PERSON

saved, profiles = {}, []
history_pressure = 0
QUERY = "billing retention_days sql"


def groups(w):
    return [e["value"]["value"]["evidence"] for e in w["entries"] if e["value"]["kind"] == "recalled_evidence"]


def catalog(w):
    return next(e["value"]["value"]["references"] for e in w["entries"] if e["value"]["kind"] == "semantic_page_references")


def resident(w):
    return {r["reference_id"] for r in catalog(w) if any(e["entry_id"] == r["group_entry_id"] for e in w["entries"])}


def oracle(stage, cli, run, view):
    def compile(query=QUERY, *flags, **kw):
        return cli("case", "context", "compile", CASE, query, "--units", "131072", *flags, "--json", **kw)

    def refresh(base, *flags, **kw):
        path = run / "refresh-working.json"
        path.write_text(json.dumps(base))
        before = cli("case", "show", CASE, "--json", raw=True)
        human = kw.pop("human", False)
        started = time.perf_counter()
        result = cli("case", "context", "refresh", CASE, "--working-file", path, *flags,
            *([] if human else ["--json"]), **kw)
        command_ms = (time.perf_counter() - started) * 1000
        assert before == cli("case", "show", CASE, "--json", raw=True), "refresh is not a canonical event"
        if isinstance(result, dict):
            result["measurements"]["product_refresh_wall_ms"] = command_ms
            old, new = base["recall"]["request"], result["working_state"]["recall"]["request"]
            assert old["compilation"]["intent"] == new["compilation"]["intent"]
            assert old["at"] == new["at"] and old["recall_required_refs"] == new["recall_required_refs"]
            assert result["compilation_mode"] == "FullRecompilation"
            assert result["measurements"]["recall"]["candidate_discovery_passes"] == 1
        return result

    def page(base, source):
        reference = next(r for r in catalog(base) if any(s["source_id"] == source for s in r["sources"]))
        path = run / "page-working.json"; path.write_text(json.dumps(base))
        result = cli("case", "context", "expand", CASE, "--working-file", path,
            "--ref", reference["reference_id"], "--page-units", "131072", "--json")
        return result["working_state"], reference

    def characterize(label, base, size):
        result = refresh(base, "--projection"); refresh_ms = result["measurements"]["product_refresh_wall_ms"]
        started = time.perf_counter(); fresh = compile(QUERY, "--ref", "resource:database", "--projection"); fresh_ms = (time.perf_counter()-started)*1000
        assert result["working_state"] == fresh["working_state"], "refresh reference path must equal fresh compile"
        assert result["projection"]["entries"] == result["working_state"]["entries"]
        profiles.append(dict(label=label, D=size, H_extra=history_pressure,
            generation=result["working_state"]["case_generation"], refresh_wall_ms=refresh_ms,
            fresh_wall_ms=fresh_ms, refresh=result["measurements"], fresh=fresh["measurements"],
            groups=len(groups(result["working_state"]))))
        return result

    if stage == "prepare":
        cli("case", "participant", "view", "admit", CASE, "--participant", PERSON, "--consumer", "model", "--view", "model_context")
        for n in range(history_pressure):
            cli("case", "participant", "role", "add", CASE, "--participant", "participant:distractor", "--role", f"refresh-distractor-{n}")
    elif stage == "initial":
        base = compile(QUERY, "--ref", "resource:database")["working_state"]
        unchanged = characterize("unchanged", base, len(view["sources"]))
        assert unchanged["posture"] == "unchanged" and unchanged["assessment"]["identity_equal"]
        assert "Task preserved" in refresh(base, human=True)
        refresh(base, "--base-id", "working-state:wrong-task", reject=True)
        refresh(base, "--participant", "participant:outsider", reject=True)
        refresh(base, "--intent", "silently change task", reject=True)
        refresh(base, "--units", "1", reject=True)
        forged = json.loads(json.dumps(base)); forged["request"]["intent"] = "different task"
        refresh(forged, reject=True)
        handbook = next(s for s in view["sources"] if s["logical_name"] == "handbook")
        config = next(s for s in view["sources"] if s["logical_name"] == "config")
        paged = compile(QUERY, "--paged", "--resident-groups", "0")["working_state"]
        expanded, reference = page(paged, config["source_id"])
        refreshed_page = refresh(expanded, "--projection")
        after = refreshed_page["working_state"]
        assert any(r["members"] == reference["members"] and r["reference_id"] in resident(after) for r in catalog(after))
        assert refreshed_page["assessment"]["selected_material_equal"]
        assert refreshed_page["projection"]["schema"] == "yai.projection.v12"
        assert refreshed_page["projection"]["entries"] == after["entries"]
        assert not resident(refresh(paged)["working_state"]), "deferred preference must not become authority/residency"
        saved.update(base=base, initial=unchanged, paged=paged, expanded=expanded, refreshed_paged=after, config=config,
            handbook=handbook, handbook_w=compile("handbook", "--ref", handbook["revision_id"])["working_state"],
            optional_book=compile("handbook")["working_state"],
            historical=compile(QUERY, "--ref", "resource:database", "--at", base["case_generation"])["working_state"],
            old_revision=next(s["revision_id"] for s in view["sources"] if s["logical_name"] == "architecture"))
        print(json.dumps({"refresh_page_preference":refreshed_page["assessment"], "paging_recompilation_us":refreshed_page["paging_recompilation_us"]}), flush=True)
    elif stage == "rebuild":
        assert refresh(saved["base"])["working_state"] == saved["initial"]["working_state"]
        assert refresh(saved["expanded"])["working_state"] == saved["refreshed_paged"]
    elif stage == "missing":
        assert "unavailable" in refresh(saved["handbook_w"], reject=True)
        incomplete = refresh(saved["optional_book"])
        # Closure is about selected semantics, not all formerly relevant data:
        # optional missing material can disappear rather than enter a partial
        # group. Neither posture certifies universal task sufficiency.
        assert incomplete["posture"] in ("incomplete", "refreshed"), incomplete["posture"]
        assert incomplete["assessment"]["current_control_equal"] and not incomplete["assessment"]["recall_identity_equal"]
        assert incomplete["working_state"]["case_generation"] == saved["optional_book"]["case_generation"]
        assert (run / "files/handbook.pdf").exists(), "never substitute live bytes"
    elif stage == "revision":
        result = characterize("source_revision", saved["base"], len(view["sources"]))
        new_revision = next(s["revision_id"] for s in view["sources"] if s["logical_name"] == "architecture")
        assert new_revision in json.dumps(groups(result["working_state"]))
        old = refresh(saved["historical"])["working_state"]
        assert new_revision not in json.dumps(groups(old))
        assert saved["old_revision"] in json.dumps(groups(old))
    elif stage == "revoke":
        for base in (saved["base"], saved["historical"], saved["expanded"], saved["paged"]):
            result = refresh(base, "--projection")
            encoded = json.dumps(result["working_state"])
            assert saved["config"]["source_id"] not in encoded and saved["config"]["revision_id"] not in encoded
            assert result["projection"]["entries"] == result["working_state"]["entries"]
    elif stage == "large":
        characterize("larger_sources", saved["base"], len(view["sources"]))
    elif stage == "evolution":
        before = refresh(saved["base"])["working_state"]
        cli("case", "participant", "role", "add", CASE, "--participant", "participant:irrelevant", "--role", "newer-is-not-relevant-778")
        irrelevant = refresh(before)["working_state"]
        assert "newer-is-not-relevant-778" not in json.dumps(groups(irrelevant))
        # Explicit external intervention, NOT a claimed YAI corrective effect.
        # Acquisition records the changed schema through governed observation.
        with sqlite3.connect(run / "database/project.sqlite") as db:
            db.execute("ALTER TABLE billing ADD COLUMN retention_days INT DEFAULT 90")
        cli("case", "sources", "acquire", CASE, "--source", "database", "--refresh", "--json")
        evolved = characterize("new_observation", saved["base"], len(view["sources"]))
        assert evolved["working_state"]["recall"]["recall_id"] != before["recall"]["recall_id"]
        assert any(u["unit"]["posture"] == "recorded_observation" and "DEFAULT 90" in str(u["unit"]["value"])
            for g in groups(evolved["working_state"]) if g["documentary"] for u in g["documentary"]["units"])
        assert any(e["event"]["kind"] == "resource_observation_recorded" for g in groups(evolved["working_state"]) for e in g["events"])
        saved["before_policy"] = evolved["working_state"]
        saved["before_policy_paged"] = compile(QUERY, "--paged", "--resident-groups", "0")["working_state"]
        print(json.dumps({"refresh_flagship":evolved, "intervention":"external SQLite change then governed YAI acquisition/Observation, no new task prompt"}), flush=True)
    elif stage == "policy_revoke":
        base = saved["before_policy"]
        assert json.loads(cli("case", "show", CASE, "--json", raw=True))["data"]["case"]["generation"] == base["case_generation"]
        # Mandatory Resource documentary backing can now be unavailable; a
        # refusal is the safe outcome, not historical permission resurrection.
        refresh(base, reject=True)
        refreshed = refresh(saved["before_policy_paged"], "--projection")
        assert not refreshed["assessment"]["current_control_equal"]
        assert not catalog(refreshed["working_state"])
        assert all(not g["documentary"] or not g["documentary"]["units"] for g in groups(refreshed["working_state"]))
        print(json.dumps({"semantic_refresh_product":"PASS", "new_prompts":0, "refresh_transitions":0,
            "model_calls":0, "existing_recall_discovery":True}), flush=True)


if __name__ == "__main__":
    for history_pressure in ([0] if "--short-only" in sys.argv else [0, 256]):
        saved.clear()
        main(oracle)
    print(json.dumps({"refresh_characterization":profiles}), flush=True)
