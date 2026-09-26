import json,os,sys,time,subprocess
from pathlib import Path
sys.path.insert(0,'tools/validation');from behavioral_corpus import Host
home_value=os.environ.get('YAI_HOME')
if not home_value or not Path(home_value).is_absolute() or Path(home_value).resolve()==(Path.home()/'.yai').resolve():
 raise SystemExit('Historical separate-Case setup is disabled in the operator profile; qualify DeepSeek in case:tech-infra-inference-service or use a fresh isolated YAI_HOME')
home=Path(home_value);h=Host(home);case='case:qualification-deepseek-native-4k';run='native-4k-setup-'+str(time.time_ns());p=Path('/home/mothx/.cache/tmp')/(run+'.jsonl');f=p.open('x');order=0
def call(op,i):
 global order
 r=h.call(op,i,run+':'+str(order));order+=1;f.write(json.dumps(dict(run_id=run,order=order,observed_at_unix_ms=int(time.time()*1000),operation=op,input=i,result=r))+'\n');f.flush();assert r['result_state']=='success',r;return r['data']
rows=call('case.list',{})
assert case not in json.dumps(rows),'Existing Case: do not mutate/reconstruct or resend'
for line in Path('labs/external-runtime/runs/20260924-deepseek-4k/exchanges.jsonl').read_text().splitlines():
 row=json.loads(line);op=row.get('operation')
 if op in ['case.create','participant.role.add','participant.principal.link','provider.case.bind','participant.view.admit','cognitive.binding.set']:
  i=row['input'];i['case_ref']=case;call(op,i)
x=call('case.summary',{'case_ref':case});assert not x['conversation']['turns']
print(json.dumps(dict(case=case,generation=x['case']['generation'],evidence=str(p))))
