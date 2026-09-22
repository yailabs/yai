import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {mkdir} from 'node:fs/promises';
import path from 'node:path';
const require=createRequire(path.resolve(import.meta.dirname,'../../studio/package.json'));
const {chromium}=require('playwright-core');
const browser=await chromium.launch({executablePath:'/usr/bin/chromium',headless:true,args:['--no-sandbox','--disable-gpu']});
const page=await browser.newPage({viewport:{width:1440,height:900}});page.setDefaultTimeout(10000);
const base=process.env.STUDIO_TEST_URL??'http://127.0.0.1:1422';
const output=process.env.STUDIO_EVIDENCE_DIR??'/tmp/yai-studio-focus';await mkdir(output,{recursive:true});
const errors=[];page.on('pageerror',error=>errors.push(String(error)));
const layout=()=>page.evaluate(()=>Object.fromEntries(['.live-sidebar','.live-context','.live-bottom'].map(selector=>{const element=document.querySelector(selector),box=element.getBoundingClientRect();return [selector,{visible:!!box.width,width:box.width,height:box.height}];})));
try {
 await page.goto(`${base}/?fixture=developer`);await page.locator('.workbench-kernel').waitFor();
 await page.locator('.live-rail button[aria-label="Environment"]').click();await page.locator('.environment-files').getByRole('button',{name:'result_access.rs',exact:true}).click();await page.locator('.cm-content').waitFor();
 await page.locator('.cm-content').click();await page.keyboard.press('Control+a');await page.keyboard.type('FOCUS_RETAINS_LOCAL_BUFFER');
 await page.evaluate(()=>{window.retainedEditor=document.querySelector('.cm-editor');window.retainedSidebar=document.querySelector('.live-sidebar');window.retainedPanel=document.querySelector('.live-bottom');});
 for(const [width,height] of [[1600,960],[1440,900],[1280,800],[1000,650]]) {
  await page.setViewportSize({width,height});const before=await layout();
  await page.getByRole('button',{name:'Focus Work Surface',exact:true}).click();
  assert.equal(await page.locator('.workbench-kernel').getAttribute('data-surface-focused'),'true');
  for(const region of Object.values(await layout()))assert.equal(region.visible,false);
  const bounds=await page.locator('.live-surface').boundingBox();assert.ok(bounds.width>width-75&&bounds.height>height-115);
  assert.equal(await page.locator('.cm-content').innerText(),'FOCUS_RETAINS_LOCAL_BUFFER');
  assert.ok(await page.evaluate(()=>window.retainedEditor===document.querySelector('.cm-editor')&&window.retainedSidebar===document.querySelector('.live-sidebar')&&window.retainedPanel===document.querySelector('.live-bottom')));
  await page.screenshot({path:`${output}/focus-${width}x${height}.png`});
  await page.getByRole('button',{name:'Restore Workbench',exact:true}).click();assert.deepEqual(await layout(),before);
 }
 await page.getByRole('button',{name:'Toggle context panel',exact:true}).click();await page.getByRole('button',{name:'Close bottom panel',exact:true}).click();const partial=await layout();
 await page.getByRole('button',{name:'Focus Work Surface',exact:true}).click();await page.getByRole('button',{name:'Restore Workbench',exact:true}).click();assert.deepEqual(await layout(),partial,'restore reopened intentionally closed regions');
 await page.getByRole('button',{name:'Focus Work Surface',exact:true}).click();await page.getByRole('button',{name:'Toggle bottom panel',exact:true}).click();assert.equal(await page.locator('.workbench-kernel').getAttribute('data-surface-focused'),'false');assert.ok((await layout())['.live-bottom'].visible);
 // The registry command remains keyboard reachable and returns to the same Surface.
 await page.keyboard.press('Control+Shift+p');await page.getByRole('textbox',{name:'Type a command'}).fill('Focus Work Surface');await page.keyboard.press('Enter');assert.equal(await page.locator('.workbench-kernel').getAttribute('data-surface-focused'),'true');
 assert.equal(await page.locator('.cm-content').innerText(),'FOCUS_RETAINS_LOCAL_BUFFER');assert.deepEqual(errors,[]);
 console.log(JSON.stringify({result:'PASS',proof:['Focus at four viewport sizes','Exact previous region visibility and sizes restored','Dirty buffer and editor instance preserved','Sidebar and Panel remain mounted','Region toggle exits focus mode predictably','Command Palette keyboard invocation']}));
}finally{await browser.close();}
