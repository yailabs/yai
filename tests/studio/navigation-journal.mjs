import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {mkdir} from 'node:fs/promises';
import path from 'node:path';
const require=createRequire(path.resolve(import.meta.dirname,'../../studio/package.json'));
const {chromium}=require('playwright-core');
const browser=await chromium.launch({executablePath:'/usr/bin/chromium',headless:true,args:['--no-sandbox','--disable-gpu']});
const page=await browser.newPage({viewport:{width:1600,height:960}});page.setDefaultTimeout(10000);
const output=process.env.STUDIO_EVIDENCE_DIR??'/tmp/yai-studio-navigation';await mkdir(output,{recursive:true});
const errors=[];page.on('pageerror',error=>errors.push(String(error)));
const back=()=>page.getByRole('button',{name:'Back',exact:true}).click();
const forward=()=>page.getByRole('button',{name:'Forward',exact:true}).click();
const current=()=>page.locator('.surface-tabs [aria-selected=true]').innerText();
try {
 await page.goto(`${process.env.STUDIO_TEST_URL??'http://127.0.0.1:1423'}/?fixture=developer`);await page.locator('.workbench-kernel').waitFor();
 assert.equal(await page.getByRole('button',{name:'Back',exact:true}).isDisabled(),true);
 await page.locator('.live-rail button[aria-label="Environment"]').click();
 await page.getByRole('searchbox',{name:'Filter Environment'}).fill('result_access');
 assert.equal(await page.locator('.environment-files button').count(),1);
 await page.locator('.environment-files').getByRole('button',{name:'result_access.rs',exact:true}).click();await page.locator('.cm-content').waitFor();
 const bytes=await page.locator('.cm-content').innerText();assert.ok(bytes.length>30);
 await page.keyboard.press('Alt+ArrowLeft');await page.locator('.live-surface h1').filter({hasText:'Environment'}).waitFor();
 await forward();assert.equal(await page.locator('.cm-content').innerText(),bytes);
 await page.getByRole('searchbox',{name:'Filter Environment'}).fill('');
 const sourceGroup=page.locator('.live-sidebar details.sidebar-group > summary').filter({hasText:/^Sources/}).locator('..');
 await sourceGroup.locator('button').first().click();await page.locator('.source-surface').waitFor();
 const sourceTitle=await current();await back();assert.equal(await page.locator('.cm-content').innerText(),bytes);await forward();assert.equal(await current(),sourceTitle);
 await page.getByRole('button',{name:'Identity / Account',exact:true}).click();await page.locator('.settings-content h2').filter({hasText:'Identity'}).waitFor();
 const section=page.getByRole('combobox',{name:'Settings section'});if(await section.isVisible())await section.selectOption('Terminal');else await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Terminal',exact:true}).click();
 await back();await page.locator('.settings-content h2').filter({hasText:'Identity'}).waitFor();await back();await page.locator('.source-surface').waitFor();
 await page.locator('.live-rail button[aria-label="Knowledge"]').click();assert.equal(await page.getByRole('button',{name:'Forward',exact:true}).isDisabled(),true,'New navigation must discard forward history');
 await page.getByRole('button',{name:'Manage',exact:true}).click();await page.getByRole('menuitem',{name:/Settings/}).click();assert.equal(await page.locator('.surface-tabs [role=tab]').filter({hasText:'Settings'}).count(),1);
 await page.getByRole('button',{name:'Quick Open',exact:true}).click();const search=page.getByRole('dialog',{name:'Quick Open'});await search.getByRole('textbox').fill('result_access');await page.keyboard.press('Enter');assert.equal(await page.locator('.cm-content').innerText(),bytes);
 await page.getByRole('button',{name:'Journal',exact:true}).click();await page.locator('.case-journal').waitFor();const events=page.locator('.journal-event');assert.ok(await events.count()>0);await events.first().click();const ref=await events.first().getAttribute('title');assert.equal(await page.locator('.inspector').getAttribute('data-inspected-ref'),ref);
 await page.getByRole('button',{name:'Pause follow',exact:true}).click();await page.getByRole('button',{name:'Resume follow',exact:true}).waitFor();await page.getByRole('searchbox',{name:'Search Journal'}).fill('NO_MATCH_SENTINEL');assert.equal(await events.count(),0);await page.getByRole('searchbox',{name:'Search Journal'}).fill('');await page.getByRole('button',{name:'Resume follow',exact:true}).click();
 await page.getByRole('button',{name:'Timeline',exact:true}).click();await page.locator('[data-surface-type="case.timeline"]').waitFor();
 assert.equal(await page.locator('.live-surface').getAttribute('data-archetype'),'canvas');
 await page.locator('.real-timeline li').last().scrollIntoViewIfNeeded();
 assert.equal(await page.locator('.real-timeline li').last().evaluate(node=>{
  const surface=node.closest('.live-surface').getBoundingClientRect(), row=node.getBoundingClientRect();
  return row.bottom<=surface.bottom+1 && row.top>=surface.top-1;
 }),true,'Canvas grammar must not clip the scrollable Timeline');
 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]) {await page.setViewportSize({width,height});assert.ok(await page.locator('.kernel-status').isVisible());await page.screenshot({path:`${output}/navigation-${width}x${height}.png`});}
 assert.deepEqual(errors,[]);console.log(JSON.stringify({result:'PASS',proof:['Perspective/File/Source history preserves exact content','Alt+Left and titlebar share history','Settings section history + singleton','New navigation clears forward entries','Identity/Manage footer','Quick Open contribution','Journal search/pause/resume/Inspector/Timeline','4 viewport matrix']}));
}catch(error){console.error({errors,body:await page.locator('body').innerText()});await page.screenshot({path:`${output}/failure.png`});throw error;}finally{await browser.close();}
