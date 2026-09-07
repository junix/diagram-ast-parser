#!/usr/bin/env python3
"""引擎仓门禁：两态守卫（引擎已演进 vs 证据漂移）+ porcelain 白名单。

所有触碰引擎仓的工具（冻结/重建/渲染链入口）都必须先跑本检查。

两态守卫（2026-09-06 refine 起，替代旧的 HEAD 一律硬锁）：
  - 「证据漂移」＝硬失败：data/frozen/manifest.txt 的 engine_head 与本脚本
    FROZEN_HEAD 常量不一致（冻结层或工具任一方被篡改）。
  - 「引擎已演进」＝stderr 警告 + 放行（rc=0）：HEAD 不等于冻结值。交付提交
    落地后主检出 HEAD 必然演进，这不是证据问题；但构建/探测会取到演进后的
    源码，完全复现冻结证据请按 README「复跑步骤」开头的冻结 worktree 配方
    （git -C <引擎仓> worktree add <dir> <冻结SHA>）把 ENGINE_REPO 指到冻结提交。

porcelain 检查在两种状态下都硬失败（允许条目，其余任何条目都不放行）：
  - 空表（干净检出/worktree）
  - `?? docs/`（引擎仓的 docs/ 目前只含本交付树，未跟踪目录以最外层目录名显示）
  - `?? docs/infographics/README.md`（外部治理进程放置，非本树产物，见偏差披露）

环境变量：ENGINE_REPO 必填；TREE_DIR 用于定位 data/frozen/manifest.txt
（存在时以其中 engine_head 为准，并与 FROZEN_HEAD 常量互校）。
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


def warn(msg: str) -> None:
    print(f"check_engine: {msg}", file=sys.stderr)


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
                            "证据漂移（硬失败）：冻结层记录的 HEAD 与本脚本常量不一致："
                            f"{m.group(1)} != {FROZEN_HEAD}"
                        )
                    expected = m.group(1)
                    break

    head = git(repo, "rev-parse", "HEAD").strip()

    porcelain = [
        line for line in git(repo, "status", "--porcelain").splitlines() if line.strip()
    ]
    bad = [line for line in porcelain if not any(p.match(line) for p in ALLOWED_PORCELAIN)]
    if bad:
        fail(f"porcelain 出现交付树之外的变化: {bad[:5]}")

    if head != expected:
        # 引擎已演进：警告放行（不硬失败）。完全复现请按 README 冻结 worktree 配方。
        warn(
            f"引擎已演进（警告，放行）: HEAD={head[:12]} != 冻结值 {expected[:12]}。"
            "交付提交落地后属正常状态；构建/探测将取到演进后的源码。"
            "完全复现冻结证据请按 README「复跑步骤」开头的配方："
            f"git -C <引擎仓> worktree add <dir> {expected} 并把 ENGINE_REPO 指向该 worktree"
        )
        print(
            f"check_engine: 引擎已演进，警告放行（配方见 README 复跑步骤）；"
            f"porcelain {len(porcelain)} 条（全部为本交付树/治理 README）"
        )
        return

    print(
        f"check_engine: HEAD={head[:12]} 与冻结值一致；"
        f"porcelain {len(porcelain)} 条（全部为本交付树/治理 README）"
    )


if __name__ == "__main__":
    main()
