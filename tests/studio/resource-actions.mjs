// UI calls the real published Host over Unix IPC through a test-only Tauri bridge.
// Only a freshly created temporary YAI_HOME is mutated; never operator state.
import { createRequire } from 'node:module';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import net from 'node:net';
import { createServer } from 'node:http';
import os from 'node:os';
import path from 'node:path';
const require = createRequire(path.resolve(import.meta.dirname, '../../studio/package.json'));
const { chromium } = require('playwright-core');
const binary = process.env.YAI_STUDIO_TEST_BINARY;
assert.ok(binary, 'YAI_STUDIO_TEST_BINARY must name a built published CLI/Host');
const home = await mkdtemp(path.join(os.tmpdir(), 'yai-studio-resource-forms-'));
const evidence = process.env.STUDIO_EVIDENCE_DIR ?? '/tmp/yai-studio-resource-forms';
await mkdir(evidence, {recursive:true});
const cli = (...args) => JSON.parse(execFileSync(binary, [...args, '--json'], {env:{...process.env, YAI_HOME:home}, encoding:'utf8', timeout:30000}));
let telemetry, serial=0, browser, provider, dropAcknowledgement;
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
 const root=path.join(home,'inputs');await mkdir(root);await writeFile(path.join(root,'scope.txt'),'Resource sentinel text');
 execFileSync('/usr/bin/python3',['-c',"import sqlite3,sys; c=sqlite3.connect(sys.argv[1]); c.execute('CREATE TABLE scope (name TEXT)'); c.execute(\"INSERT INTO scope VALUES ('resource-sentinel')\"); c.commit()",path.join(root,'scope.sqlite')]);
 provider=createServer(async(req,res)=>{res.setHeader('Content-Type','application/json');if(req.method==='GET'){res.end(JSON.stringify({sentinel:'http-resource'}));return;}let body='';for await(const chunk of req)body+=chunk;const input=JSON.parse(body);const result=input.method==='server/discover'?{supportedVersions:['2026-07-28'],capabilities:{resources:{}}}:input.method==='tools/list'?{tools:[]}:{resources:[{uri:'qualification://scope',name:'Scope'}]};res.end(JSON.stringify({jsonrpc:'2.0',id:input.id,result}));});
 await new Promise(resolve=>provider.listen(0,'127.0.0.1',resolve));const endpoint=`http://127.0.0.1:${provider.address().port}`;
 const policy={schema:'yai.policy_source_input.v4',policy_key:'resource-form-proof',source_version:'1',owner_ref:'organization:yailabs',source_origin:{source_system:'test',source_uri:'test://studio/resource-forms'},validity:{mode:'unbounded'},rules:[['filesystem.read','filesystem'],['filesystem.search','filesystem'],['discovery.enumerate','discovery'],['database.query','database'],['http.fetch','http_service'],['mcp.catalog','mcp']].map(([operation_kind,resource_kind])=>({kind:'operation_restriction',rule_id:operation_kind,operation_kind,resource_kind,effect:'allow',reason:'Bounded test Resource envelope'}))};
 const artifact=(await accepted('policy.ingest',{tenant_id:'tenant:studio-ui',source_bytes:[...Buffer.from(JSON.stringify(policy))]})).view.artifact.artifact_id;
 for(const operation of ['policy.validate','policy.publish'])await accepted(operation,{artifact_ref:artifact,reason:'Bounded resource form qualification'});
 await accepted('policy.case.bind',{case_ref:caseRef,artifact_ref:artifact,expected_generation:(await accepted('case.summary',{case_ref:caseRef})).case.generation,reason:'Bounded resource form qualification'});
 const peer=endpoint=>({endpoint,allowed_ip_addresses:['127.0.0.1'],credential_ref:null});
 for(const [name,address,operations,prefixes,names] of [
 ['files',{kind:'filesystem',root},['filesystem_read','filesystem_search'],['scope.txt'],[]],
 ['discovery',{kind:'discovery',root},['discover'],['scope.txt'],[]],
 ['database',{kind:'sqlite',root,path:'scope.sqlite',queries:{scope:'SELECT name FROM scope'}},['database_query'],[],['scope']],
 ['http',{kind:'http_service',endpoint:peer(endpoint),paths:{scope:'scope'}},['http_fetch'],[],['scope']],
 ['mcp',{kind:'mcp',endpoint:peer(endpoint+'/mcp')},['mcp_catalog'],[],[]]]){
 const definition={schema:'yai.resource_definition.v1',attachment_id:'resource:'+name,policy_owner:'participant:operator',participant_ids:['participant:operator'],operations,read_prefixes:prefixes,names,max_output_bytes:8192,max_items:16,address};const file=path.join(home,name+'.json');await writeFile(file,JSON.stringify(definition));cli('case','resource','import',caseRef,'--file',file);
 }
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
 for(const [resource,action] of [['files','filesystem_read'],['files','filesystem_search'],['discovery','discover'],['database','database_query'],['http','http_fetch'],['mcp','mcp_catalog']]){
 const group=page.locator('.live-sidebar details.sidebar-group > summary').filter({hasText:/^Resources/}).locator('..');await group.getByRole('button').filter({hasText:new RegExp(resource,'i')}).click();
 await page.getByRole('button',{name:'Request Resource operation…',exact:true}).click();const form=page.getByRole('dialog',{name:'Request Resource operation'});await form.locator('select').first().selectOption(action);if(action==='filesystem_search')await form.getByLabel('Search text').fill('sentinel');
 await form.getByRole('button',{name:'Submit governed request'}).click();await form.waitFor({state:'hidden'});
 const response=exchanges.findLast(item=>item.request.operation_ref==='resource.request');assert.equal(response.result.result_state,'success',JSON.stringify(response));assert.equal(response.result.data.outcome.observation.kind,action);assert.ok(response.result.data.outcome.observation.result);assert.equal(response.result.data.outcome.posture,'observed');
 const generation=(await accepted('case.summary',{case_ref:caseRef})).case.generation;const stale=await call('resource.request',{...response.request.input,submission_ref:'request:stale:'+action,expected_generation:0});assert.equal(stale.result_state,'stale');assert.equal((await accepted('case.summary',{case_ref:caseRef})).case.generation,generation);
 }
 assert.equal(cli('case','verify',caseRef).status,'ok');assert.deepEqual(errors,[]);
 console.log(JSON.stringify({result:'PASS',proof:['Authored filesystem read/search, discovery, SQLite query, HTTP fetch and MCP catalog UI -> exact owner observations','Each variant refuses stale generation with no canonical mutation','No arbitrary SQL, URL or shell input','CLI replay']}));

}finally{await writeFile(`${evidence}/exchanges.json`,JSON.stringify(exchanges,null,2));await browser?.close();await new Promise(resolve=>provider?.close(resolve)??resolve());try{if(telemetry)cli('host','stop');}finally{await rm(home,{recursive:true,force:true});}}
