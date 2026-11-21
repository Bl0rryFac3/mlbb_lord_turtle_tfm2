#!/usr/bin/env python3
"""Compile the PixelLab-generated frames into the mod's sprite sheets.

Slices the 6 source sheets in ./sprites/source/ into frames, trims them to their
alpha bounds, aligns every frame of a creature onto one uniform canvas (bottom /
base-line aligned, horizontally centered), and writes:

  sprites/turtle#sheet.png          sprites/turtle#anim.fanim
  sprites/elemental_lord#sheet.png  sprites/elemental_lord#anim.fanim

Re-run after replacing the source sheets:  python3 compile_sprites.py
"""
import json
from PIL import Image

SRC = "sprites/source"
OUT = "sprites"
PAD = 2  # px of transparent padding around the largest frame

# creature -> (source file, grid cols, grid rows, [ (cell_col, cell_row) in sheet order ])
PICKS = {
    "turtle": [
        ("turtle-idle.png",     3, 2, (0, 0)),  # idle_0 — base stance
        ("turtle-idle.png",     3, 2, (1, 0)),  # idle_1 — head lowered (bob)
        ("turtle-attack.png",   3, 2, (0, 1)),  # attack_0 — windback crouch, mouth open
        ("turtle-attack.png",   3, 2, (1, 0)),  # attack_1 — open-mouth lunge, head high
        ("turtle-attack.png",   3, 2, (2, 0)),  # attack_2 — follow-through, head low
        ("turtle-collapse.png", 3, 3, (1, 1)),  # dead_0 — rolled onto his side
        ("turtle-collapse.png", 3, 3, (2, 1)),  # dead_1 — belly up, legs in the air
        ("turtle-collapse.png", 3, 3, (2, 2)),  # dead_2 — flat out
    ],
    "elemental_lord": [
        ("lord-idle.png",       3, 2, (0, 0)),  # idle_0 — trident planted
        ("lord-idle.png",       3, 2, (0, 1)),  # idle_1 — lowered, heavy breath
        ("lord-attack.png",     3, 3, (1, 0)),  # attack_0 — trident raised, arm coiling
        ("lord-attack.png",     3, 3, (2, 0)),  # attack_1 — full overhead raise
        ("lord-attack.png",     3, 3, (0, 1)),  # attack_2 — trident ground-slam + splash
        ("lord-collapse.png",   3, 3, (1, 1)),  # dead_0 — slumping forward, eyes dimming
        ("lord-collapse.png",   3, 3, (0, 2)),  # dead_1 — grey, body dissolving
        ("lord-collapse.png",   3, 3, (1, 2)),  # dead_2 — faint ghost remnant + dropped trident
    ],
}

FANIM = {
    "turtle": {
        "idle":   ([(0, 0.28), (1, 0.28)]),
        "attack": ([(2, 0.12), (3, 0.16), (4, 0.18)]),
        "dead":   ([(5, 0.30), (6, 0.30), (7, 0.55)]),
    },
    "elemental_lord": {
        "idle":   ([(0, 0.30), (1, 0.30)]),
        "attack": ([(2, 0.15), (3, 0.15), (4, 0.28)]),
        "dead":   ([(5, 0.40), (6, 0.40), (7, 0.65)]),
    },
}

cache: dict[str, Image.Image] = {}
def cell_image(fname: str, cols: int, rows: int, col: int, row: int) -> Image.Image:
    key = f"{fname}:{cols}x{rows}:{col}:{row}"
    if key in cache:
        return cache[key].copy()
    im = Image.open(f"{SRC}/{fname}").convert("RGBA")
    w, h = im.size
    x0, x1 = round(col * w / cols), round((col + 1) * w / cols)
    y0, y1 = round(row * h / rows), round((row + 1) * h / rows)
    cell = im.crop((x0, y0, x1, y1))
    bbox = cell.getchannel("A").getbbox()
    if bbox is None:
        raise SystemExit(f"picked an empty cell: {fname} cell({col},{row})")
    cache[key] = cell.crop(bbox)
    return cache[key].copy()

