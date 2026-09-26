import base64,hashlib,json,os,socket,subprocess,sys,time,urllib.request
from pathlib import Path
root=Path('/home/mothx/computer-science/projects/YAI/yai');sys.path.insert(0,str(root/'tools/validation'));from behavioral_corpus import Host
home_value=os.environ.get('YAI_HOME')
if not home_value or not Path(home_value).is_absolute() or Path(home_value).resolve()==(Path.home()/'.yai').resolve():
 raise SystemExit('Historical separate-Case model qualification is disabled in the operator profile; qualify DeepSeek in case:tech-infra-inference-service or use a fresh isolated YAI_HOME')
home=Path(home_value);host=Host(home);case='case:qualification-deepseek-native-4k';target='provider-target:965ce8d788b4d6321ed0fea629f99546'
binary=root/'studio/src-tauri/target/release/yai-studio';run='native-provider-probe-'+str(time.time_ns());folder=Path('/home/mothx/.cache/tmp')/run;folder.mkdir();raw=(folder/'observations.jsonl').open('x');order=0
session=None;driver=None
with socket.socket() as listener:listener.bind(('127.0.0.1',0));port=listener.getsockname()[1]
base=f'http://127.0.0.1:{port}'
def emit(**data):
 global order;order+=1;raw.write(json.dumps(dict(run_id=run,order=order,time_unix_ms=int(time.time()*1000),**data))+'\n');raw.flush()
def call(op,inputs):
 r=host.call(op,inputs,f'{run}:{order}');emit(operation=op,input=inputs,result=r);assert r['result_state']=='success',r;return r['data']
def request(method,path,data=None):
 req=urllib.request.Request(base+path,data=json.dumps(data).encode() if data is not None else None,method=method,headers={'Content-Type':'application/json'})
 with urllib.request.urlopen(req,timeout=30) as r:return json.load(r)['value']
def js(code,*args):return request('POST',f'/session/{session}/execute/sync',{'script':code,'args':list(args)})
def wait(code,*args):
 end=time.monotonic()+30
 while time.monotonic()<end:
  value=js(code,*args)
  if value:return value
  time.sleep(.1)
 raise TimeoutError(code)
