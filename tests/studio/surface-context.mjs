import { createRequire } from 'node:module';
import path from 'node:path';
import assert from 'node:assert/strict';
const require=createRequire(path.resolve(import.meta.dirname,'../../studio/package.json'));
const {chromium}=require('playwright-core');
const browser=await chromium.launch({executablePath:'/usr/bin/chromium',headless:true,args:['--no-sandbox','--disable-gpu']});
const page=await browser.newPage();const errors=[];page.on('pageerror',e=>errors.push(String(e)));
try {
 await page.goto(`${process.env.STUDIO_TEST_URL??'http://127.0.0.1:1435'}/?gallery=1`);
 await page.evaluate(async()=>{
  document.getElementById('root').style.display='none';
  const [{default:React},{default:{createRoot}},{WorkbenchKernel},{WorkbenchRegistry},{PlatformServices},{createHostServices},{FixtureClient},{fixtureToCasePresentation}]=await Promise.all([
   import('/node_modules/.vite/deps/react.js'),import('/node_modules/.vite/deps/react-dom_client.js'),import('/src/workbench/kernel/WorkbenchKernel.tsx'),import('/src/workbench/kernel/registry.ts'),import('/src/platform/services.ts'),import('/src/platform/host.ts'),import('/src/clients/fixture.ts'),import('/src/clients/dataSource.ts')]);
  const h=React.createElement;
  function SnapshotSurface({workspace}) {
   const [value,setValue]=React.useState('No retained result');
   return h('section',null,h('output',{'data-testid':'result','data-context':JSON.stringify([workspace.case.case_ref,workspace.case.participant_ref,workspace.case.generation])},value),
    h('button',{onClick:()=>setValue(workspace.case.case_ref+'|'+workspace.case.participant_ref)},'Capture local result'),
    h('button',{onClick:()=>{const identity=workspace.case.case_ref+'|'+workspace.case.participant_ref;window.finishOldResult=()=>setValue('LATE:'+identity);}},'Start delayed read'));
  }
  const registry=new WorkbenchRegistry();
  registry.registerViewContainer({id:'Overview',title:'Overview',icon:'overview',order:0,surface:{id:'overview',identity:'overview',surfaceType:'test.snapshot',title:'Overview',pinned:true}});
  registry.registerSurfaceRenderer({type:'test.snapshot',role:'projection',capabilities:[],component:SnapshotSurface});
  const platform=new PlatformServices(createHostServices(false));
  const initial=fixtureToCasePresentation(new FixtureClient().workspace('developer'));
  const container=document.createElement('div');container.className='live-studio case-attached';document.body.appendChild(container);
  const root=createRoot(container);
  window.renderContext=(caseRef,participant,generation=1)=>root.render(h(WorkbenchKernel,{workspace:{...initial,case:{...initial.case,case_ref:caseRef,participant_ref:participant,generation}},stream:'fixture',registry,platform,refresh(){},openCaseSwitcher(){}}));
  window.renderContext('case:A','participant:A');
 });
 const output=page.getByTestId('result');
 const change=async(c,p,g=1)=>{await page.evaluate(([c,p,g])=>window.renderContext(c,p,g),[c,p,g]);await page.waitForFunction(expected=>document.querySelector('[data-testid=result]')?.dataset.context===expected,JSON.stringify([c,p,g]));};
 await page.getByRole('button',{name:'Capture local result'}).click();
 assert.equal(await output.innerText(),'case:A|participant:A');
 await change('case:A','participant:A',2);
 assert.equal(await output.innerText(),'case:A|participant:A','Ordinary snapshot refresh must preserve renderer state');
 await change('case:A','participant:B');
 await page.getByRole('button',{name:'Capture local result'}).click();
 assert.equal(await output.innerText(),'case:A|participant:B');
 await page.getByRole('button',{name:'Start delayed read'}).click();
 await change('case:A','participant:A',2);
 assert.equal(await output.innerText(),'No retained result','Restored Surface must not retain another Participant result');
 await page.evaluate(()=>window.finishOldResult());
 assert.equal(await output.innerText(),'No retained result','Old Participant response must remain detached');
 await change('case:B','participant:A');
 await page.getByRole('button',{name:'Capture local result'}).click();
 await page.getByRole('button',{name:'Start delayed read'}).click();
 await change('case:A','participant:A',2);
 assert.equal(await output.innerText(),'No retained result','Restored Surface must not retain another Case result');
 await page.evaluate(()=>window.finishOldResult());
 assert.equal(await output.innerText(),'No retained result');

 await page.evaluate(async()=>{
  const [{default:React},{default:{createRoot}},{JournalPanel}]=await Promise.all([import('/node_modules/.vite/deps/react.js'),import('/node_modules/.vite/deps/react-dom_client.js'),import('/src/contrib/case/JournalPanel.tsx')]);
  const container=document.createElement('div'),toolbar=document.createElement('div');document.body.append(container,toolbar);const root=createRoot(container);
  window.renderJournal=(participant,sequence=1)=>root.render(React.createElement(JournalPanel,{workspace:{case:{case_ref:'case:journal',participant_ref:participant},memory:{timeline:[{id:participant+sequence,kind:'observed',component:'qualification',committed_at_unix_ms:1,sequence,causal_refs:[],summary:participant+' EVIDENCE '+sequence}]}},actions:{inspect(){},openSurface(){}},visible:true,toolbarTarget:toolbar}));
  window.renderJournal('participant:A');
 });
 const journal=page.getByRole('region',{name:'Case Journal'});
 await journal.getByText('participant:A EVIDENCE 1',{exact:true}).waitFor();
 await page.getByRole('button',{name:'Pause follow',exact:true}).click();
 await page.evaluate(()=>window.renderJournal('participant:A',2));
 await journal.getByText('participant:A EVIDENCE 1',{exact:true}).waitFor();
 await page.evaluate(()=>window.renderJournal('participant:B',2));
 await journal.getByText('participant:B EVIDENCE 2',{exact:true}).waitFor({timeout:2000});
 assert.equal(await journal.getByText('participant:A EVIDENCE 1',{exact:true}).count(),0);
 await page.getByRole('button',{name:'Pause follow',exact:true}).waitFor();
 assert.deepEqual(errors,[]);
 console.log(JSON.stringify({result:'PASS',lane:'authored Workbench renderer lifecycle',proof:['same-context refresh retains local result','Case and Participant changes discard renderer-local results','late previous-context response cannot populate active Surface','paused Journal cannot retain another Participant history']}));
}finally{await browser.close();}
