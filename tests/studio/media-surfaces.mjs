import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import { readFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
const require=createRequire(path.resolve(import.meta.dirname,'../../studio/package.json'));
const {chromium}=require('playwright-core');
const evidence=process.env.STUDIO_EVIDENCE_DIR??'/tmp/yai-studio-media';await mkdir(evidence,{recursive:true});
const files=[
 {path:'docs/README.md',type:'text/markdown',bytes:Buffer.from('# Exact Markdown\n\n**Bold fact**, `source code`.\n\n| Owner | Count |\n|---|---|\n| YAI | 3 |\n\n[Table](../data.csv)\n\n![remote](https://unqualified.invalid/image.png)\n\n<script>window.untrustedExecuted=true</script>\n\n[unsafe](javascript:alert(1))')},
 {path:'data.csv',type:'text/csv',bytes:Buffer.from('Name,Count,Note\n'+Array.from({length:120},(_,i)=>`item-${i},${i},"quoted, note ${i}"`).join('\n'))},
 {path:'vector.svg',type:'image/svg+xml',bytes:Buffer.from('<svg xmlns="http://www.w3.org/2000/svg" width="240" height="120"><script>window.untrustedExecuted=true</script><rect width="240" height="120" fill="#78a9ff"/><text y="60">EXACT VECTOR</text></svg>')},
 ...await Promise.all([['runtime-contract.pdf','application/pdf'],['qualification-tone.wav','audio/wav'],['qualification-clip.webm','video/webm']].map(async([name,type])=>({path:name,type,bytes:await readFile(path.resolve(import.meta.dirname,'../../studio/public/fixtures',name))}))),
];
const material=files.map((file,index)=>({id:`media:${index}`,path:file.path,media_type:file.type,bytes:file.bytes.length,digest:`sha256:${createHash('sha256').update(file.bytes).digest('hex')}`,content:file.bytes.toString('base64'),encoding:'base64',source_ref:`source:${index}`,revision_ref:`revision:${index}`,source_label:file.path,backing:{posture:'retained'}}));
// Backend chooses UTF-8 for valid textual bytes, binary remains base64.
for(const index of [0,1,2]) {material[index].encoding='utf-8';material[index].content=files[index].bytes.toString();}
const browser=await chromium.launch({executablePath:'/usr/bin/chromium',headless:true,args:['--no-sandbox','--disable-gpu']});
const page=await browser.newPage({viewport:{width:1440,height:900}});page.setDefaultTimeout(10000);
const errors=[],external=[];page.on('pageerror',e=>errors.push(String(e)));page.on('request',r=>{if(r.url().startsWith('https://unqualified'))external.push(r.url());});
try {
 await page.goto(`${process.env.STUDIO_TEST_URL??'http://127.0.0.1:1422'}/?gallery=1`);
 await page.evaluate(async materials=>{
  document.getElementById('root').style.display='none';
  const [{default:React},{default:ReactDOM},{WorkbenchKernel},{WorkbenchRegistry},{PlatformServices},{createHostServices},{registerContributions},{builtInContributions},{FixtureClient},{fixtureToCasePresentation}]=await Promise.all([
   import('/node_modules/.vite/deps/react.js'),import('/node_modules/.vite/deps/react-dom_client.js'),import('/src/workbench/kernel/WorkbenchKernel.tsx'),import('/src/workbench/kernel/registry.ts'),import('/src/platform/services.ts'),import('/src/platform/host.ts'),import('/src/workbench/kernel/contributions.ts'),import('/src/contrib/builtins.ts'),import('/src/clients/fixture.ts'),import('/src/clients/dataSource.ts')]);
  const workspace=fixtureToCasePresentation(new FixtureClient().workspace('developer'));workspace.case.case_ref='case:media-qualification';workspace.case.generation=7;workspace.presentation={dataKind:'live',backendPosture:'resident-host-connected',materials:[]};workspace.environment.files=materials.map(({content,encoding,...file})=>file);
  const platform=new PlatformServices(createHostServices(false));const registry=new WorkbenchRegistry();registerContributions(builtInContributions,{platform,workbench:registry});window.mediaPlatform=platform;
  window.mediaReads=[];window.corruptMedia=false;
  const readMaterial=async request=>{window.mediaReads.push(request);const item=materials.find(item=>item.path===request.path);await new Promise(resolve=>setTimeout(resolve,item.path.endsWith('svg')?100:5));return{operation_ref:'material.read',correlation_ref:'authored:media',result_state:'success',data:{...item,content:window.corruptMedia?'AAAA':item.content,case_ref:request.case_ref,generation:request.expected_generation}};};
  const root=document.createElement('div');root.className='live-studio case-attached';document.body.appendChild(root);ReactDOM.createRoot(root).render(React.createElement(WorkbenchKernel,{workspace,platform,registry,stream:'live',readMaterial,refresh(){},openCaseSwitcher(){}}));
 },material);
 await page.locator('.live-rail button[aria-label="Environment"]').click();await page.keyboard.press('Control+j');
 const open=async name=>page.locator('.environment-files').getByRole('button',{name,exact:true}).click();
 const mode=async name=>{await page.evaluate(()=>window.mediaPlatform.commands.executeCommand('studio.file.openWith'));await page.getByRole('dialog',{name:/ with$/}).getByRole('button',{name,exact:true}).click();};
 await open('README.md');await page.locator('.cm-content').waitFor();await mode('Markdown Preview');
 await page.locator('.markdown-preview h1').waitFor();assert.equal(await page.locator('.markdown-preview h1').innerText(),'Exact Markdown');assert.equal(await page.locator('.markdown-preview strong').innerText(),'Bold fact');assert.equal(await page.locator('.markdown-preview table tbody tr').count(),1);
 assert.equal(await page.evaluate(()=>Boolean(window.untrustedExecuted)),false);assert.deepEqual(external,[]);
 await page.getByRole('button',{name:'Table',exact:true}).click();await page.locator('.cm-content').waitFor();await mode('Table');
 await page.locator('.table-surface tbody tr').first().waitFor();assert.equal(await page.locator('.table-surface tbody tr').count(),50);
 await page.getByRole('button',{name:'Count',exact:true}).click();await page.getByRole('button',{name:'Count',exact:true}).click();assert.match(await page.locator('.table-surface tbody tr').first().innerText(),/item-119/);
 await page.getByPlaceholder('Filter rows').fill('note 119');assert.equal(await page.locator('.table-surface tbody tr').count(),1);await page.locator('.table-surface tbody tr').click();assert.equal(await page.locator('.inspector').getAttribute('data-inspected-ref'),'media:1');
 const handle=page.getByRole('separator',{name:'Resize Name'});const width=Number(await handle.getAttribute('aria-valuenow'));await handle.focus();await page.keyboard.press('ArrowRight');assert.equal(Number(await handle.getAttribute('aria-valuenow')),width+20);
 await page.screenshot({path:`${evidence}/table.png`});
 await open('vector.svg');await page.locator('.image-stage img').waitFor();await page.waitForFunction(()=>document.querySelector('.image-stage img')?.naturalWidth===240);assert.equal(await page.evaluate(()=>Boolean(window.untrustedExecuted)),false);
 await mode('SVG/Text Editor');await page.locator('.cm-content').waitFor();await page.locator('.cm-content').click();await page.keyboard.press('Control+End');await page.keyboard.type('<!-- LOCAL SVG -->');await mode('Image Preview');await page.getByText('Preview of unsaved local SVG · inert image').waitFor();
 await open('runtime-contract.pdf');await page.locator('.pdf-stage canvas').waitFor();await page.waitForFunction(()=>document.querySelector('.pdf-stage canvas')?.width>200);assert.match(await page.locator('.pdf-toolbar').innerText(),/of 2/);await page.getByRole('button',{name:'Next page',exact:true}).click();await page.getByRole('textbox',{name:'Search PDF'}).fill('runtime');await page.getByRole('button',{name:'Find',exact:true}).click();await page.getByText(/\d+ matches/).waitFor();await page.screenshot({path:`${evidence}/pdf.png`});
 await open('qualification-tone.wav');await page.locator('audio').waitFor();await page.waitForFunction(()=>Number.isFinite(document.querySelector('audio')?.duration));
 await open('qualification-clip.webm');await page.locator('video').waitFor();await page.waitForFunction(()=>Number.isFinite(document.querySelector('video')?.duration));
 await open('README.md');await page.locator('.cm-content').waitFor();await page.locator('.cm-content').click();await page.keyboard.press('Control+End');await page.keyboard.type('\n\n## LOCAL DRAFT');await mode('Markdown Preview');await page.getByRole('heading',{name:'LOCAL DRAFT'}).waitFor();
 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]) {await page.setViewportSize({width,height});await page.waitForFunction(()=>{const tab=document.querySelector('.surface-tab[data-active="true"]')?.getBoundingClientRect();const strip=document.querySelector('.surface-tabs')?.getBoundingClientRect();return tab && strip && tab.left>=strip.left-1 && tab.right<=strip.right+1;});await page.screenshot({path:`${evidence}/markdown-${width}x${height}.png`});}
 await page.evaluate(()=>{window.corruptMedia=true;});await open('runtime-contract.pdf');await page.getByText(/Exact material identity mismatch/).waitFor();assert.equal(await page.locator('.pdf-stage canvas').count(),0);
 assert.deepEqual(errors,[]);assert.deepEqual(external,[]);
 console.log(JSON.stringify({result:'PASS',qualification:'authored exact material.read responses, not live Host evidence',proof:['Markdown semantic rendering and unsaved preview','no raw HTML or remote image requests','qualified relative link uses shared navigation','120-row CSV full search/sort and bounded 50-row DOM','column keyboard resize','inert SVG source/preview','exact PDF bytes, multipage and search','audio/video metadata from verified bytes','digest mismatch fails closed across binary renderer'],reads:await page.evaluate(()=>window.mediaReads.length)}));
} finally {await browser.close();}
