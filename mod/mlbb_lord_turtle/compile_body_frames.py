"""Display summoned Lord via followed three-tick view frames; preserve neutral art.
Run after compile_push_sprites.py. Original atlas is read from sprites/.
"""
from pathlib import Path
from PIL import Image
import json
ROOT=Path(__file__).resolve().parent
out=ROOT/'effects';out.mkdir(exist_ok=True)
source=Image.open(ROOT/'sprites/elemental_lord#sheet.png').convert('RGBA')
anims=json.loads((ROOT/'sprites/elemental_lord#anim.fanim').read_text())['anims']
base=json.loads((ROOT/'view/lord_slam.view_effects').read_text())
base['view_effects']=[v for v in base['view_effects'] if not v['name'].startswith('mlbb_lord_turtle:body_')]
for face in ['left','right']:
 image=source if face=='left' else source.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
 image.save(out/f'lord_body_{face}#sheet.png')
 tags={}
 for action,tag in [('glide','glide'),('attack','attack')]:
  for i,f in enumerate(anims[tag]['frames']):
   rect=dict(f['data'])
   if face=='right':rect['x']=source.width-rect['x']-rect['w']
   for hold in [1,2,3]:
    name=f'body_{action}_{i}_{face}_h{hold}'
    tags[name]={'frames':[{'duration':hold/60,'data':rect}]}
    base['view_effects'].append({'type':'Animation','name':'mlbb_lord_turtle:'+name,'anim':f'asset/mlbb_lord_turtle/effects/lord_body_{face}','tag':name,'z':0,'is_follow':True})
 (out/f'lord_body_{face}#anim.fanim').write_text(json.dumps({'anims':tags},indent=2)+'\n')
# Keep dimensions/metadata for stock health-bar placement. The body is rendered
# once, by view frames; do not banish/hide the targetable combat entity.
Image.new('RGBA',source.size).save(ROOT/'champions/mlbb_lord_push#sheet.png')
r=anims['idle']['frames'][0]['data']
source.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h'])).save(ROOT/'champions/mlbb_lord_push.png')
(ROOT/'view/lord_slam.view_effects').write_text(json.dumps(base,indent=2)+'\n')
print('72 followed body frame bindings; neutral atlas unchanged; summon combat sprite transparent.')
