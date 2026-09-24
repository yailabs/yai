#!/usr/bin/env python3
"""Opt-in real native SEND. Appends one question to an explicitly selected Case.

Never retries dispatch or resets the Case. Reads retained execution through Host;
provider setup/loading is an operator responsibility. Not a deterministic gate.
"""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import time
import urllib.request

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--binary', type=Path, required=True)
p.add_argument('--case', required=True)
p.add_argument('--participant', default='participant:operator')
p.add_argument('--target', required=True)
p.add_argument('--question-file', type=Path, required=True)
p.add_argument('--evidence', type=Path, required=True)
p.add_argument('--timeout', type=int, default=600)
p.add_argument('--submit', action='store_true', required=True)
a = p.parse_args()
assert os.environ.get('YAI_HOME'), 'Explicit YAI_HOME required'
assert 1 <= a.timeout <= 900
question = a.question_file.read_text().strip()
assert question and len(question.encode()) <= 65536
a.evidence.mkdir(parents=True, exist_ok=False)
run = f'native-conversation-{time.time_ns()}'
raw = (a.evidence / 'observations.jsonl').open('x')
order = 0
def emit(**data):
    global order
    order += 1
    raw.write(json.dumps(dict(run_id=run, order=order, observed_at_unix_ms=int(time.time()*1000), **data))+'\n')
    raw.flush()

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / 'tools/validation'))
from behavioral_corpus import Host
host = Host(Path(os.environ['YAI_HOME']))
def call(operation, inputs):
    result = host.call(operation, inputs, f'{run}:{order}')
    emit(operation=operation, inputs=inputs, result=result)
    assert result['result_state'] == 'success', result
    return result['data']

driver = None
session = None
driver_log = (a.evidence/'driver.log').open('w')
with socket.socket() as listener:
    listener.bind(('127.0.0.1', 0))
    port = listener.getsockname()[1]
base = f'http://127.0.0.1:{port}'
def request(method, url, data=None):
    req = urllib.request.Request(base+url, data=json.dumps(data).encode() if data is not None else None,
        method=method, headers={'Content-Type':'application/json'})
    with urllib.request.urlopen(req, timeout=30) as response:
        return json.load(response)['value']
def js(code, *args):
    return request('POST', f'/session/{session}/execute/sync', {'script':code,'args':list(args)})
def wait(code, *args):
    deadline = time.monotonic()+30
    while time.monotonic() < deadline:
        value = js(code, *args)
        if value:
            return value
        time.sleep(.1)
    raise AssertionError(code)
def shot(name):
    (a.evidence/name).write_bytes(base64.b64decode(request('GET', f'/session/{session}/screenshot')))

