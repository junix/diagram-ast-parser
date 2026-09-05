#!/usr/bin/env python3
"""产物指纹登记表：write 生成 data/fingerprints.json；check 以登记表为准核对全树。

登记范围：交付树内全部文件。排除（自指，须披露）：
  data/fingerprints.json（登记表自身——自指行记写前旧 sha 造成链式漂移）
  VERIFICATION.md（内嵌指纹表，自指）
  *.failed.txt（一次性冻结失败留档，非证据）
遇到 .DS_Store、__pycache__、*.pyc 直接硬失败（交付树内不应存在）。

环境变量：TREE_DIR 必填。用法：fingerprint.py write|check
"""
from __future__ import annotations

import hashlib
import json
import os
import sys

EXCLUDED = {"data/fingerprints.json", "VERIFICATION.md"}


def fail(msg: str) -> None:
    print(f"fingerprint: {msg}", file=sys.stderr)
    sys.exit(1)


TREE = os.environ.get("TREE_DIR") or fail("必须设置 TREE_DIR")
REG = os.path.join(TREE, "data", "fingerprints.json")


def sha256(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 16), b""):
            h.update(chunk)
    return h.hexdigest()


def scan() -> dict[str, str]:
    files: dict[str, str] = {}
    for root, dirs, names in os.walk(TREE):
        dirs[:] = [d for d in dirs if d != "__pycache__"]
        for name in names:
            full = os.path.join(root, name)
            rel = os.path.relpath(full, TREE)
            if name == ".DS_Store" or name.endswith(".pyc"):
                fail(f"交付树内出现违禁文件: {rel}")
            if rel in EXCLUDED or name.endswith(".failed.txt"):
                continue
            files[rel] = sha256(full)
    return dict(sorted(files.items()))


def write() -> None:
    files = scan()
    doc = {
        "algo": "sha256",
        "count": len(files),
        "excluded_self_referential": sorted(EXCLUDED),
        "files": files,
    }
    with open(REG, "w", encoding="utf-8") as fh:
        json.dump(doc, fh, ensure_ascii=False, indent=2, sort_keys=True)
        fh.write("\n")
    print(f"fingerprint: 登记 {len(files)} 项 -> data/fingerprints.json")


def check() -> None:
    if not os.path.isfile(REG):
        fail("登记表不存在，先执行 write")
    with open(REG, encoding="utf-8") as fh:
        reg = json.load(fh)
    disk = scan()
    registered = reg["files"]
    unregistered = sorted(set(disk) - set(registered))
    missing = sorted(set(registered) - set(disk))
    changed = sorted(k for k in set(disk) & set(registered) if disk[k] != registered[k])
    if unregistered:
        fail(f"未登记产物 {len(unregistered)} 项: {unregistered[:10]}")
    if missing:
        fail(f"登记表中已删除的产物 {len(missing)} 项: {missing[:10]}")
    if changed:
        for k in changed[:10]:
            print(f"  漂移: {k}\n    登记 {registered[k][:16]}… 实测 {disk[k][:16]}…")
        fail(f"指纹漂移 {len(changed)} 项")
    print(f"fingerprint: check 通过（{len(disk)} 项全部一致，无未登记产物）")


if __name__ == "__main__":
    if len(sys.argv) < 2 or sys.argv[1] not in ("write", "check"):
        fail("用法: fingerprint.py write|check")
    write() if sys.argv[1] == "write" else check()
