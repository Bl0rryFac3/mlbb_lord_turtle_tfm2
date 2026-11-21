"""Build summoned-unit aliases without changing the working neutral Lord."""
from pathlib import Path
import json
from PIL import Image
ROOT=Path(__file__).resolve().parent
out=ROOT/'champions';out.mkdir(exist_ok=True)
im=Image.open(ROOT/'sprites/elemental_lord#sheet.png')
a=json.loads((ROOT/'sprites/elemental_lord#anim.fanim').read_text())
a['anims']['idle']=a['anims']['glide']
im.save(out/'mlbb_lord_push#sheet.png')
(out/'mlbb_lord_push#anim.fanim').write_text(json.dumps(a,indent=2)+'\n')
r=a['anims']['idle']['frames'][0]['data']
im.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h'])).save(out/'mlbb_lord_push.png')
t=Image.new('RGBA',(1,1));t.save(out/'mlbb_lord_controller.png');t.save(out/'mlbb_lord_controller#sheet.png')
a={'anims':{tag:{'frames':[{'duration':1.0,'data':{'x':0,'y':0,'w':1,'h':1}}]}for tag in ['base','idle','run','walk','attack','dead','skill','skill2','ult']}}
(out/'mlbb_lord_controller#anim.fanim').write_text(json.dumps(a,indent=2)+'\n')
print('Summon aliases rebuilt; neutral Lord unchanged.')
