import os,sys,json,time,subprocess
from pathlib import Path
sys.path.insert(0,'tools/validation')
from behavioral_corpus import Host
home_value=os.environ.get('YAI_HOME')
if not home_value or not Path(home_value).is_absolute() or Path(home_value).resolve()==(Path.home()/'.yai').resolve():
 raise SystemExit('Historical separate-Case setup is disabled in the operator profile; qualify DeepSeek in case:tech-infra-inference-service or use a fresh isolated YAI_HOME')
run='deepseek-4k-'+str(time.time_ns()); root=Path.cwd(); out=Path('/home/mothx/.cache/tmp')/run; out.mkdir()
h=Host(Path(home_value)); order=0
f=(out/'exchanges.jsonl').open('x')
def call(op,data):
 global order
 order+=1
 result=h.call(op,data,run+':'+str(order))
 f.write(json.dumps(dict(run_id=run,order=order,cwd=str(root),yai_home=str(h.home),host_instance=h.discovery['instance_id'],operation=op,input=data,result=result))+'\n');f.flush()
 if result['result_state']!='success': raise RuntimeError(result)
 return result.get('data')
f.write(json.dumps(dict(run_id=run,order=0,sha=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),prestate='Existing operator target and qualification; create separate small Case, never modify infra Case; no provider administration',evidence_class='external_governed_host'))+'\n');f.flush()
original=call('case.summary',{'case_ref':'case:tech-infra-inference-service'})
binding=original['compute']['cognitive_bindings'][0]; target=binding['target_id']; model=next(t['model_id'] for t in original['compute']['targets'] if t['id']==target)
case='case:qualification-deepseek-4k'; scope=dict(case_ref=case,participant_ref='participant:operator')
call('case.create',dict(case_ref=case,tenant_id='tenant:studio-live-qualification'))
call('participant.role.add',dict(scope,role='operator'))
call('participant.principal.link',dict(scope,principal_ref='self'))
call('provider.case.bind',dict(scope,ordered_target_refs=[target],failover_policy='none',max_attempts_per_turn=1))
call('participant.view.admit',dict(scope,consumer='model',view_kind='model_context'))
call('cognitive.binding.set',dict(scope,role='primary',capability='primary_conversation',candidates=[dict(target_ref=target,semantic_evidence_ref=binding['semantic_evidence_id'])],replace=False))
question='Rispondi in italiano in massimo due frasi: quale scopo di qualificazione suggerisce il nome di questo Case? Distingui ciò che osservi da ciò che non puoi sapere; non inventare documenti o risultati.'
profile=dict(case_ref=case,participant_ref=scope['participant_ref'],target_ref=target,model_id=model,thread_ref='thread:qualification-4k',submission_ref='submission:'+run,question=question,question_utf8=list(question.encode()))
(out/'profile.json').write_text(json.dumps(profile,indent=2))
summary=call('case.summary',dict(case_ref=case))
inputs=dict(scope,thread_ref=profile['thread_ref'],submission_ref=profile['submission_ref'],expected_generation=summary['case']['generation'],parts=[dict(modality='text',media_type='text/plain',bytes=profile['question_utf8'])])
call('conversation.send',inputs)
query=dict(scope,execution=dict(domain='conversation',submission_ref=profile['submission_ref']))
print(json.dumps(dict(run_id=run,evidence=str(out),case_ref=case,submission=profile['submission_ref'])),flush=True)
deadline=time.monotonic()+180
while True:
 observation=call('execution.get',query)
 if observation['posture'] not in ('admitted','running','unresolved') or time.monotonic()>deadline: break
 time.sleep(2)
context=call('execution.get',dict(query,include_context=True))
(out/'context.json').write_text(json.dumps(context,indent=2))
print(json.dumps(dict(posture=observation['posture'],primary_result=observation.get('primary_result'),context_path=str(out/'context.json'))),flush=True)
f.close()
