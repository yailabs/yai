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
 await page.getByRole('button',{name:/Studio Policy Actions/}).click();await page.locator('.workbench-kernel').waitFor();await page.keyboard.press('Control+j');await page.locator('.live-rail button[aria-label="Environment"]').click();


 const root=path.join(home,'materials');await mkdir(root);await writeFile(path.join(root,'evidence.txt'),'Exact retained evidence.');
 const before=await accepted('case.summary',{case_ref:caseRef});
 for(const family of ['filesystem','discovery','sqlite','http_service','mcp']) {
  await page.getByRole('button',{name:'Attach Resource…',exact:true}).click();
  const form=page.getByRole('dialog',{name:'Attach Resource'});
  await form.getByLabel('Resource reference').fill(`resource:ui-${family}`);
  await form.getByLabel('Resource family').selectOption(family);
  if(['filesystem','discovery','sqlite'].includes(family)) await form.getByLabel('Root directory on the YAI Host').fill(root);
  else {await form.getByLabel('Endpoint',{exact:true}).fill('http://127.0.0.1:1/service');await form.getByLabel('Allowed IP addresses').fill('127.0.0.1');}
  if(['filesystem','discovery'].includes(family)) await form.getByLabel('Allowed relative path prefix').fill('evidence.txt');
  if(family==='discovery'){assert.equal(await form.getByRole('checkbox').isChecked(),false);await form.getByRole('checkbox').check();}
  if(['sqlite','http_service'].includes(family)) await form.getByLabel('Bound operation name').fill('status');
  if(family==='sqlite'){await form.getByLabel('Database path relative to root').fill('evidence.sqlite');await form.getByLabel('Bound read query').fill('SELECT 1');}
  if(family==='http_service')await form.getByLabel('Bound relative HTTP path').fill('health');
  if(family==='sqlite' || family==='discovery') {
   for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]) {
    await page.setViewportSize({width,height});await form.getByRole('button',{name:'Attach Resource',exact:true}).scrollIntoViewIfNeeded();
    const bounds=await form.boundingBox();assert.ok(bounds.x>=0 && bounds.y>=0 && bounds.x+bounds.width<=width+1 && bounds.y+bounds.height<=height+1,'Dialog must fit viewport');
    await page.screenshot({path:`${evidence}/resource-setup-${family}-${width}x${height}.png`});
   }
   await page.setViewportSize({width:1440,height:900});
  }
  if(family==='mcp')dropAcknowledgement='resource.import';
  await form.getByRole('button',{name:'Attach Resource',exact:true}).click();
  if(family==='mcp'){await form.getByText('Confirmation was lost',{exact:false}).waitFor();assert.equal(await form.getByRole('button',{name:'Attach Resource',exact:true}).isDisabled(),true);await form.getByRole('button',{name:'Close and inspect state',exact:true}).click();}
  else await form.waitFor({state:'hidden'});
  const exchange=exchanges.findLast(x=>x.request.operation_ref==='resource.import');
  assert.equal(exchange.result.result_state,'success',JSON.stringify(exchange.result));assert.equal(exchange.result.data.changed,true);
  const state=exchange.result.data.state;const attached=state.resources.find(x=>x.attachment_id===`resource:ui-${family}`);
  assert.ok(attached.access.configuration_digest.startsWith('sha256:'));assert.equal(attached.review_requirement,'require_review');
  for(const operation of ['admit_content','content_read'])assert.equal(exchange.request.input.definition.operations.includes(operation),family==='discovery','Retained content scope is explicitly selected, never inferred for other families');
  const retry=await accepted('resource.import',exchange.request.input);assert.equal(retry.changed,false);assert.deepEqual(retry.state,state);
  const definitionPath=path.join(home,`definition-${family}.json`);await writeFile(definitionPath,JSON.stringify(exchange.request.input.definition));
  const sibling=cli('case','resource','import',caseRef,'--file',definitionPath).data.value;
  assert.equal(sibling.generation,state.generation,'CLI retry must preserve generation');assert.deepEqual(sibling.resources,state.resources,'CLI and Studio share exact Resource identity');
  const changed=structuredClone(exchange.request.input);changed.definition.max_items++;
  assert.notEqual((await call('resource.import',changed)).result_state,'success');
  assert.equal((await call('resource.import',{...exchange.request.input,case_ref:'case:hidden'})).result_state,'unauthorized');
 }
 const after=await accepted('case.summary',{case_ref:caseRef});assert.equal(after.case.generation,before.case.generation+5);

 const policy=JSON.parse(await readFile(path.resolve(import.meta.dirname,'../qualification/studio-product-vertical/policy.json'),'utf8'));
 const ingested=await accepted('policy.ingest',{tenant_id:'tenant:studio-ui',source_bytes:[...Buffer.from(JSON.stringify(policy))]});
 const artifact=ingested.view.artifact.artifact_id;
 await accepted('policy.validate',{artifact_ref:artifact,reason:'Controlled acquisition qualification'});
 await accepted('policy.publish',{artifact_ref:artifact,reason:'Controlled acquisition qualification'});
 await accepted('policy.case.bind',{case_ref:caseRef,artifact_ref:artifact,expected_generation:after.case.generation,reason:'Bound discovery and content admission'});
 await page.evaluate(()=>window.qualificationPlatform.commands.executeCommand('studio.case.refresh'));
 await page.getByRole('button',{name:'Declare Source',exact:true}).click();
 let sourceForm=page.getByRole('dialog',{name:'Declare Source'});
 await sourceForm.getByLabel('Name',{exact:true}).fill('setup-evidence');
 await sourceForm.getByLabel('Perimeter label').fill('resource-setup');
 await sourceForm.getByLabel('Resource',{exact:true}).selectOption('resource:ui-discovery');
 await sourceForm.getByLabel('Relative path').fill('evidence.txt');
 await sourceForm.getByLabel('Qualified media type').fill('text/plain');
 await sourceForm.getByRole('button',{name:'Declare Source',exact:true}).click();await sourceForm.waitFor({state:'hidden'});
 await page.locator('.live-sidebar').getByRole('button',{name:/setup-evidence/}).click();
 await page.getByRole('button',{name:'Acquire Source…',exact:true}).click();
 sourceForm=page.getByRole('dialog',{name:'Acquire Source'});await sourceForm.getByRole('button',{name:'Acquire exact Source'}).click();await sourceForm.waitFor({state:'hidden'});
 const acquired=exchanges.findLast(x=>x.request.operation_ref==='source.acquire');
 assert.equal(acquired.result.result_state,'success',JSON.stringify(acquired));
 assert.equal(acquired.result.data.execution.phase,'acquired',JSON.stringify(acquired.result.data));
 const populated=await accepted('case.summary',{case_ref:caseRef});
 assert.ok(populated.environment.sources.some(x=>x.label==='setup-evidence' && x.revision_ref));
 const file=populated.environment.files.find(x=>x.path==='evidence.txt');assert.ok(file);
 await page.locator('.live-sidebar').getByRole('button',{name:/evidence\.txt/}).click();
 await page.locator('.cm-content').getByText('Exact retained evidence.',{exact:true}).waitFor();
 const material=exchanges.findLast(x=>x.request.operation_ref==='material.read');
 assert.equal(material.result.result_state,'success',JSON.stringify(material));
 assert.equal(material.result.data.content,'Exact retained evidence.');
 for(const field of ['source_ref','revision_ref','path','digest'])assert.equal(material.result.data[field],file[field]);
 assert.equal(await readFile(path.join(root,'evidence.txt'),'utf8'),'Exact retained evidence.');
 assert.deepEqual(errors,[]);assert.equal(cli('case','verify',caseRef).status,'ok');
 console.log(JSON.stringify({result:'PASS',case_ref:caseRef,proof:['Five authored Resource families through real Host','Owner configuration digest','Exact retry after acknowledgement loss','Conflict and hidden Case refused','Five canonical attachments only before acquisition','Explicit admission scope -> Source declaration -> governed acquisition -> exact editor bytes','CLI exact definition retry preserves identities and generation; canonical replay']}));
}finally{await writeFile(`${evidence}/exchanges.json`,JSON.stringify(exchanges,null,2));await browser?.close();try{if(telemetry)cli('host','stop');}finally{await rm(home,{recursive:true,force:true});}}
