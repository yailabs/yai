#!/usr/bin/env python3
"""Native portfolio qualification: read-only existing Cases or fresh-profile event/restart proof."""
import argparse,base64,hashlib,json,os,shutil,socket,subprocess,sys,tempfile,time,urllib.request
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,default=Path(__file__).resolve().parents[2]);p.add_argument('--binary',type=Path,required=True);p.add_argument('--yai',type=Path,required=True);p.add_argument('--evidence',type=Path,required=True);p.add_argument('--cases',nargs=2,required=True);p.add_argument('--fresh-profile',action='store_true',help='Create disposable manifest Cases and qualify CLI update fanout/restart; never mutate the supplied YAI_HOME');a=p.parse_args()
owned_root=None;host_started=False
if a.fresh_profile:
 manifest=json.loads((a.repo/'tests/qualification/behavioral-corpus/portfolio.json').read_text())
 assert a.cases==[case['case_ref'] for case in manifest['cases']], 'Fresh lane uses exact versioned portfolio Cases'
 owned_root=Path(tempfile.mkdtemp(prefix='yai-native-portfolio-'));home=owned_root/'home';os.environ['YAI_HOME']=str(home)
else:
 assert os.environ.get('YAI_HOME');home=Path(os.environ['YAI_HOME']).resolve()
a.evidence.mkdir(parents=True,exist_ok=True)
assert a.cases[0] != a.cases[1], 'Two different Case identities are required'
sys.path.insert(0,str(a.repo/'tools/validation'));from behavioral_corpus import Host
host=None;run=f'native-portfolio-{time.time_ns()}';order=0
raw=(a.evidence/'observations.jsonl').open('x')
def emit(value):
 global order;order+=1;raw.write(json.dumps(dict(run_id=run,order=order,**value))+'\n');raw.flush()
def digest(path):
 with path.open('rb') as stream:return hashlib.file_digest(stream,'sha256').hexdigest()
def cli(*parts):
 cmd=[str(a.yai.resolve()),*parts,'--json'];r=subprocess.run(cmd,capture_output=True,text=True,timeout=45);emit(dict(command=cmd,cwd=str(Path.cwd()),yai_home=str(home),exit=r.returncode,stdout=r.stdout,stderr=r.stderr));assert r.returncode==0, 'CLI refused; inspect raw evidence';return json.loads(r.stdout)
def status():return cli('host','status')['data']['value']
def call(operation,inputs):
 result=host.call(operation,inputs,f'{run}:{order}');emit(dict(operation=operation,input=inputs,result=result));assert result['result_state']=='success';return result['data']
def summary(ref):
 r=host.call('case.summary',{'case_ref':ref},f'{run}:{order}');emit(dict(operation='case.summary',case_ref=ref,result=r));assert r['result_state']=='success';return r['data']
class Client:
 def __init__(self,index):
  self.index=index;self.session=None
  with socket.socket() as listener:listener.bind(('127.0.0.1',0));port=listener.getsockname()[1]
  self.base=f'http://127.0.0.1:{port}';self.log=(a.evidence/f'driver-{index}.log').open('w')
  env={**os.environ,'TAURI_WEBVIEW_AUTOMATION':'true','WEBKIT_DISABLE_DMABUF_RENDERER':'1','GDK_BACKEND':'x11'}
  self.driver=subprocess.Popen(['WebKitWebDriver',f'--port={port}','--host=127.0.0.1'],env=env,stdout=self.log,stderr=self.log)
  try:
   for _ in range(100):
    try:self.request('GET','/status');break
    except Exception:time.sleep(.05)
   else:raise TimeoutError('Native WebDriver did not become ready')
   result=self.request('POST','/session',{'capabilities':{'alwaysMatch':{'webkitgtk:browserOptions':{'binary':str(a.binary.resolve()),'args':[]}}}});self.session=result['sessionId'];emit(dict(client=index,driver=result['capabilities']))
   self.wait('return document.querySelector(".live-case-row") || document.querySelector(".workbench-kernel")')
  except BaseException:
   self.close()
   raise
 def request(self,method,url,data=None):
  req=urllib.request.Request(self.base+url,data=json.dumps(data).encode() if data is not None else None,method=method,headers={'Content-Type':'application/json'})
  with urllib.request.urlopen(req,timeout=30) as r:return json.load(r)['value']
 def js(self,code,*args):return self.request('POST',f'/session/{self.session}/execute/sync',{'script':code,'args':list(args)})
 def wait(self,code,*args):
  deadline=time.monotonic()+20
  while time.monotonic()<deadline:
   value=self.js(code,*args)
   if value:return value
   time.sleep(.1)
  raise AssertionError(code)
 def open(self,ref,label):
  if self.js('return Boolean(document.querySelector(".workbench-kernel"))'):
   self.js('document.querySelector(".titlebar-case").click()')
   self.wait('return document.querySelector(".case-switcher")')
   self.js('const row=[...document.querySelectorAll(".case-switcher .ui-list-row")].find(x=>x.dataset.caseRef===arguments[0]);if(!row)throw Error("Case absent");row.click()',ref)
  else:self.js('const row=[...document.querySelectorAll(".live-case-row")].find(x=>x.dataset.caseRef===arguments[0]);if(!row)throw Error("Case absent");row.click()',ref)
  self.wait('return document.querySelector(".workbench-kernel")?.dataset.caseRef===arguments[0]',ref)
  self.js('document.querySelector(`.live-rail button[aria-label="Overview"]`).click()')
  self.wait('return document.querySelector(".case-overview h1")?.textContent===arguments[0]',label)
  self.js('document.querySelector(`.live-context .segmented button[title="Conversation"]`).click()')
  self.wait('return document.querySelector(`textarea[aria-label="Message to the Case"]`)')
 def draft(self):return self.js('return document.querySelector(`textarea[aria-label="Message to the Case"]`).value')
 def type(self,text):
  handle=self.request('GET',f'/session/{self.session}/window')
  self.request('POST',f'/session/{self.session}/window',{'handle':handle})
  element=self.request('POST',f'/session/{self.session}/element',{'using':'css selector','value':'textarea[aria-label="Message to the Case"]'})['element-6066-11e4-a52e-4f735466cecf']
  self.request('POST',f'/session/{self.session}/element/{element}/click',{})
  self.request('POST',f'/session/{self.session}/element/{element}/value',{'text':text})
  actual=self.draft();emit(dict(client=self.index,observation='typed_unsent_draft',expected=text,actual=actual))
  assert actual==text, 'Native typing did not establish the expected draft before navigation'
 def shot(self,name):
  (a.evidence/name).write_bytes(base64.b64decode(self.request('GET',f'/session/{self.session}/screenshot')))
 def close(self):
  try:
   if self.session:
    try:self.request('DELETE',f'/session/{self.session}')
    finally:self.session=None
  finally:
   if self.driver.poll() is None:
    self.driver.terminate()
    try:self.driver.wait(timeout=10)
    except subprocess.TimeoutExpired:self.driver.kill();self.driver.wait(timeout=10)
   self.log.close()
