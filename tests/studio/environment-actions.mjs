// UI calls the real published Host over Unix IPC through a test-only Tauri bridge.
// Only a freshly created temporary YAI_HOME is mutated; never operator state.
import { createRequire } from 'node:module';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import net from 'node:net';
import {createServer} from 'node:http';
import os from 'node:os';
import path from 'node:path';
const require = createRequire(path.resolve(import.meta.dirname, '../../studio/package.json'));
const { chromium } = require('playwright-core');
const binary = process.env.YAI_STUDIO_TEST_BINARY;
assert.ok(binary, 'YAI_STUDIO_TEST_BINARY must name a built published CLI/Host');
const home = await mkdtemp(path.join(os.tmpdir(), 'yai-studio-environment-'));
const evidence = process.env.STUDIO_EVIDENCE_DIR ?? '/tmp/yai-studio-environment';
await mkdir(evidence, {recursive:true});
const cli = (...args) => JSON.parse(execFileSync(binary, [...args, '--json'], {env:{...process.env, YAI_HOME:home}, encoding:'utf8', timeout:30000}));
let telemetry, serial=0, browser, dropAcknowledgement;
const exchanges = [];
let networkCalls=0;const server=createServer((_request,response)=>{networkCalls++;response.end('not requested');});
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
 const caseRef='case:studio-environment-actions';
 await accepted('case.create',{tenant_id:'tenant:studio-ui',case_ref:caseRef});
 await accepted('participant.role.add',{case_ref:caseRef,participant_ref:'participant:operator',role:'operator'});
 await accepted('participant.principal.link',{case_ref:caseRef,participant_ref:'participant:operator',principal_ref:'self'});
 const root=path.join(home,'documents');await mkdir(root);await writeFile(path.join(root,'existing.md'),'# Existing qualified Source\n');await writeFile(path.join(root,'new.md'),'# New retained candidate\n');
 const perimeter={schema:'yai.source_perimeter.v1',name:'documents',participant:'participant:operator',resources:[{schema:'yai.resource_definition.v1',attachment_id:'resource:documents',policy_owner:'participant:operator',participant_ids:['participant:operator'],operations:['discover','admit_content','content_read'],read_prefixes:['existing.md','new.md'],names:[],max_output_bytes:4096,max_items:4,address:{kind:'discovery',root}}],sources:[{name:'existing-document',resource:'resource:documents',roles:['knowledge'],action:{action:'discover',path:'existing.md'},media_type:'text/markdown',bootstrap_policy:false}]};
 await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
 execFileSync('python3',['-c','import sqlite3,sys; c=sqlite3.connect(sys.argv[1]); c.execute("create table items (name text)"); c.close()',path.join(root,'data.sqlite')]);
 const resource=(id,address,operations,names)=>({schema:'yai.resource_definition.v1',attachment_id:id,policy_owner:'participant:operator',participant_ids:['participant:operator'],operations,read_prefixes:[],names,max_output_bytes:4096,max_items:4,address});
 perimeter.resources.push(resource('resource:database',{kind:'sqlite',root,path:'data.sqlite',queries:{items:'SELECT name FROM items'}},['database_query'],['items']));
 perimeter.resources.push(resource('resource:http',{kind:'http_service',endpoint:{endpoint:`http://127.0.0.1:${server.address().port}`,allowed_ip_addresses:['127.0.0.1'],credential_ref:null},paths:{catalog:'catalog'}},['http_fetch'],['catalog']));
 const perimeterFile=path.join(home,'perimeter.json');await writeFile(perimeterFile,JSON.stringify(perimeter));cli('case','sources','declare',caseRef,'--file',perimeterFile);
 const before=await accepted('case.summary',{case_ref:caseRef});assert.equal(before.environment.sources.length,1);
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
 await page.getByRole('button',{name:/Studio Environment Actions/}).click();
 await page.locator('.workbench-kernel').waitFor();await page.keyboard.press('Control+j');
 await page.locator('.live-rail button[aria-label="Environment"]').click();
 const declare=async path=>{
  await page.getByRole('button',{name:'Declare Source',exact:true}).click();
  const form=page.getByRole('dialog',{name:'Declare Source'});
  await form.getByLabel('Name',{exact:true}).fill('new-document');await form.getByLabel('Perimeter label').fill('documents');
  await form.getByLabel('Relative path').fill(path);await form.getByLabel('Qualified media type').fill('text/markdown');
  return form;
 };
 let form=await declare('../outside.md');await form.getByRole('button',{name:'Declare Source',exact:true}).click();await form.locator('.action-result').waitFor();
 assert.notEqual(exchanges.findLast(item=>item.request.operation_ref==='source.declare').result.result_state,'success');
 assert.equal((await accepted('case.summary',{case_ref:caseRef})).case.generation,before.case.generation);
 await form.getByLabel('Relative path').fill('new.md');await form.getByRole('button',{name:'Declare Source',exact:true}).click();await form.waitFor({state:'hidden'});
 await page.locator('.live-page').getByRole('button',{name:/new-document/}).waitFor();
 const declared=await accepted('case.summary',{case_ref:caseRef});assert.equal(declared.case.generation,before.case.generation+1);assert.equal(declared.environment.sources.length,2);assert.equal(declared.environment.files.length,0,'declaration was mistaken for acquisition');
 const source=declared.environment.sources.find(item=>item.label==='new-document');assert.ok(source);
 form=await declare('new.md');await form.getByRole('button',{name:'Declare Source',exact:true}).click();await form.waitFor({state:'hidden'});
 assert.equal((await accepted('case.summary',{case_ref:caseRef})).case.generation,declared.case.generation,'identical declaration duplicated canonical state');
 form=await declare('existing.md');await form.getByRole('button',{name:'Declare Source',exact:true}).click();await form.locator('.action-result').waitFor();assert.equal(exchanges.findLast(item=>item.request.operation_ref==='source.declare').result.error.code,'source_declaration_identity_collision');await form.getByRole('button',{name:'Cancel',exact:true}).click();
 await page.locator('.live-page').getByRole('button',{name:/new-document/}).click();await page.getByRole('button',{name:'Revoke Source',exact:true}).click();
 form=page.getByRole('dialog',{name:'Revoke Source'});await form.getByLabel('Reason').fill('Disposable governed source qualification.');
 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]) {await page.setViewportSize({width,height});await page.screenshot({path:`${evidence}/revoke-${width}x${height}.png`});assert.ok(await form.evaluate(el=>{const r=el.getBoundingClientRect();return r.top>=0 && r.bottom<=innerHeight;}));}
 await form.getByRole('button',{name:'Revoke Source',exact:true}).click();await form.waitFor({state:'hidden'});
 await page.waitForFunction(()=>document.querySelector('.source-surface')?.innerText.includes('revoked'));
 assert.equal(await page.getByRole('button',{name:'Revoke Source',exact:true}).isDisabled(),true);
 const revoked=await accepted('case.summary',{case_ref:caseRef});assert.equal(revoked.case.generation,declared.case.generation+1);assert.equal(revoked.environment.sources.find(item=>item.id===source.id).posture,'revoked');assert.equal(revoked.environment.sources.length,2);
 const refusal=await call('source.revoke',{case_ref:caseRef,source_ref:'case-source:absent',reason:'No fabricated Source'});assert.notEqual(refusal.result_state,'success');assert.equal((await accepted('case.summary',{case_ref:caseRef})).case.generation,revoked.case.generation);
 await page.locator('.live-rail button[aria-label="Environment"]').click();
 for(const [resourceRef,name,label] of [['resource:database','items','Qualified query name'],['resource:http','catalog','Qualified request name']]) {
  await page.getByRole('button',{name:'Declare Source',exact:true}).click();form=page.getByRole('dialog',{name:'Declare Source'});
  await form.getByLabel('Name',{exact:true}).fill(`named-${name}`);await form.getByLabel('Perimeter label').fill('documents');await form.getByLabel('Resource',{exact:true}).selectOption(resourceRef);await form.getByLabel(label).selectOption(name);await form.getByLabel('Qualified media type').fill('application/json');await form.getByLabel('operational',{exact:true}).check();
  await page.screenshot({path:`${evidence}/declare-${name}.png`});assert.ok(await form.locator('header').evaluate(el=>el.getBoundingClientRect().top>=0));assert.ok(await form.locator('footer').evaluate(el=>el.getBoundingClientRect().bottom<=innerHeight));await form.getByRole('button',{name:'Declare Source',exact:true}).click();await form.waitFor({state:'hidden'});
  const summary=await accepted('case.summary',{case_ref:caseRef});const result=summary.environment.sources.find(item=>item.label===`named-${name}`);assert.ok(result);assert.equal(result.resource_ref,resourceRef);assert.deepEqual(result.roles,['knowledge','operational']);
  const request=exchanges.findLast(item=>item.request.operation_ref==='source.declare').request.input;
  const invalid=await call('source.declare',{...request,logical_name:`invalid-${name}`,action:{...request.action,name:'not-exposed'}});assert.notEqual(invalid.result_state,'success');assert.equal((await accepted('case.summary',{case_ref:caseRef})).case.generation,summary.case.generation);
 }
 assert.equal(networkCalls,0,'declaring an HTTP Source performed a fetch');assert.equal(await readFile(path.join(root,'new.md'),'utf8'),'# New retained candidate\n','revocation modified the external file');
 assert.equal(cli('case','verify',caseRef).status,'ok');assert.deepEqual(errors,[]);
 console.log(JSON.stringify({result:'PASS',transport:'real Unix Host via browser test bridge',case_ref:caseRef,source_ref:source.id,generations:{before:before.case.generation,declared:declared.case.generation,revoked:revoked.case.generation},proof:['noncanonical path refused without mutation','authored Source declaration','same declaration reuses canonical identity','changed declaration collision refused','declaration does not acquire files','Source revocation preserves history','absent Source refused without mutation','named SQLite and HTTP Source declarations use only qualified names','unbound query/request names refused','HTTP declaration performs no network fetch','CLI replay']}));
} finally {
 await writeFile(`${evidence}/exchanges.json`,JSON.stringify(exchanges,null,2));await browser?.close();await new Promise(resolve=>server.listening?server.close(resolve):resolve());
 try {if(telemetry)cli('host','stop');} finally {await rm(home,{recursive:true,force:true});}
}
