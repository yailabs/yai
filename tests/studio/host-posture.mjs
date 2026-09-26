import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import path from 'node:path';

const require=createRequire(path.resolve(import.meta.dirname,'../../studio/package.json'));
const {chromium}=require('playwright-core');
const browser=await chromium.launch({executablePath:'/usr/bin/chromium',headless:true,args:['--no-sandbox','--disable-gpu']});
const page=await browser.newPage({viewport:{width:1280,height:800}});
page.setDefaultTimeout(10000);
const errors=[];
page.on('pageerror',error=>errors.push(String(error)));
try {
  await page.goto(`${process.env.STUDIO_TEST_URL??'http://127.0.0.1:1434'}/?fixture=developer`);
  await page.evaluate(async()=>{
    document.getElementById('root').style.display='none';
    const [{default:React},{default:ReactDOM},{WorkbenchKernel},{WorkbenchRegistry},{PlatformServices},{registerContributions},{builtInContributions},{FixtureClient},{fixtureToCasePresentation}]=await Promise.all([
      import('/node_modules/.vite/deps/react.js'),import('/node_modules/.vite/deps/react-dom_client.js'),import('/src/workbench/kernel/WorkbenchKernel.tsx'),import('/src/workbench/kernel/registry.ts'),import('/src/platform/services.ts'),import('/src/workbench/kernel/contributions.ts'),import('/src/contrib/builtins.ts'),import('/src/clients/fixture.ts'),import('/src/clients/dataSource.ts'),
    ]);
    const telemetry={schema:'yai.host.telemetry.v1',state:'running',pid:1234,process_identity:'test-process',instance_id:'host-instance:test',uptime_ms:1000,protocol:'yai.local-host.v1',version:'test',build:'test',executable_posture:'replaced_on_disk',yai_home:'/tmp/test',yai_home_identity:'test-home',transport:'unix_domain_socket',endpoint:'/tmp/test/host.sock',endpoint_posture:'ready',application_readiness:'ready',connected_clients:1,client_kinds:{studio:1},clients:[],event_sequence:1,last_activity_unix_ms:0,runtime_supervision:'not_integrated'};
    let state={state:'live',telemetry,resync_required:false};
    const listeners=new Set();
    const host={capabilities:{kind:'web',nativeDesktop:false,terminalAvailable:false,windowControlsAvailable:false},snapshot:()=>state,subscribe(listener){listeners.add(listener);listener(state);return{dispose:()=>listeners.delete(listener)};},status:async()=>state.telemetry,start:async()=>state.telemetry,stop:async()=>state.telemetry,restart:async()=>state.telemetry,closeWindow(){},dispose(){listeners.clear();}};
    window.setHostPosture=(posture,connection='live')=>{state={state:connection,telemetry:{...telemetry,executable_posture:posture},resync_required:connection!=='live'};for(const listener of listeners)listener(state);};
    const platform=new PlatformServices(host);
    const registry=new WorkbenchRegistry();registerContributions(builtInContributions,{platform,workbench:registry});
    const workspace=fixtureToCasePresentation(new FixtureClient().workspace('developer'));
    const mount=document.createElement('div');mount.id='host-posture-probe';mount.className='live-studio case-attached';document.body.append(mount);
    ReactDOM.createRoot(mount).render(React.createElement(WorkbenchKernel,{workspace,platform,registry,stream:'fixture',refresh(){},openCaseSwitcher(){}}));
  });
  const probe=page.locator('#host-posture-probe');
  const status=probe.getByRole('contentinfo',{name:'Workbench status'}).locator('.host-status');
  await status.getByText('Restart needed').waitFor();
  assert.equal(await status.getAttribute('data-state'),'replaced_on_disk');
  await status.click();
  await page.getByRole('heading',{name:'Current host'}).waitFor();
  await page.getByRole('status').getByText(/replaced on disk/).waitFor();
  await probe.locator('.live-rail button[aria-label="Telemetry"]').click();
  await probe.locator('#telemetry-host').getByRole('status').getByText(/replaced on disk/).waitFor();
  await page.evaluate(()=>window.setHostPosture('linked'));
  await status.getByText('Connected').waitFor();
  assert.equal(await probe.locator('#telemetry-host').getByRole('status').count(),0);
  await page.evaluate(()=>window.setHostPosture('replaced_on_disk','reconnecting'));
  await status.getByText('reconnecting').waitFor();
  assert.equal(await status.getAttribute('data-state'),'reconnecting');
  assert.equal(await probe.locator('#telemetry-host').getByRole('status').count(),0);
  assert.deepEqual(errors,[]);
  console.log(JSON.stringify({result:'PASS',lane:'authored browser Host observation',proof:['replaced executable visible in status, Settings and Telemetry','linked state clears warning','disconnected state does not reuse stale process facts']}));
} finally {await browser.close();}