def shot(name):(folder/name).write_bytes(base64.b64decode(request('GET',f'/session/{session}/screenshot')))
try:
 emit(command=sys.argv,cwd=str(root),yai_home=str(home),sha=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),binary=str(binary),binary_sha256=hashlib.file_digest(binary.open('rb'),'sha256').hexdigest(),provider_mode='external_yvex',prestate='Existing bounded 4K Case; one synthetic qualification through native UI; no YVEX administration')
 before=call('case.summary',{'case_ref':case});item=next(t for t in before['compute']['targets'] if t['id']==target);emit(endpoint=item['endpoint'],exact_model=item['model_id'])
 catalog=call('provider.models',{'tenant_id':before['case']['tenant_ref'],'target_ref':target});assert item['model_id'] in catalog['models']
 env={**os.environ,'YAI_HOME':str(home),'TAURI_WEBVIEW_AUTOMATION':'true','WEBKIT_DISABLE_DMABUF_RENDERER':'1','GDK_BACKEND':'x11'}
 log=(folder/'driver.log').open('w');driver=subprocess.Popen(['WebKitWebDriver',f'--port={port}','--host=127.0.0.1'],env=env,stdout=log,stderr=log)
 for _ in range(100):
  try:request('GET','/status');break
  except OSError:time.sleep(.05)
 session=request('POST','/session',{'capabilities':{'alwaysMatch':{'webkitgtk:browserOptions':{'binary':str(binary),'args':[]}}}})['sessionId']
 request('POST',f'/session/{session}/window/rect',{'x':0,'y':0,'width':1440,'height':900})
 wait('return document.querySelector(".live-case-row") || document.querySelector(".workbench-kernel")')
 if js('return Boolean(document.querySelector(".workbench-kernel"))'):
  js('document.querySelector(".titlebar-case").click()');wait('return document.querySelector(".case-switcher")');js('const n=[...document.querySelectorAll(".case-switcher .ui-list-row")].find(x=>x.dataset.caseRef===arguments[0]);if(!n)throw Error("Case missing");n.click()',case)
 else:js('const n=[...document.querySelectorAll(".live-case-row")].find(x=>x.dataset.caseRef===arguments[0]);if(!n)throw Error("Case missing");n.click()',case)
 wait('return document.querySelector(".workbench-kernel")?.dataset.caseRef===arguments[0]',case)
 js('document.querySelector(`.live-rail button[aria-label="Providers"]`).click()')
 wait('return document.querySelector(`.deployment-compact-picker option[value="${arguments[0]}"]`)',target)
 js('const s=document.querySelector(".deployment-compact-picker select");s.value=arguments[0];s.dispatchEvent(new Event("change",{bubbles:true}))',target)
 js('[...document.querySelectorAll(".deployment-sections button")].find(n=>n.textContent==="Evidence").click()')
 wait('return document.querySelector(".provider-qualification select")')
 js('const s=document.querySelector(".provider-qualification select");s.value="text";s.dispatchEvent(new Event("change",{bubbles:true}))')
 button=wait('return [...document.querySelectorAll(".provider-qualification button")].find(n=>["Check & qualify","Run a new check"].includes(n.textContent) && !n.disabled)')
 old=js('return document.querySelector(".provider-qualification code")?.textContent')
 recovery_command=[str(root/'target/debug/yai'),'provider','probes',target,'--json']
 recovery=subprocess.run(recovery_command,env={**os.environ,'YAI_HOME':str(home)},capture_output=True,text=True,timeout=30)
 emit(recovery_command=recovery_command,exit=recovery.returncode,stdout=recovery.stdout,stderr=recovery.stderr)
 assert recovery.returncode==0
 assert json.loads(recovery.stdout)['data']['value']['runs']==[], 'Existing admissions must be observed; never dispatch here'
 assert old is None, 'Existing local receipt must be observed'
 emit(action='native_ui_click_once_after_proven_empty_admission_inventory',old_reference=old)
 js('const b=[...document.querySelectorAll(".provider-qualification button")].find(n=>n.textContent==="Check & qualify"&&!n.disabled);if(!b)throw Error("No enabled check button");b.click()')
 submission=wait('const r=document.querySelector(".provider-qualification code")?.textContent;return r && r!==arguments[0] ? r:null',old)
 emit(submission_ref=submission);deadline=time.monotonic()+600
 while time.monotonic()<deadline:
  r=call('provider.probe.get',{'target_ref':target,'submission_ref':submission})
  if r['posture']!='running':break
  time.sleep(5)
 else:raise TimeoutError('Observation deadline; never resend')
 assert r['posture']=='completed',r
 assert r['run']['evidence']['exact_model_addressed'] and r['run']['evidence']['chat_text_envelope_valid'],r
 assert 'text_to_text' in r['run']['evidence'].get('realization_shapes',[]),r
 wait('return document.querySelector(".provider-qualification").textContent.includes("Evidence recorded")');shot('completed.png')
 after=call('case.summary',{'case_ref':case});assert after['case']==before['case'];assert after['conversation']==before['conversation']
 emit(result='PASS',finding='NO_ISSUE',proof='One native Studio synthetic check through governed Host; exact real model and text shape; Case and Conversation unchanged',submission_ref=submission)
 print(json.dumps({'result':'PASS','evidence':str(folder),'submission_ref':submission}))
except BaseException as e:
 emit(result='NONPASS',error=str(e),redispatch=False)
 if session:
  try:emit(ui_failure=js('return {text:document.querySelector(".provider-qualification")?.innerText, receipts:Object.fromEntries(Object.entries(localStorage).filter(([key])=>key.startsWith("yai.studio.provider-probe.")))}'))
  except Exception:pass
 if session:
  try:shot('nonpass.png')
  except Exception:pass
 print(json.dumps({'result':'NONPASS','evidence':str(folder),'error':str(e)}));raise
finally:
 if session:
  try:request('DELETE',f'/session/{session}')
  except Exception:pass
 if driver:driver.terminate();driver.wait(timeout=10)
 raw.close()