try:
    emit(command=[sys.executable,*sys.argv], cwd=str(Path.cwd()), yai_home=os.environ['YAI_HOME'],
        evidence_class='external_provider_native_product', question=question,
        binary=str(a.binary.resolve()), binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),
        source_sha=subprocess.check_output(['git','rev-parse','HEAD'], text=True).strip(),
        source_status=subprocess.check_output(['git','status','--short'], text=True))
    before = call('case.summary', {'case_ref':a.case})
    assert before['case']['participant_ref'] == a.participant
    target = next(item for item in before['compute']['targets'] if item['id'] == a.target)
    catalog = call('provider.models', {'tenant_id':before['case']['tenant_ref'], 'target_ref':a.target})
    assert target['model_id'] in catalog['models'], 'Exact selected model is not exposed'
    old_turns = {item['id'] for item in before['conversation']['turns']}
    env = {**os.environ,'TAURI_WEBVIEW_AUTOMATION':'true','WEBKIT_DISABLE_DMABUF_RENDERER':'1','GDK_BACKEND':'x11'}
    driver = subprocess.Popen(['WebKitWebDriver',f'--port={port}','--host=127.0.0.1'], env=env, stdout=driver_log, stderr=subprocess.STDOUT)
    for _ in range(100):
        try:
            request('GET','/status')
            break
        except OSError:
            time.sleep(.05)
    result = request('POST','/session', {'capabilities':{'alwaysMatch':{'webkitgtk:browserOptions':{'binary':str(a.binary.resolve()),'args':[]}}}})
    session = result['sessionId']
    emit(driver_capabilities=result['capabilities'])
    wait('return document.querySelector(".live-case-row") || document.querySelector(".workbench-kernel")')
    if js('return Boolean(document.querySelector(".workbench-kernel"))'):
        js('document.querySelector(".titlebar-case").click()')
        wait('return document.querySelector(".case-switcher")')
        js('const row=[...document.querySelectorAll(".case-switcher .ui-list-row")].find(x=>x.dataset.caseRef===arguments[0]);if(!row)throw Error("Case missing");row.click()',a.case)
    else:
        js('const row=[...document.querySelectorAll(".live-case-row")].find(x=>x.dataset.caseRef===arguments[0]);if(!row)throw Error("Case missing");row.click()',a.case)
    wait('return document.querySelector(".workbench-kernel")?.dataset.caseRef===arguments[0]',a.case)
    js('document.querySelector(`.live-context .segmented button[title="Conversation"]`).click()')
    wait('return document.querySelector(`textarea[aria-label="Message to the Case"]`)')
    assert js('return document.querySelector(`textarea[aria-label="Message to the Case"]`).value') == '', 'Preserve an existing unsent draft'
    element = request('POST',f'/session/{session}/element',{'using':'css selector','value':'textarea[aria-label="Message to the Case"]'})['element-6066-11e4-a52e-4f735466cecf']
    request('POST',f'/session/{session}/element/{element}/click',{})
    request('POST',f'/session/{session}/element/{element}/value',{'text':question})
    assert js('return document.querySelector(`textarea[aria-label="Message to the Case"]`).value') == question
    wait('return document.querySelector(`button[aria-label="Send"]`)?.disabled===false')
    emit(action='native_composer_send_once', pre_state_version=before['case']['generation'], target=target)
    button = request('POST',f'/session/{session}/element',{'using':'css selector','value':'button[aria-label="Send"]'})['element-6066-11e4-a52e-4f735466cecf']
    request('POST',f'/session/{session}/element/{button}/click',{})
    deadline = time.monotonic()+a.timeout
    execution = None
    while time.monotonic() < deadline:
        summary = call('case.summary',{'case_ref':a.case})
        added = [turn for turn in summary['conversation']['turns'] if turn['id'] not in old_turns
            and any(part.get('text') == question for part in turn['parts'])]
        assert len(added) <= 1, 'One click duplicated the committed question'
        if added and added[0].get('execution_request_ref'):
            query = {'case_ref':a.case,'participant_ref':a.participant,
                'execution':{'domain':'cognitive_composition','request_ref':added[0]['execution_request_ref']}}
            execution = call('execution.get', query)
            if execution.get('primary_result'):
                break
            if execution['posture'] not in ['admitted','running','unresolved']:
                raise AssertionError(f"Retained outcome: {execution['posture']}; never resend automatically")
        time.sleep(5)
    else:
        raise TimeoutError('Observation deadline; submission may remain active. Do not resend it.')
    assert execution['primary_result']['selection']['selected_target_id'] == a.target
    output = execution['primary_result']['output']
    wait('return [...document.querySelectorAll(".turn-ai > p")].some(node=>node.textContent===arguments[0])',output)
    context = call('execution.get', {**query,'include_context':True})
    emit(result='PASS', proof='One real native Studio SEND; exact canonical result displayed; no automatic redispatch',
        request_ref=execution['request_ref'], result_ref=execution['primary_result']['result_id'], output=output)
    shot('completed.png')
    print(json.dumps(dict(result='PASS',run_id=run,evidence=str(a.evidence),request_ref=execution['request_ref'],output=output)))
except BaseException as error:
    emit(result='NONPASS', failure=str(error), dispatch_retry=False)
    if session:
        try:
            shot('nonpass.png')
        except Exception as capture:
            emit(capture_failure=str(capture))
    raise
finally:
    if session:
        request('DELETE',f'/session/{session}')
    if driver:
        driver.terminate()
        driver.wait(timeout=10)
    driver_log.close()
    raw.close()
