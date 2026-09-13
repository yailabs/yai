#!/usr/bin/env python3
"""Exact paging through the normal CLI and real persisted Case/source owners.

The existing fixture captures each real command/run/pre-state/exit/excerpt.
No provider, manually authored Transition or canonical page/cache owner.
"""
import json
import sys
from test_knowledge import main, CASE, PERSON

saved = {}
profiles = []
history_pressure = 0


def catalog(w):
    return next(e["value"]["value"]["references"] for e in w["entries"]
        if e["value"]["kind"] == "semantic_page_references")


def resident(w):
    ids = {e["entry_id"] for e in w["entries"]}
    return [r["reference_id"] for r in catalog(w) if r["group_entry_id"] in ids]


def current(w):
    return [e for e in w["entries"] if e["value"]["kind"] not in
        ("recalled_evidence", "recall_qualification", "semantic_page_references")]


def oracle(stage, cli, run, view):
    def compile(query="billing retention_days sql", *flags):
        defaults = [v for pair in [("--resident-groups", "1"), ("--units", "131072")]
            if pair[0] not in flags for v in pair]
        return cli("case", "context", "compile", CASE, query, "--paged", *defaults, *flags, "--json")

    def expand(base, ref, *flags, **kwargs):
        human = kwargs.pop("human", False)
        path = run / "working-input.json"
        path.write_text(json.dumps(base))
        before = cli("case", "show", CASE, "--json", raw=True)
        defaults = [v for pair in [("--base-id", base["working_state_id"]), ("--page-units", "131072")]
            if pair[0] not in flags for v in pair]
        result = cli("case", "context", "expand", CASE, "--working-file", path,
            "--ref", ref, *defaults, *flags, *([] if human else ["--json"]), **kwargs)
        assert before == cli("case", "show", CASE, "--json", raw=True), "paging may not mutate canonical state"
        return result

    def choose(w, source=None, deferred=True):
        return next(r for r in catalog(w) if not r["mandatory_task_dependency"]
            and (not deferred or r["reference_id"] not in resident(w))
            and (source is None or any(s["source_id"] == source for s in r["sources"])))

    def measure(base, page, size):
        profiles.append(dict(D=size, history_extra=history_pressure,
            generation=base["working_state"]["case_generation"], source_count=len(view["sources"]),
            initial_recall=base["measurements"], paging=page["measurements"],
            page_out=saved["page_out_measurements"],
            initial_items=base["working_state"]["bounds"]["selected_items"],
            expanded_items=page["working_state"]["bounds"]["selected_items"],
            page_units=page["page"]["semantic_units"], working_units=page["working_state"]["bounds"]["selected_semantic_units"]))

    if stage == "prepare":
        cli("case", "participant", "view", "admit", CASE, "--participant", PERSON, "--consumer", "model", "--view", "model_context")
        for n in range(history_pressure):
            cli("case", "participant", "role", "add", CASE, "--participant", "participant:distractor", "--role", f"paging-distractor-{n}")
    elif stage == "initial":
        before = cli("case", "show", CASE, "--json", raw=True)
        compiled = compile()
        assert before == cli("case", "show", CASE, "--json", raw=True)
        base = compiled["working_state"]
        assert base["schema"] == "yai.semantic_working_state.v4"
        assert catalog(base) and resident(base) and len(resident(base)) < len(catalog(base))
        architecture = next(s for s in view["sources"] if s["logical_name"] == "architecture")
        reference = choose(base, architecture["source_id"])
        page = expand(base, reference["reference_id"], "--projection")
        human = expand(base, reference["reference_id"], human=True)
        assert "SEMANTIC PAGE" in human and "Source " in human and "unresolved disagreements" in human
        assert page["measurements"]["candidate_discovery_passes"] == 0
        assert page["working_state"]["working_state_id"] != base["working_state_id"]
        assert page["working_state"]["paging"]["parent_working_state_id"] == base["working_state_id"]
        assert reference["reference_id"] in resident(page["working_state"])
        assert current(base) == current(page["working_state"])
        assert page["projection"]["entries"] == page["working_state"]["entries"]
        assert page["projection"]["schema"] == "yai.projection.v12"
        documents = [g["documentary"] for g in page["page"]["groups"] if g["documentary"]]
        assert any(d["contradictions"] for d in documents)
        assert "source_stated" in {u["unit"]["posture"] for d in documents for u in d["units"]}
        assert all(g["closure_complete"] for g in page["page"]["groups"])
        encoded = json.dumps(page["page"], ensure_ascii=False, separators=(",", ":"))
        assert len(encoded.encode()) == page["measurements"]["page_bytes"]
        assert (len(encoded) + 3) // 4 == page["page"]["semantic_units"]
        removed = expand(page["working_state"], reference["reference_id"], "--page-out")
        saved["page_out_measurements"] = removed["measurements"]
        assert reference["reference_id"] not in resident(removed["working_state"])
        assert reference["reference_id"] in removed["evicted_references"]
        again = expand(removed["working_state"], reference["reference_id"])
        assert again["page"]["groups"] == page["page"]["groups"]
        assert current(removed["working_state"]) == current(base)
        assert "atomic_closure_exceeds_budget" in expand(base, reference["reference_id"], "--page-units", "1", reject=True)
        assert "atomic_closure_exceeds_budget" in expand(base, reference["reference_id"], "--page-items", "1", reject=True)
        expand(base, reference["reference_id"], "--base-id", "working-state:another-task", reject=True)
        assert expand(base, "semantic-reference:hidden", reject=True) == expand(base, "semantic-reference:absent", reject=True)
        expand(base, reference["reference_id"], "--participant", "participant:outsider", reject=True)
        forged = json.loads(json.dumps(base)); forged["request"]["intent"] = "different task"
        expand(forged, reference["reference_id"], reject=True)
        # A measured current+reference-only envelope cannot fit an incoming
        # mandatory group, even after every optional resident has left.
        empty = compile("billing retention_days sql", "--resident-groups", "0")["working_state"]
        tight = compile("billing retention_days sql", "--resident-groups", "0", "--units", str(empty["bounds"]["selected_semantic_units"] + 64))["working_state"]
        expand(tight, choose(tight, architecture["source_id"])["reference_id"], reject=True)
        mandatory = compile("billing retention_days sql", "--ref", "resource:database", "--resident-groups", "0")["working_state"]
        required = next(r for r in catalog(mandatory) if r["mandatory_task_dependency"])
        assert required["reference_id"] in resident(mandatory)
        expand(mandatory, required["reference_id"], "--page-out", reject=True)
        handbook = next(s for s in view["sources"] if s["logical_name"] == "handbook")
        book_w = compile("handbook", "--resident-groups", "0")["working_state"]
        saved.update(base=base, ref=reference, page=page, cut=base["case_generation"], architecture=architecture,
            handbook=handbook, handbook_w=book_w, handbook_ref=choose(book_w, handbook["source_id"]), view=view)
        saved["decision"] = next(o for entry in base["entries"] if entry["value"]["kind"] == "recalled_evidence"
            for e in entry["value"]["value"]["evidence"]["events"] for o in e["event"]["object_refs"] if o.startswith("decision:"))
        measure(compiled, page, "small")
        print(json.dumps({"paging_flagship": page, "initial_W": base["working_state_id"], "deferred_reference": reference}), flush=True)
    elif stage == "rebuild":
        again = expand(saved["base"], saved["ref"]["reference_id"])
        assert again["page"] == saved["page"]["page"]
        assert again["working_state"] == saved["page"]["working_state"]
    elif stage == "missing":
        error = expand(saved["handbook_w"], saved["handbook_ref"]["reference_id"], reject=True)
        assert "unavailable" in error
        assert (run / "files/handbook.pdf").exists(), "never substitute the still-live original"
    elif stage == "revision":
        expand(saved["base"], saved["ref"]["reference_id"], reject=True)
        old = compile("billing retention_days sql", "--at", saved["cut"], "--resident-groups", "0")["working_state"]
        now = compile("billing retention_days sql", "--resident-groups", "0")["working_state"]
        historical_ref = choose(old, saved["architecture"]["source_id"])
        current_ref = choose(now, saved["architecture"]["source_id"])
        historic = expand(old, historical_ref["reference_id"])
        new_revision = next(s["revision_id"] for s in view["sources"] if s["logical_name"] == "architecture")
        assert new_revision not in json.dumps(historic["page"]["groups"])
        assert saved["architecture"]["revision_id"] in json.dumps(historic["page"]["groups"])
        assert current(historic["working_state"]) == current(now)
        expand(old, current_ref["reference_id"], reject=True)
    elif stage == "revoke":
        assert expand(saved["base"], saved["ref"]["reference_id"], reject=True) == expand(saved["base"], "semantic-reference:absent", reject=True)
        fresh = compile()["working_state"]
        revoked = next(s for s in saved["view"]["sources"] if s["logical_name"] == "config")
        assert revoked["source_id"] not in json.dumps(fresh)
        assert revoked["revision_id"] not in json.dumps(fresh)
    elif stage == "large":
        result = compile()
        w = result["working_state"]
        ref = choose(w, saved["architecture"]["source_id"])
        page = expand(w, ref["reference_id"])
        saved["page_out_measurements"] = expand(page["working_state"], ref["reference_id"], "--page-out")["measurements"]
        assert page["measurements"]["candidate_discovery_passes"] == 0
        assert page["measurements"]["source_documents_resolved"] < len(view["sources"]), "exact paging must not authorize/derive the whole corpus"
        measure(result, page, "large")
    elif stage == "evolution":
        claim = dict(id="urn:domain:billing", claims={"retention_days":90}, references=[saved["decision"]])
        def revision():
            (run / "files/architecture.md").write_text("# Billing\n```yai-knowledge-json\n" + json.dumps(claim) + "\n```\n")
            cli("case", "sources", "acquire", CASE, "--source", "architecture", "--refresh", "--json")
            w = compile("billing retention_days sql", "--resident-groups", "0")["working_state"]
            return w, expand(w, choose(w, saved["architecture"]["source_id"])["reference_id"])
        old, connected = revision()
        assert any(r["exact_reference"] == saved["decision"] for g in connected["page"]["groups"] if g["documentary"] for r in g["documentary"]["cross_references"])
        claim.pop("references")
        fresh, disconnected = revision()
        assert not any(r["exact_reference"] == saved["decision"] for g in disconnected["page"]["groups"] if g["documentary"] for r in g["documentary"]["cross_references"])
        expand(old, choose(old, saved["architecture"]["source_id"])["reference_id"], reject=True)
        saved["before_policy"] = fresh
    elif stage == "policy_revoke":
        base = saved["before_policy"]
        assert json.loads(cli("case", "show", CASE, "--json", raw=True))["data"]["case"]["generation"] == base["case_generation"]
        ref = choose(base, saved["architecture"]["source_id"])
        assert expand(base, ref["reference_id"], reject=True) == expand(base, "semantic-reference:absent", reject=True)
        print(json.dumps({"semantic_paging_product":"PASS", "no_global_discovery":True, "canonical_mutations":0, "model_calls":0}), flush=True)


if __name__ == "__main__":
    for history_pressure in ([0] if "--short-only" in sys.argv else [0, 256]):
        saved.clear()
        main(oracle)
    print(json.dumps({"paging_characterization": profiles}), flush=True)
