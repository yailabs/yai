import { createRequire } from 'node:module';
import { createServer } from 'node:http';
import { randomBytes } from 'node:crypto';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import assert from 'node:assert/strict';
const studio = path.resolve(import.meta.dirname, '../../studio');
const require = createRequire(path.join(studio, 'package.json'));
const { chromium } = require('playwright-core');
const dist = await mkdtemp(path.join(tmpdir(), 'studio-csp-'));
let browser, server;
try {
  const build = spawnSync('npm', ['exec', '--', 'vite', 'build', '--outDir', dist], {cwd:studio,env:{...process.env,VITE_STUDIO_MODE:'fixture'},encoding:'utf8'});
  if(build.status!==0) throw new Error(build.stdout+build.stderr);
  const types={'.html':'text/html','.js':'application/javascript','.css':'text/css','.svg':'image/svg+xml','.mjs':'application/javascript'};
  server=createServer(async(req,res)=>{
    try {
      const url=new URL(req.url,'http://localhost');
      const file=path.resolve(dist,`.${url.pathname==='/'?'/index.html':url.pathname}`);
      if(!file.startsWith(dist+path.sep)) {res.writeHead(403).end();return;}
      let bytes=await readFile(file);
      res.setHeader('Content-Type',types[path.extname(file)]??'application/octet-stream');
      if(file.endsWith('.html')) {
        const nonce=randomBytes(24).toString('base64');
        // Same contract as Tauri's asset handler: generated style nonce in HTML
        // and its response policy. This is a browser CSP test, not a native host.
        bytes=Buffer.from(bytes.toString().replace('<style id="studio-style-nonce">',`<style id="studio-style-nonce" nonce="${nonce}">`));
        res.setHeader('Content-Security-Policy',`default-src 'self'; script-src 'self'; style-src 'self' 'nonce-${nonce}'; img-src 'self' data:; object-src 'none'; base-uri 'self'`);
      }
      res.end(bytes);
    } catch {res.writeHead(404).end();}
  });
  await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
  browser=await chromium.launch({executablePath:process.env.CHROMIUM_PATH??'/usr/bin/chromium',headless:true,args:['--no-sandbox','--disable-gpu']});
  const page=await browser.newPage({viewport:{width:1440,height:900}});page.setDefaultTimeout(10000);
  const errors=[];page.on('pageerror',e=>errors.push(String(e)));page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});
  await page.goto(`http://127.0.0.1:${server.address().port}/?fixture=developer`);
  await page.locator('.live-rail button[aria-label="Environment"]').click();
  await page.locator('.environment-files').getByRole('button',{name:'result_access.rs',exact:true}).click();
  await page.locator('.cm-content').waitFor();
  assert.equal(await page.locator('.cm-editor').evaluate(el=>getComputedStyle(el).display),'flex');
  assert.equal(await page.locator('.cm-scroller').evaluate(el=>getComputedStyle(el).display),'flex');
  assert.ok(await page.locator('.cm-content').evaluate(el=>el.getBoundingClientRect().width>100));
  assert.deepEqual(errors,[],'trusted editor styles were rejected by CSP');
  await page.evaluate(()=>{const style=document.createElement('style');style.textContent='#root{display:none!important}';document.head.appendChild(style);});
  assert.notEqual(await page.locator('#root').evaluate(el=>getComputedStyle(el).display),'none');
  assert.ok(errors.some(error=>error.includes('Content Security Policy')||error.includes('style-src')),'untrusted inline styles were not refused');
  console.log(JSON.stringify({run_id:'studio-desktop-style-policy',result:'PASS',proof:'Production CodeMirror styles use response nonce; arbitrary inline stylesheet remains blocked',lane:'production browser with desktop-equivalent CSP'}));
} finally { await browser?.close(); await new Promise(resolve=>server?server.close(resolve):resolve()); await rm(dist,{recursive:true,force:true}); }
