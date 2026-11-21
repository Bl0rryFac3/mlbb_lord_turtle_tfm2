"""Pack the supplied 3x3 water-glide frames, preserving original RGBA pixels."""
from pathlib import Path
import json,copy
from PIL import Image
ROOT=Path(__file__).resolve().parent
p=ROOT/'sprites/elemental_lord#anim.fanim'
d=json.loads(p.read_text());anims=d['anims']
move_tags={'run','walk','move','glide'}
# Exclude prior movement rows so repeated runs do not append duplicate pixels.
core_h=max(int(f['data']['y']+f['data']['h']) for tag,a in anims.items() if tag not in move_tags for f in a['frames'])
sheet=Image.open(ROOT/'sprites/elemental_lord#sheet.png').convert('RGBA')
old_w=max(int(f['data']['x']+f['data']['w']) for tag,a in anims.items() if tag not in move_tags for f in a['frames'])
sheet=sheet.crop((0,0,old_w,core_h))
src=Image.open(ROOT/'sprites/source/lord-glide.png').convert('RGBA')
assert src.size==(384,366),src.size
frames=[]
for row in range(3):
 for col in range(3):
  f=src.crop((col*128,row*122,(col+1)*128,(row+1)*122))
  bbox=f.getchannel('A').getbbox()
  if bbox is None:raise ValueError('Empty glide frame')
  frames.append(f.crop(bbox))
cw=max(int(anims['idle']['frames'][0]['data']['w']),max(f.width for f in frames)+4)
ch=max(int(anims['idle']['frames'][0]['data']['h']),max(f.height for f in frames)+4)
combined=Image.new('RGBA',(max(sheet.width,cw*9),core_h+ch))
combined.paste(sheet,(0,0));timeline=[];preview=[]
for i,f in enumerate(frames):
 canvas=Image.new('RGBA',(cw,ch));canvas.paste(f,((cw-f.width)//2,ch-2-f.height))
 combined.paste(canvas,(i*cw,core_h))
 timeline.append({'duration':0.08,'data':{'x':i*cw,'y':core_h,'w':cw,'h':ch}})
 preview.append(canvas.resize((cw*3,ch*3),Image.Resampling.NEAREST))
for tag in move_tags:anims[tag]={'frames':copy.deepcopy(timeline)}
combined.save(ROOT/'sprites/elemental_lord#sheet.png')
p.write_text(json.dumps(d,indent=2)+'\n')
preview[0].save(ROOT/'glide_preview.gif',save_all=True,append_images=preview[1:],duration=80,loop=0,disposal=2,transparency=0)
print('Packed 9 transparent glide frames:',cw,ch,'atlas',combined.size)

import runpy
runpy.run_path(str(ROOT/"compile_push_sprites.py"), run_name="__main__")
