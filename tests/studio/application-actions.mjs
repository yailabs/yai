// UI calls the real published Host over Unix IPC through a test-only Tauri bridge.
// Only a freshly created temporary YAI_HOME is mutated; never operator state.
import { createRequire } from 'node:module';
import assert from 'node:assert/strict';
import { execFileSync, spawn } from 'node:child_process';
import { once } from 'node:events';
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import net from 'node:net';
import os from 'node:os';
import path from 'node:path';
const require = createRequire(path.resolve(import.meta.dirname, '../../studio/package.json'));
const { chromium } = require('playwright-core');
const binary = process.env.YAI_STUDIO_TEST_BINARY;
assert.ok(binary, 'YAI_STUDIO_TEST_BINARY must name a built published CLI/Host');
const home = await mkdtemp(path.join(os.tmpdir(), 'yai-studio-actions-'));
const evidence = process.env.STUDIO_EVIDENCE_DIR ?? '/tmp/yai-studio-actions';
await mkdir(evidence, {recursive:true});
const cli = (...args) => JSON.parse(execFileSync(binary, [...args, '--json'], {env:{...process.env, YAI_HOME:home}, encoding:'utf8', timeout:30000}));
let telemetry, serial=0, browser, runtime, runtimeOutput='', dropAcknowledgement;
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
 await accepted('identity.bootstrap',{tenant_id:'tenant:studio-ui',organization_ref:'organization:characterization'});
 const catalog=await accepted('application.capabilities');
 assert.ok(catalog.operations.some(op=>op.operation_id==='workflow.input.record'));
 // Normal deterministic Workflow/Resource operations create real pending Reviews.
 // Reuse the existing governed policy qualification, never fabricate review state.
 const reviewDefinition=JSON.parse(await readFile(path.resolve(import.meta.dirname,'../fixtures/workflows/review-required.v1.json'),'utf8'));
 reviewDefinition.tenant_id='tenant:studio-ui';
 const reviewWorkflow=await accepted('workflow.define',{definition:reviewDefinition});
 const reviewCases=[];
 for(const action of ['approve','deny','defer']) {
  const caseRef=`case:studio-review-${action}`; reviewCases.push({action,caseRef});
  await accepted('case.create',{tenant_id:'tenant:studio-ui',case_ref:caseRef});
  await accepted('participant.role.add',{case_ref:caseRef,participant_ref:'participant:operator',role:'operation-proposer'});
  await accepted('participant.role.add',{case_ref:caseRef,participant_ref:'subject:policy-pack',role:'resource-attachment-compatibility-owner'});
  await accepted('participant.principal.link',{case_ref:caseRef,participant_ref:'participant:operator',principal_ref:'self'});
  const resourceRoot=path.join(home,action);await mkdir(path.join(resourceRoot,'allowed'),{recursive:true});
  cli('case','attach-filesystem','--case',caseRef,'--attachment','workspace','--root',resourceRoot,'--allow-prefix','allowed','--policy-owner','subject:policy-pack','--max-bytes','4096');
  execFileSync('bash',['-e','-u','-o','pipefail','-c','source "$1"; yai_configure_governed_filesystem_case "$2" "$3" "$4" "$5" 1 allow participant:operator participant:reviewer','qualification',path.resolve(import.meta.dirname,'../characterization/lib/governed_case_policy.sh'),binary,home,caseRef,`studio-ui-${action}`],{env:{...process.env,YAI_HOME:home,YAI_TEST_TENANT_ID:'tenant:studio-ui'},encoding:'utf8',timeout:30000});
  await accepted('workflow.bind',{case_ref:caseRef,definition_ref:reviewWorkflow.workflow_definition_id,executor_bindings:[{slot:'operator',participant_id:'participant:operator'}],resource_bindings:[{slot:'workspace',attachment_id:'workspace'}],case_bindings:[]});
 }
 runtime=spawn(binary,['runtime','serve','--workers','2','--max-active-per-tenant','2','--max-queued-per-tenant','4','--max-queued-total','4'],{env:{...process.env,YAI_HOME:home},stdio:['ignore','pipe','pipe']});
 runtime.stdout.on('data',bytes=>{runtimeOutput+=bytes;});runtime.stderr.on('data',bytes=>{runtimeOutput+=bytes;});
 const pendingDeadline=Date.now()+15000;
 for(const item of reviewCases) {
  while(Date.now()<pendingDeadline) {const summary=await accepted('case.summary',{case_ref:item.caseRef});if(summary.authority.reviews.length){item.review=summary.authority.reviews[0];break;}await new Promise(resolve=>setTimeout(resolve,100));}
  assert.ok(item.review,`No real pending Review for ${item.caseRef}: ${runtimeOutput}`);
 }
 const runtimeExit=once(runtime,'exit');cli('runtime','stop');await runtimeExit;runtime=undefined;
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
 await page.getByRole('button',{name:'New Case',exact:true}).click();
 const dialog=page.getByRole('dialog',{name:'New Case',exact:true});
 await dialog.getByLabel('Case reference').fill('case:studio-ui-actions');
 await dialog.getByRole('button',{name:'Create Case',exact:true}).click();
 await page.getByRole('button',{name:'Add Participant role',exact:true}).click();
 await page.getByRole('button',{name:'Link my identity and open',exact:true}).click();
 await page.locator('.workbench-kernel').waitFor();
 await page.getByRole('dialog').waitFor({state:'hidden'});
 const created=await accepted('case.summary',{case_ref:'case:studio-ui-actions'});
 assert.equal(created.case.display_name,'Studio Ui Actions');
 const definition=JSON.parse(await readFile(path.resolve(import.meta.dirname,'../qualification/studio-product-vertical/workflow.json'),'utf8'));
 definition.tenant_id='tenant:studio-ui';
 const defined=await accepted('workflow.define',{definition});
 await accepted('participant.role.add',{case_ref:created.case.case_ref,participant_ref:created.case.participant_ref,role:'workflow-input'});
 await accepted('workflow.bind',{case_ref:created.case.case_ref,definition_ref:defined.workflow_definition_id,executor_bindings:[{slot:'operator',participant_id:created.case.participant_ref}],resource_bindings:[],case_bindings:[]});
 await page.evaluate(()=>window.qualificationPlatform.commands.executeCommand('studio.case.refresh'));
 await page.locator('.live-rail button[aria-label="Work"]').click();
 await page.getByRole('button',{name:'Provide input',exact:true}).click();
 await page.getByRole('dialog',{name:'Workflow input'}).getByLabel('Input',{exact:true}).fill('x'.repeat(257));
 await page.getByRole('button',{name:'Record input',exact:true}).click();
 await page.locator('.action-result').waitFor();
 assert.match(await page.locator('.action-result').innerText(),/YAI did not accept/);
 const before=await accepted('case.summary',{case_ref:created.case.case_ref});
 await page.getByRole('dialog',{name:'Workflow input'}).getByLabel('Input',{exact:true}).fill('Confirmed from the Studio authored Workflow form.');
 await page.getByRole('button',{name:'Record input',exact:true}).click();
 await page.getByRole('dialog').waitFor({state:'hidden'});
 await page.getByRole('button',{name:'Provide input',exact:true}).waitFor({state:'hidden'});
 const after=await accepted('case.summary',{case_ref:created.case.case_ref});
 assert.equal(after.case.generation,before.case.generation+1);
 assert.equal(after.work.nodes[0].posture,'satisfied');
 await page.screenshot({path:`${evidence}/workflow-accepted.png`});
 await page.keyboard.press('Control+j');
 await page.evaluate(()=>window.qualificationPlatform.commands.executeCommand('studio.file.settings'));
 await page.getByRole('button',{name:'Advanced',exact:true}).click();
 await page.getByRole('region',{name:'Application capabilities'}).waitFor();
 assert.match(await page.locator('.application-capabilities').innerText(),/UI debt/);
 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]) {
  await page.setViewportSize({width,height});await page.screenshot({path:`${evidence}/capabilities-${width}x${height}.png`});
  assert.ok(await page.locator('.kernel-status').evaluate(el=>el.getBoundingClientRect().bottom<=innerHeight));
 }
 await page.evaluate(()=>window.qualificationPlatform.commands.executeCommand('studio.case.close'));
 await page.getByRole('dialog',{name:'Close Case',exact:true}).getByLabel('Reason').fill('Disposable UI qualification complete.');
 await page.getByRole('button',{name:'Close this Case',exact:true}).click();
 await page.locator('.action-result').waitFor();
 assert.match(await page.locator('.action-result').innerText(),/cancellation before final closure/);
 await page.getByRole('dialog').getByRole('button',{name:'Cancel',exact:true}).click();
 await page.evaluate(()=>window.qualificationPlatform.commands.executeCommand('studio.case.cancel'));
 await page.getByRole('dialog',{name:'Cancel Case',exact:true}).getByLabel('Reason').fill('Disposable qualification cancellation before closure.');
 await page.getByRole('button',{name:'Cancel this Case',exact:true}).click();
 await page.getByRole('dialog').waitFor({state:'hidden'});
 await page.evaluate(()=>window.qualificationPlatform.commands.executeCommand('studio.case.close'));
 await page.getByRole('dialog',{name:'Close Case',exact:true}).getByLabel('Reason').fill('Disposable qualification final closure.');
 await page.getByRole('button',{name:'Close this Case',exact:true}).click();
 await page.getByRole('dialog').waitFor({state:'hidden'});
 const closed=await accepted('case.summary',{case_ref:created.case.case_ref});
 assert.equal(closed.case.case_status,'closed');
 assert.equal(cli('case','verify',created.case.case_ref).status,'ok');
 for(const item of reviewCases) {
  await page.evaluate(()=>window.qualificationPlatform.commands.executeCommand('studio.file.openCase'));
  await page.getByRole('dialog',{name:'Open Case'}).getByRole('button',{name:new RegExp(`Studio Review ${item.action[0].toUpperCase()+item.action.slice(1)}`)}).click();
  await page.waitForFunction(label=>document.title.startsWith(label),`Studio Review ${item.action[0].toUpperCase()+item.action.slice(1)}`);
  await page.locator('.live-rail button[aria-label="Authority"]').click();
  const reviewRow=page.locator('.collection-row').filter({hasText:item.review.id});await reviewRow.click();
  await page.getByRole('button',{name:`${item.action[0].toUpperCase()+item.action.slice(1)} review`,exact:true}).click();
  await page.getByRole('dialog').getByLabel('Reason').fill(`Studio UI qualification: ${item.action}.`);
  await page.getByRole('button',{name:'Submit decision',exact:true}).click();
  await page.locator('.action-result').waitFor();
  const refused=exchanges.findLast(entry=>entry.request.operation_ref===`review.${item.action}`);
  assert.notEqual(refused.result.result_state,'success');
  assert.equal(refused.result.error.code,'reviewer_not_eligible_for_case_review');
  const stillPending=await accepted('case.summary',{case_ref:item.caseRef});assert.equal(stillPending.authority.reviews[0].status,'pending');
  await accepted('participant.role.add',{case_ref:item.caseRef,participant_ref:'participant:operator',role:'operation-reviewer'});
  await page.getByRole('button',{name:'Submit decision',exact:true}).click();
  await page.getByRole('dialog').waitFor({state:'hidden'});
  const resolved=await accepted('case.summary',{case_ref:item.caseRef});
  assert.equal(resolved.authority.reviews[0].status,{approve:'approved',deny:'denied',defer:'deferred'}[item.action]);
  assert.equal(cli('case','verify',item.caseRef).status,'ok');
  await page.screenshot({path:`${evidence}/review-${item.action}.png`});
 }
 dropAcknowledgement='case.create';
 await page.evaluate(()=>window.qualificationPlatform.commands.executeCommand('studio.case.create'));
 await page.getByRole('dialog',{name:'New Case'}).getByLabel('Case reference').fill('case:studio-unconfirmed');
 await page.getByRole('button',{name:'Create Case',exact:true}).click();
 await page.getByText('Confirmation was lost',{exact:true}).waitFor();
 assert.ok(await page.getByRole('dialog').evaluate(el=>{const box=el.getBoundingClientRect();return box.top>=0 && box.bottom<=innerHeight;}));
 await page.keyboard.press('Tab');
 assert.ok(await page.getByRole('dialog').evaluate(el=>el.contains(document.activeElement)));
 assert.equal(await page.getByRole('button',{name:'Create Case',exact:true}).isDisabled(),true);
 assert.equal(exchanges.filter(item=>item.request.operation_ref==='case.create' && item.request.input.case_ref==='case:studio-unconfirmed').length,1);
 assert.ok((await accepted('case.list')).cases.some(item=>item.case_ref==='case:studio-unconfirmed'));
 await page.screenshot({path:`${evidence}/acknowledgement-lost.png`});
 await page.getByRole('button',{name:'Close and inspect state',exact:true}).click();
 assert.deepEqual(errors,[]);
 console.log(JSON.stringify({result:'PASS',transport:'real Unix Host via test-only browser bridge',operations:catalog.operations.length,case_ref:created.case.case_ref,generations:{created:created.case.generation,beforeInput:before.case.generation,afterInput:after.case.generation,closed:closed.case.generation},reviews:reviewCases.map(item=>({case_ref:item.caseRef,review_ref:item.review.id,action:item.action})),proof:['New Case + Participant + authenticated identity','Workflow input refusal -> no optimistic mutation','Workflow input success -> generation+1 -> fresh projection','Settings actual capability catalog','Cancel + Close Case -> durable lifecycle','Review approve/deny/defer: ineligible refusal then authorized success','Lost acknowledgement after commit -> no resubmit','CLI canonical replay'],consoleErrors:0}));
} finally {
 await writeFile(`${evidence}/exchanges.json`,JSON.stringify(exchanges,null,2));
 await browser?.close();
 await writeFile(`${evidence}/runtime.log`,runtimeOutput);
 if(runtime && runtime.exitCode === null){const exit=once(runtime,'exit');runtime.kill('SIGTERM');await exit;}
 try {if(telemetry)cli('host','stop');} finally {await rm(home,{recursive:true,force:true});}
}
