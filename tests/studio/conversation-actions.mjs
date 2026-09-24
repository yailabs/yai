// UI calls the real published Host over Unix IPC through a test-only Tauri bridge.
// Only a freshly created temporary YAI_HOME is mutated; never operator state.
import { createRequire } from 'node:module';
import assert from 'node:assert/strict';
import { execFileSync, spawn } from 'node:child_process';
import { mkdtemp, mkdir, readFile, rm, writeFile, stat } from 'node:fs/promises';
import net from 'node:net';
import { createServer } from 'node:http';
import os from 'node:os';
import path from 'node:path';
const require = createRequire(path.resolve(import.meta.dirname, '../../studio/package.json'));
const { chromium } = require('playwright-core');
const binary = process.env.YAI_STUDIO_TEST_BINARY;
assert.ok(binary, 'YAI_STUDIO_TEST_BINARY must name a built published CLI/Host');
const home = await mkdtemp(path.join(os.tmpdir(), 'yai-studio-conversation-'));
const evidence = process.env.STUDIO_EVIDENCE_DIR ?? '/tmp/yai-studio-conversation';
await mkdir(evidence, {recursive:true});
const cli = (...args) => JSON.parse(execFileSync(binary, [...args, '--json'], {env:{...process.env, YAI_HOME:home}, encoding:'utf8', timeout:30000}));
let telemetry, serial=0, browser, provider, dropAcknowledgement, effectChild;
let generationRequests = 0;
let rejectOversized = false;
let providerContent='Controlled provider response';
let holdResponse = false, releaseResponse;
let holdSummary = false, releaseSummary;
const exchanges = [];
const openCandidate = async locator => {const detail=locator.locator('xpath=ancestor::details[1]');if(await detail.getAttribute('open')===null)await detail.locator(':scope > summary').click();};
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
    else if(message.kind==='application_response') { exchanges.push({order:exchanges.length+1,request,result:message.result}); socket.end(); if(dropAcknowledgement===request.operation_ref){dropAcknowledgement=undefined;reject(new Error('Injected acknowledgement loss after real Host commit'));}else resolve(message.result); }
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
 const runtimeDeadline=Date.now()+10000;
 while(true) {
  telemetry=cli('host','status').data.value;
  if(telemetry.runtime_observation?.active_workers!==undefined)break;
  assert.ok(Date.now()<runtimeDeadline,'Host runtime observation missing');
  await new Promise(resolve=>setTimeout(resolve,50));
 }
 const caseRef='case:studio-policy-actions';
 await accepted('case.create',{tenant_id:'tenant:studio-ui',case_ref:caseRef});await accepted('participant.role.add',{case_ref:caseRef,participant_ref:'participant:operator',role:'operator'});await accepted('participant.principal.link',{case_ref:caseRef,participant_ref:'participant:operator',principal_ref:'self'});
 provider=createServer(async(req,res)=>{let body='';for await(const chunk of req)body+=chunk;res.setHeader('Content-Type','application/json');if(req.url==='/v1/models')res.end(JSON.stringify({data:[{id:'controlled-text-model'}]}));else if(req.url==='/v1/chat/completions'){const value=JSON.parse(body);generationRequests++;if(rejectOversized){res.statusCode=413;res.end(JSON.stringify({error:{code:'input_too_large'}}));return;}if(holdResponse)await new Promise(resolve=>releaseResponse=resolve);res.end(JSON.stringify({id:'controlled-response',model:value.model,choices:[{message:{role:'assistant',content:providerContent}}]}));}else{res.statusCode=404;res.end('{}');}});
 await new Promise(resolve=>provider.listen(0,'127.0.0.1',resolve));const endpoint=`http://127.0.0.1:${provider.address().port}`;
 browser=await chromium.launch({executablePath:'/usr/bin/chromium',headless:true,args:['--no-sandbox','--disable-gpu']});
 const page=await browser.newPage({viewport:{width:1440,height:900}});page.setDefaultTimeout(10000);
 const errors=[];page.on('pageerror',error=>errors.push(String(error)));
 await page.exposeFunction('qualificationHostCall',async request=>{const result=await rpc(request);if(holdSummary && request.operation_ref==='case.summary'){holdSummary=false;await new Promise(resolve=>releaseSummary=resolve);}return result;});
 await page.goto(`${process.env.STUDIO_TEST_URL??'http://127.0.0.1:1422'}/?gallery=1`);
 await page.evaluate(async telemetry=>{
  document.getElementById('root').style.display='none';
  const [{default:React},{default:ReactDOM},{StudioApplication},{WorkbenchRegistry},{PlatformServices},{registerContributions},{builtInContributions},{LiveClient},{LiveDataSource}] = await Promise.all([
   import('/node_modules/.vite/deps/react.js'),import('/node_modules/.vite/deps/react-dom_client.js'),import('/src/app/StudioApplication.tsx'),import('/src/workbench/kernel/registry.ts'),import('/src/platform/services.ts'),import('/src/workbench/kernel/contributions.ts'),import('/src/contrib/builtins.ts'),import('/src/clients/live.ts'),import('/src/clients/dataSource.ts')]);
  window.__TAURI__={core:{invoke:(command,args)=>{if(command==='studio_call')return window.qualificationHostCall(args.request);return Promise.resolve(undefined);}},event:{listen:async()=>()=>{}}};
  const state={state:'live',telemetry,resync_required:false};
  const host={capabilities:{kind:'web',nativeDesktop:false,terminalAvailable:false,windowControlsAvailable:false},snapshot:()=>state,subscribe:()=>({dispose(){}}),dispose(){},closeWindow(){},status:async()=>telemetry};
  const client=new LiveClient();const platform=new PlatformServices(host,client);const registry=new WorkbenchRegistry();registerContributions(builtInContributions,{platform,workbench:registry});
  const container=document.createElement('div');document.body.appendChild(container);
  ReactDOM.createRoot(container).render(React.createElement(StudioApplication,{dataSource:new LiveDataSource(client),platform,registry}));
  window.qualificationPlatform=platform;
 },telemetry);
 await page.getByRole('button',{name:/Studio Policy Actions/}).click();await page.locator('.workbench-kernel').waitFor();await page.keyboard.press('Control+j');await page.locator('.live-rail button[aria-label="Compute"]').click();


 await page.getByRole('button',{name:'Register provider target',exact:true}).click();
 let form=page.getByRole('dialog',{name:'Register provider target'});
 await form.getByLabel('Provider / runtime label').fill('controlled-provider');await form.getByLabel('Endpoint',{exact:true}).fill(endpoint);await form.getByRole('button',{name:'Discover exposed models',exact:true}).click();await form.getByText('1 exposed model(s). Discovery does not qualify inference.').waitFor();assert.deepEqual(exchanges.findLast(x=>x.request.operation_ref==='provider.models').result.data.models,['controlled-text-model']);await form.getByLabel('Exposed model').selectOption('controlled-text-model');
 await form.getByRole('button',{name:'Register target',exact:true}).click();await form.waitFor({state:'hidden'});
 let response=exchanges.findLast(item=>item.request.operation_ref==='provider.register').result;assert.equal(response.result_state,'success',JSON.stringify(response));const target=response.data;assert.equal(target.model_id,'controlled-text-model');
 assert.equal((await accepted('case.summary',{case_ref:caseRef})).compute.targets.length,0,'Registration must not silently bind');
 const candidate=page.locator('.candidate-target');await candidate.getByRole('button',{name:'Bind provider to Case',exact:true}).click();form=page.getByRole('dialog',{name:'Bind provider to Case'});await form.getByLabel('Exact target reference').fill('provider-target:missing');await form.getByRole('button',{name:'Bind target',exact:true}).click();await form.locator('.action-result').waitFor();
 assert.notEqual(exchanges.findLast(item=>item.request.operation_ref==='provider.case.bind').result.result_state,'success');await form.getByRole('button',{name:'Cancel',exact:true}).click();
 // Real bounded HTTP observations in the test harness, never browser inference or claimed production qualification.
 const started=Date.now();const catalog=await fetch(endpoint+'/v1/models').then(r=>r.json());const completion=await fetch(endpoint+'/v1/chat/completions',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({model:target.model_id,messages:[{role:'user',content:'Bounded qualification'}]})}).then(r=>r.json());
 const evidenceRecord={run_id:'studio-compute-controlled-probe',target_id:target.target_id,started_at_unix_ms:started,completed_at_unix_ms:Date.now(),transport_connected:true,exact_model_addressed:catalog.data.some(item=>item.id===target.model_id)&&completion.model===target.model_id,chat_text_envelope_valid:typeof completion.choices[0].message.content==='string',realization_shapes:['text_to_text'],structured_json_object_valid:false,usage_accounting_observed:false,health_endpoint_observed:false,extension_telemetry_observed:false,failure_codes:[]};
 await writeFile(path.join(evidence,'measured-probe.json'),JSON.stringify(evidenceRecord));
 await candidate.getByRole('button',{name:'Import qualification evidence',exact:true}).click();form=page.getByRole('dialog',{name:'Import qualification evidence'});
 await form.getByLabel('Measured evidence file').setInputFiles(path.join(evidence,'measured-probe.json'));await form.getByLabel('Qualification suite reference').fill('suite:studio-controlled-wire');await form.getByRole('button',{name:'Record evidence',exact:true}).click();await form.waitFor({state:'hidden'});
 response=exchanges.findLast(item=>item.request.operation_ref==='provider.qualify').result;assert.equal(response.result_state,'success',JSON.stringify(response));assert.equal(response.data.evidence.target_id,target.target_id);
 await candidate.getByRole('button',{name:'Set target trust',exact:true}).click();form=page.getByRole('dialog',{name:'Set target trust'});await form.getByRole('button',{name:'Record trust',exact:true}).click();await form.waitFor({state:'hidden'});
 const before=await accepted('case.summary',{case_ref:caseRef});
 await candidate.getByRole('button',{name:'Bind provider to Case',exact:true}).click();form=page.getByRole('dialog',{name:'Bind provider to Case'});await form.getByRole('button',{name:'Bind target',exact:true}).click();await form.waitFor({state:'hidden'});
 const bound=await accepted('case.summary',{case_ref:caseRef});assert.equal(bound.case.generation,before.case.generation+1);assert.equal(bound.compute.targets[0].id,target.target_id);assert.equal(bound.compute.targets[0].posture.trust.posture,'approved');
 await page.locator('.compute-target').getByText('controlled-text-model',{exact:true}).waitFor();

 await accepted('participant.view.admit',{case_ref:caseRef,participant_ref:'participant:operator',consumer:'model',view_kind:'model_context'});
 await page.evaluate(()=>window.qualificationPlatform.commands.executeCommand('studio.case.refresh'));
 await page.getByRole('button',{name:'Attest conversation suitability',exact:true}).click();
 form=page.getByRole('dialog',{name:'Attest conversation suitability'});
 await form.getByLabel('Assessment / evidence reference').fill('evidence:controlled-text-roundtrip');
 await form.getByRole('checkbox').check();await form.getByRole('button',{name:'Record my attestation'}).click();await form.waitFor({state:'hidden'});
 const attested=exchanges.findLast(x=>x.request.operation_ref==='provider.suitability.record').result;
 assert.equal(attested.result_state,'success');assert.equal(attested.data.posture,'operator_attested');
 await page.getByRole('main').getByRole('button',{name:'Assign conversation model',exact:true}).click();form=page.getByRole('dialog',{name:'Assign conversation model'});
 await form.getByRole('button',{name:'Assign model',exact:true}).click();await form.waitFor({state:'hidden'});
 let snapshot=await accepted('case.summary',{case_ref:caseRef});assert.equal(snapshot.compute.cognitive_bindings[0].target_id,target.target_id);
 await page.locator('.model-status').filter({hasText:'Assigned: controlled-text-model'}).waitFor();
 await page.locator('.live-rail button[aria-label="Providers"]').click();
 await page.getByRole('button',{name:'Check exposed model',exact:true}).click();
 await page.locator('.model-status').filter({hasText:'Exposed: controlled-text-model'}).waitFor();
 assert.match(await page.locator('.model-status').getAttribute('title'),/does not establish engine residency or capacity/);
 await page.locator('.live-rail button[aria-label="Compute"]').click();
 assert.equal(await page.locator('.model-status').textContent(),'Exposed: controlled-text-model','Observation survives navigation without another probe');

 assert.equal(snapshot.compute.targets[0].semantic_evidence[0].evidence_id,attested.data.evidence_id);
 // A refresh requested during an older read must wait for a fresh snapshot.
 await page.evaluate(()=>window.qualificationPlatform.commands.executeCommand('studio.case.refresh'));
 const refreshStart=exchanges.length;holdSummary=true;
 await page.evaluate(()=>{window.firstRefresh=window.qualificationPlatform.commands.executeCommand('studio.case.refresh');});
 const heldDeadline=Date.now()+3000;while(!releaseSummary){assert.ok(Date.now()<heldDeadline);await new Promise(resolve=>setTimeout(resolve,10));}
 await accepted('participant.role.add',{case_ref:caseRef,participant_ref:'participant:operator',role:'refresh-overlap-proof'});
 const refreshedGeneration=(await accepted('case.summary',{case_ref:caseRef})).case.generation;
 await page.evaluate(()=>{window.secondRefresh=window.qualificationPlatform.commands.executeCommand('studio.case.refresh');});
 releaseSummary();releaseSummary=undefined;
 await page.evaluate(async()=>{await Promise.all([window.firstRefresh,window.secondRefresh]);});
 const uiRefreshes=exchanges.slice(refreshStart).filter(x=>x.request.operation_ref==='case.summary' && x.request.correlation_ref.startsWith('studio:'));
 assert.ok(uiRefreshes.length>=2,'Concurrent refresh was dropped');assert.equal(uiRefreshes.at(-1).result.data.case.generation,refreshedGeneration);
 await page.getByRole('button',{name:'Conversation',exact:true}).click();
 const composer=page.getByRole('textbox',{name:'Message to the Case'});
 await page.getByRole('button',{name:'Conversation tools'}).click();
 const toolsBox=await page.getByRole('menu',{name:'Conversation tools'}).boundingBox();
 const conversationBox=await page.locator('.conversation-operational').boundingBox();
 assert.ok(toolsBox && conversationBox && toolsBox.x>=conversationBox.x && toolsBox.x+toolsBox.width<=conversationBox.x+conversationBox.width && toolsBox.y>=conversationBox.y,'Tools menu fits the Context Panel');
 await page.screenshot({path:`${evidence}/conversation-tools-1440x900.png`});
 await page.getByRole('menu',{name:'Conversation tools'}).getByRole('menuitemradio',{name:/Fast Search/}).click();
 await page.getByText('Fast Search · fallback available').waitFor();
 const baselineRequests=generationRequests;
 holdResponse=true;
 await composer.fill('First exact Studio message');await page.getByRole('button',{name:'Send',exact:true}).click();
 const inFlightDeadline=Date.now()+10000;
 while(!releaseResponse) {assert.ok(Date.now()<inFlightDeadline,'Provider never received request');await new Promise(resolve=>setTimeout(resolve,20));}
 const inFlight=exchanges.findLast(x=>x.request.operation_ref==='conversation.send');
 for(const execution of [{domain:'conversation',submission_ref:inFlight.request.input.submission_ref},
   {domain:'cognitive_composition',request_ref:inFlight.result.data.execution.request_ref}]) {
  const observed=await accepted('execution.get',{case_ref:caseRef,participant_ref:'participant:operator',execution});
  assert.equal(observed.posture,'running','Both public identities observe the same live carrier');
  assert.equal(observed.primary_result,null);
 }
 holdResponse=false;releaseResponse();releaseResponse=undefined;
 await page.locator('.conversation-answer').getByText('Controlled provider response',{exact:true}).waitFor();
 const completedDetails=page.locator('.turn-ai').first().locator(':scope > details');
 await completedDetails.locator(':scope > summary').click();
 await completedDetails.getByText('Result received',{exact:true}).waitFor();
 assert.equal(await completedDetails.getByText('200',{exact:true}).count(),1);
 await completedDetails.locator(':scope > summary').click();
 const fastSubmission=exchanges.filter(x=>x.request.operation_ref==='conversation.send').at(-1);
 assert.equal(fastSubmission.request.input.memory_search_mode,'fast');
 assert.equal(fastSubmission.result.data.memory_search.requested,'fast');
 assert.equal(fastSubmission.result.data.memory_search.effective,'standard');
 await page.getByText('Fast Search unavailable; YAI used qualified standard memory.').waitFor();
 await page.getByRole('button',{name:'Conversation tools'}).click();
 await page.getByRole('menu',{name:'Conversation tools'}).getByRole('menuitemradio',{name:/Standard/}).click();
 snapshot=await accepted('case.summary',{case_ref:caseRef});assert.equal(snapshot.conversation.turns.length,1);assert.ok(snapshot.conversation.turns[0].execution_request_ref);
 assert.equal(generationRequests,baselineRequests+1);assert.equal(await composer.inputValue(),'');
 // Exact retry after lost acknowledgement and read-only recovery must not redispatch.
 dropAcknowledgement='conversation.send';await composer.fill('Second exact Studio message');await page.getByRole('button',{name:'Send',exact:true}).click();
 await page.getByRole('button',{name:'Float Conversation',exact:true}).click();
 await page.waitForFunction(()=>document.querySelectorAll('.conversation-answer > p').length===2);
 await page.waitForFunction(()=>[...document.querySelectorAll('.conversation-answer > p')].every(p=>p.textContent==='Controlled provider response'));
 const committedRefs=(await accepted('case.summary',{case_ref:caseRef})).conversation.turns.map(turn=>turn.execution_request_ref);
 await page.getByRole('button',{name:'Close Conversation tool',exact:true}).click();
 await page.getByRole('button',{name:'Open Conversation tool',exact:true}).click();
 await page.getByRole('button',{name:'Dock Conversation',exact:true}).click();
 assert.deepEqual((await accepted('case.summary',{case_ref:caseRef})).conversation.turns.map(turn=>turn.execution_request_ref),committedRefs);
 assert.equal(generationRequests,baselineRequests+2);
 const sent=exchanges.filter(x=>x.request.operation_ref==='conversation.send').at(-1);
 const retry=await accepted('conversation.send',sent.request.input);assert.equal(retry.created,false);
 assert.equal(generationRequests,baselineRequests+2);
 // A stale generation refuses without creating a Turn or discarding the local draft.
 await composer.fill('Draft survives stale generation');
 await accepted('participant.role.add',{case_ref:caseRef,participant_ref:'participant:operator',role:'qualification-extra'});
 await page.getByRole('button',{name:'Send',exact:true}).click();
 await page.locator('.conversation-error').waitFor();
 assert.equal(exchanges.filter(x=>x.request.operation_ref==='conversation.send').at(-1).result.result_state,'stale');
 assert.equal(await composer.inputValue(),'Draft survives stale generation');

 assert.equal((await accepted('case.summary',{case_ref:caseRef})).conversation.turns.length,2);
 // Leave/re-enter the auxiliary view: results reconstructed from canonical intent refs.
 await page.getByRole('button',{name:'Inspector',exact:true}).click();await page.getByRole('button',{name:'Conversation',exact:true}).click();
 await page.waitForFunction(()=>document.querySelectorAll('.conversation-answer > p').length===2);
 assert.equal(await composer.inputValue(),'Draft survives stale generation');
 await composer.focus();
 assert.equal(await composer.evaluate(node=>getComputedStyle(node).outlineStyle),'none');
 const focusedBorder=await page.locator('.conversation-composer').evaluate(node=>getComputedStyle(node).borderColor);
 await composer.blur();
 assert.equal(await page.locator('.conversation-composer').evaluate(node=>getComputedStyle(node).borderColor),focusedBorder);
 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]) {
   await page.setViewportSize({width,height});
   const composerBox = await page.locator('.conversation-composer').boundingBox();
   assert.ok(composerBox && composerBox.y >= 0 && composerBox.y + composerBox.height <= height, 'Composer remains reachable');
   await page.getByRole('button',{name:'Conversation tools'}).click();
   const menuBox=await page.getByRole('menu',{name:'Conversation tools'}).boundingBox();
   const panelBox=await page.locator('.conversation-operational').boundingBox();
   assert.ok(menuBox && panelBox && menuBox.x>=panelBox.x && menuBox.x+menuBox.width<=panelBox.x+panelBox.width && menuBox.y>=panelBox.y,'Tools menu remains visible in the viewport matrix');
   await page.keyboard.press('Escape');
   await page.screenshot({path:`${evidence}/conversation-${width}x${height}.png`});
   assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);
 }
 // A public provider refusal must be explained, never rendered as model output.
 rejectOversized=true;
 await composer.fill('A short message still carries Case context');
 await page.getByRole('button',{name:'Send',exact:true}).click();
 await page.getByText(/The model server rejected this request as too large/).waitFor();
 assert.equal(generationRequests-baselineRequests,3);
 assert.equal(await page.locator('.conversation-answer > p').filter({hasText:'Controlled provider response'}).count(),2);
 // Overview narration reuses governed SEND; no provider shortcut or draft replacement.
 rejectOversized=false;
 await page.setViewportSize({width:1440,height:900});
 await composer.fill('Conversation draft survives narrative generation');
 await page.locator('.live-rail button[aria-label="Overview"]').click();
 const narrative=page.getByRole('region',{name:'Model explanation'});
 const narrativeBefore=generationRequests;
 dropAcknowledgement='conversation.send';
 await narrative.getByRole('button',{name:'Generate explanation',exact:true}).click();
 await narrative.locator('.narrative-text').getByText('Controlled provider response',{exact:true}).waitFor();
 assert.equal(generationRequests,narrativeBefore+1);
 assert.equal(await composer.inputValue(),'Conversation draft survives narrative generation');
 const narrativeSend=exchanges.findLast(x=>x.request.operation_ref==='conversation.send');
 assert.match(new TextDecoder().decode(new Uint8Array(narrativeSend.request.input.parts[0].bytes)),/storia operativa/);
 const narrativeRetry=await accepted('conversation.send',narrativeSend.request.input);
 assert.equal(narrativeRetry.created,false);assert.equal(generationRequests,narrativeBefore+1);
 await page.locator('.live-rail button[aria-label="Memory"]').click();
 await page.locator('.live-rail button[aria-label="Overview"]').click();
 await narrative.locator('.narrative-text').getByText('Controlled provider response',{exact:true}).waitFor();
 assert.equal(generationRequests,narrativeBefore+1,'Reopen must only observe');
 await narrative.getByRole('button',{name:'Check explanation',exact:true}).click();
 assert.equal(generationRequests,narrativeBefore+1);
 await page.locator('.live-rail button[aria-label="Telemetry"]').click();
 await page.getByRole('heading',{name:'Telemetry',exact:true}).waitFor();
 const telemetryUrl=page.url();
 assert.equal(await page.getByRole('tabpanel').count(),1);
 assert.equal(await page.locator('#telemetry-runtime').isVisible(),false);
 assert.ok((await page.locator('#telemetry-host').innerText()).includes(String(telemetry.pid)));
 await page.getByRole('tab',{name:'Host',exact:true}).focus();await page.keyboard.press('ArrowRight');
 assert.equal(await page.getByRole('tab',{name:'Runtime',exact:true}).getAttribute('aria-selected'),'true');
 assert.equal(await page.locator('#telemetry-runtime').getByText('PID',{exact:true}).locator('..').locator('dd').innerText(),String(telemetry.runtime_observation.pid));
 assert.equal(await page.locator('#telemetry-runtime').getByText('Active workers at observation',{exact:true}).locator('..').locator('dd').innerText(),String(telemetry.runtime_observation.active_workers));
 const observedSnapshot=await accepted('case.summary',{case_ref:caseRef});
 const providerHealth=observedSnapshot.compute.targets[0].posture.health;
 await page.getByRole('tab',{name:'Endpoints',exact:true}).click();
 assert.ok((await page.locator('#telemetry-endpoints').innerText()).includes(providerHealth.circuit));
 assert.ok((await page.locator('#telemetry-endpoints').innerText()).includes(String(providerHealth.consecutive_failures)));
 assert.equal(await page.locator('#telemetry-endpoints').getByText(providerHealth.observed_at_unix_ms ? `Observed: ${providerHealth.posture}` : 'Live health unknown',{exact:true}).count(),1);
 await page.locator('.telemetry-sidebar-link[data-section="clients"]').click();
 assert.equal(await page.getByRole('tab',{name:'Clients',exact:true}).getAttribute('aria-selected'),'true');
 assert.equal(await page.locator('#telemetry-clients').isVisible(),true);
 assert.equal(page.url(),telemetryUrl,'Telemetry navigation must not mutate browser hash');
 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]) {
  await page.setViewportSize({width,height});
  assert.equal(await page.getByRole('tabpanel').count(),1);
  const geometry=await page.locator('.telemetry-surface').evaluate(element=>({width:element.clientWidth,scroll:element.scrollWidth}));
  assert.ok(geometry.scroll<=geometry.width+1,'Telemetry must not overflow the Work Surface');
  await page.screenshot({path:`${evidence}/telemetry-${width}x${height}.png`});
 }
 await page.getByRole('tab',{name:'Clients',exact:true}).focus();await page.keyboard.press('End');
 assert.equal(await page.getByRole('tab',{name:'Executions',exact:true}).getAttribute('aria-selected'),'true');
 assert.equal(await page.locator('#telemetry-executions .execution-history').isVisible(),true);
 const lastTab=await page.getByRole('tab',{name:'Executions',exact:true}).boundingBox();
 const tabStrip=await page.getByRole('tablist',{name:'Telemetry sections'}).boundingBox();
 assert.ok(lastTab.x+lastTab.width<=tabStrip.x+tabStrip.width+1,'Keyboard focus reveals the last narrow tab');
 await page.keyboard.press('Home');
 assert.equal(await page.getByRole('tab',{name:'Host',exact:true}).getAttribute('aria-selected'),'true');
 assert.equal(await page.locator('#telemetry-executions .execution-history').count(),0,'Hidden telemetry execution receipts are not mounted');
 await page.setViewportSize({width:1440,height:900});
 await page.locator('.live-rail button[aria-label="Overview"]').click();
 await narrative.locator('.narrative-text').getByText('Controlled provider response',{exact:true}).waitFor();
 // Revoked current trust refuses a new model assignment (no bypass in React).
 // Explicit retained-input plan and execution: no frontend hashes or hidden SEND.
 await page.locator('.live-rail button[aria-label="Compute"]').click();
 const cognition=page.getByRole('region',{name:'Cognitive execution',exact:true});
 const planBefore=await accepted('case.summary',{case_ref:caseRef});
 const sourceTurn=planBefore.conversation.turns[0].id;
 await cognition.getByLabel('Committed Turn',{exact:true}).selectOption(sourceTurn);
 const planDispatches=generationRequests;
 await cognition.getByLabel('Cognitive capability',{exact:true}).selectOption('speech_to_text');
 await cognition.getByRole('button',{name:'Prepare execution plan',exact:true}).click();
 await cognition.getByRole('alert').waitFor();
 assert.equal(await cognition.getByRole('region',{name:'Prepared execution plan'}).count(),0,'Text input must not produce a fake audio plan');
 assert.equal(generationRequests,planDispatches);
 await cognition.getByLabel('Cognitive capability',{exact:true}).selectOption('primary_conversation');
 await cognition.getByRole('button',{name:'Prepare execution plan',exact:true}).click();
 await cognition.getByRole('region',{name:'Prepared execution plan'}).waitFor();
 const oldPlanExchange=exchanges.findLast(x=>x.request.operation_ref==='cognitive.realization.prepare');
 assert.equal(oldPlanExchange.result.data.selected_target_id,target.target_id);
 assert.deepEqual(oldPlanExchange.request.input.source_part_refs,[],'The owner resolves all exact original Turn parts');
 assert.equal(generationRequests,planDispatches,'Preparation must not invoke the model');
 assert.equal((await accepted('case.summary',{case_ref:caseRef})).case.generation,planBefore.case.generation);
 await accepted('participant.role.add',{case_ref:caseRef,participant_ref:'participant:operator',role:'plan-staleness-check'});
 await page.evaluate(()=>window.qualificationPlatform.commands.executeCommand('studio.case.refresh'));
 await cognition.getByText('The input or Case changed. Prepare a fresh plan before executing.',{exact:true}).waitFor();
 assert.equal(await cognition.getByRole('button',{name:'Execute prepared plan…',exact:true}).count(),0);
 const stalePlan=await call('cognitive.realize',{...oldPlanExchange.request.input,plan_ref:oldPlanExchange.result.data.plan_id});
 assert.notEqual(stalePlan.result_state,'success');assert.equal(generationRequests,planDispatches);
 await cognition.getByRole('button',{name:'Prepare execution plan',exact:true}).click();
 await cognition.getByRole('region',{name:'Prepared execution plan'}).waitFor();
 const planExchange=exchanges.findLast(x=>x.request.operation_ref==='cognitive.realization.prepare');
 assert.notEqual(planExchange.result.data.plan_id,oldPlanExchange.result.data.plan_id);
 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]) {
  await page.setViewportSize({width,height});await cognition.scrollIntoViewIfNeeded();
  assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);
  await page.screenshot({path:`${evidence}/cognitive-plan-${width}x${height}.png`});
 }
 await page.setViewportSize({width:1440,height:900});
 holdResponse=true;dropAcknowledgement='cognitive.realize';
 await cognition.getByRole('button',{name:'Execute prepared plan…',exact:true}).click();
 form=page.getByRole('dialog',{name:'Execute prepared input',exact:true});
 await form.getByRole('button',{name:'Submit exact plan',exact:true}).click();
 await form.getByText('Confirmation was lost',{exact:true}).waitFor();
 const realizeExchange=exchanges.findLast(x=>x.request.operation_ref==='cognitive.realize');
 assert.equal(realizeExchange.request.input.plan_ref,planExchange.result.data.plan_id);
 const planDeadline=Date.now()+10000;
 while(!releaseResponse) { assert.ok(Date.now()<planDeadline,'Explicit realization never reached controlled provider');await new Promise(resolve=>setTimeout(resolve,20)); }
 await form.getByRole('button',{name:'Close and inspect state'}).click();
 const realizationRef={domain:'cognitive_realization',plan_ref:planExchange.result.data.plan_id};
 const runningPlan=await accepted('execution.get',{case_ref:caseRef,participant_ref:'participant:operator',execution:realizationRef});
 assert.equal(runningPlan.posture,'running');assert.equal(generationRequests,planDispatches+1);
 holdResponse=false;releaseResponse();releaseResponse=undefined;
 const resultDeadline=Date.now()+10000;
 let realizedObservation;
 while(true) {const value=await accepted('execution.get',{case_ref:caseRef,participant_ref:'participant:operator',execution:realizationRef});if(value.provider_result){realizedObservation=value;break;}assert.ok(Date.now()<resultDeadline);await new Promise(resolve=>setTimeout(resolve,50));}
 assert.equal(realizedObservation.provider_result.selection.selected_target_id,target.target_id);
 assert.equal(realizedObservation.invocation_refs.length,1);
 await cognition.getByRole('button',{name:'Refresh observation',exact:true}).click();
 await cognition.locator('.cognitive-result').getByText('Controlled provider response',{exact:true}).waitFor();
 const beforeExactRepeat=await accepted('case.summary',{case_ref:caseRef});
 const exactRepeat=await accepted('cognitive.realize',realizeExchange.request.input);
 assert.equal(exactRepeat.plan_ref,realizationRef.plan_ref);assert.equal(generationRequests,planDispatches+1);
 assert.equal(exactRepeat.provider_result.result_id,realizedObservation.provider_result.result_id);
 const afterExactRepeat=await accepted('case.summary',{case_ref:caseRef});
 assert.equal(afterExactRepeat.case.generation,beforeExactRepeat.case.generation,'Exact retry creates no extra canonical consequence');
 assert.equal(afterExactRepeat.conversation.turns.length,planBefore.conversation.turns.length,'Realization must not manufacture a new conversation Turn');
 await page.locator('.live-rail button[aria-label="Memory"]').click();
 await page.locator('.live-rail button[aria-label="Compute"]').click();
 await cognition.locator('.cognitive-result').getByText('Controlled provider response',{exact:true}).waitFor();
 assert.equal(generationRequests,planDispatches+1,'Reopening only observes retained plan identity');
 for(const operation of ['cognitive.realization.prepare','cognitive.realize']) {
  const refused=await call(operation,{...(operation==='cognitive.realize'?realizeExchange.request.input:planExchange.request.input),participant_ref:'participant:hidden'});
  assert.notEqual(refused.result_state,'success');assert.equal(refused.data,undefined);
 }
 const hiddenPlan=await call('execution.get',{case_ref:caseRef,participant_ref:'participant:hidden',execution:realizationRef});
 assert.notEqual(hiddenPlan.result_state,'success');assert.equal(hiddenPlan.data,undefined);
 await page.locator('.live-rail button[aria-label="Overview"]').click();
 // A retained model candidate is normalized and submitted through explicit Studio actions.
 const effectRoot=path.join(home,'candidate-workspace');await mkdir(path.join(effectRoot,'allowed'),{recursive:true});
 cli('case','attach-filesystem','--case',caseRef,'--attachment','candidate-workspace','--root',effectRoot,'--allow-prefix','allowed','--policy-owner','participant:operator','--max-bytes','1024');
 await accepted('participant.role.add',{case_ref:caseRef,participant_ref:'participant:operator',role:'operation-proposer'});
 const effectPolicy=JSON.parse(await readFile(path.resolve(import.meta.dirname,'../fixtures/cli-product-policy.json'),'utf8'));effectPolicy.owner_ref='organization:yailabs';effectPolicy.rules.push({kind:'operation_restriction',rule_id:'process-posture',operation_kind:'process.signal',resource_kind:'process',effect:'allow',reason:'Only the bound disposable test process'});
 const effectArtifact=(await accepted('policy.ingest',{tenant_id:'tenant:studio-ui',source_bytes:[...Buffer.from(JSON.stringify(effectPolicy))]})).view.artifact.artifact_id;
 for(const operation of ['policy.validate','policy.publish'])await accepted(operation,{artifact_ref:effectArtifact,reason:'Controlled Studio candidate effect'});
 await accepted('policy.case.bind',{case_ref:caseRef,artifact_ref:effectArtifact,expected_generation:(await accepted('case.summary',{case_ref:caseRef})).case.generation,reason:'Controlled Studio candidate effect'});
 await page.evaluate(()=>window.qualificationPlatform.commands.executeCommand('studio.case.refresh'));
 // Ordinary prose is not silently turned into an operation.
 const prose=page.locator('.turn-ai').first().locator('.candidate-effect');
 await openCandidate(prose);await prose.getByRole('button',{name:'Prepare Resource action…',exact:true}).click();form=page.getByRole('dialog',{name:'Prepare Resource action',exact:true});
 await form.getByRole('button',{name:'Record proposal',exact:true}).click();await form.getByRole('alert').waitFor();
 const archivedRefusal=exchanges.findLast(x=>x.request.operation_ref==='effect.propose');assert.equal(archivedRefusal.result.error.code,'normalization_failure_result_mismatch');await form.getByText('YAI currently requires the latest provider result for a new Resource proposal. This older candidate was not recorded as an Operation.',{exact:true}).waitFor();
 await form.getByRole('button',{name:'Cancel',exact:true}).click();assert.equal(await prose.getByRole('region',{name:'Recorded Resource action'}).count(),0);
 await composer.fill('Return ordinary prose for the normalization refusal test.');await page.getByRole('button',{name:'Send',exact:true}).click();
 await page.locator('.conversation-exchange').filter({hasText:'Return ordinary prose for the normalization refusal test.'}).locator('.conversation-answer').getByText('Controlled provider response',{exact:true}).waitFor();
 await page.evaluate(()=>window.qualificationPlatform.commands.executeCommand('studio.case.refresh'));
 const currentProse=page.locator('.turn-ai').last().locator('.candidate-effect');
 await openCandidate(currentProse);await currentProse.getByRole('button',{name:'Prepare Resource action…',exact:true}).click();form=page.getByRole('dialog',{name:'Prepare Resource action',exact:true});
 await form.getByRole('button',{name:'Record proposal',exact:true}).click();await form.waitFor({state:'hidden'});
 await currentProse.getByRole('alert').waitFor();assert.equal(await currentProse.getByRole('region',{name:'Recorded Resource action'}).count(),0);
 const normalizedRefusal=exchanges.findLast(x=>x.request.operation_ref==='effect.propose');assert.equal(normalizedRefusal.result.data.posture,'normalization_refused');
 providerContent=JSON.stringify({schema:'yai.operation_proposal.filesystem_write.v1',operation:'filesystem.write',resource:'candidate-workspace',path:'allowed/explicit.txt',content:'EXACT_STUDIO_EFFECT_SENTINEL'});
 await composer.fill('Prepare the controlled file proposal for the test Resource.');await page.getByRole('button',{name:'Send',exact:true}).click();
 await page.locator('.conversation-answer').filter({hasText:'EXACT_STUDIO_EFFECT_SENTINEL'}).waitFor();
 await page.evaluate(()=>window.qualificationPlatform.commands.executeCommand('studio.case.refresh'));
 const candidateAction=page.locator('.turn-ai').filter({has:page.locator('.conversation-answer').filter({hasText:'EXACT_STUDIO_EFFECT_SENTINEL'})}).locator('.candidate-effect');
 const exactCandidateRef=await page.locator('.conversation-answer').filter({hasText:'EXACT_STUDIO_EFFECT_SENTINEL'}).getAttribute('data-result-ref');
 const beforeProposalRefusals=await accepted('case.summary',{case_ref:caseRef});
 const proposalInput={case_ref:caseRef,participant_ref:'participant:operator',candidate_ref:exactCandidateRef,resource_ref:'candidate-workspace',expected_generation:beforeProposalRefusals.case.generation};
 assert.equal((await call('effect.propose',{...proposalInput,expected_generation:0})).result_state,'stale');
 const hiddenProposal=await call('effect.propose',{...proposalInput,participant_ref:'participant:hidden'});assert.notEqual(hiddenProposal.result_state,'success');assert.equal(hiddenProposal.data,undefined);
 assert.deepEqual(await accepted('case.summary',{case_ref:caseRef}),beforeProposalRefusals,'Rejected proposals must not record an Operation');
 await openCandidate(candidateAction);await candidateAction.getByRole('button',{name:'Prepare Resource action…',exact:true}).click();form=page.getByRole('dialog',{name:'Prepare Resource action',exact:true});
 dropAcknowledgement='effect.propose';await form.getByRole('button',{name:'Record proposal',exact:true}).click();await form.getByText('Confirmation was lost',{exact:true}).waitFor();
 const firstProposal=exchanges.findLast(x=>x.request.operation_ref==='effect.propose');assert.equal(firstProposal.result.data.posture,'recorded');
 await assert.rejects(stat(path.join(effectRoot,'allowed/explicit.txt')));
 await form.getByRole('button',{name:'Close and inspect state'}).click();await candidateAction.getByRole('button',{name:'Recover exact proposal…',exact:true}).click();form=page.getByRole('dialog',{name:'Prepare Resource action',exact:true});
 await form.getByRole('button',{name:'Recover exact proposal',exact:true}).click();await form.waitFor({state:'hidden'});
 const recoveredProposal=exchanges.findLast(x=>x.request.operation_ref==='effect.propose');assert.deepEqual(recoveredProposal.request.input,firstProposal.request.input);assert.deepEqual(recoveredProposal.result.data,firstProposal.result.data);
 await candidateAction.getByRole('region',{name:'Recorded Resource action'}).waitFor();
 await candidateAction.getByText('Exact proposed content',{exact:true}).click();await candidateAction.locator('pre').getByText('EXACT_STUDIO_EFFECT_SENTINEL',{exact:true}).waitFor();
 const operationRef=firstProposal.result.data.operation.operation_id;
 const beforeSubmit=await accepted('case.summary',{case_ref:caseRef});
 const staleEffect=await call('effect.submit',{case_ref:caseRef,participant_ref:'participant:operator',operation_ref:operationRef,expected_generation:0});assert.equal(staleEffect.result_state,'stale');await assert.rejects(stat(path.join(effectRoot,'allowed/explicit.txt')));
 const hiddenEffect=await call('effect.submit',{case_ref:caseRef,participant_ref:'participant:hidden',operation_ref:operationRef,expected_generation:beforeSubmit.case.generation});assert.notEqual(hiddenEffect.result_state,'success');await assert.rejects(stat(path.join(effectRoot,'allowed/explicit.txt')));
 await candidateAction.getByRole('button',{name:'Submit recorded Operation…',exact:true}).click();form=page.getByRole('dialog',{name:'Submit recorded Operation',exact:true});
 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]){await page.setViewportSize({width,height});const titleBox=await form.getByRole('heading',{name:'Submit recorded Operation'}).boundingBox();assert.ok(titleBox && titleBox.height<70,'Dialog title must not inherit narrow chat header geometry');const footerBox=await form.locator('footer').boundingBox();assert.ok(footerBox && footerBox.y+footerBox.height<=height,'Action controls must remain reachable');await page.screenshot({path:`${evidence}/effect-submit-${width}x${height}.png`});}
 dropAcknowledgement='effect.submit';await form.getByRole('button',{name:'Submit exact Operation',exact:true}).click();await form.getByText('Confirmation was lost',{exact:true}).waitFor();
 const submittedEffect=exchanges.findLast(x=>x.request.operation_ref==='effect.submit');assert.equal(submittedEffect.result.result_state,'success');
 assert.equal(submittedEffect.result.data.progress.outcome,'applied');assert.equal(await readFile(path.join(effectRoot,'allowed/explicit.txt'),'utf8'),'EXACT_STUDIO_EFFECT_SENTINEL');
 const inode=(await stat(path.join(effectRoot,'allowed/explicit.txt'))).ino;
 await form.getByRole('button',{name:'Close and inspect state'}).click();await candidateAction.getByRole('button',{name:'Refresh observation',exact:true}).click();await candidateAction.getByText('applied',{exact:true}).waitFor();
 const afterEffect=await accepted('case.summary',{case_ref:caseRef});const providerAfterEffect=generationRequests;
 assert.deepEqual(await accepted('effect.submit',submittedEffect.request.input),submittedEffect.result.data);
 assert.equal((await stat(path.join(effectRoot,'allowed/explicit.txt'))).ino,inode);assert.deepEqual(await accepted('case.summary',{case_ref:caseRef}),afterEffect);assert.equal(generationRequests,providerAfterEffect);
 effectChild=spawn('/usr/bin/sleep',['600'],{stdio:'ignore'});await new Promise((resolve,reject)=>{effectChild.once('spawn',resolve);effectChild.once('error',reject);});
 await accepted('resource.attach_process',{case_ref:caseRef,attachment_ref:'candidate-process',pid:effectChild.pid,policy_owner_participant_ref:'participant:operator',actions:['suspend'],review_requirement:'automatic'});
 await page.evaluate(()=>window.qualificationPlatform.commands.executeCommand('studio.case.refresh'));
 providerContent=JSON.stringify({schema:'yai.operation_proposal.process_signal.v1',operation:'process.signal',resource:'candidate-process',action:'suspend'});
 await composer.fill('Prepare the explicit suspend proposal for the disposable bound process.');await page.getByRole('button',{name:'Send',exact:true}).click();
 const processCandidate=page.locator('.turn-ai').filter({has:page.locator('.conversation-answer').filter({hasText:'yai.operation_proposal.process_signal.v1'})}).locator('.candidate-effect');
 await processCandidate.waitFor({state:'attached'});await page.evaluate(()=>window.qualificationPlatform.commands.executeCommand('studio.case.refresh'));
 await openCandidate(processCandidate);await processCandidate.getByRole('button',{name:'Prepare Resource action…',exact:true}).click();form=page.getByRole('dialog',{name:'Prepare Resource action',exact:true});await form.getByLabel('Bound Resource').selectOption('candidate-process');await form.getByRole('button',{name:'Record proposal',exact:true}).click();await form.waitFor({state:'hidden'});
 await processCandidate.getByText('Recorded process action',{exact:true}).waitFor();
 const processStatus=async()=>String(await readFile(`/proc/${effectChild.pid}/status`,'utf8')).match(/^State:\s+(\w)/m)[1];
 assert.notEqual(await processStatus(),'T','Proposal must not signal the child');
 await processCandidate.getByRole('button',{name:'Submit recorded Operation…',exact:true}).click();form=page.getByRole('dialog',{name:'Submit recorded Operation',exact:true});dropAcknowledgement='effect.submit';await form.getByRole('button',{name:'Submit exact Operation',exact:true}).click();await form.getByText('Confirmation was lost',{exact:true}).waitFor();
 const processSubmission=exchanges.findLast(x=>x.request.operation_ref==='effect.submit');assert.equal(processSubmission.result.result_state,'success',JSON.stringify(processSubmission));assert.equal(processSubmission.result.data.progress.outcome,'applied');
 const stopDeadline=Date.now()+2000;while(await processStatus()!=='T'){assert.ok(Date.now()<stopDeadline);await new Promise(resolve=>setTimeout(resolve,10));}
 await form.getByRole('button',{name:'Close and inspect state'}).click();effectChild.kill('SIGCONT');
 const continueDeadline=Date.now()+2000;while(await processStatus()==='T'){assert.ok(Date.now()<continueDeadline);await new Promise(resolve=>setTimeout(resolve,10));}
 assert.deepEqual(await accepted('effect.submit',processSubmission.request.input),processSubmission.result.data);await new Promise(resolve=>setTimeout(resolve,100));assert.notEqual(await processStatus(),'T','Exact retry must not repeat process suspension');
 await accepted('participant.role.add',{case_ref:caseRef,participant_ref:'participant:reviewer',role:'operation-reviewer'});
 const reviewPolicy={...effectPolicy,policy_key:'studio.candidate.review',rules:[...effectPolicy.rules,{kind:'review_requirement',rule_id:'review-file-write',operation_kind:'filesystem.write',resource_kind:'filesystem',required:true,reason:'Explicit human review before write'},{kind:'authority_requirement',rule_id:'reviewer-role',operation_kind:'filesystem.write',resource_kind:'filesystem',subject:'reviewer',required_role:'operation-reviewer',reason:'Qualified reviewer required'}]};
 const reviewArtifact=(await accepted('policy.ingest',{tenant_id:'tenant:studio-ui',source_bytes:[...Buffer.from(JSON.stringify(reviewPolicy))]})).view.artifact.artifact_id;
 for(const operation of ['policy.validate','policy.publish'])await accepted(operation,{artifact_ref:reviewArtifact,reason:'Review gate test'});
 await accepted('policy.case.bind',{case_ref:caseRef,artifact_ref:reviewArtifact,expected_generation:(await accepted('case.summary',{case_ref:caseRef})).case.generation,reason:'Require human review'});
 await page.evaluate(()=>window.qualificationPlatform.commands.executeCommand('studio.case.refresh'));
 providerContent=JSON.stringify({schema:'yai.operation_proposal.filesystem_write.v1',operation:'filesystem.write',resource:'candidate-workspace',path:'allowed/review.txt',content:'REVIEW_REQUIRED_SENTINEL'});
 await composer.fill('Prepare the file proposal that must wait for Review.');await page.getByRole('button',{name:'Send',exact:true}).click();
 await page.locator('.conversation-answer').filter({hasText:'REVIEW_REQUIRED_SENTINEL'}).waitFor();await page.evaluate(()=>window.qualificationPlatform.commands.executeCommand('studio.case.refresh'));
 const reviewedCandidate=page.locator('.turn-ai').filter({has:page.locator('.conversation-answer').filter({hasText:'REVIEW_REQUIRED_SENTINEL'})}).locator('.candidate-effect');
 await openCandidate(reviewedCandidate);await reviewedCandidate.getByRole('button',{name:'Prepare Resource action…',exact:true}).click();form=page.getByRole('dialog',{name:'Prepare Resource action',exact:true});await form.getByLabel('Bound Resource').selectOption('candidate-workspace');await form.getByRole('button',{name:'Record proposal',exact:true}).click();await form.waitFor({state:'hidden'});
 await reviewedCandidate.getByRole('button',{name:'Submit recorded Operation…',exact:true}).click();form=page.getByRole('dialog',{name:'Submit recorded Operation',exact:true});await form.getByRole('button',{name:'Submit exact Operation',exact:true}).click();await form.waitFor({state:'hidden'});
 const waitingReview=exchanges.findLast(x=>x.request.operation_ref==='effect.submit');const reviewDecision=await accepted('decision.trajectory.inspect',{case_ref:caseRef,participant_ref:'participant:operator',decision_ref:waitingReview.result.data.progress.decision_id});assert.equal(waitingReview.result.data.progress.status,'awaiting_review',JSON.stringify(reviewDecision.decision));assert.ok(waitingReview.result.data.progress.review_id);await assert.rejects(stat(path.join(effectRoot,'allowed/review.txt')));
 await reviewedCandidate.getByText('Waiting for Review. This observation grants no permission to execute.',{exact:true}).waitFor();
 providerContent='Controlled provider response';
 await accepted('provider.trust.set',{target_ref:target.target_id,posture:'denied'});
 const rejected=await call('cognitive.binding.set',{case_ref:caseRef,participant_ref:'participant:operator',role:'primary',capability:'primary_conversation',candidates:[{target_ref:target.target_id,semantic_evidence_ref:attested.data.evidence_id}],replace:true});
 assert.notEqual(rejected.result_state,'success');
 await page.evaluate(()=>window.qualificationPlatform.commands.executeCommand('studio.case.refresh'));
 await page.locator('.live-rail button[aria-label="Compute"]').click();
 await cognition.getByLabel('Committed Turn',{exact:true}).selectOption(sourceTurn);
 await cognition.getByRole('button',{name:'Prepare execution plan',exact:true}).click();
 await cognition.getByText('unresolved',{exact:true}).waitFor();
 assert.equal(await cognition.getByRole('button',{name:'Execute prepared plan…',exact:true}).isDisabled(),true);
 assert.match(await cognition.getByRole('region',{name:'Prepared execution plan'}).innerText(),/trust not approved/);
 await page.locator('.live-rail button[aria-label="Overview"]').click();
 const beforeRefusal=generationRequests;
 await narrative.getByRole('button',{name:'Generate a new explanation',exact:true}).click();
 await page.waitForFunction(()=>{const n=document.querySelector('.overview-narrative');return n && (n.querySelector('[role="alert"]') || n.querySelector('.narrative-text')?.textContent.includes('No completed response'));});
 assert.equal(generationRequests,beforeRefusal,'Revoked trust must not dispatch narrative inference');
 assert.equal(await narrative.locator('.narrative-text').filter({hasText:'Controlled provider response'}).count(),0);

 const hidden=await call('execution.get',{case_ref:caseRef,participant_ref:'participant:hidden',execution:{domain:'conversation',submission_ref:sent.request.input.submission_ref}});assert.notEqual(hidden.result_state,'success');
 assert.equal(cli('case','verify',caseRef).status,'ok');assert.deepEqual(errors,[]);
 console.log(JSON.stringify({result:'PASS',case_ref:caseRef,provider_dispatches:generationRequests-baselineRequests,proof:['typed model discovery','authored operator attestation and primary binding','two exact committed user Turns and recorded model results','lost acknowledgement recovery and duplicate dispatch refusal','stale generation preserves draft without Turn','canonical projection restores responses','HTTP 413 is explained without fake response or redispatch','hidden execution refusal','revoked trust refusal','four viewport matrix','CLI replay','Overview narrative uses governed SEND and survives acknowledgement loss','Narrative reopen and exact retry do not redispatch','Narrative trust refusal','Telemetry shows actual Host PID','Retained Turn plan preparation without inference','Explicit exact-plan realization and lost ACK recovery','Stale plan and incompatible input refusal','Current trust excludes model route','Reopen observes without redispatch','Explicit candidate normalization, prose refusal, lost-ACK exact proposal recovery, governed file write, stale/hidden refusal, exact retry preserves inode and Case','Process suspension exact retry does not repeat the signal; required Review prevents file write']}));
} finally {
 if(effectChild && effectChild.exitCode===null){effectChild.kill('SIGCONT');effectChild.kill('SIGTERM');}
 releaseSummary?.();releaseResponse?.();await writeFile(`${evidence}/exchanges.json`,JSON.stringify(exchanges,null,2));await browser?.close();await new Promise(resolve=>provider?.close(resolve)??resolve());try{if(telemetry)cli('host','stop');}finally{await rm(home,{recursive:true,force:true});}}
