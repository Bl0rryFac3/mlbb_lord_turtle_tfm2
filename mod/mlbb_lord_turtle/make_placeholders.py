#!/usr/bin/env python3
"""Generate placeholder sprite sheets for the MLBB: Elemental Lord & Turtle TFM2 mod.

The art here is ORIGINAL crude placeholder pixel art (not extracted and not a copy of
any Moonton asset) — it exists so the mod loads and renders something in-game.
Replace every sheet with your own sprites using the same filenames and tag names
(see SPRITES.md). Run:  python3 make_placeholders.py
"""
import json
from PIL import Image, ImageDraw

OUT = "sprites"

def canvas(w, h):
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    return im, ImageDraw.Draw(im)

# ---------------- palette ----------------
TURTLE = dict(shell=(46, 140, 96), shell_dark=(28, 92, 63), rim=(120, 220, 160),
              skin=(110, 200, 140), skin_dark=(70, 150, 95), eye=(15, 30, 25))
LORD = dict(robe=(18, 84, 160), robe_dark=(10, 45, 95), trim=(90, 225, 255),
            skin=(23, 110, 205), glow=(120, 240, 255), orb=(160, 250, 255),
            crown=(255, 220, 90), eye=(190, 255, 255))

def eye(d, x, y):
    d.point((x, y), fill=(255, 255, 255))

# ---------------- TURTLE (64x64) ----------------
T = 64
def turtle_frame(pose):
    im, d = canvas(T, T)
    bob = -2 if pose == "idle_1" else 0
    lunge = 6 if pose.startswith("attack") else 0
    dead = pose == "dead"
    shell_box = (7, 24 + bob, 45, 50 + bob)
    if dead:  # flipped onto its back
        d.ellipse((8, 34, 46, 52), fill=TURTLE["shell_dark"], outline=(15, 50, 35))
        d.ellipse((12, 36, 42, 48), fill=TURTLE["shell"])
        for fx, fy in ((10, 24), (18, 20), (30, 20), (40, 24), (24, 16)):  # legs up
            d.ellipse((fx, fy, fx + 7, fy + 10), fill=TURTLE["skin_dark"], outline=(15, 50, 35))
        eye(d, 52, 44); return im
    # flippers (behind shell)
    d.ellipse((10, 42 + bob, 22, 52 + bob), fill=TURTLE["skin_dark"], outline=(15, 50, 35))
    d.ellipse((30, 42 + bob, 42, 52 + bob), fill=TURTLE["skin_dark"], outline=(15, 50, 35))
    # head
    hx = 36 + lunge
    d.ellipse((hx, 22 + bob, hx + 16, 36 + bob), fill=TURTLE["skin"], outline=(15, 50, 35))
    d.point((hx + 11, 27 + bob), fill=TURTLE["eye"])
    if pose == "attack_1":  # open mouth
        d.line((hx + 13, 32 + bob, hx + 16, 32 + bob), fill=(15, 50, 35))
    # shell: dome + rim + plate lines
    d.ellipse(shell_box, fill=TURTLE["shell"], outline=(15, 50, 35))
    d.ellipse((11, 28 + bob, 41, 46 + bob), fill=TURTLE["rim"], outline=(15, 50, 35))
    d.ellipse((15, 31 + bob, 37, 43 + bob), fill=TURTLE["shell"])
    for px in (22, 28):  # scute seams
        d.line((px, 32 + bob, px, 42 + bob), fill=TURTLE["shell_dark"])
    return im

TURTLE_FRAMES = [("idle_0", 0.22), ("idle_1", 0.22), ("attack_0", 0.10), ("attack_1", 0.14), ("dead_0", 0.5)]
turtle_sheet = Image.new("RGBA", (T * len(TURTLE_FRAMES), T), (0, 0, 0, 0))
for i, (pose, _) in enumerate(TURTLE_FRAMES):
    turtle_sheet.paste(turtle_frame(pose), (i * T, 0))
turtle_sheet.save(f"{OUT}/turtle#sheet.png")
turtle_anim = {"anims": {
    "idle":   {"frames": [{"duration": TURTLE_FRAMES[0][1], "data": {"x": 0, "y": 0, "w": T, "h": T}},
                          {"duration": TURTLE_FRAMES[1][1], "data": {"x": T, "y": 0, "w": T, "h": T}}]},
    "attack": {"frames": [{"duration": TURTLE_FRAMES[2][1], "data": {"x": 2 * T, "y": 0, "w": T, "h": T}},
                          {"duration": TURTLE_FRAMES[3][1], "data": {"x": 3 * T, "y": 0, "w": T, "h": T}}]},
    "dead":   {"frames": [{"duration": TURTLE_FRAMES[4][1], "data": {"x": 4 * T, "y": 0, "w": T, "h": T}}]},
}}
with open(f"{OUT}/turtle#anim.fanim", "w") as f:
    json.dump(turtle_anim, f, indent=2)

