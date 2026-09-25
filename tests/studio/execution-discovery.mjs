import {createRequire} from 'node:module';
import path from 'node:path';
import assert from 'node:assert/strict';
const require=createRequire(path.resolve(import.meta.dirname,'../../studio/package.json'));
const {chromium}=require('playwright-core');
const browser=await chromium.launch({executablePath:'/usr/bin/chromium',headless:true,args:['--no-sandbox','--disable-gpu']});
const page=await browser.newPage();const errors=[];page.on('pageerror',e=>errors.push(String(e)));
try {
 await page.goto(`${process.env.STUDIO_TEST_URL??'http://127.0.0.1:1422'}/?gallery=1`);
 await page.evaluate(async()=>{
  document.getElementById('root').style.display='none';
  const [{default:React},{default:{createRoot}},{ExecutionHistory},{executionKey,rememberExecution}]=await Promise.all([
   import('/node_modules/.vite/deps/react.js'),import('/node_modules/.vite/deps/react-dom_client.js'),import('/src/contrib/case/ExecutionActions.tsx'),import('/src/clients/execution.ts')]);
  const node=document.createElement('div');document.body.appendChild(node);const root=createRoot(node);
  const available={state:'available',catalog:{}};window.reads=[];window.pending=[];window.observations=[];
  const application={supports:()=>true,snapshot:()=>available,subscribe:()=>({dispose(){}}),
   executions:input=>new Promise(resolve=>{window.reads.push(input);window.pending.push({input,resolve});}),
   execution:async input=>{window.observations.push(input);return {result_state:'success',data:{case_ref:input.case_ref,participant_ref:input.participant_ref,submission_ref:input.execution.submission_ref,generation:1,posture:{state:'completed'},process:{observation_ref:'observation:exact',observed_at_unix_ms:1,status:{exit_code:0,signal:null,timed_out:false,output_limit_exceeded:false,elapsed_ms:1,timeout_ms:1000},...(input.include_output?{output:{stdout:'EXACT_RETAINED_OUTPUT',stderr:'',stdout_digest:'exact',stderr_digest:'empty',lossy_utf8:false}}:{})}}};}};
  window.renderExecutions=(caseRef='case:A',participant='participant:A',generation=1)=>root.render(React.createElement(ExecutionHistory,{platform:{application},workspace:{case:{case_ref:caseRef,participant_ref:participant,generation}}}));
  window.finishExecutions=(index,refs,override={})=>{const {input,resolve}=window.pending[index];resolve({result_state:'success',data:{schema:'yai.execution_list_projection.v1',case_ref:input.case_ref,participant_ref:input.participant_ref,generation:1,limit:input.limit,scope:'recent_visible_references_bounded_128_candidates_not_complete_history',entries:refs.map(execution=>({execution,recorded_at_unix_ms:1})),...override}});};
  window.remember=()=>rememberExecution(executionKey('case:A','participant:A'),{submission_ref:'request:A',domain:'resource_request'});
  window.renderExecutions();
 });
 const rows=page.locator('.execution-catalog-row');
 await page.waitForFunction(()=>window.reads.length===1);
 await page.evaluate(()=>window.renderExecutions('case:B','participant:B'));
 await page.waitForFunction(()=>window.reads.length===2);
 await page.evaluate(()=>window.finishExecutions(0,[{domain:'resource_request',submission_ref:'LATE_CASE_A'}]));
 assert.equal(await rows.count(),0);
 await page.evaluate(()=>window.finishExecutions(1,[{domain:'resource_request',submission_ref:'request:B'}]));
 await rows.filter({hasText:'request:B'}).waitFor();
 assert.equal(await page.evaluate(()=>window.observations.length),0,'Discovery does not fetch result bodies or dispatch work');
 await page.evaluate(()=>window.renderExecutions());
 await page.waitForFunction(()=>window.reads.length===3);
 await page.evaluate(()=>window.finishExecutions(2,[{domain:'resource_request',submission_ref:'request:A'}]));
 await rows.filter({hasText:'request:A'}).waitFor();
 await page.evaluate(()=>window.remember());
 await page.getByRole('button',{name:'Read retained process output'}).waitFor();
 assert.equal(await rows.count(),1,'Local and wire field order do not change reference identity');
 await page.getByRole('button',{name:'Read retained process output'}).click();
 await page.getByText('EXACT_RETAINED_OUTPUT',{exact:true}).waitFor();
 await page.getByRole('button',{name:'Refresh executions',exact:true}).click();
 await page.waitForFunction(()=>window.reads.length===4);
 assert.equal(await page.locator('.process-stdout').count(),0,'Catalog refresh requalifies selected local output');
 await page.evaluate(()=>window.finishExecutions(3,[]));
 await page.evaluate(()=>window.renderExecutions('case:A','participant:B'));
 await page.waitForFunction(()=>window.reads.length===5);
 assert.equal(await rows.count(),0,'Local references are Participant scoped');
 await page.evaluate(()=>window.renderExecutions('case:B','participant:B'));
 await page.waitForFunction(()=>window.reads.length===6);
 await page.evaluate(()=>window.finishExecutions(4,[{domain:'resource_request',submission_ref:'LATE_PARTICIPANT'}]));
 assert.equal(await rows.count(),0);
 await page.evaluate(()=>window.finishExecutions(5,[{domain:'resource_request',submission_ref:'WRONG_CASE'}],{case_ref:'case:other'}));
 await page.getByRole('alert').waitFor();assert.equal(await rows.count(),0);
 for(const override of [{generation:2},{participant_ref:'participant:other'},{entries:[{execution:{domain:'unknown',submission_ref:'BAD'},recorded_at_unix_ms:1}]}]){
  const index=await page.evaluate(()=>window.reads.length);
  await page.getByRole('button',{name:'Refresh executions',exact:true}).click();
  await page.waitForFunction(n=>window.reads.length===n+1,index);
  await page.evaluate(([index,override])=>window.finishExecutions(index,[],override),[index,override]);
  await page.getByRole('alert').waitFor();assert.equal(await rows.count(),0);
 }
 await page.getByLabel('Recent execution limit').selectOption('32');
 await page.waitForFunction(()=>window.reads.at(-1).limit===32);
 await page.evaluate(()=>window.finishExecutions(window.reads.length-1,[{domain:'cognitive_composition',request_ref:'request:conversation'}]));
 await rows.filter({hasText:'request:conversation'}).waitFor();
 assert.deepEqual(errors,[]);
 console.log(JSON.stringify({result:'PASS',proof:['No implicit result body reads','Canonical/local reference deduplication','Late Case/Participant responses fenced','Wrong Case/Participant/generation/unknown domain refused','Refresh removes output disclosure','Bounded 16/32 selection','Conversation composition reference discovery']}));
}finally{await browser.close();}
