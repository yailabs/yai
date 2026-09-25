// UI calls the real published Host over Unix IPC through a test-only Tauri bridge.
// Only a freshly created temporary YAI_HOME is mutated; never operator state.
import { createRequire } from 'node:module';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import net from 'node:net';
import os from 'node:os';
import path from 'node:path';
const require = createRequire(path.resolve(import.meta.dirname, '../../studio/package.json'));
const { chromium } = require('playwright-core');
const binary = process.env.YAI_STUDIO_TEST_BINARY;
assert.ok(binary, 'YAI_STUDIO_TEST_BINARY must name a built published CLI/Host');
const home = await mkdtemp(path.join(os.tmpdir(), 'yai-studio-work-'));
const evidence = process.env.STUDIO_EVIDENCE_DIR ?? '/tmp/yai-studio-work';
await mkdir(evidence, {recursive:true});
const cli = (...args) => JSON.parse(execFileSync(binary, [...args, '--json'], {env:{...process.env, YAI_HOME:home}, encoding:'utf8', timeout:30000}));
let telemetry, serial=0, browser, dropAcknowledgement;
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
 for(const ref of ['case:handoff-target','case:handoff-decline-source']) {await accepted('case.create',{tenant_id:'tenant:studio-ui',case_ref:ref});await accepted('participant.role.add',{case_ref:ref,participant_ref:'participant:operator',role:'operator'});await accepted('participant.principal.link',{case_ref:ref,participant_ref:'participant:operator',principal_ref:'self'});}
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
 await page.getByRole('button',{name:/Studio Policy Actions/}).click();await page.locator('.workbench-kernel').waitFor();await page.keyboard.press('Control+j');await page.locator('.live-rail button[aria-label="Work"]').click();


 const open=async name=>{if(name.includes('Handoff'))await page.getByRole('tab',{name:'Handoffs',exact:true}).click();await page.getByRole('button',{name,exact:true}).click();return page.getByRole('dialog',{name});};
 let form=await open('Define checkpoint Workflow');await form.getByLabel('Name',{exact:true}).fill('Operational checkpoints');await form.getByLabel('Workflow key').fill('studio-operational-checkpoints');await form.getByLabel('Checkpoints, one prompt per line').fill('Inspect source closure\nRecord product acceptance');await form.getByRole('button',{name:'Retain definition',exact:true}).click();await form.waitFor({state:'hidden'});
 let response=exchanges.findLast(item=>item.request.operation_ref==='workflow.define').result;assert.equal(response.result_state,'success',JSON.stringify(response));const definition=response.data;assert.equal(definition.nodes.length,2);
 assert.equal((await accepted('case.summary',{case_ref:caseRef})).work.nodes.length,0,'Definition must not bind implicitly');
 form=await open('Bind Workflow');assert.equal(await form.getByLabel('Definition reference').inputValue(),definition.workflow_definition_id);await form.getByRole('button',{name:'Bind definition',exact:true}).click();await form.waitFor({state:'hidden'});
 assert.equal((await accepted('case.summary',{case_ref:caseRef})).work.nodes.length,2);
 await page.locator('.workflow-node-row').filter({hasText:'step-1'}).getByRole('button',{name:'Provide input'}).click();form=page.getByRole('dialog',{name:'Workflow input'});await form.getByLabel('Input',{exact:true}).fill('Confirmed source closure');await form.getByRole('button',{name:'Record input',exact:true}).click();await form.waitFor({state:'hidden'});assert.equal((await accepted('case.summary',{case_ref:caseRef})).work.nodes.find(n=>n.node_id==='step-1').posture,'satisfied');
 const propose=async id=>{let dialog=await open('Propose Workflow checkpoint');await dialog.getByLabel('New node ID').fill(id);await dialog.getByLabel('Prompt',{exact:true}).fill('Verify '+id);await dialog.locator('select[name=after]').selectOption('step-2');await dialog.getByRole('button',{name:'Propose checkpoint',exact:true}).click();await dialog.waitFor({state:'hidden'});const result=exchanges.findLast(item=>item.request.operation_ref==='workflow.patch.propose').result;assert.equal(result.result_state,'success',JSON.stringify(result));return result.data.transition.payload.data.patch.patch_id;};
 const patchA=await propose('additional-a');const patchB=await propose('additional-b');
 form=await open('Adopt Workflow patch');await form.getByLabel('Patch reference').fill(patchA);await form.getByRole('button',{name:'Adopt patch',exact:true}).click();await form.waitFor({state:'hidden'});
 const patched=await accepted('case.summary',{case_ref:caseRef});assert.ok(patched.work.nodes.some(n=>n.node_id==='additional-a'));assert.ok(patched.work.edges.some(e=>e.from==='step-2'&&e.to==='additional-a'),'Adopted edge must be projected');assert.equal(patched.work.effective_nodes.find(n=>n.node_id==='additional-a').node.prompt,'Verify additional-a');assert.deepEqual(cli('workflow','status','--case',caseRef).data.value.effective_nodes,patched.work.effective_nodes);assert.deepEqual(cli('workflow','status','--case',caseRef).data.value.effective_edges.map(({from,to})=>({from,to})),patched.work.edges.map(({from,to})=>({from,to})));assert.equal((await call('case.summary',{case_ref:'case:hidden-workflow'})).result_state,'unauthorized');
 await page.locator('.workflow-node-row').filter({hasText:'additional-a'}).getByRole('button',{name:'Provide input'}).click();form=page.getByRole('dialog',{name:'Workflow input'});await form.getByText('Verify additional-a',{exact:true}).waitFor();await form.getByText('Maximum 4096 UTF-8 bytes, validated by YAI.',{exact:true}).waitFor();await form.getByRole('button',{name:'Cancel',exact:true}).click();await page.locator('.live-rail button[aria-label="Overview"]').click();await page.getByRole('tablist',{name:'Overview sections'}).getByRole('tab',{name:'Workflow',exact:true}).click();await page.locator('.overview-story-steps').getByText('Verify additional-a',{exact:true}).waitFor();await page.locator('.live-rail button[aria-label="Work"]').click();await page.getByRole('tab',{name:'Executions',exact:true}).click();await page.locator('.live-sidebar-content[data-view-container="Work"]').getByRole('button',{name:/additional-a/}).click();assert.equal(await page.getByRole('tab',{name:'Workflow',exact:true}).getAttribute('aria-selected'),'true');await page.getByRole('tab',{name:'Handoffs',exact:true}).click();await page.locator('.live-rail button[aria-label="Overview"]').click();await page.getByRole('button',{name:'Open workflow and dependencies',exact:true}).click();assert.equal(await page.getByRole('tab',{name:'Workflow',exact:true}).getAttribute('aria-selected'),'true');
 form=await open('Adopt Workflow patch');await form.getByLabel('Patch reference').fill(patchB);await form.getByRole('button',{name:'Adopt patch',exact:true}).click();await form.locator('.action-result').waitFor();assert.notEqual(exchanges.findLast(item=>item.request.operation_ref==='workflow.patch.adopt').result.result_state,'success');assert.equal((await accepted('case.summary',{case_ref:caseRef})).case.generation,patched.case.generation);await form.getByRole('button',{name:'Cancel',exact:true}).click();
 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]){await page.setViewportSize({width,height});await page.screenshot({path:`${evidence}/work-${width}x${height}.png`});}
 const visitedWorkCases=new Set(['Studio Policy Actions']);
 const switchCase=async label=>{await page.locator('.titlebar-case').click();await page.getByRole('button',{name:new RegExp(label)}).click();await page.locator(`.workbench-kernel[data-case-ref="case:${label.toLowerCase().replaceAll(' ','-')}"]`).waitFor();await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));await page.locator('.live-rail button[aria-label="Work"]').click();if(!visitedWorkCases.has(label)){assert.equal(await page.getByRole('tab',{name:'Workflow',exact:true}).getAttribute('aria-selected'),'true','New Case must not inherit previous Case section');visitedWorkCases.add(label);}await page.getByRole('tab',{name:'Handoffs',exact:true}).click();};
 form=await open('Offer Handoff');await form.getByLabel('Target Case reference').fill('case:handoff-target');await form.getByLabel('Request',{exact:true}).fill('Inspect the already qualified operational evidence');dropAcknowledgement='handoff.offer';await form.getByRole('button',{name:'Offer',exact:true}).click();await form.getByText('Confirmation was lost',{exact:true}).waitFor();assert.equal(await form.getByRole('button',{name:'Offer',exact:true}).isDisabled(),true);const offer=exchanges.findLast(item=>item.request.operation_ref==='handoff.offer').result.data.transition.payload.data.offer;assert.equal(exchanges.filter(item=>item.request.operation_ref==='handoff.offer').length,1);await form.getByRole('button',{name:'Close and inspect state',exact:true}).click();
 const inbox=await accepted('handoff.pending',{case_ref:'case:handoff-target'});
 assert.deepEqual(cli('case','handoff','pending','case:handoff-target').data.value,inbox.offers);
 const exact=await accepted('handoff.inspect',{case_ref:'case:handoff-target',handoff_ref:offer.handoff_id});
 assert.deepEqual(exact.offer,offer);assert.equal(exact.acceptance,null);
 assert.deepEqual(cli('case','handoff','show','case:handoff-target','--handoff',offer.handoff_id).data.value,exact);
 assert.equal((await call('handoff.inspect',{case_ref:'case:handoff-decline-source',handoff_ref:offer.handoff_id})).result_state,'unauthorized');
 await switchCase('Handoff Target');
 await page.locator('.handoff-inbox-row').filter({hasText:'Inspect the already qualified operational evidence'}).click();
 const handoffDetail=page.getByRole('region',{name:'Selected Handoff'});await handoffDetail.getByText('Inspect the already qualified operational evidence',{exact:true}).waitFor();await page.getByRole('tab',{name:'Activity',exact:true}).click();assert.equal(await handoffDetail.isVisible(),false);await page.getByRole('tab',{name:'Handoffs',exact:true}).click();await handoffDetail.getByText('Inspect the already qualified operational evidence',{exact:true}).waitFor();await page.getByRole('tab',{name:'Handoffs',exact:true}).press('Home');assert.equal(await page.getByRole('tab',{name:'Workflow',exact:true}).getAttribute('aria-selected'),'true');await page.getByRole('tab',{name:'Workflow',exact:true}).press('End');assert.equal(await page.getByRole('tab',{name:'Activity',exact:true}).getAttribute('aria-selected'),'true');await page.getByRole('tab',{name:'Handoffs',exact:true}).click();
 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]){await page.setViewportSize({width,height});await handoffDetail.scrollIntoViewIfNeeded();await page.screenshot({path:`${evidence}/handoff-inbox-${width}x${height}.png`});}
 form=await open('Accept Handoff');assert.equal(await form.getByLabel('Handoff reference').inputValue(),offer.handoff_id);assert.equal(await form.getByLabel('Source Case reference').inputValue(),caseRef);await form.getByRole('button',{name:'Accept',exact:true}).click();await form.waitFor({state:'hidden'});
 form=await open('Record Handoff result');await form.getByLabel('Handoff reference').fill(offer.handoff_id);await form.getByLabel('Result',{exact:true}).fill('Evidence reviewed through governed product operations');await form.getByRole('button',{name:'Record result',exact:true}).click();await form.waitFor({state:'hidden'});
 assert.deepEqual((await accepted('handoff.pending',{case_ref:'case:handoff-target'})).offers,[]);
 const completedHandoff=await accepted('handoff.inspect',{case_ref:'case:handoff-target',handoff_ref:offer.handoff_id});
 assert.equal(completedHandoff.result.result.value,'Evidence reviewed through governed product operations');
 await handoffDetail.getByText('Evidence reviewed through governed product operations',{exact:true}).waitFor();
 assert.deepEqual(cli('case','handoff','show','case:handoff-target','--handoff',offer.handoff_id).data.value,completedHandoff);
 await switchCase('Studio Policy Actions');form=await open('Reconcile Handoff');await form.getByLabel('Handoff reference').fill(offer.handoff_id);await form.getByRole('button',{name:'Reconcile',exact:true}).click();await form.waitFor({state:'hidden'});
 assert.equal(exchanges.findLast(item=>item.request.operation_ref==='handoff.reconcile').result.result_state,'success');
 await page.getByText('Inspect by exact reference',{exact:true}).click();await page.getByLabel('Inspect retained Handoff').fill(offer.handoff_id);await page.getByRole('button',{name:'Inspect Handoff',exact:true}).click();
 await handoffDetail.getByText('Reconciled outcome · succeeded',{exact:true}).waitFor();
 telemetry=cli('host','restart').data.value;await page.getByRole('button',{name:'Refresh Handoffs',exact:true}).click();await handoffDetail.getByText('Evidence reviewed through governed product operations',{exact:true}).waitFor();
 await switchCase('Handoff Decline Source');form=await open('Offer Handoff');await form.getByLabel('Target Case reference').fill('case:handoff-target');await form.getByLabel('Request',{exact:true}).fill('Optional second request');await form.getByRole('button',{name:'Offer',exact:true}).click();await form.waitFor({state:'hidden'});const declinedOffer=exchanges.findLast(item=>item.request.operation_ref==='handoff.offer').result.data.transition.payload.data.offer;
 await switchCase('Handoff Target');await page.locator('.handoff-inbox-row').filter({hasText:'Optional second request'}).click();await handoffDetail.getByText('Optional second request',{exact:true}).waitFor();form=await open('Decline Handoff');assert.equal(await form.getByLabel('Handoff reference').inputValue(),declinedOffer.handoff_id);assert.equal(await form.getByLabel('Source Case reference').inputValue(),'case:handoff-decline-source');await form.getByLabel('Reason',{exact:true}).fill('Not selected for this qualification');await form.getByRole('button',{name:'Decline',exact:true}).click();await form.waitFor({state:'hidden'});assert.equal(exchanges.findLast(item=>item.request.operation_ref==='handoff.decline').result.result_state,'success');
 for(const ref of [caseRef,'case:handoff-target','case:handoff-decline-source'])assert.equal(cli('case','verify',ref).status,'ok');assert.deepEqual(errors,[]);
 console.log(JSON.stringify({result:'PASS',case_ref:caseRef,definition:definition.workflow_definition_id,patch:patchA,handoff:offer.handoff_id,proof:['Define/bind two human checkpoints','Input progression','Patch adoption, exact effective edge/prompt, Overview and input dialog; CLI topology parity; hidden Case refusal','Stale topology refused without mutation','Offer acknowledgement loss never redispatches','Accept/result/reconcile cross-Case','Independent decline','Typed pending/inspect and CLI equality; unrelated Case refused; Case-local result versus explicit reconciliation','Incoming inbox selection and exact Accept/Decline prefill; readable request/result','Work section keyboard navigation, mounted Handoff selection, sidebar/Overview Workflow routing; 4 viewport matrix','CLI replay all Cases']}));

}catch(error){await browser?.contexts()[0]?.pages()[0]?.screenshot({path:`${evidence}/failure.png`});throw error;}finally{await writeFile(`${evidence}/exchanges.json`,JSON.stringify(exchanges,null,2));await browser?.close();try{if(telemetry)cli('host','stop');}finally{await rm(home,{recursive:true,force:true});}}
