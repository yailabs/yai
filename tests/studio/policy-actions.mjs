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
const home = await mkdtemp(path.join(os.tmpdir(), 'yai-studio-policy-'));
const evidence = process.env.STUDIO_EVIDENCE_DIR ?? '/tmp/yai-studio-policy';
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
 const publish=async version=>{const candidate={...definition,source_version:version,source_origin:{...definition.source_origin,source_uri:`qualification://studio-policy/${version}`}};const ingested=await accepted('policy.ingest',{tenant_id:'tenant:studio-ui',source_bytes:[...Buffer.from(JSON.stringify(candidate))]});const artifact=ingested.view.artifact.artifact_id;await accepted('policy.validate',{artifact_ref:artifact,reason:'Normal policy qualification setup'});await accepted('policy.publish',{artifact_ref:artifact,reason:'Normal policy qualification publication'});return artifact;};
 const first=await publish('1');
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
 const refresh=async()=>{await page.evaluate(()=>window.qualificationPlatform.commands.executeCommand('studio.case.refresh'));};
 const open=async(action,artifact)=>{await page.getByRole('button',{name:`${action} Policy`,exact:true}).click();const form=page.getByRole('dialog',{name:`${action} Policy`});if(artifact)await form.getByLabel('Published policy artifact').fill(artifact);await form.getByLabel('Reason').fill(`Studio ${action} Policy qualification`);return form;};
 const stale=async(form,action)=>{const before=await accepted('case.summary',{case_ref:caseRef});await accepted('participant.role.add',{case_ref:caseRef,participant_ref:'participant:operator',role:`qualified-${action.toLowerCase()}`});await form.getByRole('button',{name:`${action} Policy`,exact:true}).click();await form.locator('.action-result').waitFor();const result=exchanges.findLast(item=>item.request.operation_ref===`policy.case.${action.toLowerCase()}`).result;assert.equal(result.result_state,'stale',JSON.stringify(result));const after=await accepted('case.summary',{case_ref:caseRef});assert.equal(after.case.generation,before.case.generation+1);assert.deepEqual(after.authority.policies,before.authority.policies,'stale mutation changed policy');await page.screenshot({path:`${evidence}/${action.toLowerCase()}-stale.png`});await form.getByRole('button',{name:'Close and refresh Case',exact:true}).click();await page.waitForFunction(generation=>document.querySelector('.kernel-status')?.textContent.includes(`Generation ${generation}`),after.case.generation);return after;};
 const atBind=await stale(await open('Bind',first),'Bind');
 let form=await open('Bind',first);await form.getByRole('button',{name:'Bind Policy',exact:true}).click();await form.waitFor({state:'hidden'});
 let state=await accepted('case.summary',{case_ref:caseRef});assert.equal(state.case.generation,atBind.case.generation+1);assert.equal(state.authority.policies[0].artifact_ref,first);const binding=state.authority.policies[0];
 await page.locator('.authority-section').filter({has:page.getByText('Bound policies',{exact:true})}).getByRole('button',{name:new RegExp(definition.policy_key)}).click();
 const second=await publish('2');
 const atReplace=await stale(await open('Replace',second),'Replace');
 form=await open('Replace',second);await form.getByRole('button',{name:'Replace Policy',exact:true}).click();await form.waitFor({state:'hidden'});state=await accepted('case.summary',{case_ref:caseRef});assert.equal(state.case.generation,atReplace.case.generation+1);assert.equal(state.authority.policies.length,1);assert.equal(state.authority.policies[0].artifact_ref,second);assert.notEqual(state.authority.policies[0].id,binding.id);
 await page.locator('.authority-section').filter({has:page.getByText('Bound policies',{exact:true})}).getByRole('button',{name:new RegExp(definition.policy_key)}).click();
 const atUnbind=await stale(await open('Unbind'),'Unbind');
 form=await open('Unbind');
 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]){await page.setViewportSize({width,height});await page.screenshot({path:`${evidence}/unbind-${width}x${height}.png`});}
 await form.getByRole('button',{name:'Unbind Policy',exact:true}).click();await form.waitFor({state:'hidden'});state=await accepted('case.summary',{case_ref:caseRef});assert.equal(state.case.generation,atUnbind.case.generation+1);assert.equal(state.authority.policies.length,0);assert.equal(cli('case','verify',caseRef).status,'ok');assert.deepEqual(errors,[]);
 console.log(JSON.stringify({result:'PASS',transport:'real Unix Host via browser bridge',case_ref:caseRef,artifacts:[first,second],final_generation:state.case.generation,proof:['Bind exact published artifact','Replace same-lineage artifact','Unbind exact current binding','All three stale generations refused without policy mutation','Fresh projection after each commit','CLI canonical replay']}));
}finally{await writeFile(`${evidence}/exchanges.json`,JSON.stringify(exchanges,null,2));await browser?.close();try{if(telemetry)cli('host','stop');}finally{await rm(home,{recursive:true,force:true});}}
