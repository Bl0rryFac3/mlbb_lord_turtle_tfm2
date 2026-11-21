#!/usr/bin/env python3
"""Run after compile_sprites.py. Restore runtime tags verified from TFM2 0.7.2.
Pillow only packs existing pixels; no new art is generated.
"""
from pathlib import Path
import json,copy
from PIL import Image
ROOT=Path(__file__).resolve().parent
sprites=ROOT/'sprites'
p=sprites/'elemental_lord#anim.fanim'
d=json.loads(p.read_text());a=d['anims']
# compile_sprites.py produces eight creature frames in a single row.
# Crop to that row so this operation is idempotent.
creature_h=int(a['idle']['frames'][0]['data']['h'])
sheet=Image.open(sprites/'elemental_lord#sheet.png').convert('RGBA')
sheet=sheet.crop((0,0,sheet.width,creature_h))
fxsheet=Image.open(sprites/'source/morgard_attack_target_effect.png').convert('RGBA')
combined=Image.new('RGBA',(max(sheet.width,fxsheet.width),sheet.height+fxsheet.height))
combined.paste(sheet,(0,0));combined.paste(fxsheet,(0,sheet.height))
a['base']={'frames':[copy.deepcopy(a['idle']['frames'][0])]}
a['base']['frames'][0]['duration']=0.1
for tag in ['attack_left','attack_right']:
 anim=copy.deepcopy(a['attack']);total=sum(f['duration'] for f in anim['frames'])
 for f in anim['frames']:f['duration']*=0.4/total
 a[tag]=anim
fx=json.loads((sprites/'source/morgard_attack_target_effect.json').read_text())
for f in fx['frames']:f['data']['y']+=sheet.height
a['attack_target_effect']=fx
combined.save(sprites/'elemental_lord#sheet.png')
p.write_text(json.dumps(d,indent=2)+'\n')
p=sprites/'turtle#anim.fanim';d=json.loads(p.read_text())
d['anims']['base']={'frames':[copy.deepcopy(d['anims']['idle']['frames'][0])]}
d['anims']['base']['frames'][0]['duration']=0.1
p.write_text(json.dumps(d,indent=2)+'\n')
print('Restored Morgard base/attack_left/attack_right/attack_target_effect; Turtle base.')

# Reattach the supplied movement cycle after rebuilding the core atlas.
import runpy
runpy.run_path(str(ROOT/"compile_glide.py"), run_name="__main__")
