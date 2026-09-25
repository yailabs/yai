// Read-only rendering qualification against an explicitly captured Host projection.
import { createRequire } from 'node:module';
import { readFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
const require = createRequire(path.resolve('studio/package.json'));
const { chromium } = require('playwright-core');
const record = JSON.parse(await readFile(process.env.STUDIO_OVERVIEW_PROJECTION, 'utf8'));
assert.equal(record.operation, 'case.summary');
assert.equal(record.result.result_state, 'success');
const workspace = record.result.data;
const output = process.env.STUDIO_OVERVIEW_OUTPUT;
await mkdir(output, {recursive:true});
const browser = await chromium.launch({executablePath:'/usr/bin/chromium',headless:true,args:['--no-sandbox']});
try {
 const page=await browser.newPage(); const errors=[];
 page.on('pageerror',e=>errors.push(String(e)));
 await page.goto(process.env.STUDIO_TEST_URL);
 await page.evaluate(async workspace=>{
  const [{default:React},{default:ReactDOM},{OverviewSurface}]=await Promise.all([import('/node_modules/.vite/deps/react.js'),import('/node_modules/.vite/deps/react-dom_client.js'),import('/src/contrib/case/OverviewSurface.tsx')]);
  document.getElementById('root').style.display='none';
  const mount=document.createElement('main'); mount.style.height='100vh'; document.body.appendChild(mount);
  window.inspected=[];window.navigated=[];
  ReactDOM.createRoot(mount).render(React.createElement(OverviewSurface,{workspace,inspect:id=>window.inspected.push(id),navigate:id=>window.navigated.push(id)}));
 },workspace);
 await page.getByRole('tab',{name:'Workflow',exact:true}).click();
 await page.getByRole('region',{name:'Case story'}).waitFor();
 assert.equal(await page.locator('.overview-story-steps li').count(),workspace.work.nodes.length);
 for(const node of workspace.work.nodes){
  const definition=workspace.work.definition?.nodes?.find(item=>item.node_id===node.node_id);
  if(typeof definition?.prompt==='string') assert.ok(await page.locator('.overview-story').getByText(definition.prompt,{exact:true}).count());
 }
 await page.getByRole('tab',{name:'Sources',exact:true}).click();
 for(const source of workspace.environment.sources){
  await page.locator('.overview-story-evidence button').filter({hasText:source.label}).click();
  assert.equal(await page.evaluate(()=>window.inspected.at(-1)),source.id);
 }
 await page.getByRole('tab',{name:'Workflow',exact:true}).click();
 await page.getByRole('button',{name:'Open workflow and dependencies'}).click();
 assert.equal(await page.evaluate(()=>window.navigated.at(-1)),'Work');
 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]){
  await page.setViewportSize({width,height});
  assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),'No horizontal overflow');
  await page.screenshot({path:`${output}/overview-${width}x${height}.png`,fullPage:true});
 }
 assert.deepEqual(errors,[]);
 console.log(JSON.stringify({run_id:record.run_id,property:'Exact projected checkpoints and Source navigation; four viewport rendering',result:'PASS',case_ref:workspace.case.case_ref,generation:workspace.case.generation,provider_inference:'NOT_RUN'}));
} finally {await browser.close();}
