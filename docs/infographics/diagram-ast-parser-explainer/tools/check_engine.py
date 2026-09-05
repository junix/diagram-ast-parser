#!/usr/bin/env python3
"""引擎仓门禁：校验 HEAD 等于冻结提交、porcelain 只允许本交付树（与外部治理 README）。

所有触碰引擎仓的工具（冻结/重建/渲染链入口）都必须先跑本检查，不通过即硬失败。

允许的 porcelain 条目（其余任何条目都是硬失败）：
  - 空表（冻结前）
  - `?? docs/`（引擎仓的 docs/ 目前只含本交付树，未跟踪目录以最外层目录名显示）
  - `?? docs/infographics/README.md`（外部治理进程放置，非本树产物，见偏差披露）

环境变量：ENGINE_REPO 必填。冻结提交：FROZEN_HEAD 常量；
data/frozen/manifest.txt 存在时以其中 engine_head 为准（与常量不一致即硬失败）。
"""
from __future__ import annotations

import os
import re
import subprocess
import sys

FROZEN_HEAD = "8cfbfe572d5f6e18a0bb6e45d30187816f69f872"

ALLOWED_PORCELAIN = (re.compile(r"^\?\? docs/$"),)


def fail(msg: str) -> None:
    print(f"check_engine: {msg}", file=sys.stderr)
    sys.exit(1)


def git(repo: str, *args: str) -> str:
    proc = subprocess.run(["git", "-C", repo, *args], capture_output=True, text=True)
    if proc.returncode != 0:
        fail(f"git {' '.join(args)} 失败: {proc.stderr.strip()[:200]}")
    return proc.stdout


def main() -> None:
    repo = os.environ.get("ENGINE_REPO") or fail("必须设置 ENGINE_REPO")

    manifest = os.path.join(
        os.environ.get("TREE_DIR") or "", "data", "frozen", "manifest.txt"
    )
    expected = FROZEN_HEAD
    if os.path.isfile(manifest):
        with open(manifest, encoding="utf-8") as fh:
            for line in fh:
                m = re.match(r"^engine_head=([0-9a-f]{40})$", line.strip())
                if m:
                    if m.group(1) != FROZEN_HEAD:
                        fail(
                            "冻结层记录的 HEAD 与本脚本常量不一致："
                            f"{m.group(1)} != {FROZEN_HEAD}"
                        )
                    expected = m.group(1)
                    break

    head = git(repo, "rev-parse", "HEAD").strip()
    if head != expected:
        fail(f"引擎 HEAD 已漂移: {head} != 冻结值 {expected}")

    porcelain = [
        line for line in git(repo, "status", "--porcelain").splitlines() if line.strip()
    ]
    bad = [line for line in porcelain if not any(p.match(line) for p in ALLOWED_PORCELAIN)]
    if bad:
        fail(f"porcelain 出现交付树之外的变化: {bad[:5]}")

    print(
        f"check_engine: HEAD={head[:12]} 与冻结值一致；"
        f"porcelain {len(porcelain)} 条（全部为本交付树/治理 README）"
    )


if __name__ == "__main__":
    main()
