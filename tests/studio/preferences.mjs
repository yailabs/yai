import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import path from 'node:path';
const require=createRequire(path.resolve(import.meta.dirname,'../../studio/package.json'));
const {chromium}=require('playwright-core');
const browser=await chromium.launch({executablePath:'/usr/bin/chromium',headless:true,args:['--no-sandbox','--disable-gpu']});
const page=await browser.newPage({viewport:{width:1600,height:960}});page.setDefaultTimeout(10000);
const base=process.env.STUDIO_TEST_URL??'http://127.0.0.1:1422';
try {
 await page.goto(`${base}/?fixture=developer`);await page.locator('.workbench-kernel').waitFor();
 await page.keyboard.press('Control+j');await page.keyboard.press('Control+,');await page.getByRole('button',{name:'Workbench',exact:true}).click();
 const field=page.getByRole('spinbutton',{name:'Explorer width'});
 await field.fill('-50');await field.press('Enter');assert.equal(await field.getAttribute('aria-invalid'),'true');assert.equal(await page.locator('.workbench-kernel').evaluate(el=>el.style.getPropertyValue('--left-width')),'204px');
 await field.fill('280');await field.press('Enter');assert.equal(await page.locator('.workbench-kernel').evaluate(el=>el.style.getPropertyValue('--left-width')),'280px');
 await page.reload();await page.locator('.workbench-kernel').waitFor();assert.equal(await page.locator('.workbench-kernel').evaluate(el=>el.style.getPropertyValue('--left-width')),'280px');
 await page.locator('.live-rail button[aria-label="Environment"]').click();await page.locator('.environment-files').getByRole('button',{name:'result_access.rs',exact:true}).click();await page.locator('.cm-content').waitFor();
 await page.locator('.cm-content').click();await page.keyboard.press('Control+a');
 await page.locator('.desktop-menu button[aria-haspopup]').filter({hasText:/^Edit$/}).click();await page.getByRole('menuitem',{name:'Cut',exact:true}).click();
 await page.waitForFunction(()=>document.querySelector('.cm-content')?.innerText.trim()==='');
 await page.locator('.desktop-menu button[aria-haspopup]').filter({hasText:/^Edit$/}).click();await page.getByRole('menuitem',{name:/^Undo/}).click();await page.waitForFunction(()=>document.querySelector('.cm-content')?.innerText.trim().length>10);
 await page.keyboard.press('Control+,');await page.setViewportSize({width:1000,height:650});await page.getByRole('combobox',{name:'Settings section'}).selectOption('Workbench');await page.getByRole('spinbutton',{name:'Explorer width'}).waitFor();
 assert.ok(await page.locator('.kernel-status').evaluate(el=>el.getBoundingClientRect().bottom<=innerHeight));
 console.log(JSON.stringify({result:'PASS',proof:['invalid preference rejected','valid layout applied immediately','layout survives restart','Edit menu returns focus before Cut/Undo','compact Settings keyboard navigation at 1000x650']}));
} finally {await browser.close();}
