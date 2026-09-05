#!/usr/bin/env python3
"""拼接渲染三件套：full@2x.png（宽 2400 == 1200×dpr、高 == 页面 CSS 高 × dpr，双硬断言）、
灰度版、缩略图、分节裁剪图；PIL 拼接后再经 magick 剥除 PNG 日期/时间等辅助块
（舰队配方：magick -strip，保证双跑字节稳定）。切片为瞬态中间产物，拼接后即清理。

环境变量：TREE_DIR 必填；RENDER_DPR 默认 2；MAGICK_BIN 默认 /opt/homebrew/bin/magick。
"""
from __future__ import annotations

import json
import os
import subprocess
import sys

from PIL import Image


def fail(msg: str) -> None:
    print(f"stitch: {msg}", file=sys.stderr)
    sys.exit(1)


TREE = os.environ.get("TREE_DIR") or fail("必须设置 TREE_DIR")
DPR = int(os.environ.get("RENDER_DPR", "2"))
MAGICK = os.environ.get("MAGICK_BIN", "/opt/homebrew/bin/magick")

with open(os.path.join(TREE, "render", "layout.json"), encoding="utf-8") as fh:
    layout = json.load(fh)

if not layout.get("scroll_assertions_ok"):
    fail("layout.json 滚动断言未全部通过，拒绝拼接")

slices_dir = os.path.join(TREE, "render", "slices")
names = sorted(os.listdir(slices_dir))
if len(names) != layout["slice_count"]:
    fail(f"切片数 {len(names)} != 记录 {layout['slice_count']}")

page_h = layout["page_height"]
target_h = page_h * DPR
canvas = Image.new("RGB", (1200 * DPR, target_h), "#ffffff")

placed = 0
for i, name in enumerate(names):
    img = Image.open(os.path.join(slices_dir, name)).convert("RGB")
    if img.width != 1200 * DPR:
        fail(f"切片 {name} 宽 {img.width} != {1200 * DPR}")
    y_css = layout["scrolls"][i]["scrollY"]
    canvas.paste(img, (0, y_css * DPR))
    placed += 1

render = os.path.join(TREE, "render")
full = os.path.join(render, "full@2x.png")
canvas.save(full, format="PNG", optimize=False)

w, h = canvas.size
if w != 1200 * DPR or h != target_h:
    fail(f"full@2x 尺寸 {w}x{h} != {1200 * DPR}x{target_h}（页高 {page_h} × {DPR}）")

gray = canvas.convert("L")
gray.save(os.path.join(render, "full@2x.gray.png"), format="PNG", optimize=False)

thumb = canvas.copy()
thumb.thumbnail((360, 100000), Image.LANCZOS)
thumb.save(os.path.join(render, "thumb.png"), format="PNG", optimize=False)

sections_dir = os.path.join(render, "sections")
os.makedirs(sections_dir, exist_ok=True)
for old in os.listdir(sections_dir):
    os.remove(os.path.join(sections_dir, old))
for i, sec in enumerate(layout["sections"], 1):
    top = sec["top"] * DPR
    height = sec["height"] * DPR
    crop = canvas.crop((0, max(0, top), 1200 * DPR, min(target_h, top + height)))
    crop.save(os.path.join(sections_dir, f"{i:02d}-{sec['id']}.png"),
              format="PNG", optimize=False)

# magick 剥除 PNG 日期/时间等辅助块（舰队配方，逐字节稳定）
if os.path.isfile(MAGICK):
    for rel in ("full@2x.png", "full@2x.gray.png", "thumb.png"):
        path = os.path.join(render, rel)
        tmp = path + ".tmp.png"
        proc = subprocess.run(
            [MAGICK, path, "-strip", tmp], capture_output=True, text=True
        )
        if proc.returncode != 0:
            fail(f"magick -strip 失败（{rel}）: {proc.stderr[:200]}")
        os.replace(tmp, path)
    for name in sorted(os.listdir(sections_dir)):
        path = os.path.join(sections_dir, name)
        tmp = path + ".tmp.png"
        proc = subprocess.run(
            [MAGICK, path, "-strip", tmp], capture_output=True, text=True
        )
        if proc.returncode != 0:
            fail(f"magick -strip 失败（{name}）: {proc.stderr[:200]}")
        os.replace(tmp, path)
else:
    fail(f"magick 缺失: {MAGICK}")

for name in names:
    os.remove(os.path.join(slices_dir, name))
os.rmdir(slices_dir)

print(
    f"stitch: full@2x.png {w}x{h}（{placed} 片 · 断言通过）"
    f"+ 灰度/缩略/{len(layout['sections'])} 张分节图（已剥辅助块）"
)
