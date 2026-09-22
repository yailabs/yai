// Read-only acceptance against an explicitly selected, already-running Host.
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {execFileSync} from 'node:child_process';
import {mkdir} from 'node:fs/promises';
import net from 'node:net';
import path from 'node:path';
const require=createRequire(path.resolve(import.meta.dirname,'../../studio/package.json'));
const {chromium}=require('playwright-core');
assert.ok(process.env.YAI_STUDIO_TEST_HOME && process.env.YAI_STUDIO_TEST_BINARY,'Select the profile and published CLI explicitly');
const status=JSON.parse(execFileSync(process.env.YAI_STUDIO_TEST_BINARY,['host','status','--json'],{env:{...process.env,YAI_HOME:process.env.YAI_STUDIO_TEST_HOME},encoding:'utf8'})).data.value;
const caseRef=process.env.YAI_STUDIO_TEST_CASE??'case:studio-live-qualification';
const evidence=process.env.STUDIO_EVIDENCE_DIR??'/tmp/yai-studio-media/live';await mkdir(evidence,{recursive:true});
let serial=0;const reads=[];
const call=(operation_ref,input={})=>new Promise((resolve,reject)=>{
 assert.ok(['case.open','case.summary','material.read'].includes(operation_ref),'This acceptance is read-only');
 const request={protocol:'yai.studio.application.v1',operation_ref,correlation_ref:`preview:${process.pid}:${++serial}`,input};
 const socket=net.createConnection(status.endpoint);let buffer='',ready=false;socket.setTimeout(10000,()=>socket.destroy(new Error('Host timeout')));socket.on('error',reject);
 socket.on('connect',()=>socket.write(JSON.stringify({kind:'handshake',protocol:status.protocol,client_id:request.correlation_ref,client_kind:'qualification',pid:process.pid,yai_home_identity:status.yai_home_identity})+'\n'));
 socket.on('data',bytes=>{buffer+=bytes;let end;while((end=buffer.indexOf('\n'))>=0){const message=JSON.parse(buffer.slice(0,end));buffer=buffer.slice(end+1);if(!ready){assert.equal(message.kind,'handshake');ready=true;socket.write(JSON.stringify({kind:'application_request',request})+'\n');}else if(message.kind==='application_response'){socket.end();if(operation_ref==='material.read')reads.push(message.result);resolve(message.result);}}});
});
assert.equal((await call('case.open',{case_ref:caseRef})).result_state,'success');
const initial=await call('case.summary',{case_ref:caseRef});assert.equal(initial.result_state,'success');
const browser=await chromium.launch({executablePath:'/usr/bin/chromium',headless:true,args:['--no-sandbox','--disable-gpu']});
try {
 const page=await browser.newPage({viewport:{width:1440,height:900}});page.setDefaultTimeout(15000);
 await page.exposeFunction('readHostMaterial',input=>call('material.read',input));await page.goto(`${process.env.STUDIO_TEST_URL??'http://127.0.0.1:1422'}/?gallery=1`);
 await page.evaluate(async summary=>{
  document.getElementById('root').style.display='none';
  const [{default:React},{default:ReactDOM},{WorkbenchKernel},{WorkbenchRegistry},{PlatformServices},{createHostServices},{registerContributions},{builtInContributions}]=await Promise.all([import('/node_modules/.vite/deps/react.js'),import('/node_modules/.vite/deps/react-dom_client.js'),import('/src/workbench/kernel/WorkbenchKernel.tsx'),import('/src/workbench/kernel/registry.ts'),import('/src/platform/services.ts'),import('/src/platform/host.ts'),import('/src/workbench/kernel/contributions.ts'),import('/src/contrib/builtins.ts')]);
  const workspace={...summary,presentation:{dataKind:'live',backendPosture:'resident-host-connected'}};const platform=new PlatformServices(createHostServices(false));const registry=new WorkbenchRegistry();registerContributions(builtInContributions,{platform,workbench:registry});window.livePreviewPlatform=platform;
  const host=document.createElement('div');host.className='live-studio case-attached';document.body.appendChild(host);ReactDOM.createRoot(host).render(React.createElement(WorkbenchKernel,{workspace,platform,registry,stream:'live',readMaterial:window.readHostMaterial,refresh(){},openCaseSwitcher(){}}));
 },initial.data);
 await page.locator('.live-rail button[aria-label="Environment"]').click();await page.keyboard.press('Control+j');
 for(const filePath of ['cmd/README.md','studio/README.md']) {
  const file=initial.data.environment.files.find(file=>file.path===filePath);assert.ok(file,`No qualified file ${filePath}`);
  await page.locator(`.environment-files button[title="${filePath}"]`).click();await page.locator('.cm-content').waitFor();
  const text=await page.evaluate(async()=>{const {EditorView}=await import('/node_modules/.vite/deps/@codemirror_view.js');return EditorView.findFromDOM(document.querySelector('.cm-editor')).state.doc.toString();});
  const exact=reads.findLast(result=>result.data?.path===filePath).data;assert.equal(text,exact.content);assert.equal(exact.digest,file.digest);assert.equal(exact.revision_ref,file.revision_ref);
  await page.evaluate(()=>window.livePreviewPlatform.commands.executeCommand('studio.file.openWith'));await page.getByRole('dialog').getByRole('button',{name:'Markdown Preview',exact:true}).click();await page.locator('.markdown-preview h1').waitFor();
  await page.screenshot({path:`${evidence}/${filePath.split('/')[0]}-readme.png`});
  console.log(JSON.stringify({path:filePath,source_ref:exact.source_ref,revision_ref:exact.revision_ref,digest:exact.digest,bytes:exact.bytes,generation:exact.generation,fingerprint:text.slice(0,120),heading:await page.locator('.markdown-preview h1').first().innerText()}));
 }
 const after=await call('case.summary',{case_ref:caseRef});assert.equal(after.data.case.generation,initial.data.case.generation);
 console.log(JSON.stringify({result:'PASS',qualification:'real retained material through resident Unix Host and browser test bridge',case_ref:caseRef,generation:after.data.case.generation,mutations:0}));
}finally{await browser.close();}
