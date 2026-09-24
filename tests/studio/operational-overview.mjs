import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {mkdir} from 'node:fs/promises';
import path from 'node:path';
const require=createRequire(path.resolve(import.meta.dirname,'../../studio/package.json'));
const {chromium}=require('playwright-core');
const browser=await chromium.launch({executablePath:'/usr/bin/chromium',headless:true,args:['--no-sandbox']});
const page=await browser.newPage({viewport:{width:1440,height:900}});page.setDefaultTimeout(10000);
const output=process.env.STUDIO_EVIDENCE_DIR??'/tmp/yai-operational-overview';await mkdir(output,{recursive:true});
const errors=[];page.on('pageerror',error=>errors.push(String(error)));
try {
 await page.goto(`${process.env.STUDIO_TEST_URL??'http://127.0.0.1:1434'}/?fixture=developer`);
 await page.getByRole('region',{name:'Current Case situation'}).waitFor();
 const status=page.getByRole('contentinfo',{name:'Workbench status'});
 assert.equal(await status.getByText(/Generation/).count(),0);
 await status.getByRole('button',{name:'Telemetry',exact:true}).click();
 await page.getByRole('heading',{name:'Telemetry',exact:true}).waitFor();
 await status.locator('.model-status').click();
 await page.getByRole('heading',{name:'Compute',exact:true}).waitFor();
 await page.locator('.live-rail button[aria-label="Overview"]').click();

 assert.equal(await page.locator('.overview-identity').getByText(/Generation/).count(),0);
 assert.equal(await page.getByRole('region',{name:'Model explanation'}).getByRole('button',{name:'Generate explanation',exact:true}).isDisabled(),true);
 for(const name of ['Environment','Knowledge','Memory','Authority','Work','Compute','Providers','YVEX','Telemetry']) {
  await page.locator(`.live-rail button[aria-label="${name}"]`).click();
 }
 await page.getByRole('heading',{name:'Telemetry',exact:true}).waitFor();
 assert.ok((await page.locator('#telemetry-host').innerText()).includes('Not observed'));
 const resources=page.locator('#telemetry-resources');
 assert.equal(await resources.getByText('Availability unknown',{exact:true}).count(),await resources.locator('button.telemetry-row').count());
 assert.ok(await page.locator('.surface-tab[data-archetype="product"]').count()>5);
 const selector=page.getByRole('combobox',{name:'Switch open Surface'});
 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]) {
  await page.setViewportSize({width,height});
  assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);
  for(const button of await status.locator('button').all()) {
   const bounds=await button.boundingBox();assert.ok(bounds && bounds.width>0 && bounds.x>=0 && bounds.x+bounds.width<=width);
  }
  const box=await selector.boundingBox();assert.ok(box && box.x>=0 && box.x+box.width<=width);
  await page.screenshot({path:`${output}/telemetry-${width}x${height}.png`});
 }
 await selector.selectOption({label:'Overview — product'});
 await page.getByRole('region',{name:'Current Case situation'}).waitFor();
 await page.getByRole('button',{name:'Toggle context panel',exact:true}).click();
 await page.getByRole('button',{name:'Toggle bottom panel',exact:true}).click();
 await page.screenshot({path:`${output}/overview-1000x650.png`,fullPage:true});
 // Candidate Markdown is inert; only currently disclosed object refs navigate.
 const outbound=[];page.on('request',request=>{if(request.url().includes('untrusted.invalid'))outbound.push(request.url());});
 await page.evaluate(async()=>{
  const react=await import('/node_modules/.vite/deps/react.js');
  const React=react.default??react;
  const client=await import('/node_modules/.vite/deps/react-dom_client.js');
  const createRoot=client.createRoot??client.default.createRoot;
  const {default:NarrativeText}=await import('/src/contrib/case/NarrativeText.tsx');
  const host=document.createElement('div');host.id='narrative-security-probe';document.body.append(host);
  createRoot(host).render(React.createElement(NarrativeText,{text:'[Evidence](source:known) [External](https://untrusted.invalid/link) ![Image](https://untrusted.invalid/image) <script>window.unsafeNarrative=true</script>',references:['source:known'],inspect:ref=>{window.inspectedNarrative=ref;}}));
 });
 await page.locator('#narrative-security-probe').getByRole('button',{name:'Evidence'}).click();
 assert.equal(await page.evaluate(()=>window.inspectedNarrative),'source:known');
 assert.equal(await page.locator('#narrative-security-probe a, #narrative-security-probe img, #narrative-security-probe script').count(),0);
 assert.deepEqual(outbound,[]);assert.equal(await page.evaluate(()=>window.unsafeNarrative),undefined);
 assert.deepEqual(errors,[]);
 console.log(JSON.stringify({result:'PASS',lane:'fixture only',proof:['Unknown telemetry stays unknown','Narrative unavailable without governed model','Product tabs and overflow selector','No Generation header','Four viewport matrix']}));
} finally {await browser.close();}
