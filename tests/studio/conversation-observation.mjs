// Presentation-only qualification: no provider or canonical Case mutation.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import path from 'node:path';
const require = createRequire(path.resolve(import.meta.dirname, '../../studio/package.json'));
const { chromium } = require('playwright-core');
const browser = await chromium.launch({ executablePath: '/usr/bin/chromium', headless: true, args: ['--no-sandbox', '--disable-gpu'] });
try {
  const page = await browser.newPage();
  const errors = [];
  page.on('pageerror', error => errors.push(String(error)));
  await page.goto(`${process.env.STUDIO_TEST_URL ?? 'http://127.0.0.1:1422'}/?gallery=1`);
  await page.evaluate(async () => {
    document.getElementById('root').style.display = 'none';
    const [{ default: React }, { default: ReactDOM }, { ConversationView }, { OverviewNarrative }] = await Promise.all([
      import('/node_modules/.vite/deps/react.js'), import('/node_modules/.vite/deps/react-dom_client.js'),
      import('/src/contrib/case/ConversationView.tsx'),
      import('/src/contrib/case/OverviewNarrative.tsx'),
    ]);
    const availability = { state: 'ready' };
    const calls = window.observationCalls = [];
    window.sendCalls = 0;
    let recovered = false, markdownOutput, attempts=[];
    const application = {
      snapshot: () => availability, subscribe: () => ({ dispose() {} }),
      supports: operation => ['execution.get', 'conversation.send'].includes(operation), reason: () => 'Observation fixture',
      sendConversation: async () => { window.sendCalls++; throw Error('No dispatch allowed'); },
      observeConversation: async input => {
        calls.push(input);
        const ref = input.execution.request_ref ?? 'request:narrative';
        const count = calls.filter(call => call.execution.request_ref === ref).length;
        const completed = ref === 'request:active' ? count >= 2 : recovered;
        return { result_state: 'success', data: {
          case_ref: input.case_ref, participant_ref: input.participant_ref,
          submission_ref: input.execution.submission_ref ?? ref, request_ref: ref, turn_ref: ref.replace('request:', 'turn:'),
          observed_generation: 7, posture: completed ? 'completed' : ref === 'request:active' ? 'running' : 'unresolved',
          invocation_refs: [], attempt_outcomes: attempts,
          primary_result: completed ? { result_id: `result:${ref}`, output: markdownOutput ?? `Retained answer for ${ref}`, invocation_id: 'invocation:fixture' } : null,
        } };
      },
    };
    const workspace = {
      case: { case_ref: 'case:observation', participant_ref: 'participant:reader', generation: 7, case_status: 'open' },
      overview: { participants: [{ id: 'participant:reader', is_current: true, roles: [], model_context_admitted: true }] },
      compute: { targets: [] },
      environment: { sources: [], resources: [], files: [] }, knowledge: { units: [] },
      authority: { policies: [] }, work: { nodes: [] }, memory: { timeline: [] },
      conversation: { turns: ['unknown', 'active'].map(name => ({ id: `turn:${name}`, execution_request_ref: `request:${name}`,
        participant_ref: 'participant:reader', generation: 7, parts: [{ text: `Question ${name}` }] })) },
    };
    const node = document.createElement('div'); document.body.appendChild(node);
    const root = ReactDOM.createRoot(node);
    sessionStorage.setItem('yai.studio.narrative.v1:case:observation:participant:reader', JSON.stringify({
      case_ref: 'case:observation', participant_ref: 'participant:reader', submission_ref: 'submission:narrative',
      parts: [], expected_generation: 7,
    }));
    const platform = { application, commands: { executeCommand: async () => undefined } };
    const render = () => root.render(React.createElement(React.Fragment, null,
      React.createElement(ConversationView, { workspace, platform, actions: { openPerspective() {}, inspect(ref) { window.inspectedReference=ref; } } }),
      React.createElement(OverviewNarrative, { workspace, platform, configure() {}, inspect() {} })));
    window.recoverExecution = () => { recovered = true; };
    window.advanceCase = () => { workspace.case = { ...workspace.case, generation: 8 }; render(); };
    window.showMarkdown = text => { markdownOutput=text; workspace.case={...workspace.case,generation:9}; render(); };
    window.showAttempts = values => { attempts=values; workspace.case={...workspace.case,generation:workspace.case.generation+1}; render(); };
    render();
  });
  await page.getByRole('button', { name: 'Check status', exact: true }).waitFor();
  assert.ok((await page.locator('.conversation-messages').innerText()).includes('no recorded response or active execution'));
  await page.locator('.conversation-answer').getByText('Retained answer for request:active', { exact: true }).waitFor();
  // More than two normal observation intervals: settled/unknown requests stop polling.
  await page.waitForTimeout(3200);
  const counts = await page.evaluate(() => Object.fromEntries(['unknown', 'active'].map(name => [name,
    window.observationCalls.filter(call => call.execution.request_ref === `request:${name}`).length])));
  assert.deepEqual(counts, { unknown: 1, active: 2 });
  assert.equal(await page.evaluate(() => window.observationCalls.filter(call => call.execution.submission_ref === 'submission:narrative').length), 1);
  await page.evaluate(() => window.recoverExecution());
  await page.getByRole('button', { name: 'Check status', exact: true }).click();
  await page.locator('.conversation-answer').getByText('Retained answer for request:unknown', { exact: true }).waitFor();
  assert.equal(await page.getByRole('button', { name: 'Check status', exact: true }).count(), 0);
  await page.getByRole('button', { name: 'Check explanation', exact: true }).click();
  await page.getByText('Retained answer for request:narrative', { exact: true }).waitFor();
  const before = await page.evaluate(() => window.observationCalls.length);
  await page.evaluate(() => window.advanceCase());
  await page.waitForFunction(count => window.observationCalls.length > count, before);
  assert.equal(await page.evaluate(() => window.sendCalls), 0);
  const external=[];
  page.on('request',request=>{if(request.url().includes('blocked.example'))external.push(request.url());});
  const markdown='## Evidence\n\n- **Known fact**\n- Missing evidence\n\n```sh\nprintf "candidate only"\n```\n\n| Fact | State |\n| --- | --- |\n| Input | Retained |\n\n[Inspect Case](case:observation) [external](https://blocked.example)\n\n![remote](https://blocked.example/image.png) <script>window.injected=true</script>';
  await page.evaluate(text=>window.showMarkdown(text),markdown);
  const answer=page.locator('.conversation-answer').first();
  await answer.getByRole('heading',{name:'Evidence'}).waitFor();
  assert.equal(await answer.locator('li').count(),2);
  assert.equal(await answer.locator('pre code').textContent(),'printf "candidate only"\n');
  assert.equal(await answer.locator('table').count(),1);
  assert.equal(await answer.locator('a,img,script,iframe').count(),0);
  await answer.getByRole('button',{name:'Inspect Case'}).click();
  assert.equal(await page.evaluate(()=>window.inspectedReference),'case:observation');
  assert.equal(await page.locator('.conversation-response-source').first().textContent(),markdown,'Exact candidate bytes remain inspectable');
  assert.equal(await page.evaluate(()=>Boolean(window.injected)),false);
  assert.deepEqual(external,[]);
  const outcomes=[
    {outcome_id:'attempt:received',attempt_number:1,delivery:'result_received',stage:'completed',request_bytes_written:8311,response_status:200,recorded_at_unix_ms:1790283497093},
    {outcome_id:'attempt:uncertain',attempt_number:2,delivery:'delivery_indeterminate',stage:'response_body',request_bytes_written:12456,response_status:null,failure_class:'delivery_or_response_unknown'},
    {outcome_id:'attempt:refused',attempt_number:3,delivery:'not_dispatched',stage:'request_serialized',request_bytes_written:0,response_status:null,failure_class:'request_capacity_refused'},
    {outcome_id:'attempt:partial'},
  ];
  await page.evaluate(values=>window.showAttempts(values),outcomes);
  const exchange=page.locator('.turn-ai').first();
  await exchange.locator(':scope > details > summary').click();
  await exchange.getByText('Result received',{exact:true}).waitFor();
  assert.equal(await exchange.getByText('Delivery uncertain',{exact:true}).count(),1);
  assert.equal(await exchange.getByText('Not sent',{exact:true}).count(),1);
  const receipts=exchange.getByRole('region',{name:'Provider attempt'});
  assert.equal(await receipts.count(),4);
  assert.match(await receipts.nth(1).innerText(),/may have executed/);
  assert.match(await receipts.nth(2).innerText(),/request capacity refused/);
  assert.match(await receipts.nth(3).innerText(),/Delivery not classified/);
  assert.match(await receipts.nth(3).innerText(),/Not recorded/);
  assert.equal(await receipts.nth(3).getByText('200',{exact:true}).count(),0);
  assert.equal(await receipts.nth(3).getByText('Attempt 4',{exact:true}).count(),0);
  await receipts.first().getByText('Exact transport evidence',{exact:true}).click();
  assert.deepEqual(JSON.parse(await receipts.first().locator('pre').textContent()),outcomes[0]);
  assert.equal(await page.evaluate(()=>window.sendCalls),0);
  assert.deepEqual(errors, []);
  console.log('PASS: Conversation and Overview active-only observation, unresolved explicit check, Case update resync; no redispatch (presentation fixture).');
} finally { await browser.close(); }
