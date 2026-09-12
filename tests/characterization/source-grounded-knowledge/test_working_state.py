#!/usr/bin/env python3
"""Actual task -> Recall v2 -> W product proof; no model or second intake.

Runs the existing admitted multi-format source/operation fixture twice, with
independent run IDs. No retained PASS cache; each assertion reopens real LMDB.
"""
import json
import sqlite3
from test_knowledge import main, CASE, PERSON

saved = {}
history_pressure = 0
profiles = []


def groups(w):
    return [e["value"]["value"]["evidence"] for e in w["entries"] if e["value"]["kind"] == "recalled_evidence"]


def control(w):
    return [e for e in w["entries"] if e["posture"] in ("control_state", "unresolved")
            and e["value"]["kind"] != "execution_intent"]


def measure(r, view, size):
    w = r["working_state"]
    evidence = groups(w)
    return dict(D=size, H_extra=history_pressure, generation=w["case_generation"], sources=len(view["sources"]),
        working_items=len(w["entries"]), working_groups=len(evidence),
        working_events=len({e["event"]["transition_id"] for g in evidence for e in g["events"]}),
        working_documentary_units=len({u["unit"]["id"] for g in evidence if g["documentary"] for u in g["documentary"]["units"]}),
        **r["measurements"])


def oracle(stage, cli, run, view):
    def working(query="billing retention_days sql", *flags, **kwargs):
        return cli("case", "context", "compile", CASE, query, "--units", "131072", *flags, "--json", **kwargs)

    def plain(query, *flags, **kwargs):
        return cli("case", "context", "compile", CASE, query, *flags, "--json", **kwargs)

    def unchanged(f):
        before = cli("case", "show", CASE, "--json", raw=True)
        result = f()
        assert before == cli("case", "show", CASE, "--json", raw=True)
        return result

    if stage == "prepare":
        cli("case", "participant", "view", "admit", CASE, "--participant", PERSON,
            "--consumer", "model", "--view", "model_context")
        for n in range(history_pressure):
            cli("case", "participant", "role", "add", CASE,
                "--participant", "participant:distractor", "--role", f"unrelated-{n}")
    elif stage == "initial":
        r = unchanged(lambda: working("billing retention_days sql", "--ref", "resource:database", "--projection"))
        w = r["working_state"]
        assert w["schema"] == "yai.semantic_working_state.v3"
        encoded = json.dumps(w, separators=(",", ":"), ensure_ascii=False)
        assert (len(encoded) + 3) // 4 == w["bounds"]["selected_semantic_units"]
        assert len(encoded.encode()) == r["measurements"]["output_bytes"]
        assert w["recall"]["recall_request"]["schema"] == "yai.recall_request.v2"
        assert r["compilation_mode"] == "FullRecompilation"
        assert r["projection"]["entries"] == w["entries"], "lowering must not select again"
        assert r["projection"]["schema"] == "yai.projection.v11"
        qualified = next(e["value"]["value"] for e in w["entries"] if e["value"]["kind"] == "recall_qualification")
        assert qualified["working_omitted_items"] == w["bounds"]["omitted_items"]
        assert qualified["closure_complete"] == w["recall"]["recall_closure_complete"]
        assert control(w) and any(e["value"]["kind"] == "effective_authority" for e in control(w))
        assert any(e["value"]["kind"] == "execution_intent" for e in w["entries"])
        recalled = groups(w)
        assert recalled
        documents = [u["unit"] for g in recalled if g["documentary"] for u in g["documentary"]["units"]]
        print(json.dumps({"working_selection_observed": {"bounds": w["bounds"], "groups": len(recalled), "documentary_postures": [u["posture"] for u in documents], "recall": w["recall"], "decisions": w["decisions"]}}), flush=True)
        assert "source_stated" in {u["posture"] for u in documents}
        assert any(e["event"]["kind"] == "resource_observation_recorded" for g in recalled for e in g["events"])
        assert any(g["documentary"]["contradictions"] for g in recalled if g["documentary"])
        assert any(e["event"]["kind"] == "decision_recorded" for g in recalled for e in g["events"])
        assert all(e["posture"] == "derived_memory" for e in w["entries"] if e["value"]["kind"] == "recalled_evidence")
        assert all("candidates" not in g and "request" not in g and "trace_id" not in g for g in recalled)
        ordinary = working()["working_state"]
        other = working("authentication", "--resource", "resource:unrelated")["working_state"]
        assert not groups(other), "exact task focus excludes unsupported attractive memory"
        assert control(ordinary) == control(other), "task switch retains Case constraints"
        assert ordinary["working_state_id"] != other["working_state_id"]
        # Qualified default envelope, then atomically pressured envelope. Exact
        # dependency cannot be replaced or truncated to make either fit.
        default = plain("billing retention_days sql")["working_state"]
        assert default["bounds"]["selected_semantic_units"] <= 32768
        assert control(default) == control(ordinary)
        assert all(g in groups(ordinary) for g in groups(default)), "budget cannot split a qualified evidence group"
        assert any(g["documentary"] and g["documentary"]["units"] for g in groups(default))
        pressured = plain("billing retention_days sql", "--units", "16384")["working_state"]
        assert pressured["bounds"]["omitted_by_budget"] > 0
        assert control(pressured) == control(ordinary)
        assert all(g in groups(ordinary) for g in groups(pressured))
        plain("billing", "--ref", "resource:database", "--units", "1", reject=True)
        plain("billing", "--ref", "decision:absent", reject=True)
        plain("billing", "--participant", "participant:unlinked", reject=True)
        saved.update(initial=w, ordinary=ordinary, cut=w["case_generation"], view=view,
            handbook=next(s for s in view["sources"] if s["logical_name"] == "handbook"))
        saved["decision"] = next(ref for g in recalled for e in g["events"] for ref in e["event"]["object_refs"] if ref.startswith("decision:"))
        saved["handbook_w"] = working("handbook")["working_state"]
        profiles.append(measure(r, view, "small"))
        print(json.dumps({"working_flagship": w, "default_omissions": default["bounds"]}), flush=True)
        assert "CASE WORKING STATE" in cli("case", "context", "compile", CASE, "billing")
    elif stage == "rebuild":
        w = unchanged(lambda: working("billing retention_days sql", "--ref", "resource:database"))["working_state"]
        assert w == saved["initial"], "restart and graph/index/memory rebuild must preserve W identity"
    elif stage == "missing":
        w = unchanged(lambda: working("handbook"))["working_state"]
        assert w["case_generation"] == saved["handbook_w"]["case_generation"]
        assert w["working_state_id"] != saved["handbook_w"]["working_state_id"], "R changes outside S generation"
        assert all(not g["documentary"] or all(u["unit"]["source"] != saved["handbook"]["id"] for u in g["documentary"]["units"]) for g in groups(w))
        working("handbook", "--ref", saved["handbook"]["revision_id"], reject=True)
        assert (run/"files/handbook.pdf").exists(), "live file cannot replace lost exact backing"
    elif stage == "revision":
        current = working()["working_state"]
        old = working("billing retention_days sql", "--at", saved["cut"])["working_state"]
        assert control(current) == control(old), "as-of evidence does not rewind authority"
        new_revision = next(s["revision_id"] for s in view["sources"] if s["logical_name"] == "architecture")
        assert new_revision not in json.dumps(groups(old))
        assert new_revision in json.dumps(groups(current))
        old_revision = next(s["revision_id"] for s in saved["view"]["sources"] if s["logical_name"] == "architecture")
        historical = working("billing", "--ref", old_revision)["working_state"]
        assert any(not s["source"]["current_revision"] for g in groups(historical) if g["documentary"] for s in g["documentary"]["sources"])
    elif stage == "revoke":
        revoked = next(s for s in saved["view"]["sources"] if s["logical_name"] == "config")
        for flags in [(), ("--at", saved["cut"])]:
            w = working("billing retention_days sql", *flags)["working_state"]
            assert revoked["id"] not in json.dumps(w)
            assert revoked["revision_id"] not in json.dumps(w)
        assert working("billing", "--ref", revoked["revision_id"], reject=True) == working("billing", "--ref", "source-revision:absent", reject=True)
    elif stage == "large":
        r = unchanged(lambda: working("billing sql source knowledge", "--ref", "resource:database"))
        w = r["working_state"]
        assert any(e["event"]["kind"] == "decision_recorded" for g in groups(w) for e in g["events"])
        assert control(w)
        profiles.append(measure(r, view, "large"))
    elif stage == "evolution":
        claim = dict(id="urn:domain:billing", claims={"retention_days": 90}, references=[saved["decision"]])
        path = run / "files/architecture.md"
        def revision():
            path.write_text("# Billing corrective investigation\n```yai-knowledge-json\n" + json.dumps(claim) + "\n```\n")
            cli("case", "sources", "acquire", CASE, "--source", "architecture", "--refresh", "--json")
        revision()
        # External world intervention, explicitly NOT a claimed YAI effect.
        # YAI then acquires/records the changed schema through governed access.
        with sqlite3.connect(run / "database/project.sqlite") as db:
            db.execute("ALTER TABLE billing ADD COLUMN retention_days INT DEFAULT 90")
        inventory = cli("case", "sources", "acquire", CASE, "--source", "database", "--refresh", "--json")
        revision_id = next(s["progress"]["revision"]["revision_id"] for s in inventory["sources"] if s["name"] == "database")
        w = unchanged(lambda: working("billing retention_days sql", "--ref", "resource:database", "--require", revision_id))["working_state"]
        g = groups(w)
        assert any(r["exact_reference"] == saved["decision"] for e in g if e["documentary"] for r in e["documentary"]["cross_references"])
        assert any(u["unit"]["posture"] == "recorded_observation" and "DEFAULT 90" in str(u["unit"]["value"]) for e in g if e["documentary"] for u in e["documentary"]["units"])
        assert len({e["event"]["transition_id"] for group in g for e in group["events"] if e["event"]["kind"] == "resource_observation_recorded"}) >= 2
        print(json.dumps({"working_evolution": w, "intervention": "external SQLite ALTER followed by YAI-governed observation; not a claimed YAI corrective effect"}), flush=True)
        claim.pop("references")
        revision()
        counter = working("billing retention_days sql", "--ref", saved["decision"])["working_state"]
        assert not any(r["exact_reference"] == saved["decision"] for e in groups(counter) if e["documentary"] for r in e["documentary"]["cross_references"])
        saved["before_policy_revoke"] = working()["working_state"]
    elif stage == "policy_revoke":
        w = working()["working_state"]
        assert w["case_generation"] == saved["before_policy_revoke"]["case_generation"]
        assert w["working_state_id"] != saved["before_policy_revoke"]["working_state_id"]
        assert control(w) != control(saved["before_policy_revoke"])
        assert all(not g["documentary"] or not g["documentary"]["units"] for g in groups(w))
        print(json.dumps({"working_product": "PASS", "models": 0, "reads_append_transitions": 0,
            "current_authority_not_recalled_authority": True, "cache_restart_identity": "equal"}), flush=True)


if __name__ == "__main__":
    main(oracle)
    history_pressure = 512
    saved.clear()
    main(oracle)
    print(json.dumps({"working_D_H_characterization": profiles}), flush=True)
