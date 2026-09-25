import json,sys,time,subprocess
from pathlib import Path
root=Path.cwd();sys.path.insert(0,str(root/'tools/validation'))
from behavioral_corpus import Host
source=Path('/home/mothx/.cache/tmp/native-provider-probe-1790314416245781447/observations.jsonl')
rows=[json.loads(x) for x in source.read_text().splitlines()]
before=next(x['result']['data'] for x in rows if x.get('operation')=='case.summary')
ref='studio-probe:63a3e514-45b1-4bad-8b4b-4fb8e71aab0e';target='provider-target:965ce8d788b4d6321ed0fea629f99546'
run='provider-probe-continuity-read-'+str(time.time_ns());host=Host(Path('/home/mothx/.yai'))
print(json.dumps({'run_id':run,'order':1,'source_run':str(source),'sha':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'prestate':'Retained completed probe; prior screenshot timeout; observation only','command':sys.argv,'cwd':str(root),'yai_home':'/home/mothx/.yai'}))
after=host.call('case.summary',{'case_ref':before['case']['id']},run+':summary') if 'id' in before['case'] else host.call('case.summary',{'case_ref':'case:qualification-deepseek-native-4k'},run+':summary')
print(json.dumps({'run_id':run,'order':2,'operation':'case.summary','result':after}))
assert after['result_state']=='success'
assert after['data']['case']==before['case']
assert after['data']['conversation']==before['conversation']
probe=host.call('provider.probe.get',{'target_ref':target,'submission_ref':ref},run+':probe')
print(json.dumps({'run_id':run,'order':3,'operation':'provider.probe.get','result':probe}))
assert probe['result_state']=='success' and probe['data']['posture']=='completed'
assert probe['data']['run']['evidence']['exact_model_addressed']
print(json.dumps({'run_id':run,'order':4,'result':'PASS','proof':'Read-only retained probe and unchanged Case/Conversation compared with linked pre-state; no redispatch'}))
