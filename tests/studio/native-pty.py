#!/usr/bin/env python3
"""Native WebKitGTK/Tauri PTY acceptance. Explicit operator profile; no Case mutation.

Requires WebKitWebDriver and an X display (xvfb-run is supported). The driver
launches only the supplied desktop binary and closes only its own session.
"""
import argparse
import base64
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import time
import urllib.error
import urllib.request

p = argparse.ArgumentParser()
p.add_argument('--binary', type=Path, required=True)
p.add_argument('--case', default='case:studio-live-qualification')
p.add_argument('--evidence', type=Path, required=True)
p.add_argument('--context-tools', action='store_true')
p.add_argument('--operational-overview', action='store_true')
p.add_argument('--decision-history', action='store_true')
p.add_argument('--execution-discovery', action='store_true', help='Read one retained process result through native Work and compare with Host; no dispatch')
a = p.parse_args()
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / 'tools/validation'))
from behavioral_corpus import Host
assert os.environ.get('YAI_HOME'), 'Explicit YAI_HOME required'
a.evidence.mkdir(parents=True, exist_ok=True)
with socket.socket() as listener:
    listener.bind(('127.0.0.1', 0))
    port = listener.getsockname()[1]
env = {**os.environ, 'TAURI_WEBVIEW_AUTOMATION':'true', 'WEBKIT_DISABLE_DMABUF_RENDERER':'1', 'GDK_BACKEND':'x11'}
log = (a.evidence/'driver.log').open('w')
driver = subprocess.Popen(['WebKitWebDriver',f'--port={port}','--host=127.0.0.1'],env=env,stdout=log,stderr=subprocess.STDOUT)
base = f'http://127.0.0.1:{port}'
session = None
def request(method, url, data=None):
    req = urllib.request.Request(base+url, data=json.dumps(data).encode() if data is not None else None, method=method, headers={'Content-Type':'application/json'})
    try:
        result=json.load(urllib.request.urlopen(req, timeout=30))
    except urllib.error.HTTPError as e:
        raise RuntimeError(e.read().decode()) from e
    return result['value']
def script(text, *args):
    return request('POST',f'/session/{session}/execute/sync',{'script':text,'args':list(args)})
def wait(text, *args):
    end=time.monotonic()+20
    while time.monotonic()<end:
        value=script(text,*args)
        if value: return value
        time.sleep(.1)
    raise AssertionError(f'Native UI timeout: {text}')
def shot(name):
    (a.evidence/f'{name}.png').write_bytes(base64.b64decode(request('GET',f'/session/{session}/screenshot')))
