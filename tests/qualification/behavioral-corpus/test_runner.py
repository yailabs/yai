import copy
import importlib.util
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[3]
spec = importlib.util.spec_from_file_location("corpus", ROOT / "tools/validation/behavioral_corpus.py")
corpus = importlib.util.module_from_spec(spec)
spec.loader.exec_module(corpus)


class Client:
    def __init__(self):
        self.calls = []

    def call(self, operation, inputs, correlation):
        self.calls.append((operation, inputs))
        return {"result_state": "success", "data": {"case_ref": inputs["case_ref"]}}


class CorpusTest(unittest.TestCase):
    def observation_test(self):
        test = copy.deepcopy(self.test)
        test['steps'][0]['observe'] = dict(path='/data/posture', **{'while':['running']}, max_observations=3, interval_ms=0)
        return test

    def test_bounded_observation_uses_read_only_exact_inputs(self):
        host = Client()
        original = host.call
        def call(*args):
            result = original(*args)
            result['data']['posture'] = 'running' if len(host.calls) < 3 else 'completed'
            return result
        host.call = call
        records = []
        self.assertEqual(corpus.evaluate(self.observation_test(), {}, {'case_ref':'case:A'}, host,
            {'case.summary':'read'}, False, records.append), 'PASS')
        self.assertEqual(host.calls, [('case.summary', {'case_ref':'case:A'})] * 3)
        self.assertEqual([row['observation'] for row in records], [0,1,2])

    def test_pending_is_not_failure_or_success_and_never_resubmits(self):
        host = Client()
        original = host.call
        def call(*args):
            result = original(*args); result['data']['posture'] = 'running'; return result
        host.call = call
        with self.assertRaises(corpus.PendingObservation):
            corpus.evaluate(self.observation_test(), {}, {'case_ref':'case:A'}, host,
                {'case.summary':'read'}, False, lambda _:None)
        self.assertEqual(len(host.calls), 3)

    def test_observation_cannot_repeat_effects_or_derived_computation(self):
        for impact in ['canonical_mutation', 'external_effect', 'derived_computation']:
            host = Client()
            with self.assertRaises(ValueError):
                corpus.evaluate(self.observation_test(), {}, {'case_ref':'case:A'}, host,
                    {'case.summary':impact}, True, lambda _:None)
            self.assertEqual(host.calls, [])

    def test_observation_transport_loss_is_not_retried(self):
        host = Client()
        def call(*args):
            host.calls.append(args); raise OSError('lost connection')
        host.call = call
        with self.assertRaises(OSError):
            corpus.evaluate(self.observation_test(), {}, {'case_ref':'case:A'}, host,
                {'case.summary':'read'}, False, lambda _:None)
        self.assertEqual(len(host.calls), 1)

    def setUp(self):
        self.test = dict(id="identity", dimensions=["ISOLATES"], meaning="Exact Case identity",
                         preconditions=[], required_refs=[], allowed_actions=["case.summary"],
                         forbidden_actions=[], expected_consequences=[], repeatability="read", mode="deterministic",
                         steps=[dict(id="read", operation="case.summary", input={"case_ref": {"$ref": "/profile/case_ref"}})],
                         assertions=[dict(path="/results/read/data/case_ref", op="equal", value={"$ref": "/profile/case_ref"})])

    def test_variant_identity_order_independent_and_bounded(self):
        test = copy.deepcopy(self.test)
        test["variants"] = {"participant": ["A", "B"], "cut": [1, 2]}
        a = corpus.variants(dict(schema=corpus.SCHEMA, evaluations=[test]), 4)
        test["variants"] = {"cut": [1, 2], "participant": ["A", "B"]}
        b = corpus.variants(dict(schema=corpus.SCHEMA, evaluations=[test]), 4)
        self.assertEqual([x[2] for x in a], [x[2] for x in b])
        self.assertEqual(len({x[2] for x in a}), 4)
        with self.assertRaises(ValueError):
            corpus.variants(dict(schema=corpus.SCHEMA, evaluations=[test]), 3)
        test["variants"]["cut"] = [1, 1]
        with self.assertRaises(ValueError):
            corpus.variants(dict(schema=corpus.SCHEMA, evaluations=[test]), 64)

    def test_semantic_mismatch_is_failure(self):
        host = Client()
        self.test["assertions"][0]["value"] = "case:B"
        with self.assertRaises(AssertionError):
            corpus.evaluate(self.test, {}, {"case_ref": "case:A"}, host,
                            {"case.summary": "read"}, False, lambda _: None)
        self.assertEqual(host.calls, [("case.summary", {"case_ref": "case:A"})])

    def test_effect_requires_opt_in_before_dispatch(self):
        host = Client()
        with self.assertRaises(ValueError):
            corpus.evaluate(self.test, {}, {"case_ref": "case:A"}, host,
                            {"case.summary": "external_effect"}, False, lambda _: None)
        self.assertEqual(host.calls, [])

    def test_forbidden_operation_and_unknown_catalog_fail_closed(self):
        host = Client()
        for impacts in ({}, {"case.summary": "read"}):
            self.test["forbidden_actions"] = ["case.summary"]
            with self.assertRaises(ValueError):
                corpus.evaluate(self.test, {}, {}, host, impacts, True, lambda _: None)
        self.assertEqual(host.calls, [])

    def test_no_steps_never_passes(self):
        self.test["steps"] = []
        self.assertEqual(corpus.evaluate(self.test, {}, {}, Client(), {}, False, lambda _: None), "NOT_RUN")

    def test_invalid_later_step_refuses_before_any_dispatch(self):
        host = Client()
        self.test["steps"].append(copy.deepcopy(self.test["steps"][0]))
        with self.assertRaises(ValueError):
            corpus.evaluate(self.test, {}, {"case_ref": "case:A"}, host,
                            {"case.summary": "read"}, True, lambda _: None)
        self.assertEqual(host.calls, [])
        self.test["steps"][1].update(id="later", operation="case.cancel")
        with self.assertRaises(ValueError):
            corpus.evaluate(self.test, {}, {"case_ref": "case:A"}, host,
                            {"case.summary": "read", "case.cancel": "mutation"}, True, lambda _: None)
        self.assertEqual(host.calls, [])

    def test_exact_identity_and_evidence(self):
        events = []
        self.assertEqual(corpus.evaluate(self.test, {}, {"case_ref": "case:A"}, Client(),
                                        {"case.summary": "read"}, False, events.append), "PASS")
        self.assertEqual(events[0]["result"]["data"]["case_ref"], "case:A")

    def test_failed_precondition_stops_before_later_effect(self):
        host = Client()
        self.test["steps"][0]["assertions"] = [dict(path="/results/read/data/case_ref", op="equal", value="case:expected")]
        self.test["allowed_actions"].append("effect.submit")
        self.test["steps"].append(dict(id="effect", operation="effect.submit", input={"case_ref":"case:A"}))
        with self.assertRaises(AssertionError):
            corpus.evaluate(self.test, {}, {"case_ref":"case:A"}, host,
                            {"case.summary":"read", "effect.submit":"external_effect"}, True, lambda _: None)
        self.assertEqual(len(host.calls), 1)

    def test_transport_loss_does_not_retry(self):
        class Lost(Client):
            def call(self, *args):
                self.calls.append(args)
                raise ConnectionError("lost acknowledgement")
        host = Lost()
        with self.assertRaises(ConnectionError):
            corpus.evaluate(self.test, {}, {"case_ref": "case:A"}, host,
                            {"case.summary": "read"}, False, lambda _: None)
        self.assertEqual(len(host.calls), 1)

    def test_overlay_preserves_generic_semantic_identity(self):
        base = dict(schema=corpus.SCHEMA, evaluations=[self.test])
        merged = corpus.overlay(base, dict(schema=corpus.SCHEMA,
            evaluations=[dict(id="identity", preconditions=["Case-specific generation"])]))
        self.assertEqual(merged["evaluations"][0]["meaning"], self.test["meaning"])
        self.assertEqual(base["evaluations"][0]["preconditions"], [])

    def test_some_does_not_accept_other_source_or_truthy_substitute(self):
        assertion = dict(path="/sources", op="some", value={"ref": "source:A", "visible": True})
        with self.assertRaises(AssertionError):
            corpus.assert_result(assertion, {"sources": [{"ref": "source:B", "visible": True}]})
        with self.assertRaises(AssertionError):
            corpus.assert_result(assertion, {"sources": [{"ref": "source:A", "visible": 1}]})
        nested = dict(path="/sources", op="some", value={"/source/revision_id":"revision:A", "/source/digest":"sha256:exact"})
        with self.assertRaises(AssertionError):
            corpus.assert_result(nested, {"sources":[{"source":{"revision_id":"revision:B","digest":"sha256:exact"}}]})
        corpus.assert_result(nested, {"sources":[{}, {"source":{"revision_id":"revision:A","digest":"sha256:exact"}}]})

    def test_nested_authority_values_are_not_python_truthiness(self):
        actual = {"state": {"grants": [{"admitted": 1}]}}
        with self.assertRaises(AssertionError):
            corpus.assert_result(dict(path="/state", op="equal", value={"grants":[{"admitted":True}]}), actual)
        with self.assertRaises(AssertionError):
            corpus.assert_result(dict(path="/state/grants", op="contains", value={"admitted":True}), actual)
        corpus.assert_result(dict(path="/state/grants", op="excludes", value={"admitted":True}), actual)


if __name__ == "__main__":
    unittest.main()
