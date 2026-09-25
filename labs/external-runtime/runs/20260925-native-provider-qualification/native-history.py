import base64,hashlib,json,os,socket,subprocess,sys,time,urllib.request
from pathlib import Path
root=Path('/home/mothx/computer-science/projects/YAI/yai');sys.path.insert(0,str(root/'tools/validation'));from behavioral_corpus import Host
home=Path('/home/mothx/.yai');host=Host(home);case='case:qualification-deepseek-native-4k';target='provider-target:965ce8d788b4d6321ed0fea629f99546'
binary=root/'studio/src-tauri/target/release/yai-studio';run='native-provider-history-read-'+str(time.time_ns());folder=Path('/home/mothx/.cache/tmp')/run;folder.mkdir();raw=(folder/'observations.jsonl').open('x');order=0
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
def shot(name):
 handle=request('GET',f'/session/{session}/window');request('POST',f'/session/{session}/window',{'handle':handle})
 (folder/name).write_bytes(base64.b64decode(request('GET',f'/session/{session}/screenshot')))
try:
 emit(command=sys.argv,cwd=str(root),yai_home=str(home),sha=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),binary=str(binary),binary_sha256=hashlib.file_digest(binary.open('rb'),'sha256').hexdigest(),provider_mode='external_yvex',prestate='Existing bounded 4K Case; read-only retained qualification in native UI; no dispatch or YVEX administration')
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
 submission='studio-probe:63a3e514-45b1-4bad-8b4b-4fb8e71aab0e'
 r=call('provider.probe.get',{'target_ref':target,'submission_ref':submission});assert r['posture']=='completed'
 wait('return document.querySelector(".provider-check-history")')
 js('document.querySelector(".provider-check-history").open=true')
 wait('return document.querySelector(".provider-check-history").textContent.includes(arguments[0])',submission)
 js('const h=document.querySelector(".provider-check-history");h.querySelector("details").open=true;h.scrollIntoView({block:"center"})')
 emit(observation='native_retained_history',ui=js('return document.querySelector(".provider-check-history").innerText'))
 shot('retained-check.png')
 after=call('case.summary',{'case_ref':case});assert after['case']==before['case'];assert after['conversation']==before['conversation']
 emit(result='PASS',finding='NO_ISSUE',proof='Native Studio observes existing real-model check through current authority; no dispatch; Case and Conversation unchanged',submission_ref=submission)
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