for name, picks in PICKS.items():
    frames = [cell_image(f, c, r, col, row) for (f, c, r, (col, row)) in picks]
    cw = max(f.width for f in frames) + 2 * PAD
    ch = max(f.height for f in frames) + 2 * PAD
    sheet = Image.new("RGBA", (cw * len(frames), ch), (0, 0, 0, 0))
    rects = []
    for i, f in enumerate(frames):
        x = i * cw + (cw - f.width) // 2          # horizontally centered
        y = ch - PAD - f.height                    # base-line aligned at the bottom
        sheet.paste(f, (x, y), f)
        rects.append({"x": i * cw, "y": 0, "w": cw, "h": ch})
    sheet.save(f"{OUT}/{name}#sheet.png")
    # Base-path fallback: the mod loader only registers #sheet/#anim as sub-assets,
    # so override remappings pointing at the bare name need a real file to exist.
    # Emit the first idle frame as <name>.png (static fallback if the engine ever
    # prefers the base image over #sheet/#anim).
    f0 = json.loads(json.dumps(rects[0]))
    base_frame = sheet.crop((f0["x"], f0["y"], f0["x"] + f0["w"], f0["y"] + f0["h"]))
    base_bbox = base_frame.getchannel("A").getbbox()
    if base_bbox:
        base_frame = base_frame.crop(base_bbox)
    base_frame.save(f"{OUT}/{name}.png")
    anims = {}
    for tag, spec in FANIM[name].items():
        anims[tag] = {"frames": [
            {"duration": dur, "data": rects[idx]} for idx, dur in spec
        ]}
    # Defensive aliases: if the view asks for any of these tags, resolve it
    # to frames we do have instead of an empty lookup (unwrap panic risk).
    alias = {
        # locomotion / stance / spawn — engine may request these at any state
        "run": "idle", "walk": "idle", "move": "idle", "chase": "idle",
        "spawn": "idle", "start": "idle", "begin": "idle", "emerge": "idle",
        "summon": "idle", "activate": "idle", "wake": "idle", "idle_elite": "idle",
        # hit reactions / flinch
        "hit": "attack", "hurt": "attack", "hit1": "attack", "hit2": "attack",
        "on_hit": "attack", "flinch": "attack", "stun": "attack", "knockback": "attack",
        # combat casts / attacks / abilities
        "cast": "attack", "skill": "attack", "skill1": "attack", "skill2": "attack",
        "skill3": "attack", "ult": "attack", "ultimate": "attack", "ability": "attack",
        "poke": "attack", "swing": "attack", "slash": "attack", "slam": "attack",
        "smash": "attack", "aoe": "attack", "slam_aoe": "attack", "casting": "attack",
        "telegraph": "attack", "charge": "attack", "windup": "attack", "wind_up": "attack",
        "punch": "attack", "bite": "attack", "chomp": "attack", "attack2": "attack",
        "attack3": "attack", "attack_down": "attack", "ranged": "attack", "melee": "attack",
        "ranged_attack": "attack", "projectile": "attack", "rage": "attack",
        "enrage": "attack", "howl": "attack", "roar": "attack", "buff": "attack",
        "empower": "attack", "attack_buff": "attack",
        # death
        "die": "dead", "death": "dead", "death0": "dead", "despawn": "dead",
        "collapse": "dead", "defeat": "dead", "ko": "dead",
    }
    for a, target in alias.items():
        if target in anims and a not in anims:
            anims[a] = anims[target]
    with open(f"{OUT}/{name}#anim.fanim", "w") as fp:
        json.dump({"anims": anims}, fp, indent=2)
    print(f"{name}: canvas {cw}x{ch}, sheet {sheet.size[0]}x{sheet.size[1]}")

# Preserve the verified objective runtime contract after regenerating art.
import runpy
from pathlib import Path
runpy.run_path(str(Path(__file__).resolve().with_name("repair_runtime_tags.py")), run_name="__main__")
