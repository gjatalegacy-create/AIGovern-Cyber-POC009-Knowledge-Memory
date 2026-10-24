"""Copy the published POC008 PD/codec substrate, never the private runtime."""
from pathlib import Path
import hashlib, json, shutil
root = Path(__file__).resolve().parents[1]
source = root.parent / 'poc008-continuity'
paths = ['src/pd_memory.rs','src/projection.rs','src/zero_noi.rs','src/framing.rs','src/fnv.rs','src/pd_spine_contract.rs']
paths += ['src/progressive_debatic/'+p for p in ['checkpoint.rs','core.rs','genius_detector.rs','types.rs','mod.rs']]
entries=[]
for name in paths:
    data=(source/name).read_bytes()
    dest=root/name
    dest.parent.mkdir(parents=True,exist_ok=True)
    dest.write_bytes(data)
    entries.append({'source':name,'source_sha256':hashlib.sha256(data).hexdigest(),'output':name,'transformation':'none; byte-identical published POC008 substrate'})
for name in ['LICENSE','NOTICE.md','provenance_engine.json','provenance_contracts.json']:
    dest = root/name if name=='LICENSE' else root/'provenance'/'poc008'/name
    dest.parent.mkdir(parents=True,exist_ok=True)
    shutil.copyfile(source/name,dest)
(root/'base_provenance.json').write_text(json.dumps({'source_repository':'https://github.com/gjatalegacy-create/AIGovern-Cyber-POC008-Continuity','source_tag':'poc008-public-v0.1.0','source_doi':'10.5281/zenodo.23613613','entries':entries},indent=2)+'\n',encoding='utf-8')
print('Copied',len(entries),'source files from public POC008')
