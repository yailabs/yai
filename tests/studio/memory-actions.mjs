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
const home = await mkdtemp(path.join(os.tmpdir(), 'yai-studio-memory-'));
const evidence = process.env.STUDIO_EVIDENCE_DIR ?? '/tmp/yai-studio-memory';
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

 const artifact=await publish('1');const initial=await accepted('case.summary',{case_ref:caseRef});
 await accepted('policy.case.bind',{case_ref:caseRef,artifact_ref:artifact,expected_generation:initial.case.generation,reason:'Qualified bounded memory source admission'});
 const root=path.join(home,'documents');await mkdir(root);await writeFile(path.join(root,'evidence.md'),'# Memory qualification\n\nThe intake property keeps policy candidates separate from effective authority.\n\nDocumentary evidence remains source-stated.\n');
 const perimeter={schema:'yai.source_perimeter.v1',name:'memory',participant:'participant:operator',resources:[{schema:'yai.resource_definition.v1',attachment_id:'resource:documents',policy_owner:'participant:operator',participant_ids:['participant:operator'],operations:['discover','admit_content','content_read'],read_prefixes:['evidence.md'],names:[],max_output_bytes:4096,max_items:4,address:{kind:'discovery',root}}],sources:[{name:'memory-evidence',resource:'resource:documents',roles:['knowledge'],action:{action:'discover',path:'evidence.md'},media_type:'text/markdown',bootstrap_policy:false}]};
 const file=path.join(home,'perimeter.json');await writeFile(file,JSON.stringify(perimeter));cli('case','sources','declare',caseRef,'--file',file);cli('case','sources','acquire',caseRef);
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
 await page.getByRole('button',{name:/Studio Policy Actions/}).click();await page.locator('.workbench-kernel').waitFor();await page.keyboard.press('Control+j');await page.locator('.live-rail button[aria-label="Memory"]').click();

 await page.locator('.live-surface').getByRole('button',{name:/Recall/}).click();
 await page.getByRole('button',{name:'Recall for a task',exact:true}).click();
 let form=page.getByRole('dialog',{name:'Recall for a task'});
 await form.getByLabel('Task / query').fill('intake policy');
 await form.getByRole('button',{name:'Run Recall',exact:true}).click();await form.waitFor({state:'hidden'});
 let response=exchanges.findLast(item=>item.request.operation_ref==='semantic.recall').result;
 assert.equal(response.result_state,'success',JSON.stringify(response));assert.ok(response.data.trace.documentary.units.some(item=>item.unit.text.includes('intake property')));
 await page.getByText('The intake property keeps policy candidates separate from effective authority.',{exact:true}).waitFor();
 await page.locator('.live-sidebar').getByRole('button',{name:/Working State/}).click();
 const compile=async()=>{await page.getByRole('button',{name:'Compile Working State',exact:true}).click();const dialog=page.getByRole('dialog',{name:'Compile Working State'});await dialog.getByLabel('Task / query').fill('intake policy');await dialog.getByRole('button',{name:'Compile',exact:true}).click();return dialog;};
 form=await compile();await form.locator('.action-result').waitFor();
 response=exchanges.findLast(item=>item.request.operation_ref==='semantic.working_state.compile').result;assert.notEqual(response.result_state,'success');
 const before=await accepted('case.summary',{case_ref:caseRef});
 await form.getByRole('button',{name:'Cancel',exact:true}).click();
 await page.getByRole('button',{name:'Admit model-context view…',exact:true}).click();form=page.getByRole('dialog',{name:'Admit model-context view'});await form.getByRole('button',{name:'Admit view',exact:true}).click();await form.waitFor({state:'hidden'});
 assert.equal((await accepted('case.summary',{case_ref:caseRef})).case.generation,before.case.generation+1);
 form=await compile();await form.waitFor({state:'hidden'});
 response=exchanges.findLast(item=>item.request.operation_ref==='semantic.working_state.compile').result;assert.equal(response.result_state,'success',JSON.stringify(response));
 const first=response.data.working_state;assert.equal(first.case_id,caseRef);assert.ok(first.entries.length);
 await page.getByRole('button',{name:'Refresh exact task',exact:true}).click();
 await page.waitForFunction(()=>!document.querySelector('.memory-operational button')?.disabled);
 await new Promise(resolve=>setTimeout(resolve,100));
 response=exchanges.findLast(item=>item.request.operation_ref==='semantic.working_state.refresh').result;assert.equal(response.result_state,'success',JSON.stringify(response));
 const pages=page.getByRole('button',{name:'Defer group',exact:true});
 assert.ok(await pages.count()>0,'Real acquired evidence must expose a W4 group');
 await pages.first().click();await page.getByRole('button',{name:'Expand group',exact:true}).first().waitFor();
 assert.equal(exchanges.findLast(item=>item.request.operation_ref==='semantic.working_state.page').result.result_state,'success');
 await page.getByRole('button',{name:'Expand group',exact:true}).first().click();
 await page.getByRole('button',{name:'Defer group',exact:true}).first().waitFor();
 assert.equal(exchanges.findLast(item=>item.request.operation_ref==='semantic.working_state.page').result.result_state,'success');
 await page.getByRole('button',{name:'Defer group',exact:true}).first().click();
 await page.getByRole('button',{name:'Expand group',exact:true}).first().waitFor();
 await page.getByRole('button',{name:'Defer group',exact:true}).last().click();
 await page.waitForFunction(()=>[...document.querySelectorAll('button')].filter(b=>b.textContent==='Expand group').length===2);
 await page.getByRole('button',{name:'Refresh exact task',exact:true}).click();
 await page.getByRole('button',{name:'Prepare frontier',exact:true}).click();form=page.getByRole('dialog',{name:'Prepare frontier'});await form.getByRole('button',{name:'Prepare',exact:true}).click();await form.waitFor({state:'hidden'});
 const frontierCall=exchanges.findLast(item=>item.request.operation_ref==='decision.frontier.prepare');assert.equal(frontierCall.result.result_state,'success',JSON.stringify(frontierCall.result));assert.ok(frontierCall.result.data.frontier.candidates.length>=2);
 await page.getByRole('button',{name:'Prepare decision request',exact:true}).click();form=page.getByRole('dialog',{name:'Prepare decision request'});await form.getByRole('button',{name:'Prepare',exact:true}).click();await form.waitFor({state:'hidden'});
 const decision=exchanges.findLast(item=>item.request.operation_ref==='decision.request.prepare').result;assert.equal(decision.result_state,'success',JSON.stringify(decision));assert.deepEqual(decision.data.candidates.map(c=>c.candidate_id),frontierCall.result.data.frontier.candidates.map(c=>c.candidate.candidate_id));
 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]){await page.setViewportSize({width,height});await page.screenshot({path:`${evidence}/memory-${width}x${height}.png`});}
 const after=await accepted('case.summary',{case_ref:caseRef});assert.equal(after.case.generation,before.case.generation+1,'Recall/W/page/refresh must not mutate the Case');
 await accepted('participant.role.add',{case_ref:caseRef,participant_ref:'participant:operator',role:'qualification-observer'});
 const stale=await call('decision.frontier.prepare',frontierCall.request.input);assert.notEqual(stale.result_state,'success','Changed Case must refuse the prior Working State');
 await page.getByRole('button',{name:'Refresh Case',exact:true}).click();await page.waitForFunction(()=>[...document.querySelectorAll('button')].find(b=>b.textContent==='Prepare frontier')?.disabled);
 assert.equal(cli('case','verify',caseRef).status,'ok');assert.deepEqual(errors,[]);
 console.log(JSON.stringify({result:'PASS',case_ref:caseRef,working_state:first.working_state_id,generation:after.case.generation,proof:['Real documentary acquisition -> Recall exact text','W refuses absent admitted view','Explicit view admission only canonical change','W compile + refresh','Explicit W4 page-out and page-in','Case generation unchanged by derived operations','4 viewport matrix','Typed Decision Frontier and request preserve exact candidates','Stale Working State refused','CLI replay']}));
}finally{await writeFile(`${evidence}/exchanges.json`,JSON.stringify(exchanges,null,2));await browser?.close();try{if(telemetry)cli('host','stop');}finally{await rm(home,{recursive:true,force:true});}}
