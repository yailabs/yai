#!/usr/bin/env python3
"""Read-only native portfolio qualification; only unsent window-local drafts change."""
import argparse,base64,hashlib,json,os,socket,subprocess,sys,time,urllib.request
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,default=Path(__file__).resolve().parents[2]);p.add_argument('--binary',type=Path,required=True);p.add_argument('--yai',type=Path,required=True);p.add_argument('--evidence',type=Path,required=True);p.add_argument('--cases',nargs=2,required=True);a=p.parse_args()
assert os.environ.get('YAI_HOME');home=Path(os.environ['YAI_HOME']).resolve();a.evidence.mkdir(parents=True,exist_ok=True)
assert a.cases[0] != a.cases[1], 'Two different Case identities are required'
sys.path.insert(0,str(a.repo/'tools/validation'));from behavioral_corpus import Host
host=Host(home);run=f'native-portfolio-{time.time_ns()}';order=0
raw=(a.evidence/'observations.jsonl').open('x')
def emit(value):
 global order;order+=1;raw.write(json.dumps(dict(run_id=run,order=order,**value))+'\n');raw.flush()
def digest(path):
 with path.open('rb') as stream:return hashlib.file_digest(stream,'sha256').hexdigest()
def status():
 cmd=[str(a.yai),'host','status','--json'];r=subprocess.run(cmd,capture_output=True,text=True,check=True,timeout=15);emit(dict(command=cmd,exit=r.returncode,stdout=r.stdout,stderr=r.stderr));return json.loads(r.stdout)['data']['value']
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
  element=self.request('POST',f'/session/{self.session}/element',{'using':'css selector','value':'textarea[aria-label="Message to the Case"]'})['element-6066-11e4-a52e-4f735466cecf']
  self.request('POST',f'/session/{self.session}/element/{element}/value',{'text':text})
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
           evidence_class='native_local_product',provider='not_invoked',material_pre_state='existing operator Cases; only unsent window-local drafts'))
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
 emit(dict(result='PASS',cases=a.cases,host_pid=baseline['pid'],host_instance=baseline['instance_id'],native_clients=new_studios,proof=['Two independently launched native Studios share Host','Case switching restores isolated local drafts','Same Case in two clients does not share drafts','Closing one Studio leaves Host and other Studio functional','Case identity/version and canonical Conversation unchanged']))
 print(json.dumps(dict(result='PASS',run_id=run,evidence=str(a.evidence))))
finally:
 errors=[]
 for client in clients:
  try:client.close()
  except Exception as error:errors.append(str(error))
 raw.close()
 if errors:raise RuntimeError('Native client cleanup failed: '+ '; '.join(errors))
