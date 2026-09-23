// Opt-in operator evidence. Reads the selected persistent Case through the real
// Host; optional enrichment invokes only the reviewed CLI qualification procedure.
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {execFileSync, spawn} from 'node:child_process';
import {mkdir, readFile, writeFile} from 'node:fs/promises';
import net from 'node:net';
import path from 'node:path';
const require=createRequire(path.resolve(import.meta.dirname,'../../studio/package.json'));
const {chromium}=require('playwright-core');
const binary=process.env.YAI_STUDIO_TEST_BINARY, home=process.env.YAI_STUDIO_TEST_HOME;
assert.ok(binary&&home,'Explicit operator profile and published CLI required');
const env={...process.env,YAI_HOME:home};
const caseRef=process.env.YAI_STUDIO_TEST_CASE??'case:studio-live-qualification';
const evidence=process.env.STUDIO_EVIDENCE_DIR??'/tmp/yai-studio-operational-live';await mkdir(evidence,{recursive:true});
const telemetry=JSON.parse(execFileSync(binary,['host','status','--json'],{env,encoding:'utf8'})).data.value;
assert.equal(telemetry.state,'running');
let serial=0;const exchanges=[], events=[], sockets=[];
function connect(onFrame, suffix){
 const socket=net.createConnection(telemetry.endpoint);let buffer='';sockets.push(socket);
 socket.on('connect',()=>socket.write(JSON.stringify({kind:'handshake',protocol:telemetry.protocol,client_id:`studio-operational:${process.pid}:${++serial}:${suffix}`,client_kind:'qualification',pid:process.pid,yai_home_identity:telemetry.yai_home_identity})+'\n'));
 socket.on('data',chunk=>{buffer+=chunk;let i;while((i=buffer.indexOf('\n'))>=0){const frame=JSON.parse(buffer.slice(0,i));buffer=buffer.slice(i+1);onFrame(frame,socket);}});return socket;
}
function rpc(request){return new Promise((resolve,reject)=>{const socket=connect((frame,socket)=>{
 if(frame.kind==='handshake')socket.write(JSON.stringify({kind:'application_request',request})+'\n');
 if(frame.kind==='application_response'){exchanges.push({order:exchanges.length+1,request,result:frame.result});socket.end();resolve(frame.result);}
 if(frame.kind==='error'){socket.destroy();reject(new Error(JSON.stringify(frame)));}
},'request');socket.on('error',reject);socket.setTimeout(20000,()=>socket.destroy(new Error('Host timeout')));});}
const call=(operation_ref,input)=>rpc({protocol:'yai.studio.application.v1',operation_ref,correlation_ref:`operator-proof:${++serial}`,input});
const before=await call('case.summary',{case_ref:caseRef});assert.equal(before.result_state,'success');
const browser=await chromium.launch({executablePath:'/usr/bin/chromium',headless:true,args:['--no-sandbox','--disable-gpu']});
const errors=[];let command;
try{
 const pages=[];
 for(let client=0;client<2;client++){
  const context=await browser.newContext({viewport:{width:1440,height:900}});const page=await context.newPage();pages.push(page);page.setDefaultTimeout(20000);page.on('pageerror',e=>errors.push(String(e)));
  await page.exposeFunction('qualificationHostCall',rpc);await page.goto(`${process.env.STUDIO_TEST_URL??'http://127.0.0.1:1422'}/?gallery=1&case=${encodeURIComponent(caseRef)}`);
  await page.evaluate(async telemetry=>{
   document.getElementById('root').style.display='none';
   const [{default:React},{default:ReactDOM},{StudioApplication},{WorkbenchRegistry},{PlatformServices},{registerContributions},{builtInContributions},{LiveClient},{LiveDataSource}]=await Promise.all([
    import('/node_modules/.vite/deps/react.js'),import('/node_modules/.vite/deps/react-dom_client.js'),import('/src/app/StudioApplication.tsx'),import('/src/workbench/kernel/registry.ts'),import('/src/platform/services.ts'),import('/src/workbench/kernel/contributions.ts'),import('/src/contrib/builtins.ts'),import('/src/clients/live.ts'),import('/src/clients/dataSource.ts')]);
   const listeners=new Set();window.forwardHostEvent=payload=>listeners.forEach(handler=>handler({payload}));
   window.__TAURI__={core:{invoke:(command,args)=>command==='studio_call'?window.qualificationHostCall(args.request):Promise.resolve(undefined)},event:{listen:async(_name,handler)=>{listeners.add(handler);return()=>listeners.delete(handler);}}};
   const state={state:'live',telemetry,resync_required:false};const host={capabilities:{kind:'web',nativeDesktop:false,terminalAvailable:false,windowControlsAvailable:false},snapshot:()=>state,subscribe:()=>({dispose(){}}),dispose(){},closeWindow(){},status:async()=>telemetry};
   const client=new LiveClient(), platform=new PlatformServices(host,client), registry=new WorkbenchRegistry();registerContributions(builtInContributions,{platform,workbench:registry});window.qualificationPlatform=platform;
   const container=document.createElement('div');document.body.appendChild(container);ReactDOM.createRoot(container).render(React.createElement(StudioApplication,{dataSource:new LiveDataSource(client),platform,registry}));
  },telemetry);
  await page.locator('.workbench-kernel').waitFor();
  await new Promise((resolve,reject)=>{const socket=connect((frame,socket)=>{
   if(frame.kind==='handshake')socket.write(JSON.stringify({kind:'subscribe'})+'\n');
   if(frame.kind==='subscribed')resolve();
   if(frame.kind==='event'){events.push({client,...frame.event});void page.evaluate(event=>window.forwardHostEvent(event),frame.event).catch(()=>{});}
  },`events-${client}`);socket.on('error',reject);});
  await page.getByRole('button',{name:'Journal',exact:true}).click();
 }
 await pages[0].getByRole('button',{name:'Pause follow',exact:true}).click();
 const paused=await pages[0].locator('.journal-events').innerText();
 if(process.env.STUDIO_ENTERPRISE_WORKFLOW==='1'){
  const recipe=JSON.parse(await readFile(new URL('../qualification/behavioral-corpus/enterprise-workflow.json',import.meta.url),'utf8'));
  assert.equal(caseRef,recipe.case_ref,'Enterprise recipe cannot mutate another operator Case');
  assert.equal(recipe.schema,'yai.studio_checkpoint_recipe.v1');
  // Existing work is operator-owned. Replaying this procedure never replaces
  // a binding, supplies a fictional assessment, or duplicates its graph.
  if(!before.data.work.nodes.length){
   const page=pages[0];await page.locator('.live-rail button[aria-label="Work"]').click();
   await page.getByRole('button',{name:'Define checkpoint Workflow',exact:true}).click();
   let form=page.getByRole('dialog',{name:'Define checkpoint Workflow'});
   await form.getByLabel('Name',{exact:true}).fill(recipe.name);await form.getByLabel('Workflow key').fill(recipe.key);
   await form.getByLabel('Version',{exact:true}).fill(recipe.version);await form.getByLabel('Description',{exact:true}).fill(recipe.description);
   await form.getByLabel('Checkpoints, one prompt per line').fill(recipe.prompts.join('\n'));
   await form.getByLabel('Required Participant roles, comma separated').fill(recipe.roles.join(','));
   await form.getByRole('button',{name:'Retain definition',exact:true}).click();await form.waitFor({state:'hidden'});
   const definition=exchanges.findLast(item=>item.request.operation_ref==='workflow.define').result;
   assert.equal(definition.result_state,'success');assert.equal(definition.data.nodes.length,recipe.prompts.length);
   await page.getByRole('button',{name:'Bind Workflow',exact:true}).click();form=page.getByRole('dialog',{name:'Bind Workflow'});
   assert.equal(await form.getByLabel('Definition reference').inputValue(),definition.data.workflow_definition_id);
   await form.getByRole('button',{name:'Bind definition',exact:true}).click();await form.waitFor({state:'hidden'});
   const binding=exchanges.findLast(item=>item.request.operation_ref==='workflow.bind').result;
   assert.equal(binding.result_state,'success');
  }
  command={operation:'Studio workflow.define / workflow.bind',recipe:recipe.key,existing_binding_preserved:Boolean(before.data.work.nodes.length)};
 }
 if(process.env.STUDIO_ADVANCE_WORLD==='1'){
  assert.ok(process.env.STUDIO_WORLD_ROOT,'Persistent qualification input root required');
  command=['python3','tests/qualification/studio-product-vertical/operational_world.py','advance','--yai',binary,'--case',caseRef,'--root',process.env.STUDIO_WORLD_ROOT,'--evidence',`${evidence}/commands.jsonl`];
  if(process.env.STUDIO_REFRESH_HTTP==='1')command.push('--refresh-http','--http-revision',process.env.STUDIO_HTTP_REVISION??'2');
  const stdout=[];const child=spawn(command[0],command.slice(1),{env,cwd:path.resolve(import.meta.dirname,'../..')});child.stdout.on('data',b=>stdout.push(b));child.stderr.on('data',b=>stdout.push(b));
  const exit=await new Promise((resolve,reject)=>{child.on('error',reject);child.on('close',resolve);});await writeFile(`${evidence}/advance.log`,Buffer.concat(stdout));assert.equal(exit,0,'Inspect advance.log; no reset/retry fabricated');
 }
 const after=await call('case.summary',{case_ref:caseRef});assert.equal(after.result_state,'success');
 if(process.env.STUDIO_ENTERPRISE_WORKFLOW==='1'){
  assert.ok(after.data.work.nodes.length>0,'A real bound Workflow must be projected');
  const checked=JSON.parse(execFileSync(binary,['case','verify',caseRef,'--json'],{env,encoding:'utf8'}));
  assert.equal(checked.status,'ok');await writeFile(`${evidence}/cli-verify.json`,JSON.stringify(checked,null,2));
 }
 for(const page of pages)await page.getByLabel('Workbench status').getByText(`Generation ${after.data.case.generation}`,{exact:true}).waitFor();
 if(after.data.case.generation!==before.data.case.generation){
  for(let client=0;client<2;client++)assert.ok(events.some(e=>e.client===client&&e.case_ref===caseRef&&e.generation>before.data.case.generation),'Real Host events for each attachment');
  assert.equal(await pages[0].locator('.journal-events').innerText(),paused,'Paused Journal retains previous cut');
  await pages[0].getByRole('button',{name:'Resume follow',exact:true}).click();
  await pages[0].waitForFunction(generation=>[...document.querySelectorAll('.journal-sequence')].at(-1)?.textContent===String(generation),after.data.case.generation);
  assert.notEqual(await pages[0].locator('.journal-events').innerText(),paused);
 }
 const page=pages[0];await page.keyboard.press('Control+j');
 for(const name of ['Overview','Environment','Knowledge','Memory','Authority','Work','Compute']){
  await page.locator(`.live-rail button[aria-label="${name}"]`).click();
  await page.locator('.live-page').waitFor();await page.screenshot({path:`${evidence}/${name.toLowerCase()}.png`});
 }
 assert.deepEqual(errors,[]);await writeFile(`${evidence}/summary.json`,JSON.stringify(after,null,2));
 console.log(JSON.stringify({result:'PASS',case_ref:caseRef,before:before.data.case.generation,after:after.data.case.generation,host_instance:telemetry.instance_id,host_pid:telemetry.pid,attachments:2,events:events.length,resources:after.data.environment.resources.map(r=>r.kind),sources:after.data.environment.sources.map(s=>({id:s.id,kind:s.kind,revision:s.revision_ref,posture:s.posture})),proof:after.data.case.generation!==before.data.case.generation?'Real Unix Host event fanout -> two LiveClients -> authoritative resync; Journal pause/resume':'Two live Host attachments and read-only surface inspection; mutation event proof not selected',mutation:command??'none'}));
}finally{await writeFile(`${evidence}/exchanges.json`,JSON.stringify({exchanges,events},null,2));sockets.forEach(s=>s.destroy());await browser.close();}
