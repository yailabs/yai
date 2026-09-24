import copy
import json
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
    def test_not_equal_is_structural_and_type_strict(self):
        corpus.assert_result(dict(path="/value",op="not_equal",value=True),dict(value=1))
        corpus.assert_result(dict(path="/value",op="not_equal",value=None),dict(value={"id":"retained"}))
        for value in [None, "", {"id":"same"}]:
            with self.subTest(value=value), self.assertRaises(AssertionError):
                corpus.assert_result(dict(path="/value",op="not_equal",value=value),dict(value=value))

    def test_governed_question_corpus_rejects_content_or_identity_substitution(self):
        suite = json.loads((Path(__file__).parent / "conversation.json").read_text())
        test = suite['evaluations'][0]
        profile = dict(case_ref='case:A', participant_ref='participant:A', target_ref='target:A',
            model_id='model:A', thread_ref='thread:A', submission_ref='submission:A',
            question='Question A', question_utf8=list(b'Question A'))
        observation = dict(case_id='case:A',participant_id='participant:A',target_id='target:A',model_id='model:A',refusal=None)
        context = dict(omitted_invocations=0, invocations=[dict(unavailable_reason=None,
            working_state={'id':'W'},projection={'id':'P'},frame={'id':'F'},input_observation=observation)])
        execution = dict(case_ref='case:A',participant_ref='participant:A',turn_ref='turn:A',request_ref='request:A',
            posture='completed',primary_result=dict(output='Candidate answer',selection=dict(selected_target_id='target:A')))
        class Replay:
            def __init__(self, alter=None): self.sends=0; self.alter=alter
            def call(self, operation, inputs, correlation):
                if operation == 'case.summary':
                    data=dict(case=dict(case_ref='case:A',generation=1),conversation=dict(turns=[dict(id='turn:A',parts=[dict(text='Question A')])]))
                elif operation == 'conversation.send':
                    self.sends+=1;data=dict(created=self.sends==1,execution=copy.deepcopy(execution))
                elif inputs['participant_ref'] != 'participant:A': return dict(result_state='unauthorized')
                else:
                    data=copy.deepcopy(execution)
                    if inputs.get('include_context'): data['prepared_context']=copy.deepcopy(context)
                response=dict(result_state='success',data=data)
                if self.alter:self.alter(operation,inputs,response)
                return response
        impacts={'case.summary':'read','execution.get':'read','conversation.send':'external_effect'}
        self.assertEqual(corpus.evaluate(test,{},profile,Replay(),impacts,True,lambda _:None),'PASS')
        def change(path, value):
            def alter(operation,inputs,response):
                if operation!='execution.get' or not inputs.get('include_context'):return
                target=response['data']
                for key in path[:-1]:target=target[key]
                target[path[-1]]=value
            return alter
        for path,value in [(['case_ref'],'case:B'),(['primary_result','output'],''),
            (['primary_result','selection','selected_target_id'],'target:B'),
            (['prepared_context','invocations',0,'working_state'],None),
            (['prepared_context','invocations',0,'input_observation','model_id'],'model:B')]:
            with self.subTest(path=path),self.assertRaises(AssertionError):
                corpus.evaluate(test,{},profile,Replay(change(path,value)),impacts,True,lambda _:None)
        host=Replay()
        with self.assertRaises(ValueError):corpus.evaluate(test,{},profile,host,impacts,False,lambda _:None)
        self.assertEqual(host.sends,0)

    def test_retained_conversation_checks_exact_input_and_current_disclosure_without_send(self):
        suite=json.loads((Path(__file__).parent / 'retained-conversation.json').read_text())
        profile=dict(case_ref='case:A',participant_ref='participant:A',request_ref='request:A',
            turn_ref='turn:A',result_ref='result:A',target_ref='target:A',invocation_ref='invocation:A',
            input_observation_ref='input:A',serialized_request_digest='digest:A',model_id='model:A')
        class Reader:
            def __init__(self, alteration=None): self.alteration=alteration; self.calls=[]
            def call(self, op, inputs, correlation):
                self.calls.append(op)
                if op=='case.summary': return dict(result_state='success',data={'case':{'generation':7}})
                if inputs['participant_ref']=='participant:behavioral-unlinked':
                    return dict(result_state='unauthorized', **({'data':{'output':'secret'}} if self.alteration=='leak' else {}))
                data={k:profile[k] for k in ['case_ref','participant_ref','request_ref','turn_ref']}
                observation=dict(observation_id='input:A',serialized_request_digest='digest:A',case_id='case:A',
                    participant_id='participant:A',model_id='model:A',target_id='target:A',refusal=None)
                data.update(posture='completed',primary_result=dict(result_id='result:A',selection={'selected_target_id':'target:A'}),
                    prepared_context={'invocations':[{'invocation_ref':'invocation:A','input_observation':observation}]})
                if self.alteration=='digest': observation['serialized_request_digest']='different'
                if self.alteration=='result': data['primary_result']['result_id']='different'
                if self.alteration=='context': data['prepared_context']['invocations']=[]
                return dict(result_state='success',data=data)
        impacts={'case.summary':'read','execution.get':'read'}
        for test in suite['evaluations']:
            host=Reader()
            self.assertEqual(corpus.evaluate(test,{},profile,host,impacts,False,lambda _:None),'PASS')
            self.assertTrue(set(host.calls)<=set(impacts))
        for mutation in ['digest','result','context']:
            with self.subTest(mutation=mutation),self.assertRaises(AssertionError):
                corpus.evaluate(suite['evaluations'][0],{},profile,Reader(mutation),impacts,False,lambda _:None)
        with self.assertRaises(AssertionError):
            corpus.evaluate(suite['evaluations'][1],{},profile,Reader('leak'),impacts,False,lambda _:None)

    def retained_action(self):
        suite = json.loads((Path(__file__).parent / "retained-action.json").read_text())
        profile = dict(case_ref="case:A", participant_ref="participant:A", submission_ref="request:A",
            operation_ref="operation:A", receipt_ref="receipt:A", effect_ref="effect:A", result_ref="result:A",
            outcome="applied", external_execution_started=True)
        class RetainedClient:
            def call(self, operation, inputs, correlation):
                if operation == "case.summary":
                    return dict(result_state="success", data=dict(case=dict(case_ref="case:A", generation=7)))
                if inputs['participant_ref'] != 'participant:A':
                    return dict(result_state="unauthorized")
                return dict(result_state="success", data=dict(case_ref="case:A", participant_ref="participant:A",
                    operation_ref="operation:A", posture=dict(state="effect_recorded", receipt_ref="receipt:A",
                    effect_ref="effect:A", result_ref="result:A", outcome="applied", external_execution_started=True)))
        return suite, profile, RetainedClient()

    def test_retained_action_suite_exact_receipt_and_hidden_participant(self):
        suite, profile, host = self.retained_action()
        for test, variant, _ in corpus.variants(suite, 3):
            self.assertEqual(corpus.evaluate(test, variant, profile, host,
                {'case.summary':'read', 'execution.get':'read'}, False, lambda _:None), 'PASS')

    def test_retained_action_suite_rejects_identity_and_outcome_substitution(self):
        suite, profile, host = self.retained_action()
        for field in ['case_ref', 'participant_ref', 'operation_ref', 'receipt_ref', 'effect_ref', 'result_ref',
                      'outcome', 'external_execution_started']:
            wrong = dict(profile, **{field:False if field == 'external_execution_started' else 'wrong'})
            with self.subTest(field=field), self.assertRaises(AssertionError):
                corpus.evaluate(suite['evaluations'][0], {}, wrong, host,
                    {'case.summary':'read', 'execution.get':'read'}, False, lambda _:None)

    def test_retained_action_suite_rejects_disclosure_in_refusal(self):
        suite, profile, host = self.retained_action()
        original = host.call
        def leaking(*args):
            result = original(*args)
            if result['result_state'] == 'unauthorized':
                result['data'] = {'receipt_ref':'receipt:A'}
            return result
        host.call = leaking
        with self.assertRaises(AssertionError):
            corpus.evaluate(suite['evaluations'][2], {}, profile, host,
                {'case.summary':'read', 'execution.get':'read'}, False, lambda _:None)

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
