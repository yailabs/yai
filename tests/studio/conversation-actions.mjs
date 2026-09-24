// UI calls the real published Host over Unix IPC through a test-only Tauri bridge.
// Only a freshly created temporary YAI_HOME is mutated; never operator state.
import { createRequire } from 'node:module';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
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
let telemetry, serial=0, browser, provider, dropAcknowledgement;
let generationRequests = 0;
let rejectOversized = false;
const exchanges = [];
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
 const caseRef='case:studio-policy-actions';
 await accepted('case.create',{tenant_id:'tenant:studio-ui',case_ref:caseRef});await accepted('participant.role.add',{case_ref:caseRef,participant_ref:'participant:operator',role:'operator'});await accepted('participant.principal.link',{case_ref:caseRef,participant_ref:'participant:operator',principal_ref:'self'});
 provider=createServer(async(req,res)=>{let body='';for await(const chunk of req)body+=chunk;res.setHeader('Content-Type','application/json');if(req.url==='/v1/models')res.end(JSON.stringify({data:[{id:'controlled-text-model'}]}));else if(req.url==='/v1/chat/completions'){const value=JSON.parse(body);generationRequests++;if(rejectOversized){res.statusCode=413;res.end(JSON.stringify({error:{code:'input_too_large'}}));return;}res.end(JSON.stringify({id:'controlled-response',model:value.model,choices:[{message:{role:'assistant',content:'Controlled provider response'}}]}));}else{res.statusCode=404;res.end('{}');}});
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
  const container=document.createElement('div');document.body.appendChild(container);
  ReactDOM.createRoot(container).render(React.createElement(StudioApplication,{dataSource:new LiveDataSource(client),platform,registry}));
  window.qualificationPlatform=platform;
 },telemetry);
 await page.getByRole('button',{name:/Studio Policy Actions/}).click();await page.locator('.workbench-kernel').waitFor();await page.keyboard.press('Control+j');await page.locator('.live-rail button[aria-label="Compute"]').click();


 await page.getByRole('button',{name:'Register provider target',exact:true}).click();
 let form=page.getByRole('dialog',{name:'Register provider target'});
 await form.getByLabel('Provider / runtime label').fill('controlled-provider');await form.getByLabel('Endpoint',{exact:true}).fill(endpoint);await form.getByRole('button',{name:'Discover exposed models',exact:true}).click();await form.getByText('1 exposed model(s). Discovery does not qualify inference.').waitFor();assert.deepEqual(exchanges.findLast(x=>x.request.operation_ref==='provider.models').result.data.models,['controlled-text-model']);await form.getByLabel('Exact model identity').fill('controlled-text-model');
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
 await page.getByRole('button',{name:'Assign conversation model',exact:true}).click();form=page.getByRole('dialog',{name:'Assign conversation model'});
 await form.getByRole('button',{name:'Assign model',exact:true}).click();await form.waitFor({state:'hidden'});
 let snapshot=await accepted('case.summary',{case_ref:caseRef});assert.equal(snapshot.compute.cognitive_bindings[0].target_id,target.target_id);
 assert.equal(snapshot.compute.targets[0].semantic_evidence[0].evidence_id,attested.data.evidence_id);
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
 await composer.fill('First exact Studio message');await page.getByRole('button',{name:'Send',exact:true}).click();
 await page.locator('.turn-ai').getByText('Controlled provider response',{exact:true}).waitFor();
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
 await page.waitForFunction(()=>document.querySelectorAll('.turn-ai > p').length===2);
 await page.waitForFunction(()=>[...document.querySelectorAll('.turn-ai > p')].every(p=>p.textContent==='Controlled provider response'));
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
 await page.waitForFunction(()=>document.querySelectorAll('.turn-ai > p').length===2);
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
 assert.equal(await page.locator('.turn-ai > p').filter({hasText:'Controlled provider response'}).count(),2);
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
 assert.ok((await page.locator('#telemetry-host').innerText()).includes(String(telemetry.pid)));
 const observedSnapshot=await accepted('case.summary',{case_ref:caseRef});
 const providerHealth=observedSnapshot.compute.targets[0].posture.health;
 assert.ok((await page.locator('#telemetry-endpoints').innerText()).includes(providerHealth.circuit));
 assert.ok((await page.locator('#telemetry-endpoints').innerText()).includes(String(providerHealth.consecutive_failures)));
 assert.equal(await page.locator('#telemetry-endpoints').getByText(providerHealth.observed_at_unix_ms ? `Observed: ${providerHealth.posture}` : 'Live health unknown',{exact:true}).count(),1);
 await page.locator('.live-rail button[aria-label="Overview"]').click();
 await narrative.locator('.narrative-text').getByText('Controlled provider response',{exact:true}).waitFor();
 // Revoked current trust refuses a new model assignment (no bypass in React).
 await accepted('provider.trust.set',{target_ref:target.target_id,posture:'denied'});
 const rejected=await call('cognitive.binding.set',{case_ref:caseRef,participant_ref:'participant:operator',role:'primary',capability:'primary_conversation',candidates:[{target_ref:target.target_id,semantic_evidence_ref:attested.data.evidence_id}],replace:true});
 assert.notEqual(rejected.result_state,'success');
 const beforeRefusal=generationRequests;
 await narrative.getByRole('button',{name:'Generate a new explanation',exact:true}).click();
 await page.waitForFunction(()=>{const n=document.querySelector('.overview-narrative');return n && (n.querySelector('[role="alert"]') || n.querySelector('.narrative-text')?.textContent.includes('No completed response'));});
 assert.equal(generationRequests,beforeRefusal,'Revoked trust must not dispatch narrative inference');
 assert.equal(await narrative.locator('.narrative-text').filter({hasText:'Controlled provider response'}).count(),0);

 const hidden=await call('execution.get',{case_ref:caseRef,participant_ref:'participant:hidden',execution:{domain:'conversation',submission_ref:sent.request.input.submission_ref}});assert.notEqual(hidden.result_state,'success');
 assert.equal(cli('case','verify',caseRef).status,'ok');assert.deepEqual(errors,[]);
 console.log(JSON.stringify({result:'PASS',case_ref:caseRef,provider_dispatches:generationRequests-baselineRequests,proof:['typed model discovery','authored operator attestation and primary binding','two exact committed user Turns and recorded model results','lost acknowledgement recovery and duplicate dispatch refusal','stale generation preserves draft without Turn','canonical projection restores responses','HTTP 413 is explained without fake response or redispatch','hidden execution refusal','revoked trust refusal','four viewport matrix','CLI replay','Overview narrative uses governed SEND and survives acknowledgement loss','Narrative reopen and exact retry do not redispatch','Narrative trust refusal','Telemetry shows actual Host PID']}));
} finally {await writeFile(`${evidence}/exchanges.json`,JSON.stringify(exchanges,null,2));await browser?.close();await new Promise(resolve=>provider?.close(resolve)??resolve());try{if(telemetry)cli('host','stop');}finally{await rm(home,{recursive:true,force:true});}}