try:
    for _ in range(100):
        try: request('GET','/status'); break
        except Exception: time.sleep(.05)
    attached=request('POST','/session',{'capabilities':{'alwaysMatch':{'webkitgtk:browserOptions':{'binary':str(a.binary.resolve()),'args':[]}}}})
    session=attached['sessionId']
    scale=script('return devicePixelRatio')
    request('POST',f'/session/{session}/window/rect',{'x':0,'y':0,'width':round(1440*scale),'height':round(900*scale)})
    wait('return document.querySelector(".workbench-kernel") || document.querySelector(".live-case-row")')
    if not script('return Boolean(document.querySelector(".workbench-kernel"))'):
        script('const row=[...document.querySelectorAll(".live-case-row")].find(x=>x.dataset.caseRef===arguments[0]); if(!row)throw Error("Exact Case absent");row.click()',a.case)
    wait('return document.querySelector(".workbench-kernel")?.dataset.caseRef===arguments[0]',a.case)
    generation=script('return document.querySelector(".kernel-status").innerText')
    host = Host(Path(os.environ['YAI_HOME']))
    before = host.call('case.summary', {'case_ref':a.case}, f'native-before:{time.time_ns()}')
    assert before['result_state'] == 'success'
    (a.evidence/'case-before.json').write_text(json.dumps(before))
    if a.operational_overview:
        script('document.querySelector(`.live-rail button[aria-label="Overview"]`).click()')
        wait('return document.querySelector(`section[aria-label="Current Case situation"]`)')
        assert script('return !document.querySelector(".overview-identity").textContent.includes("Generation")')
        assert script('return Boolean(document.querySelector(`section[aria-label="Model explanation"]`))')
        shot('native-rich-overview')
        script('document.querySelector(`.live-rail button[aria-label="Telemetry"]`).click()')
        wait('return document.querySelector("#telemetry-host")')
        assert script('return [...document.querySelectorAll("#telemetry-host dl > div")].some(row=>row.querySelector("dt")?.textContent==="PID" && /^[0-9]+$/.test(row.querySelector("dd")?.textContent))')
        assert script('return document.querySelectorAll("#telemetry-resources button.telemetry-row").length') == len(before['data']['environment']['resources'])
        shot('native-telemetry')
        script('document.querySelector(`.live-rail button[aria-label="Overview"]`).click()')
        wait('return document.querySelector(".case-overview")')
        print(json.dumps({'result':'PASS','proof':'Native rich Overview, explicit narrative control, observed Host PID and exact Case Resource inventory'}))
    if a.decision_history:
        script('document.querySelector(`.live-rail button[aria-label="Work"]`).click()')
        wait('return document.querySelector(`[data-work-section="Decisions"]`)')
        script('document.querySelector(`[data-work-section="Decisions"]`).click()')
        wait('return document.querySelector(".decision-history")?.getBoundingClientRect().height > 0')
        script('const button=[...document.querySelectorAll(".decision-history button")].find(item=>item.textContent==="Load Decision history");button.click()')
        wait('return document.querySelector(".decision-corpus .fact-row")')
        decision_ref=script('return document.querySelector(".decision-corpus .fact-row small").textContent')
        script('document.querySelector(".decision-corpus .fact-row").click()')
        wait('return document.querySelector(".decision-reconstruction")')
        observation=host.call('decision.trajectory.inspect',{'case_ref':a.case,'participant_ref':before['data']['case']['participant_ref'],'decision_ref':decision_ref},f'native-trajectory:{time.time_ns()}')
        assert observation['result_state']=='success'
        assert script('return document.querySelector(".decision-reconstruction header p").textContent')==observation['data']['decision']['reason']
        (a.evidence/'decision-trajectory.json').write_text(json.dumps(observation))
        script('document.querySelector(".decision-reconstruction").scrollIntoView()')
        shot('native-decision-history')
        script('document.querySelector(`.live-rail button[aria-label="Overview"]`).click()')
        wait('return document.querySelector(".case-overview")')
        print(json.dumps({'result':'PASS','proof':'Native Work exact historical Decision identity/reason equals typed Host read','decision_ref':decision_ref}))
    if a.execution_discovery:
        participant=before['data']['case']['participant_ref']
        catalog=host.call('execution.list',{'case_ref':a.case,'participant_ref':participant,'limit':32},f'native-catalog:{time.time_ns()}')
        assert catalog['result_state']=='success'
        (a.evidence/'execution-catalog.json').write_text(json.dumps(catalog))
        expected=None
        for entry in catalog['data']['entries']:
            ref=entry['execution']
            if ref['domain']!='resource_request':continue
            observed=host.call('execution.get',{'case_ref':a.case,'participant_ref':participant,'execution':ref,'include_output':True},f'native-process:{time.time_ns()}')
            if observed['result_state']=='success' and observed['data'].get('process',{}).get('output'):
                expected=observed;exact=ref;break
        assert expected, 'An authorized retained process result is required; this lane never creates one'
        (a.evidence/'execution-output.json').write_text(json.dumps(expected))
        script('document.querySelector(`.live-rail button[aria-label="Work"]`).click()')
        wait('return document.querySelector(`[data-work-section="Executions"]`)')
        script('document.querySelector(`[data-work-section="Executions"]`).click()')
        wait('return document.querySelector(".work-surface .execution-catalog-row")?.getBoundingClientRect().height > 0')
        script('const select=document.querySelector(`.work-surface select[aria-label="Recent execution limit"]`);select.value="32";select.dispatchEvent(new Event("change",{bubbles:true}))')
        wait('return [...document.querySelectorAll(".work-surface .execution-catalog-row")].some(row=>row.title===arguments[0])',exact['submission_ref'])
        script('const row=[...document.querySelectorAll(".work-surface .execution-catalog-row")].find(row=>row.title===arguments[0]);row.click();row.scrollIntoView()',exact['submission_ref'])
        wait('return document.querySelector(".work-surface .process-result")')
        assert script('return !document.querySelector(".work-surface .process-stdout")')
        script('const button=[...document.querySelectorAll(".work-surface .process-result button")].find(b=>b.textContent==="Read retained process output");button.click()')
        wait('return document.querySelector(".work-surface .process-stdout")')
        output=expected['data']['process']['output']
        assert script('return document.querySelector(".work-surface .process-stdout").textContent')==(output['stdout'] or 'No standard output recorded.')
        assert script('return document.querySelector(".work-surface .process-stderr").textContent')==(output['stderr'] or 'No standard error recorded.')
        script('document.querySelector(".work-surface .process-result").scrollIntoView()')
        shot('native-retained-process')
        script('document.querySelector(`.live-rail button[aria-label="Overview"]`).click()')
        wait('return document.querySelector(".case-overview")')
        print(json.dumps({'result':'PASS','proof':'Native execution discovery and opt-in retained process bytes equal typed Host; no dispatch','reference':exact,'observation_ref':expected['data']['process']['observation_ref']}))
    # Use the Workbench's ordinary Terminal tab/command; no direct IPC invocation.
    script('const tab=[...document.querySelectorAll("button")].find(x=>x.textContent==="Terminal" && x.closest(".live-bottom")); if(tab)tab.click(); else document.dispatchEvent(new KeyboardEvent("keydown",{key:"`",ctrlKey:true,bubbles:true}))')
    wait('return document.querySelector(".xterm-helper-textarea")')
    wait('return [...document.querySelectorAll(".xterm-rows > div")].some(x=>x.textContent.trim())')
    terminal=request('POST',f'/session/{session}/element',{'using':'css selector','value':'.xterm-helper-textarea'})
    element=terminal['element-6066-11e4-a52e-4f735466cecf']
    request('POST',f'/session/{session}/element/{element}/value',{'text':"printf 'YAI_NATIVE_PTY_QUALIFIED\\n'\n"})
    wait('return [...document.querySelectorAll(".xterm-rows > div")].some(x=>x.textContent.trim()==="YAI_NATIVE_PTY_QUALIFIED")')
    assert script('return !document.querySelector(".terminal-instance-pane")'), 'One shell must not have a duplicate shell sidebar'
    for width,height in [(1600,960),(1440,900),(1280,800),(1000,650)]:
        scale=script('return devicePixelRatio')
        request('POST',f'/session/{session}/window/rect',{'width':round(width*scale),'height':round(height*scale)})
        wait('return Math.abs(innerWidth-arguments[0])<=2 && Math.abs(innerHeight-arguments[1])<=2',width,height)
        wait('const r=document.querySelector(".kernel-status")?.getBoundingClientRect();return r && r.bottom<=innerHeight+1')
        assert script('const e=document.querySelector(".xterm-viewport");return e && e.scrollWidth<=e.clientWidth+1'), 'Native horizontal terminal overflow'
        assert script('return [...document.querySelectorAll(".panel-tabs > button, .desktop-menu-trigger")].every(e=>e.getBoundingClientRect().height<=40)'), 'Compact header labels must remain on one row'
        shot(f'native-{width}x{height}')
    # Telemetry uses the real desktop PTY owner, independently of YAI Host data.
    script('document.querySelector(`.live-rail button[aria-label="Telemetry"]`).click()')
    wait('return document.querySelectorAll(".telemetry-shell").length===1')
    assert script('return [...document.querySelectorAll("#telemetry-shells dl > div")].some(row=>row.querySelector("dt")?.textContent==="Studio PID" && /^[0-9]+$/.test(row.querySelector("dd")?.textContent))')
    first_shell = script('return document.querySelector(".telemetry-shell").dataset.terminalId')
    assert script('return /^PID [0-9]+$/.test(document.querySelector(".telemetry-shell > span").textContent)')
    script("document.querySelector('button[aria-label=\"New terminal\"]').click()")
    wait('return document.querySelectorAll(".terminal-instance-pane [role=tab]").length===2')
    wait('return document.querySelectorAll(".telemetry-shell").length===2')
    script('document.querySelector(`.telemetry-sidebar-link[data-section="shells"]`).click()')
    wait('const r=document.querySelector("#telemetry-shells")?.getBoundingClientRect();return r && r.top>=0 && r.top<innerHeight/2')
    shot('native-shell-telemetry')
    script("document.querySelector('.terminal-instance-pane button[title=\"Kill Terminal\"]').click()")
    wait('return !document.querySelector(".terminal-instance-pane")')
    wait('return document.querySelectorAll(".telemetry-shell").length===1 && document.querySelector(".telemetry-shell").dataset.terminalId!==arguments[0]', first_shell)
    script("document.querySelector('button[aria-label=\"Kill active terminal\"]').click()")
    wait('return document.querySelector(".live-bottom")?.hidden')
    wait('return document.querySelectorAll(".telemetry-shell").length===0')
    script('document.querySelector(`.live-rail button[aria-label="Overview"]`).click()')
    wait('return document.querySelector(".case-overview")')
    script('document.dispatchEvent(new KeyboardEvent("keydown",{key:"j",ctrlKey:true,bubbles:true}))')
    wait('return document.querySelector(".xterm-helper-textarea")')
    assert script('return document.querySelector(".kernel-status").innerText')==generation
    if a.context_tools:
        script('const button=[...document.querySelectorAll(".live-context .segmented button")].find(b=>b.title==="Conversation");button.click()')
        wait('return document.querySelector(`textarea[aria-label="Message to the Case"]`)')
        composer=request('POST',f'/session/{session}/element',{'using':'css selector','value':'textarea[aria-label="Message to the Case"]'})['element-6066-11e4-a52e-4f735466cecf']
        request('POST',f'/session/{session}/element/{composer}/value',{'text':'NATIVE_UNSENT_CONTEXT_DRAFT'})
        script('window.nativeComposer=document.querySelector(`textarea[aria-label="Message to the Case"]`);document.querySelector(`button[aria-label="Float Conversation"]`).click()')
        wait('return document.querySelector(".context-tool-floating[data-tool=Conversation]")')
        script('document.querySelector(`button[aria-label="Open Inspector tool"]`).click()')
        wait('return document.querySelector(`button[aria-label="Float Inspector"]`)')
        script('document.querySelector(`button[aria-label="Float Inspector"]`).click()')
        for width,height in [(1600,960),(1440,900),(1280,800),(1000,650)]:
            scale=script('return devicePixelRatio')
            request('POST',f'/session/{session}/window/rect',{'width':round(width*scale),'height':round(height*scale)})
            wait('return [...document.querySelectorAll(".context-tool-floating")].every(e=>{const r=e.getBoundingClientRect();return r.x>=12&&r.y>=72&&r.right<=innerWidth-11&&r.bottom<=innerHeight-31})')
            shot(f'native-context-{width}x{height}')
        script('document.querySelector(`button[aria-label="Close Conversation tool"]`).click()')
        wait('return document.querySelector(".context-tool[data-tool=Conversation]").hidden')
        script('document.querySelector(`button[aria-label="Open Conversation tool"]`).click()')
        wait('return !document.querySelector(".context-tool[data-tool=Conversation]").hidden')
        script('document.querySelector(`button[aria-label="Dock Conversation"]`).click()')
        assert script('return window.nativeComposer===document.querySelector(`textarea[aria-label="Message to the Case"]`) && window.nativeComposer.value==="NATIVE_UNSENT_CONTEXT_DRAFT"')
        assert script('return document.querySelector(".kernel-status").innerText')==generation
        print(json.dumps({'result':'PASS','proof':'Native dock/float/close retains exact composer; two independent bounded cards; four viewport sizes; stable visible status'}))
    after = host.call('case.summary', {'case_ref':a.case}, f'native-after:{time.time_ns()}')
    (a.evidence/'case-after.json').write_text(json.dumps(after))
    assert after['result_state'] == 'success'
    assert after['data']['case'] == before['data']['case'], 'Authoritative Case identity/state changed during native acceptance'
    print(json.dumps({'result':'PASS','case_ref':a.case,'status':generation,'driver':attached['capabilities'],'proof':['Native release WebKitGTK/Tauri Workbench','Real PTY shell output','Four CSS viewport sizes','Single-row headers','Status bar remains visible','No horizontal terminal overflow','Two shells -> conditional sidebar -> last close hides panel -> Ctrl+J recreates shell','Case generation unchanged']}))
except Exception:
    if session:
        try:
            details=script('return {terminalText:document.querySelector(".terminal-surface")?.innerText, active:document.activeElement?.tagName, visibility:document.visibilityState, focused:document.hasFocus(), size:[innerWidth,innerHeight], panelHidden:document.querySelector(".live-bottom")?.hidden}')
            (a.evidence/'failure-ui.json').write_text(json.dumps(details))
            print(json.dumps(details),flush=True)
            shot('native-failure')
        except Exception as error: print(str(error),flush=True)
    raise
finally:
    if session:
        try: request('DELETE',f'/session/{session}')
        except Exception: pass
    driver.terminate();driver.wait(timeout=10);log.close()
