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
const home = await mkdtemp(path.join(os.tmpdir(), 'yai-studio-policy-intake-'));
const evidence = process.env.STUDIO_EVIDENCE_DIR ?? '/tmp/yai-studio-policy-intake';
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
 const definition=JSON.parse(await readFile(path.resolve(import.meta.dirname,'../qualification/studio-product-vertical/policy.json'),'utf8'));
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

 const initial=await accepted('case.summary',{case_ref:caseRef});
 const importDoc=async(name,bytes)=>{
   await page.getByRole('button',{name:'Import policy documents',exact:true}).click();
   const form=page.getByRole('dialog',{name:'Import policy documents'});
   await form.getByLabel('Policy documents',{exact:true}).setInputFiles({name,mimeType:'application/octet-stream',buffer:Buffer.from(bytes)});
   await form.getByRole('button',{name:'Import candidates',exact:true}).click();
   return form;
 };
 // Extension and MIME are not policy classifiers; ordinary JSON refuses.
 let form=await importDoc('policy.json','{"ordinary":"document, not policy"}');
 await form.locator('.action-result').waitFor();
 assert.notEqual(exchanges.findLast(item=>item.request.operation_ref==='policy.ingest').result.result_state,'success');
 assert.equal((await accepted('case.summary',{case_ref:caseRef})).case.generation,initial.case.generation);
 await form.getByRole('button',{name:'Cancel',exact:true}).click();
 const source=JSON.stringify(definition);
 form=await importDoc('rules.md','```yai-policy-json\n'+source+'\n```\n');
 await form.waitFor({state:'hidden'});
 const ingested=exchanges.findLast(item=>item.request.operation_ref==='policy.ingest').result.data;
 const artifact=ingested.view.artifact.artifact_id;
 assert.equal(ingested.view.lifecycle,'candidate');
 assert.equal(ingested.view.artifact.policy_ir.rules.length,definition.rules.length);
 assert.equal((await accepted('case.summary',{case_ref:caseRef})).authority.policies.length,0);
 const panel=page.getByRole('article',{name:`Policy ${definition.policy_key}`});
 await panel.getByText('Rules returned by YAI',{exact:true}).waitFor();
 assert.equal(await panel.locator('.policy-rule').count(),definition.rules.length);
 for(const rule of definition.rules) await panel.getByText(rule.reason,{exact:true}).waitFor();
 const lifecycle=async(action,title)=>{
  await panel.getByRole('button',{name:title,exact:true}).click();
  const dialog=page.getByRole('dialog',{name:title});
  await dialog.getByLabel('Reason').fill(`Studio policy intake qualification: ${action}`);
  await dialog.getByRole('button',{name:title,exact:true}).click();
  await dialog.waitFor({state:'hidden'});
  const response=exchanges.findLast(item=>item.request.operation_ref===`policy.${action}`).result;
  assert.equal(response.result_state,'success',JSON.stringify(response));
  assert.equal(response.data.view.artifact.artifact_id,artifact);
  return response.data.view;
 };
 assert.equal((await lifecycle('validate','Validate candidate')).lifecycle,'validated');
 assert.equal((await lifecycle('publish','Publish policy')).lifecycle,'published');
 assert.equal((await accepted('case.summary',{case_ref:caseRef})).authority.policies.length,0,'publication must not bind');
 await panel.getByRole('button',{name:'Bind Policy',exact:true}).click();
 form=page.getByRole('dialog',{name:'Bind Policy'});
 assert.equal(await form.getByLabel('Published policy artifact').inputValue(),artifact);
 await form.getByLabel('Reason').fill('Explicit Case binding after inspection');
 await form.getByRole('button',{name:'Bind Policy',exact:true}).click();await form.waitFor({state:'hidden'});
 const bound=await accepted('case.summary',{case_ref:caseRef});assert.equal(bound.authority.policies[0].artifact_ref,artifact);
 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]) {
  await page.setViewportSize({width,height});await panel.scrollIntoViewIfNeeded();
  assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);
  await page.screenshot({path:`${evidence}/policy-intake-${width}x${height}.png`});
 }
 // Retire and revoke affect the real Tenant artifact; neither rewrites history.
 assert.equal((await lifecycle('retire','Retire policy')).lifecycle,'retired');
 assert.equal((await lifecycle('revoke','Revoke policy')).lifecycle,'revoked');
 const revoked=cli('policy','inspect',artifact);assert.match(JSON.stringify(revoked),/revoked/); assert.match(JSON.stringify(revoked),new RegExp(artifact));
 const other='case:studio-policy-refusal'; await accepted('case.create',{tenant_id:'tenant:studio-ui',case_ref:other});
 await accepted('participant.role.add',{case_ref:other,participant_ref:'participant:operator',role:'operator'}); await accepted('participant.principal.link',{case_ref:other,participant_ref:'participant:operator',principal_ref:'self'});
 const empty=await accepted('case.summary',{case_ref:other});
 const blocked=await call('policy.case.bind',{case_ref:other,artifact_ref:artifact,expected_generation:empty.case.generation,reason:'Refusal qualification'});
 assert.notEqual(blocked.result_state,'success');
 const after=await accepted('case.summary',{case_ref:caseRef});assert.deepEqual(after.authority.policies,bound.authority.policies);
 // Real lost acknowledgement after intake commit: no automatic duplicate send.
 dropAcknowledgement='policy.ingest';
 form=await importDoc('repeat.json',source);
 await form.getByText('Confirmation was lost',{exact:true}).waitFor();
 assert.equal(await form.getByRole('button',{name:'Import candidates',exact:true}).isDisabled(),true);
 const count=exchanges.filter(item=>item.request.operation_ref==='policy.ingest').length;
 await new Promise(resolve=>setTimeout(resolve,250));
 assert.equal(exchanges.filter(item=>item.request.operation_ref==='policy.ingest').length,count);
 assert.equal(cli('case','verify',caseRef).status,'ok');assert.deepEqual(errors,[]);
 console.log(JSON.stringify({result:'PASS',case_ref:caseRef,artifact,original_source:ingested.view.artifact.source_id,generations:{initial:initial.case.generation,bound:bound.case.generation,after:after.case.generation},proof:['ordinary JSON refusal without Case mutation','Markdown original -> backend typed rules','validate + publish without auto-binding','explicit generation-fenced binding','retire + revoke with durable lifecycle','revoked binding refused without Case mutation','lost import acknowledgement is not retried','4 viewport matrix','CLI canonical replay']}));
}finally{await writeFile(`${evidence}/exchanges.json`,JSON.stringify(exchanges,null,2));await browser?.close();try{if(telemetry)cli('host','stop');}finally{await rm(home,{recursive:true,force:true});}}
