// UI calls the real published Host over Unix IPC through a test-only Tauri bridge.
// Only a freshly created temporary YAI_HOME is mutated; never operator state.
import { createRequire } from 'node:module';
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import net from 'node:net';
import http from 'node:http';
import { createHash } from 'node:crypto';
import os from 'node:os';
import path from 'node:path';
const require = createRequire(path.resolve(import.meta.dirname, '../../studio/package.json'));
const { chromium } = require('playwright-core');
const binary = process.env.YAI_STUDIO_TEST_BINARY;
assert.ok(binary, 'YAI_STUDIO_TEST_BINARY must name a built published CLI/Host');
const home = await mkdtemp(path.join(os.tmpdir(), 'yai-studio-execution-'));
const evidence = process.env.STUDIO_EVIDENCE_DIR ?? '/tmp/yai-studio-execution';
await mkdir(evidence, {recursive:true});
const cli = (...args) => JSON.parse(execFileSync(binary, [...args, '--json'], {env:{...process.env, YAI_HOME:home}, encoding:'utf8', timeout:30000}));
let telemetry, serial=0, browser, dropAcknowledgement, abandonResponse;
let sourceServer, serveSource = false; const sourceRequests = [];
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
    if(!handshaken) { if(message.kind!=='handshake') {socket.destroy();reject(new Error(JSON.stringify(message)));return;} handshaken=true;socket.write(JSON.stringify({kind:'application_request',request})+'\n', () => { if(abandonResponse===request.operation_ref){abandonResponse=undefined;exchanges.push({order:exchanges.length+1,request,action:'close_without_reading_response'});socket.end();reject(new Error('Injected connection loss while real Source request is in flight'));} }); }
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
 const root=path.join(home,'documents');await mkdir(root);await writeFile(path.join(root,'document.md'),'# Source qualification\nActual governed documentary content.\n');
 const resource={schema:'yai.resource_definition.v1',attachment_id:'resource:documents',policy_owner:'participant:operator',participant_ids:['participant:operator'],operations:['discover','admit_content','content_read'],read_prefixes:['policy.json','document.md'],names:[],max_output_bytes:8192,max_items:4,address:{kind:'discovery',root}};
 const perimeter={schema:'yai.source_perimeter.v1',name:'source-policy',participant:'participant:operator',resources:[resource],sources:[{name:'policy-candidate',resource:resource.attachment_id,roles:['policy'],action:{action:'discover',path:'policy.json'},media_type:'application/json',bootstrap_policy:true},{name:'document',resource:resource.attachment_id,roles:['knowledge'],action:{action:'discover',path:'document.md'},media_type:'text/markdown',bootstrap_policy:false}]};
 const file=path.join(home,'perimeter.json');await writeFile(file,JSON.stringify(perimeter));cli('case','sources','declare',caseRef,'--file',file);
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
 await page.getByRole('button',{name:/Studio Policy Actions/}).click();await page.locator('.workbench-kernel').waitFor();await page.keyboard.press('Control+j');await page.locator('.live-rail button[aria-label="Authority"]').click();


 const before=await accepted('case.summary',{case_ref:caseRef});
 const policy=before.environment.sources.find(item=>item.label==='policy-candidate');
 await page.locator('.live-sidebar').getByRole('button',{name:/policy-candidate/}).click();
 await page.getByRole('button',{name:'Acquire Source…',exact:true}).click();
 let form=page.getByRole('dialog',{name:'Acquire Source'});await form.getByRole('button',{name:'Acquire exact Source'}).click();await form.waitFor({state:'hidden'});
 let acquired=exchanges.findLast(item=>item.request.operation_ref==='source.acquire');assert.equal(acquired.result.result_state,'success',JSON.stringify(acquired));
 assert.equal(acquired.result.data.execution.attempt,1);assert.notEqual(acquired.result.data.execution.phase,'acquired');
 let summary=await accepted('case.summary',{case_ref:caseRef});let progress=summary.environment.sources.find(item=>item.id===policy.id);assert.equal(progress.attempt,1);assert.ok(progress.progress_ref);
 const sourceReceipt=page.locator('.source-acquisition .execution-receipt');
 await sourceReceipt.getByText(acquired.result.data.execution.posture.replaceAll('_',' '),{exact:true}).waitFor();
 assert.ok((await sourceReceipt.textContent()).includes(policy.id));
 const sourceSubmissions=exchanges.filter(item=>['source.acquire','source.resume'].includes(item.request.operation_ref)).length;
 await sourceReceipt.getByRole('button',{name:'Refresh observation',exact:true}).click();
 await sourceReceipt.getByRole('button',{name:'Refresh observation',exact:true}).waitFor();
 assert.equal(exchanges.filter(item=>['source.acquire','source.resume'].includes(item.request.operation_ref)).length,sourceSubmissions);
 assert.equal((await accepted('case.summary',{case_ref:caseRef})).case.generation,summary.case.generation,'Source observation must not advance Case history');
 await writeFile(path.join(root,'policy.json'),await readFile(path.resolve(import.meta.dirname,'../qualification/studio-product-vertical/policy.json')));
 await page.getByRole('button',{name:'Resume acquisition…',exact:true}).click();form=page.getByRole('dialog',{name:'Resume acquisition'});await form.getByRole('button',{name:'Resume exact attempt'}).click();await form.waitFor({state:'hidden'});
 const resumed=exchanges.findLast(item=>item.request.operation_ref==='source.resume');assert.equal(resumed.result.result_state,'success',JSON.stringify(resumed));assert.equal(resumed.request.input.previous_progress_ref,progress.progress_ref);assert.equal(resumed.result.data.execution.attempt,1);assert.equal(resumed.result.data.execution.phase,'acquired');
 await sourceReceipt.getByText('acquired',{exact:true}).first().waitFor();
 await page.getByRole('button',{name:'Publish Source policy…',exact:true}).click();form=page.getByRole('dialog',{name:'Publish Source policy'});await form.getByLabel('Review reason').fill('Inspect bounded source policy');await form.getByRole('button',{name:'Publish and bind'}).click();await form.waitFor({state:'hidden'});
 await page.locator('.live-rail button[aria-label="Environment"]').click();await page.locator('.live-sidebar').getByRole('button',{name:'document',exact:true}).click();
 await page.getByRole('button',{name:'Acquire Source…',exact:true}).click();form=page.getByRole('dialog',{name:'Acquire Source'});dropAcknowledgement='source.acquire';await form.getByRole('button',{name:'Acquire exact Source'}).click();await form.getByText('Confirmation was lost',{exact:true}).waitFor();assert.equal(await form.getByRole('button',{name:'Acquire exact Source'}).isEnabled(),false);await form.getByRole('button',{name:'Close and inspect state'}).click();
 acquired=exchanges.findLast(item=>item.request.operation_ref==='source.acquire');assert.equal(acquired.result.data.execution.phase,'acquired');
 const ordinary=acquired.request.input.source_ref;const stable=(await accepted('case.summary',{case_ref:caseRef})).case.generation;
 assert.equal((await accepted('source.acquire',acquired.request.input)).created,false);assert.equal((await accepted('case.summary',{case_ref:caseRef})).case.generation,stable);
 const observation={case_ref:caseRef,participant_ref:'participant:operator',execution:{domain:'source_acquisition',source_ref:ordinary,attempt:1}};
 assert.equal((await accepted('execution.get',observation)).phase,'acquired');
 await sourceReceipt.getByText('acquired',{exact:true}).first().waitFor();
 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]) {
  await page.setViewportSize({width,height});
  await sourceReceipt.scrollIntoViewIfNeeded();
  await page.screenshot({path:`${evidence}/source-observation-${width}x${height}.png`});
 }
 await page.setViewportSize({width:1440,height:900});
 const hidden=await call('execution.get',{...observation,participant_ref:'participant:hidden'});assert.equal(hidden.result_state,'unauthorized');assert.equal(hidden.data,undefined);
 const stale=await call('source.acquire',{...acquired.request.input,attempt:2,expected_generation:0});assert.notEqual(stale.result_state,'success');assert.equal((await accepted('case.summary',{case_ref:caseRef})).case.generation,stable);
 await page.getByRole('button',{name:'Refresh Case',exact:true}).click();await page.locator('.live-rail button[aria-label="Work"]').click();await page.getByRole('tab',{name:'Executions',exact:true}).click();
 await page.locator('.work-surface .execution-receipt').filter({hasText:ordinary}).waitFor();
 const submissions=exchanges.filter(item=>item.request.operation_ref==='source.acquire').length;
 telemetry=cli('host','restart').data.value;await page.getByRole('button',{name:'Refresh Case',exact:true}).click();await page.locator('.work-surface .execution-receipt').filter({hasText:ordinary}).getByRole('button',{name:'Refresh observation'}).click();
 assert.equal(exchanges.filter(item=>item.request.operation_ref==='source.acquire').length,submissions,'Restart observation must never redispatch');
 await page.locator('.live-rail button[aria-label="Knowledge"]').click();
 const queries=page.locator('.knowledge-queries');await queries.getByRole('searchbox').fill('qualification');await queries.getByRole('button',{name:'Search Knowledge',exact:true}).click();await queries.locator('.fact-row').first().waitFor();
 const search=exchanges.findLast(item=>item.request.operation_ref==='knowledge.search');assert.equal(search.result.result_state,'success',JSON.stringify(search));assert.ok(search.result.data.hits.length);
 await queries.locator('.fact-row').first().click();await queries.locator('.knowledge-resolved').waitFor();const resolved=exchanges.findLast(item=>item.request.operation_ref==='knowledge.resolve');assert.equal(resolved.result.result_state,'success',JSON.stringify(resolved));assert.ok((await queries.locator('.knowledge-resolved').textContent()).includes(resolved.result.data.unit.text));
 await queries.getByRole('button',{name:'Inspect current Knowledge'}).click();for(let attempt=0;attempt<100 && !exchanges.some(item=>item.request.operation_ref==='knowledge.inspect');attempt++) await new Promise(resolve=>setTimeout(resolve,25));assert.equal(exchanges.findLast(item=>item.request.operation_ref==='knowledge.inspect').result.result_state,'success');
 await queries.getByRole('button',{name:'Build documentary navigation'}).click();await queries.getByText('Navigation returned by YAI',{exact:true}).waitFor();assert.equal(exchanges.findLast(item=>item.request.operation_ref==='knowledge.navigation').result.result_state,'success');
 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]){await page.setViewportSize({width,height});await page.screenshot({path:`${evidence}/knowledge-${width}x${height}.png`});}
 await accepted('source.revoke',{case_ref:caseRef,source_ref:ordinary,reason:'Current backing refusal qualification'});
 assert.notEqual((await call('knowledge.resolve',resolved.request.input)).result_state,'success');
 await page.getByRole('button',{name:'Refresh Case',exact:true}).click();await queries.locator('.knowledge-resolved').waitFor({state:'hidden'});
 // A real HTTP acquisition remains blocked in its producer while the UI loses
 // its acknowledgement. Kill only the disposable Host after observing its lease.
 sourceServer=http.createServer((request,response)=>{sourceRequests.push({path:request.url});response.on('error',()=>{});if(serveSource){const bytes=Buffer.from('{"qualification":"reviewed-source-exact"}');response.writeHead(200,{'Content-Type':'application/json','Content-Length':bytes.length});response.end(bytes);}});
 await new Promise(resolve=>sourceServer.listen(0,'127.0.0.1',resolve));
 const binding={schema:'yai.local_resource_access_binding.v1',case_id:caseRef,attachment_id:'resource:interrupted-http',address:{kind:'http_service',endpoint:{endpoint:`http://127.0.0.1:${sourceServer.address().port}`,allowed_ip_addresses:['127.0.0.1'],credential_ref:null},paths:{document:'document'}}};
 await accepted('resource.attach',{binding,access:{schema:'yai.resource_access.v1',configuration_digest:'sha256:'+createHash('sha256').update(JSON.stringify(binding)).digest('hex'),participant_ids:['participant:operator'],operations:['http_fetch'],read_prefixes:[],names:['document'],max_output_bytes:4096,max_items:8},policy_owner_participant_ref:'participant:operator',review_requirement:'automatic'});
 const httpPolicy={schema:'yai.policy_source_input.v4',policy_key:'source-crash',source_version:'1',owner_ref:'organization:yailabs',source_origin:{source_system:'qualification',source_uri:'test://source-crash'},validity:{mode:'unbounded'},rules:[{kind:'operation_restriction',rule_id:'fetch',operation_kind:'http.fetch',resource_kind:'http_service',effect:'allow',reason:'One bounded local source'}]};
 const artifact=(await accepted('policy.ingest',{tenant_id:'tenant:studio-ui',source_bytes:[...Buffer.from(JSON.stringify(httpPolicy))]})).view.artifact.artifact_id;
 for(const operation of ['policy.validate','policy.publish'])await accepted(operation,{artifact_ref:artifact,reason:'Interrupted Source UI qualification'});
 await accepted('policy.case.bind',{case_ref:caseRef,artifact_ref:artifact,expected_generation:(await accepted('case.summary',{case_ref:caseRef})).case.generation,reason:'Interrupted Source UI qualification'});
 await accepted('source.declare',{case_ref:caseRef,participant_ref:'participant:operator',perimeter:'qualification',logical_name:'interrupted-http',resource_ref:binding.attachment_id,roles:['knowledge'],action:{action:'http_fetch',name:'document'},bootstrap_policy:false,media_type:'application/json'});
 await page.getByRole('button',{name:'Refresh Case',exact:true}).click();
 await page.locator('.live-rail button[aria-label="Environment"]').click();
 await page.locator('.live-sidebar').getByRole('button',{name:'interrupted-http',exact:true}).click();
 await page.getByRole('button',{name:'Acquire Source…',exact:true}).click();
 form=page.getByRole('dialog',{name:'Acquire Source'});abandonResponse='source.acquire';
 await form.getByRole('button',{name:'Acquire exact Source'}).click();
 await form.getByText('Confirmation was lost',{exact:true}).waitFor();
 assert.equal(await form.getByRole('button',{name:'Acquire exact Source'}).isEnabled(),false);
 await form.getByRole('button',{name:'Close and inspect state'}).click();
 for(let attempt=0;attempt<100 && sourceRequests.length===0;attempt++)await new Promise(resolve=>setTimeout(resolve,25));
 assert.equal(sourceRequests.length,1);
 const interrupted=exchanges.findLast(item=>item.action==='close_without_reading_response').request.input;
 const observeInterrupted={case_ref:caseRef,participant_ref:'participant:operator',execution:{domain:'source_acquisition',source_ref:interrupted.source_ref,attempt:1}};
 const active=await accepted('execution.get',observeInterrupted);assert.equal(active.posture,'running');
 await page.getByRole('button',{name:'Refresh Case',exact:true}).click();
 await sourceReceipt.getByText('running',{exact:true}).waitFor();
 assert.equal(await page.getByRole('button',{name:'Acquire Source…',exact:true}).isEnabled(),false);
 assert.equal(await page.getByRole('button',{name:'Resume acquisition…',exact:true}).isEnabled(),false);
 assert.equal(path.resolve(telemetry.yai_home),path.resolve(home),'Never kill an operator Host');
 exchanges.push({order:exchanges.length+1,action:'kill_disposable_host_in_flight',pid:telemetry.pid,process_identity:telemetry.process_identity,execution:active});
 process.kill(telemetry.pid,'SIGKILL');
 sourceServer.closeAllConnections();
 telemetry=cli('host','start').data.value;
 await page.getByRole('button',{name:'Refresh Case',exact:true}).click();
 await sourceReceipt.getByRole('button',{name:'Refresh observation',exact:true}).click();
 await sourceReceipt.getByText('delivery indeterminate',{exact:true}).waitFor();
 await sourceReceipt.getByText(/no confirmed result.*no longer observed as active/).waitFor();
 const recovered=await accepted('execution.get',observeInterrupted);
 assert.deepEqual(recovered,{...active,posture:'delivery_indeterminate'});
 const generationAfterCrash=(await accepted('case.summary',{case_ref:caseRef})).case.generation;
 assert.equal(await page.getByRole('button',{name:'Acquire Source…',exact:true}).isEnabled(),false);
 assert.equal(await page.getByRole('button',{name:'Resume acquisition…',exact:true}).isEnabled(),false);
 const beforeObservation=exchanges.filter(item=>['source.acquire','source.resume'].includes(item.request?.operation_ref)).length;
 await sourceReceipt.getByRole('button',{name:'Refresh observation',exact:true}).click();
 await sourceReceipt.getByRole('button',{name:'Refresh observation',exact:true}).waitFor();
 assert.equal(exchanges.filter(item=>['source.acquire','source.resume'].includes(item.request?.operation_ref)).length,beforeObservation);
 const repeated=await accepted('source.acquire',interrupted);assert.equal(repeated.created,false);assert.deepEqual(repeated.execution,recovered);
 assert.notEqual((await call('source.resume',{...interrupted,expected_generation:generationAfterCrash,previous_progress_ref:active.progress_ref})).result_state,'success');
 assert.notEqual((await call('source.acquire',{...interrupted,attempt:2,expected_generation:generationAfterCrash})).result_state,'success');
 const hiddenInterrupted=await call('execution.get',{...observeInterrupted,participant_ref:'participant:hidden'});assert.equal(hiddenInterrupted.result_state,'unauthorized');assert.equal(hiddenInterrupted.data,undefined);
 assert.equal((await accepted('case.summary',{case_ref:caseRef})).case.generation,generationAfterCrash);
 assert.equal(sourceRequests.length,1,'Restart, observation and explicit retry cannot redispatch uncertain acquisition');
 await page.screenshot({path:`${evidence}/source-carrier-lost.png`});
 // A settled Review differs from unknown delivery: only the former can resume.
 const reviewPolicy={...httpPolicy,policy_key:'source-review',rules:[...httpPolicy.rules,
  {kind:'review_requirement',rule_id:'review',operation_kind:'http.fetch',resource_kind:'http_service',required:true,reason:'Review before acquisition'},
  {kind:'authority_requirement',rule_id:'reviewer',operation_kind:'http.fetch',resource_kind:'http_service',subject:'reviewer',required_role:'operator',reason:'Exact eligible reviewer'}]};
 const reviewedArtifact=(await accepted('policy.ingest',{tenant_id:'tenant:studio-ui',source_bytes:[...Buffer.from(JSON.stringify(reviewPolicy))]})).view.artifact.artifact_id;
 for(const operation of ['policy.validate','policy.publish'])await accepted(operation,{artifact_ref:reviewedArtifact,reason:'Source Review continuation qualification'});
 await accepted('policy.case.bind',{case_ref:caseRef,artifact_ref:reviewedArtifact,expected_generation:(await accepted('case.summary',{case_ref:caseRef})).case.generation,reason:'Review before explicit continuation'});
 await accepted('source.declare',{case_ref:caseRef,participant_ref:'participant:operator',perimeter:'qualification',logical_name:'reviewed-http',resource_ref:binding.attachment_id,roles:['knowledge'],action:{action:'http_fetch',name:'document'},bootstrap_policy:false,media_type:'application/json'});
 await page.getByRole('button',{name:'Refresh Case',exact:true}).click();
 await page.locator('.live-sidebar').getByRole('button',{name:'reviewed-http',exact:true}).click();
 await page.getByRole('button',{name:'Acquire Source…',exact:true}).click();form=page.getByRole('dialog',{name:'Acquire Source'});
 await form.getByRole('button',{name:'Acquire exact Source'}).click();await form.waitFor({state:'hidden'});
 const waiting=exchanges.findLast(x=>x.request?.operation_ref==='source.acquire');assert.equal(waiting.result.result_state,'success');assert.equal(waiting.result.data.execution.posture,'waiting_for_review');
 await sourceReceipt.getByText('waiting for review',{exact:true}).waitFor();
 assert.equal(sourceRequests.length,1,'Review-required acquisition must not fetch');
 const pendingSummary=await accepted('case.summary',{case_ref:caseRef});
 const reviewRef=pendingSummary.authority.reviews.find(review=>review.status==='pending').id;
 const hiddenApproval=await call('review.approve',{case_ref:caseRef,participant_ref:'participant:hidden',review_ref:reviewRef,reason:'Must refuse'});assert.equal(hiddenApproval.result_state,'unauthorized');assert.equal(sourceRequests.length,1);
 await page.locator('.live-rail button[aria-label="Authority"]').click();
 await page.locator('.collection-row').filter({hasText:reviewRef}).click();
 await page.getByRole('button',{name:'Approve review',exact:true}).click();
 await page.getByRole('dialog').getByLabel('Reason').fill('Read the exact bounded HTTP source.');
 await page.getByRole('button',{name:'Submit decision',exact:true}).click();await page.getByRole('dialog').waitFor({state:'hidden'});
 assert.equal(exchanges.findLast(x=>x.request?.operation_ref==='review.approve').result.data.external_effect,false);
 assert.equal(sourceRequests.length,1,'Approval alone must not fetch');
 const oldHost=telemetry.instance_id;telemetry=cli('host','restart').data.value;assert.notEqual(telemetry.instance_id,oldHost);
 await page.getByRole('button',{name:'Refresh Case',exact:true}).click();
 await page.locator('.live-rail button[aria-label="Environment"]').click();
 await page.locator('.live-sidebar').getByRole('button',{name:'reviewed-http',exact:true}).click();
 await sourceReceipt.getByText('waiting for review',{exact:true}).waitFor();
 await page.getByRole('button',{name:'Resume acquisition…',exact:true}).click();form=page.getByRole('dialog',{name:'Resume acquisition'});
 // Case mutation invalidates the confirmation before it can dispatch.
 await accepted('participant.role.add',{case_ref:caseRef,participant_ref:'participant:operator',role:'observation-reader'});
 await page.evaluate(()=>window.qualificationPlatform.commands.executeCommand('studio.case.refresh'));
 assert.equal(await form.getByRole('button',{name:'Resume exact attempt'}).isEnabled(),false);assert.equal(sourceRequests.length,1);
 await form.getByRole('button',{name:'Cancel',exact:true}).click();
 serveSource=true;
 await page.getByRole('button',{name:'Resume acquisition…',exact:true}).click();form=page.getByRole('dialog',{name:'Resume acquisition'});dropAcknowledgement='source.resume';
 await form.getByRole('button',{name:'Resume exact attempt'}).click();await form.getByText('Confirmation was lost',{exact:true}).waitFor();
 assert.equal(await form.getByRole('button',{name:'Resume exact attempt'}).isEnabled(),false);
 const completed=exchanges.findLast(x=>x.request?.operation_ref==='source.resume');
 assert.equal(completed.result.result_state,'success');assert.equal(completed.request.input.attempt,1);assert.equal(completed.request.input.source_ref,waiting.request.input.source_ref);
 assert.equal(completed.request.input.previous_progress_ref,waiting.result.data.execution.progress_ref);
 assert.equal(completed.result.data.execution.phase,'acquired');assert.equal(sourceRequests.length,2);
 await form.getByRole('button',{name:'Close and inspect state'}).click();
 await page.getByRole('button',{name:'Refresh Case',exact:true}).click();await sourceReceipt.getByText('acquired',{exact:true}).first().waitFor();
 const terminal=(await accepted('case.summary',{case_ref:caseRef})).case.generation;
 const exactRetry=await accepted('source.resume',completed.request.input);assert.equal(exactRetry.created,false);assert.deepEqual(exactRetry.execution,completed.result.data.execution);
 assert.equal((await accepted('case.summary',{case_ref:caseRef})).case.generation,terminal);assert.equal(sourceRequests.length,2,'Completed resume retry must not fetch again');
 await page.screenshot({path:`${evidence}/source-review-resumed.png`});
 // Later current work must not hide the earlier approved retained Source.
 const nextRequest=await accepted('resource.request',{case_ref:caseRef,participant_ref:'participant:operator',resource_ref:binding.attachment_id,submission_ref:'request:later-reviewed-http',expected_generation:terminal,request:{schema:'yai.resource_request.v1',configuration_digest:'sha256:'+createHash('sha256').update(JSON.stringify(binding)).digest('hex'),action:{action:'http_fetch',name:'document'}}});
 assert.equal(nextRequest.execution.posture.state,'waiting_for_review');assert.equal(sourceRequests.length,2,'Historical approval cannot authorize a fresh request');
 await page.getByRole('button',{name:'Refresh Case',exact:true}).click();
 const readSummary=await accepted('case.summary',{case_ref:caseRef});
 const reviewedFile=readSummary.environment.files.find(file=>file.path==='reviewed-http');assert.ok(reviewedFile,'Approved exact Source material must be projected');
 const readRequest={case_ref:caseRef,source_ref:reviewedFile.source_ref,revision_ref:reviewedFile.revision_ref,path:reviewedFile.path,expected_generation:readSummary.case.generation};
 const retained=await accepted('material.read',readRequest);assert.ok(retained.content.includes('reviewed-source-exact'));
 for(const field of ['source_ref','revision_ref','path','digest'])assert.equal(retained[field],reviewedFile[field]);
 await page.locator('.source-surface .fact-row').filter({hasText:'reviewed-http'}).click();
 await page.locator('.cm-content').filter({hasText:'reviewed-source-exact'}).waitFor();
 const cliRead=cli('case','sources','read',caseRef,'--source',waiting.request.input.source_ref).data.value;
 exchanges.push({order:exchanges.length+1,cli:['case','sources','read',caseRef,'--source',waiting.request.input.source_ref],result:cliRead});
 assert.ok(JSON.stringify(cliRead).includes('reviewed-source-exact'));
 assert.equal(sourceRequests.length,2,'Reading retained evidence cannot fetch again');
 await accepted('policy.revoke',{artifact_ref:reviewedArtifact,reason:'Current reviewed Source disclosure must refuse'});
 assert.equal((await accepted('case.summary',{case_ref:caseRef})).case.generation,readSummary.case.generation,'Tenant policy revocation does not require a Case generation change');
 const revokedRead=await call('material.read',readRequest);assert.notEqual(revokedRead.result_state,'success');assert.equal(revokedRead.data,undefined);
 const refusedCli=spawnSync(binary,['case','sources','read',caseRef,'--source',waiting.request.input.source_ref,'--json'],{env:{...process.env,YAI_HOME:home},encoding:'utf8',timeout:30000});
 assert.notEqual(refusedCli.status,0);assert.equal(refusedCli.error,undefined);assert.equal(refusedCli.stdout,'');assert.ok(!refusedCli.stderr.includes('reviewed-source-exact'));
 exchanges.push({order:exchanges.length+1,cli:['case','sources','read',caseRef,'--source',waiting.request.input.source_ref],exit:refusedCli.status,stdout:refusedCli.stdout,stderr:refusedCli.stderr});
 await page.getByRole('button',{name:'Refresh Case',exact:true}).click();
 await page.locator('.cm-content').filter({hasText:'reviewed-source-exact'}).waitFor({state:'hidden'});
 assert.equal(sourceRequests.length,2);


 assert.equal(cli('case','verify',caseRef).status,'ok');assert.deepEqual(errors,[]);
 console.log(JSON.stringify({result:'PASS',case_ref:caseRef,source:ordinary,generation:stable,proof:['UI acquire missing material -> resume same exact attempt','UI source policy publication and binding','Lost acknowledgement retained reference and no auto-retry','Exact execution observation after real Host restart','Duplicate submit does not create generation','Stale generation and hidden Participant refused','Owner Knowledge inspect/search/resolve/navigation','Revocation refuses old resolution and clears UI','Four sizes and CLI replay','Real in-flight HTTP Source carrier loss: UI running -> delivery indeterminate, disabled redispatch, exact retry and hidden refusal, one remote request','Source Review approval does not fetch; explicit same-attempt resume after Host restart with stale confirmation and lost-ACK retry preserves one fetch','Reviewed material exact identity matches Studio/CLI; current policy revocation hides clean cached content at the same Case generation']}));

}finally{await writeFile(`${evidence}/exchanges.json`,JSON.stringify(exchanges,null,2));await browser?.close();sourceServer?.closeAllConnections();sourceServer?.close();try{if(telemetry)cli('host','stop');}finally{await rm(home,{recursive:true,force:true});}}
