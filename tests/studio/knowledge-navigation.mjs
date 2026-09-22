import { createRequire } from 'node:module';
import assert from 'node:assert/strict';
import path from 'node:path';
import { readFile, mkdir } from 'node:fs/promises';
const require = createRequire(path.resolve(import.meta.dirname, '../../studio/package.json'));
const { chromium } = require('playwright-core');
const base = process.env.STUDIO_TEST_URL ?? 'http://127.0.0.1:1422';
// Optional read-only replay of a captured Host projection, never presented as a live transport test.
const capture = process.env.STUDIO_PROJECTION ? JSON.parse(await readFile(process.env.STUDIO_PROJECTION, 'utf8')).data : null;
const evidence = process.env.STUDIO_EVIDENCE_DIR ?? '/tmp/yai-studio-knowledge';
await mkdir(evidence, { recursive: true });
const browser = await chromium.launch({ executablePath: '/usr/bin/chromium', headless: true, args: ['--no-sandbox', '--disable-gpu'] });
const page = await browser.newPage({ viewport: { width: 1600, height: 960 } });
page.setDefaultTimeout(8000);
const errors = []; page.on('pageerror', error => errors.push(String(error)));
try {
 await page.goto(`${base}/?gallery=1`);
 const facts = await page.evaluate(async capture => {
  document.getElementById('root').style.display = 'none';
  const [{default: React}, {default: ReactDOM}, {WorkbenchKernel}, {WorkbenchRegistry}, {PlatformServices}, {createHostServices}, {registerContributions}, {builtInContributions}, {FixtureClient}, {fixtureToCasePresentation}, {knowledgeGraph}] = await Promise.all([
   import('/node_modules/.vite/deps/react.js'), import('/node_modules/.vite/deps/react-dom_client.js'), import('/src/workbench/kernel/WorkbenchKernel.tsx'), import('/src/workbench/kernel/registry.ts'), import('/src/platform/services.ts'), import('/src/platform/host.ts'), import('/src/workbench/kernel/contributions.ts'), import('/src/contrib/builtins.ts'), import('/src/clients/fixture.ts'), import('/src/clients/dataSource.ts'), import('/src/contrib/case/graph.ts')]);
  const workspace = capture ? {...capture, presentation: {dataKind: 'live', backendPosture: 'resident-host-connected'}} : fixtureToCasePresentation(new FixtureClient().workspace('developer'));
  if (!capture) {
   workspace.knowledge = {status:'qualified', message:'Authored navigation qualification', sources:[{id:'document:test',source_ref:workspace.environment.sources[0].id,label:'Qualification document',path:'qualification.md',revision_ref:'revision:authored',digest:'authored',media_type:'text/markdown',extractor:'authored',status:'fixture',detail:'Deterministic test data'}], units:Array.from({length:205},(_,i)=>({id:`unit:${i}`,source_ref:'document:test',kind:'text_block',text:`UNIQUE_CONTENT_${String(i).padStart(3,'0')}\nAuthored exact unit ${i}`,posture:'fixture',references:[],topics:[]})),entities:[],topics:[{name:'Navigation',units:['unit:204']}],contradictions:[],relations:Array.from({length:205},(_,i)=>({id:`relation:${i}`,from:'document:test',to:`unit:${i}`,kind:'contains'}))};
   workspace.knowledge.relations.push({id:'relation:reference',from:'document:test',to:'unprojected:reference',kind:'references'});
  }
  const root = document.createElement('div'); root.className='live-studio case-attached'; document.body.appendChild(root);
  const platform = new PlatformServices(createHostServices(false)); const registry = new WorkbenchRegistry(); registerContributions(builtInContributions,{platform,workbench:registry});
  ReactDOM.createRoot(root).render(React.createElement(WorkbenchKernel,{workspace,stream:'live',platform,registry,refresh(){},openCaseSwitcher(){}}));
  const graph = knowledgeGraph(workspace);
  return {generation:workspace.case.generation,units:workspace.knowledge.units.length,relations:graph.edges.length,nodes:graph.nodes.length,first:workspace.knowledge.units[0],last:workspace.knowledge.units.at(-1)};
 }, capture);
 await page.locator('.workbench-kernel').waitFor();
 await page.keyboard.press('Control+j');
 await page.locator('.live-rail button[aria-label="Knowledge"]').click();
 await page.locator('.collection-categories').getByRole('button',{name:/^Units/}).click();
 assert.ok(await page.locator('.knowledge-page .collection-row').count() <= 40);
 await page.locator('.knowledge-page .collection-row').first().click();
 assert.equal(await page.locator('.inspector-excerpt').innerText(), facts.first.text);
 assert.ok(await page.locator('.inspector-excerpt').evaluate(element => element.clientWidth > 160));
 await page.screenshot({path:`${evidence}/unit-inspector.png`});
 assert.equal(await page.locator('.inspector').getAttribute('data-inspected-ref'), facts.first.id);
 await page.getByRole('searchbox',{name:'Search Units',exact:true}).fill(facts.last.id);
 assert.equal(await page.locator('.knowledge-page .collection-row').count(),1);
 await page.locator('.knowledge-page .collection-row').click();
 assert.equal(await page.locator('.inspector-excerpt').innerText(), facts.last.text);
 await page.getByRole('searchbox',{name:'Search Units',exact:true}).fill('');
 await page.getByRole('button',{name:'Open graph',exact:true}).click();
 await page.locator('.graph-viewport').waitFor();
 assert.ok(await page.locator('.graph-node').count() <= 20);
 assert.match(await page.locator('.graph-summary').innerText(), new RegExp(`of ${facts.relations} relations`));
 const firstNode = page.locator('.graph-node').first(); await firstNode.focus(); await page.keyboard.press('Enter');
 assert.equal(await firstNode.getAttribute('aria-pressed'),'true');
 await page.getByRole('button',{name:'Show related',exact:true}).click();
 await page.getByRole('button',{name:'Fit',exact:true}).click();
 const node = page.locator('.graph-node').first();
 const before = await node.boundingBox();
 await page.mouse.move(before.x+before.width/2, before.y+before.height/2); await page.mouse.down(); await page.mouse.move(before.x+before.width/2+40,before.y+before.height/2+30,{steps:5}); await page.mouse.up();
 const after = await node.boundingBox(); console.log(JSON.stringify({dragBefore:before,dragAfter:after})); assert.ok(Math.abs(after.x-before.x-40)<3); assert.ok(Math.abs(after.y-before.y-30)<3);
 await page.getByRole('button',{name:'Fit',exact:true}).click();
 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]) {
  await page.setViewportSize({width,height});
  assert.ok(await page.locator('.kernel-status').evaluate(el=>el.getBoundingClientRect().bottom<=innerHeight));
  await page.screenshot({path:`${evidence}/graph-${width}x${height}.png`});
 }
 await page.getByRole('button',{name:'Show all',exact:true}).click();
 await page.getByRole('searchbox',{name:'Filter graph nodes',exact:true}).fill(facts.last.id);
 assert.equal(await page.locator('.graph-node').count(),1);
 await page.locator('.graph-node').focus(); await page.keyboard.press('Enter');
 assert.equal(await page.locator('.inspector-excerpt').innerText(), facts.last.text);
 await page.locator('.live-rail button[aria-label="Knowledge"]').click();
 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]) {
  await page.setViewportSize({width,height});
  await page.screenshot({path:`${evidence}/knowledge-${width}x${height}.png`});
 }
 assert.deepEqual(errors,[]);
 console.log(JSON.stringify({result:'PASS',qualification:capture?'recorded Host projection replay':'authored deterministic Case projection',generation:facts.generation,units:facts.units,nodes:facts.nodes,relations:facts.relations,proof:['full-set search','bounded DOM','exact Inspector content','endpoint completeness','keyboard selection','drag in SVG coordinates','fit','four viewport sizes'],consoleErrors:0}));
} finally { await browser.close(); }
