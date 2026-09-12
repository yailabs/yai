#!/usr/bin/env python3
"""One actual product Case, existing M07 bootstrap fixture, integrated Recall.

No independent intake, model, persistence fixture substitute or cached PASS.
"""
import json
import sqlite3
from test_knowledge import main, CASE

saved = {}
history_pressure = 0
characterizations = []


def oracle(stage, cli, run, view):
    def recall(query="retention_days billing", *flags, **kwargs):
        return cli("case", "recall", CASE, query, *flags, "--json", **kwargs)

    if stage == "prepare":
        for n in range(history_pressure):
            cli("case", "participant", "role", "add", CASE,
                "--participant", "participant:distractor", "--role", f"unrelated-{n}")
    elif stage == "initial":
        r = recall()
        t = r["trace"]
        assert t["schema"] == "yai.recall_trace.v2"
        d = t["documentary"]
        assert {u["unit"]["value"] for u in d["units"] if u["unit"]["predicate"] == "retention_days"} >= {30, 90}
        assert d["contradictions"] and all("unresolved" in c["posture"] for c in d["contradictions"])
        assert all(u["unit"]["posture"] == "source_stated" for u in d["units"] if u["unit"]["predicate"] == "retention_days")
        # Ordinary shared Resource anchor is resolved below presentation, not a
        # document text match. It preserves real Decisions and observations.
        anchored = recall("billing sql", "--ref", "resource:database")["trace"]
        assert any(e["event"]["kind"] == "decision_recorded" for e in anchored["events"])
        assert any(e["event"]["kind"] == "resource_observation_recorded" for e in anchored["events"])
        assert anchored["documentary"]["units"]
        assert {u["unit"]["posture"] for u in anchored["documentary"]["units"]} >= {"source_stated", "recorded_observation"}
        assert any(r["kind"] == "decision_observation" for r in anchored["relations"])
        saved.update(initial=t, cut=t["generation"], view=view, anchored=anchored)
        saved["decision"] = next(ref for e in anchored["events"] for ref in e["event"]["object_refs"] if ref.startswith("decision:"))
        for s in d["sources"]:
            assert s["admitted_at_generation"] <= t["generation"] and s["admission_transition"]
        policy = next(s for s in view["sources"] if s["logical_name"] == "policy")
        dual = recall("policy", "--ref", policy["revision_id"])["trace"]
        assert dual["documentary"]["sources"] and dual["closure_complete"]
        assert all(u["unit"]["posture"] != "control_state" for u in dual["documentary"]["units"])
        saved["small"] = r["measurements"]
        characterizations.append(dict(D="small", H_extra=history_pressure, generation=t["generation"], sources=len(view["sources"]), **r["measurements"]))
        print(json.dumps({"integrated_flagship": anchored, "raw_knowledge_hits": cli("case", "knowledge", "search", CASE, "billing sql", "--json")["hits"]}), flush=True)
    elif stage == "rebuild":
        assert recall()["trace"] == saved["initial"]
        assert recall("billing sql", "--ref", "resource:database")["trace"] == saved["anchored"]
    elif stage == "missing":
        s = next(s for s in view["sources"] if s["logical_name"] == "handbook")
        t = recall("handbook", "--ref", s["revision_id"])["trace"]
        assert not t["closure_complete"]
        assert any(s["source"]["status"] == "backing_unavailable" for s in t["documentary"]["sources"])
        assert all(u["unit"]["source"] != s["id"] for u in t["documentary"]["units"])
    elif stage == "revision":
        now = recall()["trace"]
        old = recall("retention_days billing", "--at", saved["cut"])["trace"]
        current_source = next(s for s in view["sources"] if s["logical_name"] == "architecture")
        old_source = next(s for s in saved["view"]["sources"] if s["logical_name"] == "architecture")
        assert current_source["revision_id"] not in json.dumps(old)
        assert any(s["source"]["revision_id"] == old_source["revision_id"] for s in old["documentary"]["sources"])
        assert all(s["source"]["revision_id"] != old_source["revision_id"] for s in now["documentary"]["sources"])
        historical = recall("retention_days", "--ref", old_source["revision_id"])["trace"]
        assert any(not s["source"]["current_revision"] and not s["applicable_at_cut"] for s in historical["documentary"]["sources"])
        recall("billing", "--at", saved["cut"], "--ref", current_source["revision_id"], reject=True)
    elif stage == "revoke":
        revoked = next(s for s in saved["view"]["sources"] if s["logical_name"] == "config")
        for flags in [(), ("--at", saved["cut"])]:
            result = recall("retention_days billing", *flags)
            assert revoked["id"] not in json.dumps(result)
            assert revoked["revision_id"] not in json.dumps(result)
        hidden = recall("billing", "--ref", revoked["revision_id"], reject=True)
        absent = recall("billing", "--ref", "source-revision:absent", reject=True)
        assert hidden == absent
    elif stage == "large":
        r = recall("billing sql source knowledge", "--ref", saved["decision"])
        assert any(saved["decision"] in e["event"]["object_refs"] for e in r["trace"]["events"])
        assert r["trace"]["documentary"]["units"]
        assert len(r["trace"]["candidates"]) <= 16
        assert r["trace"]["candidates"][0]["plane"] != "knowledge_bm25"
        saved["large"] = r["measurements"]
        characterizations.append(dict(D="large", H_extra=history_pressure, generation=r["trace"]["generation"], sources=len(view["sources"]), **r["measurements"]))
    elif stage == "evolution":
        # Exact documentary reference, not an assertion that the document caused
        # the operation. The SQLite edit is an external fixture intervention;
        # only its subsequent governed observation is a YAI-owned lifecycle.
        claim = dict(id="urn:domain:billing", claims={"retention_days":90}, references=[saved["decision"]])
        path = run/"files/architecture.md"
        def revision():
            path.write_text("# Billing corrective investigation\n```yai-knowledge-json\n"+json.dumps(claim)+"\n```\n")
            cli("case", "sources", "acquire", CASE, "--source", "architecture", "--refresh", "--json")
        revision()
        with sqlite3.connect(run/"database/project.sqlite") as db:
            db.execute("ALTER TABLE billing ADD COLUMN retention_days INT DEFAULT 90")
        cli("case", "sources", "acquire", CASE, "--source", "database", "--refresh", "--json")
        before = cli("case", "show", CASE, "--json", raw=True)
        t = recall("billing retention_days sql", "--ref", "resource:database")["trace"]
        assert t["closure_complete"]
        assert any(r["exact_reference"] == saved["decision"] for r in t["documentary"]["cross_references"])
        observations = [e for e in t["events"] if e["event"]["kind"] == "resource_observation_recorded"]
        assert len(observations) >= 2
        assert any(saved["decision"] in e["event"]["object_refs"] for e in t["events"])
        assert any(s["source"]["logical_name"] == "database" and s["source"]["current_revision"] for s in t["documentary"]["sources"])
        assert any(u["unit"]["posture"] == "recorded_observation" and "DEFAULT 90" in str(u["unit"]["value"]) for u in t["documentary"]["units"])
        assert cli("case", "show", CASE, "--json", raw=True) == before
        saved["story"] = dict(trace=t["trace_id"], generation=t["generation"],
            documentary=[u for u in t["documentary"]["units"] if u["unit"]["predicate"]],
            events=[dict(refs=e["event"]["object_refs"], posture=e["event"]["posture"], at=e["event"]["recorded_generation"]) for e in t["events"]],
            cross_references=t["documentary"]["cross_references"],
            relations=[r["kind"] for r in t["relations"]],
            intervention="external fixture SQLite ALTER; not a claimed YAI corrective effect")
        claim.pop("references")
        revision()
        counterfactual = recall("billing retention_days sql", "--ref", saved["decision"])["trace"]
        assert not any(r["exact_reference"] == saved["decision"] for r in counterfactual["documentary"]["cross_references"])
        # Exact original Decision remains mandatory even after its documentary
        # reference is removed. Removing a relation does not erase the Decision.
        assert any(saved["decision"] in e["event"]["object_refs"] for e in counterfactual["events"])
    elif stage == "policy_revoke":
        t = recall()["trace"]
        assert not t["documentary"]["sources"] and not t["documentary"]["units"]
        for s in saved["view"]["sources"]:
            assert s["id"] not in json.dumps(t)
        print(json.dumps({"integrated_recall_product": "PASS", "small": saved["small"], "large": saved["large"],
            "v1_preserved": "separate Rust regression", "asof_future_revision": "excluded", "historical_permission": "not resurrected",
            "rebuild_restart": "equal", "missing_backing": "explicit despite live original", "models": 0, "W_integration": False}), flush=True)
        print(json.dumps({"qualified_product_story": saved["story"]}), flush=True)


if __name__ == "__main__":
    main(oracle)
    history_pressure = 512
    saved.clear()
    main(oracle)
    print(json.dumps({"D_H_characterization": characterizations}), flush=True)
