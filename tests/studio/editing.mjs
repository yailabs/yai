import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import path from 'node:path';
const require=createRequire(path.resolve(import.meta.dirname,'../../studio/package.json'));
const {chromium}=require('playwright-core');
const browser=await chromium.launch({executablePath:'/usr/bin/chromium',headless:true,args:['--no-sandbox','--disable-gpu']});
const context=await browser.newContext({viewport:{width:1440,height:900},permissions:['clipboard-read','clipboard-write']});
const page=await context.newPage();page.setDefaultTimeout(10000);
const errors=[];page.on('pageerror',error=>errors.push(String(error)));
const base=process.env.STUDIO_TEST_URL??'http://127.0.0.1:1422';
const menu=async name=>{await page.locator('.desktop-menu button[aria-haspopup]').filter({hasText:/^Edit$/}).click();await page.getByRole('menuitem',{name,exact:true}).click();};
const open=async name=>{await page.locator('.environment-files').getByRole('button',{name,exact:true}).click();await page.locator('.cm-content').waitFor();};
const text=()=>page.locator('.cm-content').innerText();
try {
 await page.goto(`${base}/?fixture=developer`);await page.locator('.workbench-kernel').waitFor();await page.keyboard.press('Control+j');
 await page.locator('.live-rail button[aria-label="Environment"]').click();await open('result_access.rs');
 const initial=await text();await page.locator('.cm-content').click();await page.keyboard.press('Control+a');
 await menu('Copy');assert.equal((await page.evaluate(()=>navigator.clipboard.readText())).trim(),initial.trim());
 await page.evaluate(()=>navigator.clipboard.writeText('CLIPBOARD_SENTINEL_A'));await menu('Paste');await page.waitForFunction(()=>document.querySelector('.cm-content')?.innerText.trim()==='CLIPBOARD_SENTINEL_A');
 await menu('Undo');assert.equal((await text()).trim(),initial.trim());await menu('Redo');assert.equal((await text()).trim(),'CLIPBOARD_SENTINEL_A');
 await page.keyboard.press('Control+Shift+p');await page.getByRole('textbox',{name:'Type a command'}).fill('Select All');await page.keyboard.press('Enter');
 await menu('Cut');await page.waitForFunction(()=>document.querySelector('.cm-content')?.innerText.trim()==='');assert.equal(await page.evaluate(()=>navigator.clipboard.readText()),'CLIPBOARD_SENTINEL_A');await menu('Undo');
 // Real Clipboard API succeeded above; control response timing to qualify identity fencing.
 await page.evaluate(()=>{window.originalClipboardRead=navigator.clipboard.readText.bind(navigator.clipboard);Object.defineProperty(navigator.clipboard,'readText',{configurable:true,value:()=>new Promise(resolve=>window.finishClipboardRead=resolve)});});
 await menu('Paste');await page.waitForFunction(()=>typeof window.finishClipboardRead==='function');
 await open('result_reuse.test');const second=await text();await page.evaluate(()=>window.finishClipboardRead('MUST_NOT_ENTER_FILE_B'));
 await page.getByRole('alert').filter({hasText:'Paste cancelled'}).waitFor();assert.equal(await text(),second);await open('result_access.rs');assert.equal((await text()).trim(),'CLIPBOARD_SENTINEL_A');
 await page.getByRole('button',{name:'Dismiss editing notice'}).click();await page.locator('.cm-content').click();
 await page.evaluate(()=>Object.defineProperty(navigator.clipboard,'readText',{configurable:true,value:()=>Promise.reject(new DOMException('Denied','NotAllowedError'))}));
 await menu('Paste');await page.getByRole('alert').filter({hasText:'permission was refused'}).waitFor();assert.equal((await text()).trim(),'CLIPBOARD_SENTINEL_A');
 await page.getByRole('button',{name:'Dismiss editing notice'}).click();
 // A permission prompt that resolves after typing must not overwrite the changed buffer.
 await page.evaluate(()=>Object.defineProperty(navigator.clipboard,'readText',{configurable:true,value:()=>new Promise(resolve=>window.finishClipboardRead=resolve)}));
 await page.locator('.cm-content').click();await page.keyboard.press('Control+a');await menu('Paste');await page.keyboard.type('TYPED_WHILE_CLIPBOARD_PENDING');await page.evaluate(()=>window.finishClipboardRead('LATE_TEXT'));
 await page.getByRole('alert').filter({hasText:'Paste cancelled'}).waitFor();assert.equal((await text()).trim(),'TYPED_WHILE_CLIPBOARD_PENDING');
 await page.evaluate(()=>Object.defineProperty(navigator.clipboard,'readText',{configurable:true,value:window.originalClipboardRead}));await page.getByRole('button',{name:'Dismiss editing notice'}).click();
 await page.keyboard.press('Control+,');await page.getByRole('textbox',{name:'Search Settings'}).fill('');await page.evaluate(()=>navigator.clipboard.writeText('scrollback'));await menu('Paste');
 await page.waitForFunction(()=>document.querySelector('input[aria-label="Search settings"]')?.value==='scrollback');await page.getByText('Terminal scrollback',{exact:true}).waitFor();
 assert.deepEqual(errors,[]);
 console.log(JSON.stringify({result:'PASS',proof:['OS clipboard Copy/Paste/Cut','Undo/Redo preserve editor history','Command Palette Select All targets the editor','Late Paste cannot cross files','Denied clipboard preserves local content and explains refusal','Typing during pending Paste cancels overwrite','Settings text field follows normal React input events']}));
} catch(error) { console.error(await page.evaluate(()=>({focus:document.activeElement?.outerHTML,notice:document.querySelector('.editing-notice')?.textContent,value:document.querySelector('input[aria-label="Search settings"]')?.value,selection:window.getSelection()?.toString()})));throw error; } finally {await browser.close();}