# ---------------- ELEMENTAL LORD (128x128) ----------------
L = 128
def lord_frame(pose):
    im, d = canvas(L, L)
    bob = -3 if pose == "idle_1" else 0
    raised = pose.startswith("attack")
    dead = pose == "dead"
    robe = LORD["robe_dark"] if dead else LORD["robe"]
    trim = (60, 120, 140) if dead else LORD["trim"]
    # robe (trapezoid) + inner trim
    d.polygon([(34, 126), (44, 38 + bob), (84, 38 + bob), (94, 126)], fill=robe, outline=(6, 25, 55))
    d.polygon([(46, 126), (53, 48 + bob), (75, 48 + bob), (82, 126)], fill=LORD["robe_dark"])
    d.line((44, 38 + bob, 34, 126), fill=trim, width=2)
    d.line((84, 38 + bob, 94, 126), fill=trim, width=2)
    # belt + shoulders
    d.rectangle((46, 84 + bob, 82, 90 + bob), fill=trim)
    d.ellipse((36, 34 + bob, 56, 48 + bob), fill=LORD["skin"], outline=(6, 25, 55))
    d.ellipse((72, 34 + bob, 92, 48 + bob), fill=LORD["skin"], outline=(6, 25, 55))
    # head
    d.ellipse((48, 8 + bob, 80, 40 + bob), fill=LORD["skin"], outline=(6, 25, 55))
    d.rectangle((54, 30 + bob, 74, 40 + bob), fill=LORD["robe_dark"])  # beard shadow
    # glowing eyes
    d.point((57, 22 + bob), fill=LORD["eye"]); d.point((69, 22 + bob), fill=LORD["eye"])
    if dead:
        d.line((55, 21 + bob, 59, 23 + bob), fill=(0, 0, 0)); d.line((67, 23 + bob, 71, 21 + bob), fill=(0, 0, 0))
    # crown of coral spikes
    for cx in (52, 62, 72):
        d.polygon([(cx, 12 + bob), (cx + 4, 12 + bob), (cx + 2, 2 + bob)], fill=LORD["crown"], outline=(120, 90, 20))
    # chest core (glow)
    core = (30, 55, 70) if dead else LORD["glow"]
    d.ellipse((56, 56 + bob, 72, 72 + bob), fill=core, outline=(6, 25, 55))
    # trident (right hand)
    tx = 96
    ty0, ty1 = ((20, 86) if raised else (34, 110))
    ty0 += bob; ty1 += bob
    d.line((tx, ty0, tx, ty1), fill=(200, 230, 255), width=3)
    d.line((tx, ty0, tx - 8, ty0 - 12), fill=(200, 230, 255), width=2)
    d.line((tx, ty0, tx, ty0 - 16), fill=(200, 230, 255), width=2)
    d.line((tx, ty0, tx + 8, ty0 - 12), fill=(200, 230, 255), width=2)
    # orb in left hand while attacking
    if pose == "attack_1":
        d.ellipse((18, 56 + bob, 38, 76 + bob), fill=LORD["orb"], outline=(6, 25, 55))
        for ang in (0, 60, 120, 180, 240, 300):
            import math
            x = 28 + int(13 * math.cos(math.radians(ang)))
            y = 66 + bob + int(13 * math.sin(math.radians(ang)))
            d.point((x, y), fill=LORD["orb"])
    return im

LORD_FRAMES = [("idle_0", 0.25), ("idle_1", 0.25), ("attack_0", 0.16), ("attack_1", 0.22), ("dead_0", 0.6)]
lord_sheet = Image.new("RGBA", (L * len(LORD_FRAMES), L), (0, 0, 0, 0))
for i, (pose, _) in enumerate(LORD_FRAMES):
    lord_sheet.paste(lord_frame(pose), (i * L, 0))
lord_sheet.save(f"{OUT}/elemental_lord#sheet.png")
lord_anim = {"anims": {
    "idle":   {"frames": [{"duration": LORD_FRAMES[0][1], "data": {"x": 0, "y": 0, "w": L, "h": L}},
                          {"duration": LORD_FRAMES[1][1], "data": {"x": L, "y": 0, "w": L, "h": L}}]},
    "attack": {"frames": [{"duration": LORD_FRAMES[2][1], "data": {"x": 2 * L, "y": 0, "w": L, "h": L}},
                          {"duration": LORD_FRAMES[3][1], "data": {"x": 3 * L, "y": 0, "w": L, "h": L}}]},
    "dead":   {"frames": [{"duration": LORD_FRAMES[4][1], "data": {"x": 4 * L, "y": 0, "w": L, "h": L}}]},
}}
with open(f"{OUT}/elemental_lord#anim.fanim", "w") as f:
    json.dump(lord_anim, f, indent=2)

