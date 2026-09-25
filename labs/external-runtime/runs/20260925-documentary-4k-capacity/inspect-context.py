import json,sys,time
from pathlib import Path
sys.path.insert(0,'tools/validation');from behavioral_corpus import Host
op='execution.get';i=dict(case_ref='case:qualification-deepseek-material-4k',participant_ref='participant:operator',execution=dict(domain='cognitive_composition',request_ref='cognitive-composition:sha256:24d9f71d24c8bf4d1e3624acbe2576713dead7d8f4848f4a89eb1ec509ef30c4'),include_context=True)
r=Host('/home/mothx/.yai').call(op,i,'documentary-context:'+str(time.time_ns()))
Path('/tmp/documentary-context.json').write_text(json.dumps(dict(operation=op,input=i,result=r),indent=2))
assert r['result_state']=='success'
for x in r['data']['prepared_context']['invocations']:
 o=x.get('input_observation'); print(json.dumps(o,indent=2)); print('frame keys',list((x.get('frame') or {}).keys()))
