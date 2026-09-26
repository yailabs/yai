import json,os,sys,time,subprocess
from pathlib import Path
sys.path.insert(0,'tools/validation')
from behavioral_corpus import Host
home_value=os.environ.get('YAI_HOME')
if not home_value or not Path(home_value).is_absolute() or Path(home_value).resolve()==(Path.home()/'.yai').resolve():
 raise SystemExit('Historical separate-Case setup is disabled in the operator profile; qualify DeepSeek in case:tech-infra-inference-service or use a fresh isolated YAI_HOME')
root=Path.cwd(); home=Path(home_value); h=Host(home)
case='case:qualification-deepseek-material-4k'; run='documentary-4k-'+str(time.time_ns())
out=Path('/tmp')/run;out.mkdir(); stream=(out/'setup.jsonl').open('x');order=0

def emit(record):
 global order
 order+=1;stream.write(json.dumps(dict(run_id=run,order=order,**record))+'\n');stream.flush()
def call(op,i):
 r=h.call(op,i,run+':'+str(order));emit(dict(operation=op,input=i,result=r));assert r['result_state']=='success',r;return r['data']
def cli(*parts):
 argv=[str(root/'target/debug/yai'),*parts,'--json'];p=subprocess.run(argv,env={**os.environ,'YAI_HOME':str(home)},capture_output=True,text=True,timeout=120)
 emit(dict(command=argv,cwd=str(root),yai_home=str(home),exit=p.returncode,stdout=p.stdout,stderr=p.stderr));assert p.returncode==0,p.stderr
 return json.loads(p.stdout)
rows=call('case.list',{});assert case not in json.dumps(rows),'Existing Case: inspect, never repeat setup'
for line in (root/'labs/external-runtime/runs/20260924-deepseek-4k/exchanges.jsonl').read_text().splitlines():
 r=json.loads(line);op=r.get('operation')
 if op in ['case.create','participant.role.add','participant.principal.link','provider.case.bind','participant.view.admit','cognitive.binding.set']:
  i=r['input'];i['case_ref']=case;call(op,i)
call('participant.role.add',dict(case_ref=case,participant_ref='participant:operator',role='operation-proposer'))
policy='tests/qualification/studio-product-vertical/policy.json';doc='application/yai-application/Cargo.toml'
resource=dict(schema='yai.resource_definition.v1',attachment_id='resource:documentary-4k',policy_owner='participant:operator',participant_ids=['participant:operator'],operations=['discover','admit_content','content_read'],read_prefixes=[policy,doc],names=[],max_output_bytes=16384,max_items=2,address=dict(kind='discovery',root=str(root)))
sources=[dict(name='documentary-policy',resource=resource['attachment_id'],roles=['policy'],action=dict(action='discover',path=policy),media_type='application/json',bootstrap_policy=True),dict(name='application-manifest',resource=resource['attachment_id'],roles=['knowledge'],action=dict(action='discover',path=doc),media_type='text/plain',bootstrap_policy=False)]
perimeter=out/'perimeter.json';perimeter.write_text(json.dumps(dict(schema='yai.source_perimeter.v1',name='documentary-4k',participant='participant:operator',resources=[resource],sources=sources)))
cli('case','sources','declare',case,'--file',str(perimeter))
cli('case','sources','acquire',case,'--source','documentary-policy')
cli('case','sources','publish',case,'--source','documentary-policy','--reason','Bounded documentary qualification only; no infrastructure effects')
cli('case','sources','resume',case,'--source','application-manifest')
cli('case','knowledge','build',case)
cli('case','verify',case)
cli('case','sources','inventory',case)
s=call('case.summary',dict(case_ref=case));emit(dict(result='prepared',generation=s['case']['generation'],inference_submitted=False))
print(json.dumps(dict(case_ref=case,evidence=str(out),generation=s['case']['generation'])))
