// UI calls the real published Host over Unix IPC through a test-only Tauri bridge.
// Only a freshly created temporary YAI_HOME is mutated; never operator state.
import { createRequire } from 'node:module';
import assert from 'node:assert/strict';
import { execFileSync, spawn } from 'node:child_process';
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import net from 'node:net';
import { createHash } from 'node:crypto';
import { realpath } from 'node:fs/promises';
import { createServer } from 'node:http';
import os from 'node:os';
import path from 'node:path';
const require = createRequire(path.resolve(import.meta.dirname, '../../studio/package.json'));
const { chromium } = require('playwright-core');
const binary = process.env.YAI_STUDIO_TEST_BINARY;
assert.ok(binary, 'YAI_STUDIO_TEST_BINARY must name a built published CLI/Host');
const home = await mkdtemp(path.join(os.tmpdir(), 'yai-studio-effects-'));
const evidence = process.env.STUDIO_EVIDENCE_DIR ?? '/tmp/yai-studio-effects';
await mkdir(evidence, {recursive:true});
const cli = (...args) => JSON.parse(execFileSync(binary, [...args, '--json'], {env:{...process.env, YAI_HOME:home}, encoding:'utf8', timeout:30000}));
let telemetry, serial=0, browser, provider, child, releaseProvider, dropAcknowledgement;
let dispatches=0;
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
 const workspace=path.join(home,'workspace');await mkdir(path.join(workspace,'allowed'),{recursive:true});
 cli('case','attach-filesystem','--case',caseRef,'--attachment','workspace','--root',workspace,'--allow-prefix','allowed','--policy-owner','participant:operator','--max-bytes','1024');
 await accepted('participant.role.add',{case_ref:caseRef,participant_ref:'participant:operator',role:'operation-proposer'});
 await accepted('participant.view.admit',{case_ref:caseRef,participant_ref:'participant:operator',consumer:'model',view_kind:'model_context'});
 const policy=JSON.parse(await readFile(path.resolve(import.meta.dirname,'../fixtures/cli-product-policy.json'),'utf8'));policy.owner_ref='organization:yailabs';
 policy.rules.push({kind:'operation_restriction',rule_id:'controlled-runner',operation_kind:'process.run',resource_kind:'process_runner',effect:'allow',reason:'Only the pre-bound verification command'});
 const artifact=(await accepted('policy.ingest',{tenant_id:'tenant:studio-ui',source_bytes:[...Buffer.from(JSON.stringify(policy))]})).view.artifact.artifact_id;
 for(const operation of ['policy.validate','policy.publish'])await accepted(operation,{artifact_ref:artifact,reason:'Controlled product execution'});
 await accepted('policy.case.bind',{case_ref:caseRef,artifact_ref:artifact,expected_generation:(await accepted('case.summary',{case_ref:caseRef})).case.generation,reason:'Controlled product execution'});
 const executable=await realpath('/usr/bin/python3');const processRoot=path.join(home,'process');await mkdir(path.join(processRoot,'work'),{recursive:true});
 const definition={schema:'yai.resource_definition.v1',attachment_id:'resource:runner',policy_owner:'participant:operator',participant_ids:['participant:operator'],operations:['process_run'],read_prefixes:[],names:['verify'],max_output_bytes:8192,max_items:8,address:{kind:'process_runner',root:processRoot,runners:{verify:{executable,executable_digest:'sha256:'+createHash('sha256').update(await readFile(executable)).digest('hex'),argv:['-I','-B','-c',"import time; print('controlled-effect', time.time_ns())"],working_directory:'work',environment:{},timeout_ms:1000}}}};
 const resourceFile=path.join(home,'runner.json');await writeFile(resourceFile,JSON.stringify(definition));cli('case','resource','import',caseRef,'--file',resourceFile);
 const providerGate=new Promise(resolve=>{releaseProvider=resolve;});
 provider=createServer(async(req,res)=>{let body='';for await(const chunk of req)body+=chunk;const input=JSON.parse(body);const active=input.messages.some(message=>String(message.content).includes('YAI typed ContextFrame:'));if(active){dispatches++;await providerGate;}const content=active?JSON.stringify({schema:'yai.case_runtime_turn.v1',outcome:'complete',summary:'bounded controlled completion'}):'{"probe":true}';res.setHeader('Content-Type','application/json');res.end(JSON.stringify({id:'controlled',object:'chat.completion',model:input.model,choices:[{index:0,message:{role:'assistant',content},finish_reason:'stop'}],usage:{prompt_tokens:1,completion_tokens:1,total_tokens:2}}));});
 await new Promise(resolve=>provider.listen(0,'127.0.0.1',resolve));const endpoint=`http://127.0.0.1:${provider.address().port}/v1/chat/completions`;
 const target=await accepted('provider.register',{tenant_id:'tenant:studio-ui',provider_key:'controlled-execution',adapter:'open_ai_compatible',endpoint,model_id:'controlled-model',credential_ref:'none',locality:'loopback'});
 const started=Date.now();const proof=await fetch(endpoint,{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({model:'controlled-model',messages:[{role:'user',content:'probe'}]})}).then(response=>response.json());assert.equal(JSON.parse(proof.choices[0].message.content).probe,true);
 await accepted('provider.qualify',{target_ref:target.target_id,suite_ref:'test:studio-execution',evidence:{run_id:path.basename(home),target_id:target.target_id,started_at_unix_ms:started,completed_at_unix_ms:Date.now(),transport_connected:true,exact_model_addressed:proof.model==='controlled-model',chat_text_envelope_valid:true,structured_json_object_valid:true,usage_accounting_observed:true,health_endpoint_observed:false,extension_telemetry_observed:false,failure_codes:[]}});
 await accepted('provider.trust.set',{target_ref:target.target_id,posture:'approved'});await accepted('provider.case.bind',{case_ref:caseRef,participant_ref:'participant:operator',ordered_target_refs:[target.target_id],failover_policy:'none',max_attempts_per_turn:1});
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


 await page.locator('.live-rail button[aria-label="Environment"]').click();
 const resources=page.locator('.live-sidebar details.sidebar-group > summary').filter({hasText:/^Resources/}).locator('..');await resources.getByRole('button').filter({hasText:'runner'}).click();
 await page.getByRole('button',{name:'Request Resource operation…',exact:true}).click();let form=page.getByRole('dialog',{name:'Request Resource operation'});dropAcknowledgement='resource.request';await form.getByRole('button',{name:'Submit governed request'}).click();await form.getByText('Confirmation was lost',{exact:true}).waitFor();assert.equal(await form.getByRole('button',{name:'Submit governed request'}).isEnabled(),false);await form.getByRole('button',{name:'Close and inspect state'}).click();
 const effect=exchanges.findLast(item=>item.request.operation_ref==='resource.request');assert.equal(effect.result.result_state,'success',JSON.stringify(effect));assert.equal(effect.result.data.execution.posture.state,'effect_recorded');assert.match(effect.result.data.outcome.observation.result.stdout,/^controlled-effect [0-9]+/);assert.equal(effect.result.data.outcome.observation.result.exit_code,0);
 const effectGeneration=(await accepted('case.summary',{case_ref:caseRef})).case.generation;const duplicate=await accepted('resource.request',effect.request.input);assert.equal(duplicate.outcome,null);assert.equal(duplicate.execution.operation_ref,effect.result.data.execution.operation_ref);assert.deepEqual(duplicate.execution,effect.result.data.execution,'Retry returns the same durable receipt, without redispatching');assert.equal((await accepted('case.summary',{case_ref:caseRef})).case.generation,effectGeneration);
 const before=(await accepted('case.summary',{case_ref:caseRef})).case.generation;const invalid=await call('resource.request',{...effect.request.input,submission_ref:'request:invalid-config',expected_generation:before,request:{...effect.request.input.request,configuration_digest:'sha256:'+'0'.repeat(64)}});assert.notEqual(invalid.result_state,'success');assert.equal((await call('execution.get',{case_ref:caseRef,participant_ref:'participant:operator',execution:{domain:'resource_request',submission_ref:'request:invalid-config'}})).result_state,'unauthorized');assert.match(effect.result.data.outcome.observation.result.stdout,/^controlled-effect [0-9]+/);assert.equal(effect.result.data.outcome.observation.result.exit_code,0);
 await page.getByRole('button',{name:'Refresh Case',exact:true}).click();await page.locator('.live-rail button[aria-label="Environment"]').click();
 await page.getByRole('button',{name:'Attach process…',exact:true}).click();form=page.getByRole('dialog',{name:'Attach process Resource'});await form.getByLabel('Resource reference').fill('resource:owned-process');await form.getByLabel('Exact PID').fill('2147483647');await form.getByLabel('suspend',{exact:true}).check();await form.getByRole('button',{name:'Attach process',exact:true}).click();await form.locator('.action-result').waitFor();assert.notEqual(exchanges.findLast(item=>item.request.operation_ref==='resource.attach_process').result.result_state,'success');
 child=spawn('/usr/bin/sleep',['60']);await form.getByLabel('Exact PID').fill(String(child.pid));await form.getByRole('button',{name:'Attach process',exact:true}).click();await form.waitFor({state:'hidden'});assert.equal(exchanges.findLast(item=>item.request.operation_ref==='resource.attach_process').result.result_state,'success');assert.ok((await accepted('case.summary',{case_ref:caseRef})).environment.resources.some(item=>item.id==='resource:owned-process'));assert.equal(child.exitCode,null,'Attachment does not execute a signal');
 await page.locator('.live-rail button[aria-label="Work"]').click();await page.getByRole('button',{name:'Run bounded work…',exact:true}).click();form=page.getByRole('dialog',{name:'Run bounded work'});await form.getByLabel('Task',{exact:true}).fill('Complete one bounded controlled task');await form.getByLabel('Resource',{exact:true}).selectOption('workspace');await form.getByLabel('Maximum invocations').fill('2');dropAcknowledgement='case.run';await form.getByRole('button',{name:'Submit work'}).click();await form.getByText('Confirmation was lost',{exact:true}).waitFor();await form.getByRole('button',{name:'Close and inspect state'}).click();
 const run=exchanges.findLast(item=>item.request.operation_ref==='case.run');assert.equal(run.result.result_state,'success',JSON.stringify(run));
 const receipt=page.locator('.work-surface .execution-receipt').filter({hasText:run.request.input.submission_ref});await receipt.waitFor();
 for(let attempt=0;attempt<100 && !dispatches;attempt++)await new Promise(resolve=>setTimeout(resolve,50));assert.equal(dispatches,1,JSON.stringify(await accepted('execution.get',{case_ref:caseRef,participant_ref:'participant:operator',execution:{domain:'runtime_work',submission_ref:run.request.input.submission_ref}})));
 await receipt.getByRole('button',{name:'Refresh observation'}).click();await receipt.getByRole('button',{name:'Stop this run…'}).click();form=page.getByRole('dialog',{name:'Stop this run'});await form.getByRole('button',{name:'Request stop'}).click();await form.waitFor({state:'hidden'});const stop=exchanges.findLast(item=>item.request.operation_ref==='case.stop');assert.equal(stop.result.result_state,'success',JSON.stringify(stop));assert.equal(stop.result.data.runner.stop_requested,true);
 releaseProvider();const observe={case_ref:caseRef,participant_ref:'participant:operator',execution:{domain:'runtime_work',submission_ref:run.request.input.submission_ref}};let finished;
 for(let attempt=0;attempt<100;attempt++){finished=await accepted('execution.get',observe);if(['completed','cancelled'].includes(finished.state))break;await new Promise(resolve=>setTimeout(resolve,50));}assert.ok(['completed','cancelled'].includes(finished.state),JSON.stringify(finished));
 telemetry=cli('host','restart').data.value;assert.equal((await accepted('execution.get',observe)).execution_ref,finished.execution_ref);assert.equal((await accepted('case.run',run.request.input)).created,false);assert.equal(dispatches,1);
 const badStop=await call('case.stop',{...stop.request.input,run_ref:'run:wrong'});assert.notEqual(badStop.result_state,'success');assert.equal((await call('execution.get',{...observe,participant_ref:'participant:hidden'})).result_state,'unauthorized');
 await page.getByRole('button',{name:'Refresh Case',exact:true}).click();await receipt.getByRole('button',{name:'Refresh observation'}).click();
 await page.screenshot({path:`${evidence}/work-executions.png`});assert.equal(cli('case','verify',caseRef).status,'ok');assert.deepEqual(errors,[]);
 console.log(JSON.stringify({result:'PASS',case_ref:caseRef,execution:finished.execution_ref,effect:effect.result.data.execution.operation_ref,proof:['UI controlled Resource effect; lost acknowledgement then exact retry, one dispatch','Configuration digest mismatch refused with no effect','UI process attach positive/absent PID refusal; no signal','UI bounded Case run, real supervised controlled provider','UI exact cooperative stop','Host restart, execution observation and no duplicate provider dispatch','Wrong runner and hidden Participant refused','CLI replay']}));

}finally{await writeFile(`${evidence}/exchanges.json`,JSON.stringify(exchanges,null,2));await browser?.close();child?.kill();releaseProvider?.();await new Promise(resolve=>provider?.close(resolve)??resolve());try{if(telemetry)cli('host','stop');}finally{await rm(home,{recursive:true,force:true});}}