# ---------------- LORD BUFF ANIMATION (48x48, empower indicator) ----------------
B = 48
def buff_frame(shift, bright):
    im, d = canvas(B, B)
    col = (140, 240, 255) if bright else (80, 190, 230)
    for sx in (10, 24, 38):  # rising tide wisps
        base = 40 - shift
        d.line((sx, base, sx - 3, base - 8), fill=col, width=2)
        d.line((sx - 3, base - 8, sx + 1, base - 15), fill=col, width=2)
    for dx, dy in ((8, 28 - shift), (20, 36 - shift), (33, 26 - shift), (40, 34 - shift)):
        d.point((dx, dy), fill=(200, 250, 255))
    return im

buff_sheet = Image.new("RGBA", (B * 2, B), (0, 0, 0, 0))
buff_sheet.paste(buff_frame(0, False), (0, 0))
buff_sheet.paste(buff_frame(8, True), (B, 0))
buff_sheet.save(f"{OUT}/lord_buff_animation#sheet.png")
buff_anim = {"anims": {
    "idle": {"frames": [{"duration": 0.3, "data": {"x": 0, "y": 0, "w": B, "h": B}},
                        {"duration": 0.3, "data": {"x": B, "y": 0, "w": B, "h": B}}]},
}}
with open(f"{OUT}/lord_buff_animation#anim.fanim", "w") as f:
    json.dump(buff_anim, f, indent=2)

# ---------------- STICKERS (256x256, stills) ----------------
def sticker(fname, kind, team):
    im, d = canvas(256, 256)
    bg = (36, 88, 196) if team == "blue" else (196, 55, 55)
    d.ellipse((8, 8, 248, 248), fill=bg, outline=(245, 245, 245), width=10)
    inner = (18, 52, 130) if team == "blue" else (130, 30, 30)
    d.ellipse((36, 36, 220, 220), fill=inner)
    if kind == "turtle":
        d.ellipse((60, 96, 160, 190), fill=TURTLE["shell"], outline=(10, 40, 25), width=4)
        d.ellipse((72, 108, 148, 178), fill=TURTLE["rim"])
        d.ellipse((84, 120, 136, 166), fill=TURTLE["shell"])
        d.ellipse((140, 96, 206, 152), fill=TURTLE["skin"], outline=(10, 40, 25), width=4)
        d.point((186, 116), fill=(255, 255, 255)); d.point((187, 116), fill=TURTLE["eye"])
    else:
        d.polygon([(78, 220), (96, 96), (160, 96), (178, 220)], fill=LORD["robe"], outline=(5, 20, 50))
        d.ellipse((98, 34, 158, 100), fill=LORD["skin"], outline=(5, 20, 50))
        for cx in (104, 124, 144):
            d.polygon([(cx, 44), (cx + 8, 44), (cx + 4, 22)], fill=LORD["crown"])
        d.ellipse((116, 140, 140, 164), fill=LORD["glow"])
        d.line((176, 84, 176, 196), fill=(210, 235, 255), width=6)
        d.line((176, 84, 164, 64), fill=(210, 235, 255), width=4)
        d.line((176, 84, 176, 58), fill=(210, 235, 255), width=4)
        d.line((176, 84, 188, 64), fill=(210, 235, 255), width=4)
    im.save(f"{OUT}/{fname}.png")

sticker("turtle_sticker_blue", "turtle", "blue")
sticker("turtle_sticker_red", "turtle", "red")
sticker("lord_sticker_blue", "lord", "blue")
sticker("lord_sticker_red", "lord", "red")

# ---------------- THUMBNAIL (256x256) ----------------
im, d = canvas(256, 256)
d.rectangle((0, 0, 255, 255), fill=(10, 22, 40))
d.rectangle((0, 128, 255, 255), fill=(16, 34, 60))
turtle_s = turtle_frame("idle_0").resize((96, 96), Image.NEAREST)
lord_s = lord_frame("idle_0").resize((150, 150), Image.NEAREST)
im.paste(turtle_s, (18, 138), turtle_s)
im.paste(lord_s, (108, 96), lord_s)
d.rectangle((0, 236, 255, 255), fill=(90, 225, 255))
im.save("thumbnail.png")

print("done")
