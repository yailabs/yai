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
import time
import urllib.error
import urllib.request

p = argparse.ArgumentParser()
p.add_argument('--binary', type=Path, required=True)
p.add_argument('--case', default='case:studio-live-qualification')
p.add_argument('--evidence', type=Path, required=True)
a = p.parse_args()
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
        script('const row=[...document.querySelectorAll(".live-case-row")].find(x=>x.textContent.includes(arguments[0])); if(!row)throw Error("Case absent");row.click()',a.case.removeprefix('case:').replace('-',' ').title())
    wait('return document.querySelector(".workbench-kernel")?.dataset.caseRef===arguments[0]',a.case)
    generation=script('return document.querySelector(".kernel-status").innerText')
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
    script("document.querySelector('button[aria-label=\"New terminal\"]').click()")
    wait('return document.querySelectorAll(".terminal-instance-pane [role=tab]").length===2')
    script("document.querySelector('.terminal-instance-pane button[title=\"Kill Terminal\"]').click()")
    wait('return !document.querySelector(".terminal-instance-pane")')
    script("document.querySelector('button[aria-label=\"Kill active terminal\"]').click()")
    wait('return document.querySelector(".live-bottom")?.hidden')
    script('document.dispatchEvent(new KeyboardEvent("keydown",{key:"j",ctrlKey:true,bubbles:true}))')
    wait('return document.querySelector(".xterm-helper-textarea")')
    assert script('return document.querySelector(".kernel-status").innerText')==generation
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
