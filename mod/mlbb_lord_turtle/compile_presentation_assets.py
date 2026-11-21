"""Run after compile_body_frames.py; restore static UI portrait and small icons."""
from pathlib import Path
from PIL import Image
import json
R=Path(__file__).resolve().parent
im=Image.open(R/'sprites/elemental_lord#sheet.png').convert('RGBA')
r=json.loads((R/'sprites/elemental_lord#anim.fanim').read_text())['anims']['idle']['frames'][0]['data']
im.crop((r['x'],r['y'],r['x']+r['w'],r['y']+r['h'])).save(R/'champions/mlbb_lord_push.png')
for team in ['blue','red']:
 Image.open(R/f'sprites/lord_sticker_{team}.png').convert('RGBA').resize((14,14),Image.Resampling.NEAREST).save(R/f'sprites/lord_minimap_{team}.png')
print('Visible static Lord portrait and 14px team minimap icons generated; combat sheet unchanged.')
