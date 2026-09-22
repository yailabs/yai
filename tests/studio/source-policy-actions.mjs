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
const home = await mkdtemp(path.join(os.tmpdir(), 'yai-studio-source-policy-'));
const evidence = process.env.STUDIO_EVIDENCE_DIR ?? '/tmp/yai-studio-source-policy';
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
 const root=path.join(home,'documents');await mkdir(root);await writeFile(path.join(root,'policy.json'),await readFile(path.resolve(import.meta.dirname,'../qualification/studio-product-vertical/policy.json')));await writeFile(path.join(root,'document.md'),'# Source qualification\nActual governed documentary content.\n');
 const resource={schema:'yai.resource_definition.v1',attachment_id:'resource:documents',policy_owner:'participant:operator',participant_ids:['participant:operator'],operations:['discover','admit_content','content_read'],read_prefixes:['policy.json','document.md'],names:[],max_output_bytes:8192,max_items:4,address:{kind:'discovery',root}};
 const perimeter={schema:'yai.source_perimeter.v1',name:'source-policy',participant:'participant:operator',resources:[resource],sources:[{name:'policy-candidate',resource:resource.attachment_id,roles:['policy'],action:{action:'discover',path:'policy.json'},media_type:'application/json',bootstrap_policy:true},{name:'document',resource:resource.attachment_id,roles:['knowledge'],action:{action:'discover',path:'document.md'},media_type:'text/markdown',bootstrap_policy:false}]};
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
 await page.getByRole('button',{name:/Studio Policy Actions/}).click();await page.locator('.workbench-kernel').waitFor();await page.keyboard.press('Control+j');await page.locator('.live-rail button[aria-label="Authority"]').click();


 const before=await accepted('case.summary',{case_ref:caseRef});assert.equal(before.authority.policies.length,0);const policy=before.environment.sources.find(item=>item.label==='policy-candidate');
 await page.locator('.live-sidebar').getByRole('button',{name:/policy-candidate/}).click();await page.locator('.source-surface').waitFor();
 await page.getByRole('button',{name:'Publish Source policy…',exact:true}).click();let form=page.getByRole('dialog',{name:'Publish Source policy'});await form.getByLabel('Review reason').fill('Review exact bounded qualification policy');await form.getByRole('button',{name:'Publish and bind',exact:true}).click();await form.waitFor({state:'hidden'});
 const published=exchanges.findLast(item=>item.request.operation_ref==='source.publish').result;assert.equal(published.result_state,'success',JSON.stringify(published));assert.equal((await accepted('case.summary',{case_ref:caseRef})).authority.policies.length,1);
 cli('case','sources','acquire',caseRef);await page.getByRole('button',{name:'Refresh Case',exact:true}).click();
 await page.locator('.live-rail button[aria-label="Environment"]').click();assert.equal(await page.locator('.environment-files').getByRole('button',{name:'policy.json',exact:true}).count(),0);await page.locator('.environment-files').getByRole('button',{name:'document.md',exact:true}).waitFor();
 const resources=page.locator('.live-sidebar details.sidebar-group > summary').filter({hasText:/^Resources/}).locator('..');await resources.getByRole('button').first().click();await page.locator('.resource-requestability').getByText('Requestable',{exact:true}).first().waitFor();
 const capability=exchanges.findLast(item=>item.request.operation_ref==='case.capabilities').result;assert.equal(capability.result_state,'success',JSON.stringify(capability));assert.ok(capability.data.entries.some(item=>item.resource.attachment_id===resource.attachment_id));
 const hidden=await call('case.capabilities',{case_ref:caseRef,participant_ref:'participant:missing'});assert.notEqual(hidden.result_state,'success');
 const ordinary=(await accepted('case.summary',{case_ref:caseRef})).environment.sources.find(item=>item.label==='document');const preRefusal=(await accepted('case.summary',{case_ref:caseRef})).case.generation;const refused=await call('source.publish',{case_ref:caseRef,source_ref:ordinary.id,reason:'Knowledge is not policy'});assert.notEqual(refused.result_state,'success');assert.equal((await accepted('case.summary',{case_ref:caseRef})).case.generation,preRefusal);
 assert.equal(cli('case','verify',caseRef).status,'ok');assert.deepEqual(errors,[]);
 console.log(JSON.stringify({result:'PASS',case_ref:caseRef,source:policy.id,generation:preRefusal,proof:['Real bootstrap acquisition -> explicit UI source.publish','Policy Source in Authority, ordinary document in Environment','Current Participant-scoped Resource requestability','Unknown Participant refused','Knowledge-only policy publication refused without mutation','CLI replay']}));

}finally{await writeFile(`${evidence}/exchanges.json`,JSON.stringify(exchanges,null,2));await browser?.close();try{if(telemetry)cli('host','stop');}finally{await rm(home,{recursive:true,force:true});}}
