// UI calls the real published Host over Unix IPC through a test-only Tauri bridge.
// Only a freshly created temporary YAI_HOME is mutated; never operator state.
import { createRequire } from 'node:module';
import assert from 'node:assert/strict';
import { execFileSync, execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import net from 'node:net';
import { createServer } from 'node:http';
import os from 'node:os';
import path from 'node:path';
const require = createRequire(path.resolve(import.meta.dirname, '../../studio/package.json'));
const { chromium } = require('playwright-core');
const binary = process.env.YAI_STUDIO_TEST_BINARY;
assert.ok(binary, 'YAI_STUDIO_TEST_BINARY must name a built published CLI/Host');
const home = await mkdtemp(path.join(os.tmpdir(), 'yai-studio-compute-'));
const evidence = process.env.STUDIO_EVIDENCE_DIR ?? '/tmp/yai-studio-compute';
await mkdir(evidence, {recursive:true});
const cli = (...args) => JSON.parse(execFileSync(binary, [...args, '--json'], {env:{...process.env, YAI_HOME:home}, encoding:'utf8', timeout:30000}));
let refuseInventory=false;
let telemetry, serial=0, browser, provider, dropAcknowledgement, holdModelResponse=false, heldModelResponse;
const exchanges = [];
const probeHttp=[];let holdProbe=false,releaseProbe,wrongProbeModel=false,mismatchProbeObservation=false;
let catalogModels = [{id:"controlled-text-model"}];
function rpc(request) {
 return new Promise((resolve,reject) => {
  const socket = net.createConnection(telemetry.endpoint); let buffer='', handshaken=false;
  socket.setTimeout(10000, () => socket.destroy(new Error('Host response timed out')));
  socket.on('error',reject);
  socket.on('connect',()=>socket.write(JSON.stringify({kind:'handshake',protocol:telemetry.protocol,client_id:`studio-ui-qualification:${process.pid}:${++serial}`,client_kind:'qualification',pid:process.pid,yai_home_identity:telemetry.yai_home_identity})+'\n'));
  socket.on('data', bytes => {
   buffer += bytes.toString(); let end;
   while((end=buffer.indexOf('\n'))>=0) {
    const message=JSON.parse(buffer.slice(0,end));buffer=buffer.slice(end+1);
    if(!handshaken) { if(message.kind!=='handshake') {socket.destroy();reject(new Error(JSON.stringify(message)));return;} handshaken=true;socket.write(JSON.stringify({kind:'application_request',request})+'\n'); }
    else if(message.kind==='application_response') { exchanges.push({order:exchanges.length+1,request,result:message.result}); socket.end(); if(refuseInventory && request.operation_ref==='provider.inventory'){resolve({...message.result,result_state:'refused',data:undefined,error:{code:'controlled_inventory_refusal',safe_message:'Controlled inventory refusal'}});}else if(dropAcknowledgement===request.operation_ref){dropAcknowledgement=undefined;reject(new Error('Injected acknowledgement loss after real Host commit'));}else if(holdModelResponse && request.operation_ref==='provider.models'){holdModelResponse=false;heldModelResponse=()=>resolve(message.result);}else if(mismatchProbeObservation && request.operation_ref==='provider.probe.get'){mismatchProbeObservation=false;const wrong=structuredClone(message.result);wrong.data.target_ref='provider-target:unrelated';resolve(wrong);}else resolve(message.result); }
    else if(message.kind==='error') {socket.destroy();reject(new Error(JSON.stringify(message)));}
   }
  });
 });
}
const call = (operation_ref,input={})=>rpc({protocol:'yai.studio.application.v1',operation_ref,correlation_ref:`qualification:${++serial}`,input});
const accepted = async (operation,input) => {const result=await call(operation,input); assert.equal(result.result_state,'success',JSON.stringify(result));return result.data;};
try {
 telemetry=cli('host','start').data.value;
 await accepted('identity.bootstrap',{tenant_id:'tenant:studio-ui',organization_ref:'organization:yailabs'});
 const caseRef='case:studio-policy-actions';
 await accepted('case.create',{tenant_id:'tenant:studio-ui',case_ref:caseRef});await accepted('participant.role.add',{case_ref:caseRef,participant_ref:'participant:operator',role:'operator'});await accepted('participant.principal.link',{case_ref:caseRef,participant_ref:'participant:operator',principal_ref:'self'});
 provider=createServer(async(req,res)=>{
  let body='';for await(const chunk of req)body+=chunk;
  res.setHeader('Content-Type','application/json');
  if(req.url==='/v1/models'){res.end(JSON.stringify({data:catalogModels}));return;}
  const value=body?JSON.parse(body):{};
  probeHttp.push({path:req.url,body:value});
  if(holdProbe){holdProbe=false;await new Promise(resolve=>{releaseProbe=resolve;});}
  if(req.url==='/v1/embeddings'){res.end(JSON.stringify({model:value.model,data:[{index:0,embedding:[0.25,0.5]}],usage:{prompt_tokens:3,total_tokens:3}}));return;}
  if(req.url==='/v1/chat/completions'){
    let content='Controlled provider response', finish='stop', tools;
    if(value.tool_choice==='none')content=value.messages.at(-1).content;
    else if(value.tools){content=null;finish='tool_calls';tools=[{id:'call_exact',type:'function',function:{name:'yai_contract_echo',arguments:JSON.stringify({value:'yai-contract'})}}];}
    else if(value.response_format)content=JSON.stringify({ok:true});
    res.end(JSON.stringify({id:'controlled-response',model:wrongProbeModel?'wrong-model':value.model,choices:[{finish_reason:finish,message:{role:'assistant',content,...(tools?{tool_calls:tools}:{})}}],usage:{prompt_tokens:8,completion_tokens:4,total_tokens:12}}));return;
  }
  res.statusCode=404;res.end('{}');
 });
 await new Promise(resolve=>provider.listen(0,'127.0.0.1',resolve));const endpoint=`http://127.0.0.1:${provider.address().port}`;
 browser=await chromium.launch({executablePath:'/usr/bin/chromium',headless:true,args:['--no-sandbox','--disable-gpu']});
 const page=await browser.newPage({viewport:{width:1440,height:900}});page.setDefaultTimeout(10000);
 const errors=[];page.on('pageerror',error=>errors.push(String(error)));
 await page.exposeFunction('qualificationHostCall',rpc);
 await page.goto(`${process.env.STUDIO_TEST_URL??'http://127.0.0.1:1422'}/?gallery=1`);
 await page.evaluate(async telemetry=>{
  document.getElementById('root').style.display='none';
  const [{default:React},{default:ReactDOM},{StudioApplication},{WorkbenchRegistry},{PlatformServices},{registerContributions},{builtInContributions},{LiveClient},{LiveDataSource}] = await Promise.all([
   import('/node_modules/.vite/deps/react.js'),import('/node_modules/.vite/deps/react-dom_client.js'),import('/src/app/StudioApplication.tsx'),import('/src/workbench/kernel/registry.ts'),import('/src/platform/services.ts'),import('/src/workbench/kernel/contributions.ts'),import('/src/contrib/builtins.ts'),import('/src/clients/live.ts'),import('/src/clients/dataSource.ts')]);
  window.__TAURI__={core:{invoke:(command,args)=>{if(command==='studio_call')return window.qualificationHostCall(args.request);return Promise.resolve(undefined);}},event:{listen:async()=>()=>{}}};
  const state={state:'live',telemetry,resync_required:false};
  const host={capabilities:{kind:'web',nativeDesktop:false,terminalAvailable:false,windowControlsAvailable:false},snapshot:()=>state,subscribe:()=>({dispose(){}}),dispose(){},closeWindow(){},status:async()=>telemetry};
  const client=new LiveClient();const platform=new PlatformServices(host,client);const registry=new WorkbenchRegistry();registerContributions(builtInContributions,{platform,workbench:registry});
  // Presentation-only contributions qualify pinning without inventing Case objects.
  for(const [id,order] of [['Qualification Tool A',30],['Qualification Tool B',31]]) registry.registerViewContainer({id,title:id,icon:'case',order,rail:{section:'pinned',fixed:false,defaultPinned:true},surface:{identity:id,surfaceType:'case.overview',title:id,icon:'case',pinned:true}});
  const container=document.createElement('div');document.body.appendChild(container);
  ReactDOM.createRoot(container).render(React.createElement(StudioApplication,{dataSource:new LiveDataSource(client),platform,registry}));
  window.qualificationPlatform=platform;
 },telemetry);
 await page.getByRole('button',{name:/Studio Policy Actions/}).click();await page.locator('.workbench-kernel').waitFor();await page.keyboard.press('Control+j');await page.locator('.live-rail button[aria-label="Compute"]').click();


 await page.getByRole('button',{name:'Register provider target',exact:true}).click();
 let form=page.getByRole('dialog',{name:'Register provider target'});
 await form.getByLabel('Provider / runtime label').fill('controlled-provider');await form.getByLabel('Endpoint',{exact:true}).fill(endpoint);await form.getByRole('button',{name:'Discover exposed models',exact:true}).click();await form.getByLabel('Exposed model').selectOption('controlled-text-model');

 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]){await page.setViewportSize({width,height});await page.screenshot({path:`${evidence}/connect-${width}x${height}.png`});}
 await page.setViewportSize({width:1440,height:900});
 // Changing endpoint invalidates both the selected model and a late catalog.
 holdModelResponse=true;
 await form.getByRole('button',{name:'Discover exposed models',exact:true}).click();
 for(let attempt=0;!heldModelResponse && attempt<100;attempt++)await new Promise(resolve=>setTimeout(resolve,20));
 assert.ok(heldModelResponse,'Host response held for overlap regression');
 await form.getByLabel('Endpoint',{exact:true}).fill(endpoint+'/other');
 heldModelResponse();heldModelResponse=undefined;
 await page.waitForTimeout(100);
 assert.equal(await form.getByLabel('Exposed model').inputValue(),'');
 assert.equal(await form.getByLabel('Exposed model').locator('option').count(),1,'Old catalog cannot populate new endpoint');
 await form.getByLabel('Endpoint',{exact:true}).fill(endpoint);
 await form.getByRole('button',{name:'Discover exposed models',exact:true}).click();
 await form.getByLabel('Exposed model').selectOption('controlled-text-model');
 await form.getByRole('button',{name:'Register target',exact:true}).click();await form.waitFor({state:'hidden'});
 let response=exchanges.findLast(item=>item.request.operation_ref==='provider.register').result;assert.equal(response.result_state,'success',JSON.stringify(response));const target=response.data;assert.equal(target.model_id,'controlled-text-model');
 assert.equal((await accepted('case.summary',{case_ref:caseRef})).compute.targets.length,0,'Registration must not silently bind');
 const candidate=page.locator('.candidate-target');await candidate.getByRole('button',{name:'Bind provider to Case',exact:true}).click();form=page.getByRole('dialog',{name:'Bind provider to Case'});await form.getByLabel('Exact target reference').fill('provider-target:missing');await form.getByRole('button',{name:'Bind target',exact:true}).click();await form.locator('.action-result').waitFor();
 assert.notEqual(exchanges.findLast(item=>item.request.operation_ref==='provider.case.bind').result.result_state,'success');await form.getByRole('button',{name:'Cancel',exact:true}).click();
 // Real bounded HTTP observations in the test harness, never browser inference or claimed production qualification.
 const started=Date.now();const catalog=await fetch(endpoint+'/v1/models').then(r=>r.json());const completion=await fetch(endpoint+'/v1/chat/completions',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({model:target.model_id,messages:[{role:'user',content:'Bounded qualification'}]})}).then(r=>r.json());
 const evidenceRecord={run_id:'studio-compute-controlled-probe',target_id:target.target_id,started_at_unix_ms:started,completed_at_unix_ms:Date.now(),transport_connected:true,exact_model_addressed:catalog.data.some(item=>item.id===target.model_id)&&completion.model===target.model_id,chat_text_envelope_valid:typeof completion.choices[0].message.content==='string',structured_json_object_valid:false,usage_accounting_observed:false,health_endpoint_observed:false,extension_telemetry_observed:false,failure_codes:[]};
 await writeFile(path.join(evidence,'measured-probe.json'),JSON.stringify(evidenceRecord));
 await candidate.getByRole('button',{name:'Import qualification evidence',exact:true}).click();form=page.getByRole('dialog',{name:'Import qualification evidence'});
 await form.getByLabel('Measured evidence file').setInputFiles(path.join(evidence,'measured-probe.json'));await form.getByLabel('Qualification suite reference').fill('suite:studio-controlled-wire');await form.getByRole('button',{name:'Record evidence',exact:true}).click();await form.waitFor({state:'hidden'});
 response=exchanges.findLast(item=>item.request.operation_ref==='provider.qualify').result;assert.equal(response.result_state,'success',JSON.stringify(response));assert.equal(response.data.evidence.target_id,target.target_id);
 await candidate.getByRole('button',{name:'Set target trust',exact:true}).click();form=page.getByRole('dialog',{name:'Set target trust'});await form.getByRole('button',{name:'Record trust',exact:true}).click();await form.waitFor({state:'hidden'});
 const before=await accepted('case.summary',{case_ref:caseRef});
 await candidate.getByRole('button',{name:'Bind provider to Case',exact:true}).click();form=page.getByRole('dialog',{name:'Bind provider to Case'});await form.getByRole('button',{name:'Bind target',exact:true}).click();await form.waitFor({state:'hidden'});
 const bound=await accepted('case.summary',{case_ref:caseRef});assert.equal(bound.case.generation,before.case.generation+1);assert.equal(bound.compute.targets[0].id,target.target_id);assert.equal(bound.compute.targets[0].posture.trust.posture,'approved');
 await page.locator('.compute-target').getByText('controlled-text-model',{exact:true}).waitFor();
 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]){await page.setViewportSize({width,height});await page.screenshot({path:`${evidence}/compute-${width}x${height}.png`});}
 await page.locator('.compute-target').getByRole('button',{name:'Set target trust',exact:true}).click();form=page.getByRole('dialog',{name:'Set target trust'});await form.getByLabel('Trust decision').selectOption('denied');await form.getByRole('button',{name:'Record trust',exact:true}).click();await form.waitFor({state:'hidden'});

 assert.equal((await accepted('case.summary',{case_ref:caseRef})).compute.targets[0].posture.trust.posture,'denied');assert.equal((await accepted('case.summary',{case_ref:caseRef})).case.generation,bound.case.generation,'Tenant trust changes are not Case transitions');

 await page.setViewportSize({width:1440,height:900});
 await page.locator('.live-rail').getByRole('button',{name:'Providers',exact:true}).click();
 await page.getByRole('heading',{name:'Providers',exact:true}).waitFor();
 await page.locator('.compute-target').getByText('controlled-text-model',{exact:true}).waitFor();
 assert.equal(await page.locator('.live-surface').getAttribute('data-archetype'),'product');
 await page.locator('.compute-target').getByRole('button',{name:'controlled-provider',exact:true}).click();
 const boundInspector=page.locator('.inspector-view');
 await boundInspector.getByRole('heading',{name:'controlled-provider',exact:true}).waitFor();
 await boundInspector.getByText('Bound',{exact:true}).waitFor();
 assert.equal(await boundInspector.getByText(endpoint,{exact:true}).count(),1,'Bound deployment retains the same detailed Inspector');
 await boundInspector.getByText('denied',{exact:true}).waitFor();
 refuseInventory=true;
 await page.getByRole('button',{name:'Refresh Case',exact:true}).click();
 await boundInspector.getByRole('status').filter({hasText:'Tenant inventory unavailable'}).waitFor();
 assert.ok((await boundInspector.textContent()).includes('Current Case projection.'));
 assert.equal(await boundInspector.getByText(endpoint,{exact:true}).count(),1,'Only the disclosed Case target remains under inventory refusal');
 refuseInventory=false;
 await page.getByRole('button',{name:'Refresh Case',exact:true}).click();
 await boundInspector.getByText('Current Tenant inventory.',{exact:false}).waitFor();


 await page.getByRole('button',{name:'Register provider target',exact:true}).click();
 form=page.getByRole('dialog',{name:'Register provider target'});
 await form.getByLabel('Provider / runtime label').fill('unbound-inventory-target');
 await form.getByLabel('Endpoint',{exact:true}).fill(endpoint);
 await form.getByLabel('Enter an exact model ID manually').check();await form.getByLabel('Exact model identity').fill('controlled-text-model');
 await form.getByRole('button',{name:'Register target',exact:true}).click();await form.waitFor({state:'hidden'});
 await page.getByRole('button',{name:'Refresh inventory',exact:true}).click();
 await page.waitForFunction(()=>document.querySelectorAll('.compute-target').length===2);
 const inventory=await accepted('provider.inventory',{tenant_id:'tenant:studio-ui'});
 assert.equal(inventory.targets.length,2);
 assert.equal((await accepted('case.summary',{case_ref:caseRef})).compute.targets.length,1,'Tenant registration does not expand Case binding');
 assert.equal(inventory.case_usage,'not_projected');
 await page.locator('.compute-target').getByRole('button',{name:'unbound-inventory-target',exact:true}).click();
 await page.locator('.inspector-view').getByRole('heading',{name:'unbound-inventory-target',exact:true}).waitFor();
 await page.locator('.inspector-view').getByText('Not bound',{exact:true}).waitFor();
 await page.getByRole('region',{name:'Observed deployment health'}).waitFor();
 await page.getByRole('button',{name:'Check exposed model',exact:true}).click();
 await page.getByText('Model exposed',{exact:true}).waitFor();
 assert.equal((await page.locator('.provider-status').textContent()).includes('Catalog reachable'),false,'Unbound deployment observation must not change Case provider status');
 const catalogRead=exchanges.filter(item=>item.request.operation_ref==='provider.models').at(-1);
 assert.equal(catalogRead.request.input.target_ref,inventory.targets.find(item=>item.provider_key==='unbound-inventory-target').id);
 assert.equal(catalogRead.request.input.endpoint,undefined,'Saved connection resolved by YAI');
 const cliResult=await promisify(execFile)(binary,['provider','models',catalogRead.request.input.target_ref,'--tenant','tenant:studio-ui','--json'],{env:{...process.env,YAI_HOME:home},timeout:30000});
 const cliCatalog=JSON.parse(cliResult.stdout).data.value;
 assert.deepEqual(cliCatalog.models,catalogRead.result.data.models,'CLI and Studio observe the same exact model identities');
 catalogModels=[];
 await page.getByRole('button',{name:'Check exposed model',exact:true}).click();
 await page.getByRole('alert').filter({hasText:'No models exposed.'}).waitFor();
 catalogModels=[{id:'different-model'}];
 await page.getByRole('button',{name:'Check exposed model',exact:true}).click();
 await page.getByText('Model not exposed',{exact:true}).waitFor();
 assert.equal(await page.getByText('Model exposed',{exact:true}).count(),0,'Empty catalog replaces previous positive observation');
 catalogModels=[{id:'controlled-text-model'}];
 holdModelResponse=true;heldModelResponse=undefined;
 await page.getByRole('button',{name:'Check exposed model',exact:true}).click();
 const catalogDeadline=Date.now()+10000;
 while(!heldModelResponse && Date.now()<catalogDeadline) await new Promise(resolve=>setTimeout(resolve,20));
 assert.ok(heldModelResponse,"Catalog response arrived within the bounded test deadline");
 const otherTarget=inventory.targets.find(item=>item.id!==catalogRead.request.input.target_ref);
 await page.locator('.compute-target').getByRole('button',{name:otherTarget.provider_key,exact:true}).click();
 heldModelResponse();
 await page.waitForTimeout(50);
 assert.equal(await page.getByText('Model exposed',{exact:true}).count(),0,'Late catalog cannot describe another selected deployment');
 // Now observe the actual Case-bound target; the footer and workspace share one result.
 await page.getByRole('button',{name:'Check exposed model',exact:true}).click();
 await page.getByText('Model exposed',{exact:true}).waitFor();
 await page.locator('.provider-status').filter({hasText:'Catalog reachable'}).waitFor();
 assert.match(await page.locator('.provider-status').getAttribute('title'),/Catalog observed:/);
 catalogModels=[];
 await page.getByRole('button',{name:'Check exposed model',exact:true}).click();
 await page.locator('.provider-status').filter({hasText:'No models exposed'}).waitFor();
 assert.equal(await page.locator('.provider-status').getAttribute('data-state'),'unavailable');
 catalogModels=[{id:'controlled-text-model'}];
 await page.getByRole('button',{name:'Check exposed model',exact:true}).click();
 await page.locator('.provider-status').filter({hasText:'Catalog reachable'}).waitFor();
 await page.evaluate(()=>window.qualificationPlatform.application.refresh());
 await page.locator('.provider-status').filter({hasText:'Last health:'}).waitFor();
 assert.equal(await page.getByText('Model exposed',{exact:true}).count(),0,'Host contract refresh clears both consumers');
 await page.locator('.compute-target').getByRole('button',{name:'unbound-inventory-target',exact:true}).click();


 assert.equal(await page.locator('.platform-surface.live-page').count(),0,'Platform uses full workspace, not document page');
 await page.getByRole('navigation',{name:'Deployment sections'}).getByRole('button',{name:'Platform',exact:true}).click();
 await page.getByText('No typed management connection',{exact:true}).first().waitFor();
 await page.getByRole('navigation',{name:'Deployment sections'}).getByRole('button',{name:'Runtime',exact:true}).click();
 await page.getByText('Not disclosed by this connection',{exact:true}).waitFor();
 await page.getByRole('searchbox',{name:'Filter deployments'}).fill('unbound-inventory-target');
 assert.equal(await page.locator('.deployment-list .compute-target').count(),1);
 await page.getByRole('searchbox',{name:'Filter deployments'}).fill('');

 // Tenant state can change without a Case generation change: toolbar refresh
 // must invalidate both the inventory Surface and the selected Inspector.
 const unbound=inventory.targets.find(item=>item.provider_key==='unbound-inventory-target');
 // New typed probe: admission survives acknowledgement loss and Surface movement.
 const caseBeforeProbe=await accepted('case.summary',{case_ref:caseRef});
 await page.getByRole('navigation',{name:'Deployment sections'}).getByRole('button',{name:'Evidence',exact:true}).click();
 const check=page.getByRole('region',{name:'Deployment qualification'});
 const beforeStorageRefusal=exchanges.filter(item=>item.request.operation_ref==='provider.probe').length;
 await page.evaluate(()=>{window.savedProbeSetItem=Storage.prototype.setItem;Storage.prototype.setItem=function(key,value){if(key.startsWith('yai.studio.provider-probe.'))throw new Error('Controlled storage refusal');return window.savedProbeSetItem.call(this,key,value);};});
 await check.getByRole('button',{name:'Check & qualify',exact:true}).click();
 await check.getByRole('alert').filter({hasText:'Cannot retain the recovery reference'}).waitFor();
 assert.equal(exchanges.filter(item=>item.request.operation_ref==='provider.probe').length,beforeStorageRefusal);
 await page.evaluate(()=>{Storage.prototype.setItem=window.savedProbeSetItem;delete window.savedProbeSetItem;});
 dropAcknowledgement='provider.probe'; holdProbe=true;
 await check.getByRole('button',{name:'Check & qualify',exact:true}).click();
 const probeDeadline=Date.now()+10000;
 while(!releaseProbe && Date.now()<probeDeadline) await new Promise(resolve=>setTimeout(resolve,20));
 assert.ok(releaseProbe,'Synthetic HTTP is held after durable admission');
 const submitted=exchanges.findLast(item=>item.request.operation_ref==='provider.probe');
 assert.equal(submitted.result.result_state,'success');
 const probeInput=submitted.request.input;
 assert.equal(probeInput.target_ref,unbound.id);
 await check.getByText('Checking',{exact:true}).waitFor();
 await page.getByRole('navigation',{name:'Deployment sections'}).getByRole('button',{name:'Runtime',exact:true}).click();
 await page.locator('.compute-target').getByRole('button',{name:otherTarget.provider_key,exact:true}).click();
 await page.getByRole('navigation',{name:'Deployment sections'}).getByRole('button',{name:'Evidence',exact:true}).click();
 assert.equal(await check.getByText('Checking',{exact:true}).count(),0,'Another deployment cannot inherit an in-flight check');
 releaseProbe();releaseProbe=undefined;
 await page.locator('.compute-target').getByRole('button',{name:unbound.provider_key,exact:true}).click();
 await check.getByText('Evidence recorded',{exact:true}).waitFor();
 const retained=await accepted('provider.probe.get',{target_ref:unbound.id,submission_ref:probeInput.submission_ref});
 assert.equal(retained.posture,'completed');assert.equal(retained.run.evidence.exact_model_addressed,true);
 assert.deepEqual(retained.run.evidence.realization_shapes,['text_to_text']);
 assert.equal(exchanges.filter(item=>item.request.operation_ref==='provider.probe').length,1,'No automatic redispatch after lost acknowledgement');
 const requestsBeforeRetry=probeHttp.length;
 const exactRetry=await accepted('provider.probe',probeInput);
 assert.equal(exactRetry.created,false);assert.deepEqual(exactRetry.run,retained.run);
 assert.equal(probeHttp.length,requestsBeforeRetry);
 assert.deepEqual(await accepted('case.summary',{case_ref:caseRef}),caseBeforeProbe,'Synthetic checks cannot mutate the Case');
 assert.equal((await accepted('provider.inventory',{tenant_id:'tenant:studio-ui'})).targets.find(item=>item.id===unbound.id).posture.trust,null);
 // CLI observes the same retained identity, rather than probing again.
 const qualifiedCLI=await promisify(execFile)(binary,['provider','qualify',unbound.id,'--submission-ref',probeInput.submission_ref,'--realization-shape','text_to_text','--json'],{env:{...process.env,YAI_HOME:home},timeout:30000});
 assert.ok(qualifiedCLI.stdout.includes(retained.run.qualification.qualification_id));
 assert.equal(probeHttp.length,requestsBeforeRetry);
 for(const [mode,expected] of [['tools',['text_to_text','text_to_json_object','text_functions_to_text_or_call']],['embedding',[]]]) {
   await check.getByLabel('Check capabilities').selectOption(mode);
   await check.getByRole('button',{name:'Run a new check',exact:true}).click();
   await check.getByText('Evidence recorded',{exact:true}).waitFor();
   const latest=exchanges.findLast(item=>item.request.operation_ref==='provider.probe').request.input;
   const observed=await accepted('provider.probe.get',{target_ref:unbound.id,submission_ref:latest.submission_ref});
   assert.deepEqual([...(observed.run.evidence.realization_shapes ?? [])].sort(),expected.sort());
   if(mode==='embedding')assert.equal(observed.run.evidence.embedding_dimension,2);
 }
 wrongProbeModel=true;
 await check.getByLabel('Check capabilities').selectOption('text');
 await check.getByRole('button',{name:'Run a new check',exact:true}).click();
 await check.getByText('Exact model not proven',{exact:true}).waitFor();
 const badInput=exchanges.findLast(item=>item.request.operation_ref==='provider.probe').request.input;
 const bad=await accepted('provider.probe.get',{target_ref:unbound.id,submission_ref:badInput.submission_ref});
 assert.equal(bad.run.evidence.exact_model_addressed,false);
 assert.equal(bad.run.qualification.capabilities.some(item=>['chat_text','model_exact_addressing'].includes(item.capability)),false);
 assert.ok(bad.run.evidence.failure_codes.includes('chat_response_model_mismatch'));
 wrongProbeModel=false;
 mismatchProbeObservation=true;
 await check.getByRole('button',{name:'Observe result',exact:true}).click();
 await check.getByRole('alert').filter({hasText:'Provider check identity mismatch'}).waitFor();
 assert.equal(await check.getByText('Exact model not proven',{exact:true}).count(),0,'Mismatched result cannot be displayed');
 const beforeUIRetry=probeHttp.length;
 await check.getByRole('button',{name:'Retry exact request',exact:true}).click();
 await check.getByText('Exact model not proven',{exact:true}).waitFor();
 assert.equal(probeHttp.length,beforeUIRetry,'Explicit exact retry observes the retained result without another HTTP request');
 // Losing browser storage must not hide durable checks or dispatch a replacement.
 const beforeHistoryRecovery=exchanges.filter(item=>item.request.operation_ref==='provider.probe').length;
 await page.evaluate(()=>{for(const key of Object.keys(localStorage))if(key.startsWith('yai.studio.provider-probe.'))localStorage.removeItem(key);});
 await page.getByRole('navigation',{name:'Deployment sections'}).getByRole('button',{name:'Runtime',exact:true}).click();
 await page.getByRole('navigation',{name:'Deployment sections'}).getByRole('button',{name:'Evidence',exact:true}).click();
 const history=check.locator('.provider-check-history');
 await history.locator(':scope > summary').click();
 await history.locator('details').last().locator('summary').click();
 assert.ok((await history.textContent()).includes(probeInput.submission_ref),'Durable request remains discoverable without local storage');
 assert.equal(exchanges.filter(item=>item.request.operation_ref==='provider.probe').length,beforeHistoryRecovery);
 assert.equal(probeHttp.length,beforeUIRetry,'History is observation only');
 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]) {
   await page.setViewportSize({width,height});
   await page.screenshot({path:`${evidence}/qualification-${width}x${height}.png`});
   assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);
 }
 await page.setViewportSize({width:1440,height:900});
 await page.getByRole('navigation',{name:'Deployment sections'}).getByRole('button',{name:'Runtime',exact:true}).click();
 const unchangedGeneration=(await accepted('case.summary',{case_ref:caseRef})).case.generation;
 await accepted('provider.trust.set',{target_ref:unbound.id,posture:'denied'});
 const inventoryReads=exchanges.filter(x=>x.request.operation_ref==='provider.inventory').length;
 await page.getByRole('button',{name:'Refresh Case',exact:true}).click();
 await page.locator('.compute-target').filter({hasText:'unbound-inventory-target'}).getByText('denied',{exact:true}).waitFor();
 await page.locator('.inspector-view').getByText('denied',{exact:true}).waitFor();
 assert.ok(exchanges.filter(x=>x.request.operation_ref==='provider.inventory').length>=inventoryReads+2);
 assert.equal((await accepted('case.summary',{case_ref:caseRef})).case.generation,unchangedGeneration);
 assert.ok(await page.locator('.kernel-status .provider-status').innerText());
 // Selected-target governance needs no manually copied reference. Polling must
 // subsequently observe a Tenant-only change in both workspace and Inspector.
 await page.getByRole('navigation',{name:'Deployment sections'}).getByRole('button',{name:'Evidence',exact:true}).click();
 await page.getByRole('button',{name:'Set target trust',exact:true}).click();
 form=page.getByRole('dialog',{name:'Set target trust'});
 assert.equal(await form.getByLabel('Exact target reference').count(),0);
 await form.getByRole('button',{name:'Record trust',exact:true}).click();await form.waitFor({state:'hidden'});
 assert.equal(exchanges.findLast(x=>x.request.operation_ref==='provider.trust.set').request.input.target_ref,unbound.id);
 await page.locator('.deployment-list').getByText('approved',{exact:true}).waitFor();
 await page.locator('.inspector-view').getByText('approved',{exact:true}).waitFor();
 await page.getByRole('navigation',{name:'Deployment sections'}).getByRole('button',{name:'Runtime',exact:true}).click();
 await accepted('provider.trust.set',{target_ref:unbound.id,posture:'denied'});
 await page.locator('.deployment-list').getByText('denied',{exact:true}).last().waitFor({timeout:15000});
 await page.locator('.inspector-view').getByText('denied',{exact:true}).waitFor({timeout:15000});
 assert.equal((await accepted('case.summary',{case_ref:caseRef})).case.generation,unchangedGeneration);


 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]) {
   await page.setViewportSize({width,height});await page.screenshot({path:`${evidence}/providers-${width}x${height}.png`});
   assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);
 }
 await page.locator('.live-rail').getByRole('button',{name:'YVEX',exact:true}).click();
 await page.getByRole('heading',{name:'Select a deployment',exact:true}).waitFor();
 assert.equal(await page.locator('.compute-target').count(),0,'Generic target is not claimed as YVEX-compatible');
 await page.locator('.live-rail').getByRole('button',{name:'Compute',exact:true}).click();
 await page.waitForFunction(()=>document.querySelectorAll('.compute-target').length===1);
 await page.evaluate(()=>window.qualificationPlatform.commands.executeCommand('studio.view.customizeRail'));
 const railDialog=page.getByRole('dialog',{name:'Customize Activity Rail'});
 await railDialog.getByRole('button',{name:'Unpin Qualification Tool A',exact:true}).click();
 assert.equal(await page.locator('.live-rail').getByRole('button',{name:'Qualification Tool A',exact:true}).count(),0);
 assert.equal(await railDialog.getByRole('button',{name:'Unpin Overview',exact:true}).count(),0,'Core grammar cannot be unpinned');
 await railDialog.getByRole('button',{name:'Pin Qualification Tool A',exact:true}).click();
 await railDialog.getByRole('button',{name:'Move Qualification Tool B up',exact:true}).click();
 assert.deepEqual(await page.locator('.live-rail > button[data-rail-section="pinned"]').evaluateAll(nodes=>nodes.map(node=>node.getAttribute('aria-label'))),['Qualification Tool B','Qualification Tool A']);
 const retainedRail=await page.evaluate(async()=>{
   const {ConfigurationService}=await import('/src/platform/configuration.ts');
   const restored=new ConfigurationService({'workbench.rail.hidden':[],'workbench.rail.order':[]});
   return restored.get('workbench.rail.order');
 });
 assert.deepEqual(retainedRail,['Telemetry','Qualification Tool B','Qualification Tool A']);
 await railDialog.getByRole('button',{name:'Restore rail defaults',exact:true}).click();
 assert.deepEqual(await page.locator('.live-rail > button[data-rail-section="pinned"]').evaluateAll(nodes=>nodes.map(node=>node.getAttribute('aria-label'))),['Qualification Tool A','Qualification Tool B']);
 await page.keyboard.press('Escape');await railDialog.waitFor({state:'hidden'});
 assert.equal(cli('case','verify',caseRef).status,'ok');assert.deepEqual(errors,[]);
 console.log(JSON.stringify({result:'PASS',case_ref:caseRef,target:target.target_id,generation:bound.case.generation,proof:['Synthetic text, JSON/tool roundtrip and embeddings through typed Host; no Case mutation or automatic trust','Lost acknowledgement and target/Surface changes retain exact run; CLI retry performs no HTTP','Storage refusal prevents dispatch; wrong response-model identity visibly fails','Retained history recovers missing local receipts without HTTP or replacement submission','Shared provider footer observation, exact target isolation and invalidation','Rail pin/unpin/reorder persists locally, protects core and restores defaults','Tenant Providers discovers unbound targets without expanding Case binding','YVEX excludes targets without the compatibility extension','Product Surface and four-size Providers matrix','UI register exact target','Unknown exact target bind refused','Real controlled HTTP evidence imported through typed Application','Explicit trust then binding','Denied trust visibly reported; Tenant trust mutation does not invent a Case Transition','4 viewport matrix','CLI replay']}));

}finally{await writeFile(`${evidence}/exchanges.json`,JSON.stringify(exchanges,null,2));await browser?.close();await new Promise(resolve=>provider?.close(resolve)??resolve());try{if(telemetry)cli('host','stop');}finally{await rm(home,{recursive:true,force:true});}}
