#!/usr/bin/env python3
"""真空比对：两棵交付树逐文件比对。

判据（README 预声明）：文本类逐字节 cmp；PNG 首选逐字节，不一致退路为
像素零差 + PNG 辅助块归一化（剥 tIME/tEXt/iTXt/zTXt 后逐字节）。
输出人类可读报告；发现任何实质差异 rc=1。

用法：TREE_DIR=<目录> cmp_artifacts.py <treeA> <treeB>
（TREE_DIR 用于跳过两树各自的登记表/验证文档自指文件。）
"""
from __future__ import annotations

import os
import struct
import sys

from PIL import Image

SKIP = {"data/fingerprints.json", "VERIFICATION.md"}
AUX_CHUNKS = {b"tIME", b"tEXt", b"iTXt", b"zTXt", b"eXIf"}


def fail(msg: str) -> None:
    print(f"cmp_artifacts: {msg}", file=sys.stderr)
    sys.exit(1)


def walk(root: str) -> dict[str, str]:
    out: dict[str, str] = {}
    for dirpath, dirnames, names in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in ("__pycache__",)]
        for n in names:
            if n == ".DS_Store" or n.endswith(".failed.txt"):
                continue
            full = os.path.join(dirpath, n)
            out[os.path.relpath(full, root)] = full
    return out


def png_norm(path: str) -> bytes:
    with open(path, "rb") as fh:
        data = fh.read()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        return data
    out = bytearray(data[:8])
    i = 8
    while i + 8 <= len(data):
        length = struct.unpack(">I", data[i:i + 4])[0]
        ctype = data[i + 4:i + 8]
        if ctype not in AUX_CHUNKS:
            out += data[i:i + 12 + length]
        i += 12 + length
    return bytes(out)


def png_pixels_equal(pa: str, pb: str) -> bool:
    ia, ib = Image.open(pa).convert("RGBA"), Image.open(pb).convert("RGBA")
    return ia.size == ib.size and ia.tobytes() == ib.tobytes()


def main() -> None:
    if len(sys.argv) != 3:
        fail("用法: cmp_artifacts.py <treeA> <treeB>")
    tree_a, tree_b = sys.argv[1], sys.argv[2]
    fa, fb = walk(tree_a), walk(tree_b)
    only_a = sorted(set(fa) - set(fb) - SKIP)
    only_b = sorted(set(fb) - set(fa) - SKIP)
    common = sorted(set(fa) & set(fb) - SKIP)
    problems: list[str] = []
    byte_eq = 0
    pixel_eq = 0
    text_eq = 0
    for rel in common:
        pa, pb = fa[rel], fb[rel]
        with open(pa, "rb") as fh:
            da = fh.read()
        with open(pb, "rb") as fh:
            db = fh.read()
        if da == db:
            if rel.endswith(".png"):
                byte_eq += 1
            else:
                text_eq += 1
            continue
        if rel.endswith(".png"):
            if png_norm(pa) == png_norm(pb) and png_pixels_equal(pa, pb):
                pixel_eq += 1
                print(f"  位图走退路（像素零差 + 辅助块归一化一致）: {rel}")
                continue
            problems.append(f"位图实质差异: {rel}")
        else:
            problems.append(f"文本字节差异: {rel}")
    for rel in only_a:
        problems.append(f"仅 A 树存在: {rel}")
    for rel in only_b:
        problems.append(f"仅 B 树存在: {rel}")
    print(f"比对文件 {len(common)} 项：文本逐字节一致 {text_eq}，"
          f"位图逐字节一致 {byte_eq}，位图退路一致 {pixel_eq}")
    print(f"独有文件：A {len(only_a)} / B {len(only_b)}（登记表与验证文档自指文件已跳过）")
    if problems:
        for p in problems[:20]:
            print(f"  ✗ {p}")
        fail(f"实质差异 {len(problems)} 项")
    print("cmp_artifacts: 两树产物等价（判据内一致）")


if __name__ == "__main__":
    main()
