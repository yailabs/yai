import json,sys,time
from pathlib import Path
sys.path.insert(0,'tools/validation')
from behavioral_corpus import Host
p=Path(__file__).parent
records=[json.loads(l) for l in (p/'exchanges.jsonl').read_text().splitlines()]
original=next(r for r in records if r.get('operation')=='conversation.send')
h=Host('/home/mothx/.yai'); requests=[]
def call(op,inputs):
 r=h.call(op,inputs,'4k-retained:'+str(time.time_ns()));requests.append(dict(operation=op,input=inputs,result=r));return r
scope={k:original['input'][k] for k in ('case_ref','participant_ref')}
before=call('case.summary',{'case_ref':scope['case_ref']})
retry=call('conversation.send',original['input'])
assert retry['result_state']=='success',retry
assert retry['data']['created'] is False
assert retry['data']['execution']['request_ref']==original['result']['data']['execution']['request_ref']
query=dict(scope,execution=dict(domain='conversation',submission_ref=original['input']['submission_ref']),include_context=True)
observed=call('execution.get',query)
assert observed['data']['posture']=='completed'
assert observed['data']['primary_result']['output']
hidden=call('execution.get',dict(query,participant_ref='participant:behavioral-unlinked'))
assert hidden['result_state']=='unauthorized' and 'data' not in hidden,hidden
after=call('case.summary',{'case_ref':scope['case_ref']})
assert before['data']==after['data'],'Retry or observation changed canonical state'
assert len(observed['data']['attempt_outcomes'])==1
(p/'retry-exchanges.json').write_text(json.dumps(requests,indent=2))
print(json.dumps(dict(result='PASS',case_ref=scope['case_ref'],request_ref=retry['data']['execution']['request_ref'],result_ref=observed['data']['primary_result']['result_id'],attempts=1,generation_before=before['data']['case']['generation'],generation_after=after['data']['case']['generation'],hidden='unauthorized',language_quality='NOT_ASSESSED')))
