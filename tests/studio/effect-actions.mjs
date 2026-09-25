// UI calls the real published Host over Unix IPC through a test-only Tauri bridge.
// Only a freshly created temporary YAI_HOME is mutated; never operator state.
import { createRequire } from 'node:module';
import assert from 'node:assert/strict';
import { execFile, execFileSync, spawn } from 'node:child_process';
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import net from 'node:net';
import { createHash } from 'node:crypto';
import { realpath } from 'node:fs/promises';
import { createServer } from 'node:http';
import os from 'node:os';
import { promisify } from 'node:util';
const execFileAsync = promisify(execFile);
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
let dispatches=0, recoveryProposal;
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
 provider=createServer(async(req,res)=>{let body='';for await(const chunk of req)body+=chunk;const input=JSON.parse(body);const active=input.messages.some(message=>String(message.content).includes('YAI typed ContextFrame:'));if(active){dispatches++;await providerGate;}const content=active?JSON.stringify(recoveryProposal ?? (dispatches===1?{schema:'yai.operation_proposal.filesystem_write.v1',operation:'filesystem.write',resource:'workspace',path:'allowed/resume-proof.txt',content:'one governed write before stop'}:{schema:'yai.case_runtime_turn.v1',outcome:'complete',summary:'continued without repeating the effect'})):'{"probe":true}';res.setHeader('Content-Type','application/json');res.end(JSON.stringify({id:'controlled',object:'chat.completion',model:input.model,choices:[{index:0,message:{role:'assistant',content},finish_reason:'stop'}],usage:{prompt_tokens:1,completion_tokens:1,total_tokens:2}}));});
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
 await page.locator('.live-rail button[aria-label="Work"]').click();await page.getByRole('tab',{name:'Executions',exact:true}).click();await page.getByRole('button',{name:'Run bounded work…',exact:true}).click();form=page.getByRole('dialog',{name:'Run bounded work'});await form.getByLabel('Task',{exact:true}).fill('Complete one bounded controlled task');await form.getByLabel('Resource',{exact:true}).selectOption('workspace');await form.getByLabel('Maximum invocations').fill('2');dropAcknowledgement='case.run';await form.getByRole('button',{name:'Submit work'}).click();await form.getByText('Confirmation was lost',{exact:true}).waitFor();await form.getByRole('button',{name:'Close and inspect state'}).click();
 const run=exchanges.findLast(item=>item.request.operation_ref==='case.run');assert.equal(run.result.result_state,'success',JSON.stringify(run));
 const receipt=page.locator('.work-surface .execution-receipt').filter({hasText:run.request.input.submission_ref});await receipt.waitFor();
 for(let attempt=0;attempt<100 && !dispatches;attempt++)await new Promise(resolve=>setTimeout(resolve,50));assert.equal(dispatches,1,JSON.stringify(await accepted('execution.get',{case_ref:caseRef,participant_ref:'participant:operator',execution:{domain:'runtime_work',submission_ref:run.request.input.submission_ref}})));
 await receipt.getByRole('button',{name:'Refresh observation'}).click();await receipt.getByRole('button',{name:'Stop this run…'}).click();form=page.getByRole('dialog',{name:'Stop this run'});await form.getByRole('button',{name:'Request stop'}).click();await form.waitFor({state:'hidden'});const stop=exchanges.findLast(item=>item.request.operation_ref==='case.stop');assert.equal(stop.result.result_state,'success',JSON.stringify(stop));assert.equal(stop.result.data.runner.stop_requested,true);
 releaseProvider();const observe={case_ref:caseRef,participant_ref:'participant:operator',execution:{domain:'runtime_work',submission_ref:run.request.input.submission_ref}};let finished;
 for(let attempt=0;attempt<100;attempt++){finished=await accepted('execution.get',observe);if(['completed','cancelled'].includes(finished.state))break;await new Promise(resolve=>setTimeout(resolve,50));}assert.ok(['completed','cancelled'].includes(finished.state),JSON.stringify(finished));
 telemetry=cli('host','restart').data.value;assert.equal((await accepted('execution.get',observe)).execution_ref,finished.execution_ref);assert.equal((await accepted('case.run',run.request.input)).created,false);assert.equal(dispatches,1);
 const badStop=await call('case.stop',{...stop.request.input,run_ref:'run:wrong'});assert.notEqual(badStop.result_state,'success');assert.equal((await call('execution.get',{...observe,participant_ref:'participant:hidden'})).result_state,'unauthorized');
 await page.getByRole('button',{name:'Refresh Case',exact:true}).click();await receipt.getByRole('button',{name:'Refresh observation'}).click();

 assert.equal(finished.runner.posture,'operator_stopped',JSON.stringify(finished));
 const effectsBefore=(await accepted('case.summary',{case_ref:caseRef})).memory.timeline.filter(event=>/effect|receipt/i.test(event.kind));assert.ok(effectsBefore.length>0);
 await receipt.getByRole('button',{name:'Resume stopped work…',exact:true}).click();
 const resumeLimit=Number(process.env.STUDIO_RESUME_LIMIT??2);assert.ok([1,2].includes(resumeLimit));
 form=page.getByRole('dialog',{name:'Resume stopped work'});await form.getByLabel('Total invocation limit').fill(String(resumeLimit));dropAcknowledgement='case.resume';
 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]){await page.setViewportSize({width,height});await page.screenshot({path:`${evidence}/resume-${width}x${height}.png`});}
 await form.getByRole('button',{name:'Submit continuation',exact:true}).click();await form.getByText('Confirmation was lost',{exact:true}).waitFor();
 assert.equal(await form.getByRole('button',{name:'Submit continuation',exact:true}).isEnabled(),false);
 const continuation=exchanges.findLast(item=>item.request.operation_ref==='case.resume');assert.equal(continuation.result.result_state,'success',JSON.stringify(continuation));
 await form.getByRole('button',{name:'Close and inspect state'}).click();
 const resumedRef={...observe,execution:{domain:'runtime_work',submission_ref:continuation.request.input.submission_ref}};let resumed;
 for(let attempt=0;attempt<100;attempt++){resumed=await accepted('execution.get',resumedRef);if(['completed','cancelled'].includes(resumed.state))break;await new Promise(resolve=>setTimeout(resolve,50));}
 assert.equal(resumed.runner.run_ref,finished.runner.run_ref);assert.notEqual(resumed.execution_ref,finished.execution_ref);
 assert.equal(resumed.runner.posture,resumeLimit===1?'invocation_budget_exhausted':'completed');assert.equal(dispatches,resumeLimit,'Continuation keeps consumed budget and dispatches only the remaining invocation');
 assert.equal(await readFile(path.join(workspace,'allowed/resume-proof.txt'),'utf8'),'one governed write before stop');
 assert.deepEqual((await accepted('case.summary',{case_ref:caseRef})).memory.timeline.filter(event=>/effect|receipt/i.test(event.kind)),effectsBefore,'Continuation must not create a duplicate canonical effect/receipt');
 const retried=await accepted('case.resume',continuation.request.input);assert.equal(retried.created,false);assert.equal(retried.execution.execution_ref,resumed.execution_ref);assert.equal(dispatches,resumeLimit);
 assert.equal((await call('case.resume',{...continuation.request.input,participant_ref:'participant:hidden'})).result_state,'unauthorized');
 assert.equal((await call('case.resume',{...continuation.request.input,submission_ref:'request:stale-resume',checkpoint_digest:'wrong'})).result_state,'stale');
 telemetry=cli('host','restart').data.value;assert.equal((await accepted('execution.get',resumedRef)).execution_ref,resumed.execution_ref);assert.equal(dispatches,resumeLimit);
 await page.getByRole('button',{name:'Refresh Case',exact:true}).click();
 await page.locator('.work-surface .execution-catalog-row').filter({hasText:continuation.request.input.submission_ref}).click();
 await page.locator('.work-surface .execution-receipt').filter({hasText:continuation.request.input.submission_ref}).getByRole('button',{name:'Refresh observation'}).click();
 // Historical Decisions are read through the same typed Host dispatcher.
 await page.getByRole('tab',{name:'Decisions',exact:true}).click();const history=page.getByRole('region',{name:'Decision history'});
 await history.getByRole('button',{name:'Load Decision history',exact:true}).click();
 await history.locator('.decision-corpus .fact-row').first().waitFor();
 const corpus=exchanges.findLast(item=>item.request.operation_ref==='decision.trajectory.corpus').result;
 assert.equal(corpus.result_state,'success');assert.ok(corpus.data.trajectories.length>0);
 const selectedDecision=corpus.data.trajectories[0];
 const historyBefore=await accepted('case.summary',{case_ref:caseRef});
 await history.locator('.decision-corpus .fact-row').first().click();
 await history.locator('.decision-reconstruction').waitFor();
 const inspected=exchanges.findLast(item=>item.request.operation_ref==='decision.trajectory.inspect').result;
 assert.equal(inspected.result_state,'success');assert.equal(inspected.data.decision.decision_id,selectedDecision.decision.decision_id);
 assert.ok(inspected.data.pre_decision.cut_generation<inspected.data.decision_generation);
 await history.getByText(selectedDecision.decision.reason,{exact:true}).waitFor();
 // Navigate a real historical filesystem Decision to its current controlled-effect receipt.
 let controlledDecision, controlledEvidence;
 for(const item of corpus.data.trajectories) {
  const result=await call('execution.get',{case_ref:caseRef,participant_ref:'participant:operator',execution:{domain:'controlled_effect',operation_ref:item.decision.operation_id}});
  if(result.result_state==='success' && result.data.progress?.receipt_id) {controlledDecision=item;controlledEvidence=result.data;break;}
 }
 assert.ok(controlledDecision,'Real governed file effect missing from Decision history');
 await history.getByLabel('Exact Decision reference').fill(controlledDecision.decision.decision_id);
 await history.getByRole('button',{name:'Inspect Decision',exact:true}).click();
 await history.getByRole('button',{name:'Observe Operation effect',exact:true}).click();
 const controlled=history.getByRole('region',{name:'Current effect observation',exact:true});
 await controlled.getByText('Effect identities',{exact:true}).click();
 await controlled.getByText(controlledEvidence.progress.receipt_id,{exact:true}).waitFor();
 await controlled.getByText('applied',{exact:true}).waitFor();
 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]){await page.setViewportSize({width,height});await controlled.scrollIntoViewIfNeeded();await page.screenshot({path:`${evidence}/controlled-effect-${width}x${height}.png`});}
 const retainedBefore=await accepted('case.summary',{case_ref:caseRef});
 await controlled.getByRole('button',{name:'Refresh observation',exact:true}).click();
 await controlled.getByText(controlledEvidence.progress.receipt_id,{exact:true}).waitFor();
 assert.deepEqual(await accepted('case.summary',{case_ref:caseRef}),retainedBefore,'Effect observation must not change canonical state');
 assert.equal(dispatches,resumeLimit,'Effect observation must not redispatch provider');
 const hiddenEffect=await call('execution.get',{case_ref:caseRef,participant_ref:'participant:hidden',execution:{domain:'controlled_effect',operation_ref:controlledDecision.decision.operation_id}});
 assert.notEqual(hiddenEffect.result_state,'success');assert.equal(hiddenEffect.data,undefined);
 await page.getByRole('tab',{name:'Executions',exact:true}).click();await page.getByRole('button',{name:'Observe exact execution…',exact:true}).click();
 const manual=page.locator('.execution-observe-form');await manual.getByLabel('Execution family').selectOption('controlled_effect');await manual.getByLabel('Exact reference').fill('operation:not-visible');await manual.getByRole('button',{name:'Observe reference',exact:true}).click();
 await page.locator('.execution-history .execution-receipt').filter({hasText:'operation:not-visible'}).getByRole('alert').waitFor();
 assert.equal(await page.locator('.execution-history .execution-receipt').filter({hasText:'operation:not-visible'}).getByRole('region',{name:'Controlled effect evidence'}).count(),0);

 await page.getByRole('tab',{name:'Decisions',exact:true}).click();await history.getByRole('button',{name:'Evaluate reconstruction',exact:true}).click();await history.locator('.decision-evaluation').waitFor();
 const evaluated=exchanges.findLast(item=>item.request.operation_ref==='decision.trajectory.evaluate').result;
 assert.equal(evaluated.result_state,'success');assert.equal(evaluated.data.trajectory_count,corpus.data.trajectories.length);
 assert.equal(evaluated.data.cross_case_leakage_violations,0);
 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]){await page.setViewportSize({width,height});await history.scrollIntoViewIfNeeded();await page.screenshot({path:`${evidence}/decision-history-${width}x${height}.png`});}
 await history.getByLabel('Exact Decision reference').fill('decision:not-visible');await history.getByRole('button',{name:'Inspect Decision',exact:true}).click();await history.getByRole('alert').waitFor();
 assert.notEqual(exchanges.findLast(item=>item.request.operation_ref==='decision.trajectory.inspect').result.result_state,'success');
 assert.equal(await history.locator('.decision-reconstruction,.decision-corpus,.decision-evaluation').count(),0,'Refused re-read must not retain prior disclosures');
 for(const operation of ['decision.trajectory.corpus','decision.trajectory.evaluate'])assert.equal((await call(operation,{case_ref:caseRef,participant_ref:'participant:hidden',max_decisions:16})).result_state,'unauthorized');
 assert.deepEqual(await accepted('case.summary',{case_ref:caseRef}),historyBefore,'Historical inspection/evaluation must not mutate the Case');
 // Seed a real PREPARE without dispatch through the existing diagnostic CLI failpoint.
 // Product interaction below uses typed Host calls; the CLI is fixture setup only.
 cli('case','attach-provider','--case',caseRef,'--subject','participant:operator','--base-url',endpoint,'--model','controlled-model');
 for(const retry of [false,true]) {
  const relative=`allowed/reconcile-${retry}.txt`;
  recoveryProposal={schema:'yai.operation_proposal.filesystem_write.v1',operation:'filesystem.write',resource:'workspace',path:relative,content:`exact recovery ${retry}`};
  let crashed;
  try {await execFileAsync(binary,['effect','filesystem-write','--case',caseRef,'--subject','participant:operator','--attachment','workspace','--prompt','Propose the exact controlled recovery write','--base-url',endpoint,'--model','controlled-model','--failpoint','after_prepare_before_effect'],{env:{...process.env,YAI_HOME:home},timeout:30000});}
  catch(error){crashed=error;}
  assert.ok(crashed,'Failpoint must interrupt before external execution');
  assert.match(crashed.stderr,/controlled_effect_crash_injected: after_prepare_before_effect/);
  await writeFile(`${evidence}/prepare-${retry}.json`,JSON.stringify({argv:crashed.cmd,exit:crashed.code,stdout:crashed.stdout,stderr:crashed.stderr}));
  await assert.rejects(readFile(path.join(workspace,relative)),{code:'ENOENT'});
  const decisions=await accepted('decision.trajectory.corpus',{case_ref:caseRef,participant_ref:'participant:operator',max_decisions:32});
  let prepared;
  for(const item of decisions.trajectories){const response=await call('execution.get',{case_ref:caseRef,participant_ref:'participant:operator',execution:{domain:'controlled_effect',operation_ref:item.decision.operation_id}});if(response.result_state==='success' && response.data.progress?.status==='indeterminate'){prepared=response.data;break;}}
  assert.ok(prepared,'Prepared effect must remain observable after CLI interruption');
  await page.evaluate(()=>window.qualificationPlatform.commands.executeCommand('studio.case.refresh'));
  await page.getByRole('tab',{name:'Executions',exact:true}).click();await page.getByRole('button',{name:'Observe exact execution…',exact:true}).click();
  await manual.getByLabel('Execution family').selectOption('controlled_effect');await manual.getByLabel('Exact reference').fill(prepared.operation_ref);await manual.getByRole('button',{name:'Observe reference',exact:true}).click();
  const recovery=page.locator('.execution-history .execution-receipt').filter({hasText:prepared.operation_ref});
  await recovery.getByRole('button',{name:'Reconcile outcome…',exact:true}).click();
  form=page.getByRole('dialog',{name:'Reconcile effect outcome',exact:true});
  assert.equal(await form.getByRole('checkbox').isChecked(),false);
  if(!retry)for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]){await page.setViewportSize({width,height});const box=await form.boundingBox();assert.ok(box.x>=0 && box.y>=0 && box.x+box.width<=width+1 && box.y+box.height<=height+1,'Recovery dialog must fit');await page.screenshot({path:`${evidence}/reconcile-dialog-${width}x${height}.png`});}
  await page.setViewportSize({width:1440,height:900});
  // A dialog opened before a Case change cannot submit against a new snapshot.
  await accepted('participant.role.add',{case_ref:caseRef,participant_ref:'participant:operator',role:`reconcile-stale-${retry}`});
  await page.evaluate(()=>window.qualificationPlatform.commands.executeCommand('studio.case.refresh'));
  assert.equal(await form.getByRole('button',{name:'Reconcile exact effect',exact:true}).isEnabled(),false);
  await form.getByRole('button',{name:'Cancel',exact:true}).click();
  await recovery.getByRole('button',{name:'Reconcile outcome…',exact:true}).click();
  if(retry)await form.getByRole('checkbox').check();
  const noDispatch=dispatches;dropAcknowledgement='effect.reconcile';
  await form.getByRole('button',{name:'Reconcile exact effect',exact:true}).click();await form.getByText('Confirmation was lost',{exact:true}).waitFor();
  const reconciled=exchanges.findLast(item=>item.request.operation_ref==='effect.reconcile');
  assert.equal(reconciled.result.result_state,'success',JSON.stringify(reconciled));
  assert.equal(reconciled.request.input.retry_no_effect,retry);
  assert.equal(reconciled.result.data.progress.outcome,retry?'applied':'no_effect');
  assert.equal(await form.getByRole('button',{name:'Reconcile exact effect',exact:true}).isEnabled(),false);
  await form.getByRole('button',{name:'Close and inspect state',exact:true}).click();await recovery.getByRole('button',{name:'Refresh observation',exact:true}).click();
  await recovery.getByText(retry?'applied':'no effect',{exact:true}).waitFor();
  assert.equal(await recovery.getByRole('button',{name:'Reconcile outcome…',exact:true}).count(),0);
  const retained=await accepted('case.summary',{case_ref:caseRef});
  const repeated=await accepted('effect.reconcile',reconciled.request.input);
  assert.equal(repeated.progress.receipt_id,reconciled.result.data.progress.receipt_id);
  assert.deepEqual(await accepted('case.summary',{case_ref:caseRef}),retained);
  const hidden=await call('effect.reconcile',{...reconciled.request.input,participant_ref:'participant:hidden'});assert.notEqual(hidden.result_state,'success');assert.equal(hidden.data,undefined);
  const wrong=await call('effect.reconcile',{...reconciled.request.input,effect_ref:'effect:wrong'});assert.notEqual(wrong.result_state,'success');
  if(retry)assert.equal(await readFile(path.join(workspace,relative),'utf8'),recoveryProposal.content);else await assert.rejects(readFile(path.join(workspace,relative)),{code:'ENOENT'});
  assert.equal(dispatches,noDispatch,'Reconciliation must not invoke the provider again');
  await page.screenshot({path:`${evidence}/reconcile-${retry}.png`});
 }
 await page.screenshot({path:`${evidence}/work-executions.png`});assert.equal(cli('case','verify',caseRef).status,'ok');assert.deepEqual(errors,[]);
 console.log(JSON.stringify({result:'PASS',case_ref:caseRef,execution:finished.execution_ref,effect:effect.result.data.execution.operation_ref,proof:['UI controlled Resource effect; lost acknowledgement then exact retry, one dispatch','Configuration digest mismatch refused with no effect','UI process attach positive/absent PID refusal; no signal','UI bounded Case run, real supervised controlled provider','UI exact cooperative stop','Host restart, execution observation and no duplicate provider dispatch','Wrong runner and hidden Participant refused','UI Decision corpus/inspect/evaluate; exact pre-cut and real refusal; no mutation','UI stopped checkpoint resume; lost ACK exact retry; stale/hidden refusal; retained budgets and run identity','Historical Decision to exact current controlled-effect receipt; hidden/unknown refusal and no canonical mutation','UI exact effect reconciliation: observation-only/no-effect and opted-in filesystem recovery; lost ACK/idempotent receipt; stale dialog, hidden/wrong identity refusals','CLI replay']}));

}finally{await writeFile(`${evidence}/exchanges.json`,JSON.stringify(exchanges,null,2));await browser?.close();child?.kill();releaseProvider?.();await new Promise(resolve=>provider?.close(resolve)??resolve());try{if(telemetry)cli('host','stop');}finally{await rm(home,{recursive:true,force:true});}}