clients=[]
try:
 emit(dict(command=[sys.executable,*sys.argv],cwd=str(Path.cwd()),yai_home=str(home),
           source_sha=subprocess.check_output(['git','rev-parse','HEAD'],cwd=a.repo,text=True).strip(),
           desktop_binary=str(a.binary.resolve()),desktop_sha256=digest(a.binary),
           evidence_class='native_local_product',provider='not_invoked',material_pre_state='fresh manifest-derived disposable profile' if a.fresh_profile else 'existing operator Cases; only unsent window-local drafts'))
 if a.fresh_profile:
  host_started=True;cli('host','start');host=Host(home)
  emit(dict(manifest=manifest))
  call('identity.bootstrap',dict(tenant_id=manifest['tenant'],organization_ref=manifest['organization']))
  for ref in a.cases:
   call('case.create',dict(case_ref=ref,tenant_id=manifest['tenant']))
   call('participant.role.add',dict(case_ref=ref,participant_ref=manifest['participant'],role='operator'))
   call('participant.principal.link',dict(case_ref=ref,participant_ref=manifest['participant'],principal_ref='self'))
 else:host=Host(home)
 baseline=status();assert baseline['state']=='running'
 before={ref:summary(ref) for ref in a.cases};labels={ref:value['case']['display_name'] for ref,value in before.items()}
 first=Client(0);clients.append(first);first.open(a.cases[0],labels[a.cases[0]]);assert first.draft()=='';first.type('UNSENT_NATIVE_ALPHA')
 second=Client(1);clients.append(second);second.open(a.cases[1],labels[a.cases[1]]);assert second.draft()=='';second.type('UNSENT_NATIVE_BETA')
 deadline=time.monotonic()+10
 while True:
  attached=status()
  if attached['connected_clients']>=baseline['connected_clients']+2:break
  assert time.monotonic()<deadline;time.sleep(.1)
 assert attached['pid']==baseline['pid'] and attached['instance_id']==baseline['instance_id']
 old_clients={client['client_id'] for client in baseline['clients']}
 new_studios=[client for client in attached['clients'] if client['client_kind']=='studio' and client['client_id'] not in old_clients]
 assert len(new_studios)==2, 'Exactly two new native Studio attachments required'
 assert len({client['pid'] for client in new_studios})==2, 'Native Studios must have different process identities'
 studio_ids={client['client_id'] for client in new_studios}
 first.open(a.cases[1],labels[a.cases[1]]);assert first.draft()=='';assert second.draft()=='UNSENT_NATIVE_BETA'
 first.open(a.cases[0],labels[a.cases[0]]);assert first.draft()=='UNSENT_NATIVE_ALPHA'
 first.shot('native-case-alpha.png');second.shot('native-case-beta.png')
 if a.fresh_profile:
  second.open(a.cases[0],labels[a.cases[0]]);assert second.draft()=='';second.type('UNSENT_SECOND_ALPHA')
  for client in clients:
   client.js('const tab=[...document.querySelectorAll(".live-bottom button")].find(x=>x.textContent==="Journal");if(!tab)throw Error("Journal absent");tab.click()')
   client.wait('return document.querySelector(".journal-events")')
  role='native-event-observer'
  cli('case','participant','role','add','--case',a.cases[0],'--participant',manifest['participant'],'--role',role)
  advanced=summary(a.cases[0]);assert advanced['case']['generation']==before[a.cases[0]]['case']['generation']+1
  event=advanced['memory']['timeline'][-1]
  for client in clients:
   client.wait('return [...document.querySelectorAll(".journal-event")].some(row=>row.title===arguments[0] && row.querySelector(".journal-sequence")?.textContent===String(arguments[1]))',event['id'],event['sequence'])
   client.shot(f'native-event-client-{client.index}.png')
  assert first.draft()=='UNSENT_NATIVE_ALPHA' and second.draft()=='UNSENT_SECOND_ALPHA'
  cli('case','participant','role','add','--case',a.cases[0],'--participant',manifest['participant'],'--role',role)
  assert summary(a.cases[0])==advanced, 'Exact repeated CLI request duplicated Case history'
  cli('host','restart');host=Host(home);restarted=status()
  assert restarted['instance_id']!=baseline['instance_id'] and restarted['pid']!=baseline['pid']
  for client in clients:
   client.wait('return document.querySelector(".kernel-status")?.innerText.includes("YAI ●")')
  # A second post-restart mutation proves fresh subscriptions, not cached UI.
  cli('case','participant','role','add','--case',a.cases[0],'--participant',manifest['participant'],'--role','native-reconnected-observer')
  recovered=summary(a.cases[0]);assert recovered['case']['generation']==advanced['case']['generation']+1
  event=recovered['memory']['timeline'][-1]
  for client in clients:
   client.wait('return [...document.querySelectorAll(".journal-event")].some(row=>row.title===arguments[0])',event['id'])
  assert first.draft()=='UNSENT_NATIVE_ALPHA' and second.draft()=='UNSENT_SECOND_ALPHA'
  assert summary(a.cases[1])==before[a.cases[1]], 'Independent Case changed'
  emit(dict(result='PASS',proof='Canonical CLI mutation reaches both native Journals before and after Host restart; exact repeat is idempotent; drafts and independent Case preserved',before=before[a.cases[0]]['case']['generation'],after=recovered['case']['generation'],old_host=baseline['instance_id'],new_host=restarted['instance_id']))
  before[a.cases[0]]=recovered
  second.open(a.cases[1],labels[a.cases[1]]);assert second.draft()=='UNSENT_NATIVE_BETA'
  baseline=restarted;attached=status()
  reconnected_studios=[client for client in attached['clients'] if client['client_kind']=='studio']
  assert len(reconnected_studios)==2
  assert {client['pid'] for client in reconnected_studios}=={client['pid'] for client in new_studios}
  assert not studio_ids.intersection(client['client_id'] for client in reconnected_studios), 'Restart reused ephemeral client attachment'
  new_studios=reconnected_studios;studio_ids={client['client_id'] for client in new_studios}
 first.close();clients.remove(first)
 second.js('document.querySelector(`button[aria-label="Refresh Case"]`).click()');assert second.draft()=='UNSENT_NATIVE_BETA'
 deadline=time.monotonic()+10
 while True:
  remaining=status()
  if remaining['connected_clients']<attached['connected_clients']:break
  assert time.monotonic()<deadline;time.sleep(.1)
 assert remaining['pid']==baseline['pid'] and remaining['instance_id']==baseline['instance_id']
 assert len(studio_ids.intersection(client['client_id'] for client in remaining['clients']))==1
 for ref in a.cases:
  after=summary(ref);assert after['case']==before[ref]['case'];assert after['conversation']==before[ref]['conversation']
 emit(dict(result='PASS',cases=a.cases,host_pid=baseline['pid'],host_instance=baseline['instance_id'],native_clients=new_studios,proof=['Two independently launched native Studios share Host','Case switching restores isolated local drafts','Same Case in two clients does not share drafts','Closing one Studio leaves Host and other Studio functional','Expected canonical Case state and Conversation preserved']))
 print(json.dumps(dict(result='PASS',run_id=run,evidence=str(a.evidence))))
except BaseException:
 for client in clients:
  try:
   emit(dict(client=client.index,failure_ui=client.js('return {case_ref:document.querySelector(".workbench-kernel")?.dataset.caseRef,draft:document.querySelector(`textarea[aria-label="Message to the Case"]`)?.value,conversation_keys:Object.keys(sessionStorage).filter(key=>key.includes("conversation"))}')))
   client.shot(f'failure-client-{client.index}.png')
  except Exception as error:emit(dict(client=client.index,failure_capture=str(error)))
 raise
finally:
 errors=[]
 for client in clients:
  try:client.close()
  except Exception as error:errors.append(str(error))
 if owned_root:
  try:
   if host_started:cli('host','stop')
   shutil.rmtree(owned_root)
   emit(dict(cleanup='fresh profile removed after Host stop'))
  except Exception as error:errors.append(f'Owned profile retained at {owned_root}: {error}')
 raw.close()
 if errors:raise RuntimeError('Native client cleanup failed: '+ '; '.join(errors))
