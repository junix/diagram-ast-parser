#!/usr/bin/env python3
"""解析一次性冻结日志 -> data/frozen/build-summary.json（结构化摘要）。

从暂存目录（FROZEN_STAGING）读取 build-run.txt / test-run.txt / cli-matrix.txt /
err-*.rc，输出：耗时、测试计数、产物尺寸、7 格式 CLI 摘要（rc/bytes/sha）、
错误演示 rc、auto 识别矩阵。失败即硬失败（摘要不完整不给冻结）。
环境变量：FROZEN_STAGING、TREE_DIR 必填。
"""
from __future__ import annotations

import json
import os
import re
import sys


def fail(msg: str) -> None:
    print(f"parse_freeze: {msg}", file=sys.stderr)
    sys.exit(1)


STAGE = os.environ.get("FROZEN_STAGING") or fail("必须设置 FROZEN_STAGING")
TREE = os.environ.get("TREE_DIR") or fail("必须设置 TREE_DIR")


def read(name: str) -> str:
    path = os.path.join(STAGE, name)
    with open(path, encoding="utf-8") as fh:
        return fh.read()


def reals(text: str) -> list[float]:
    return [float(m) for m in re.findall(r"^real\s+([0-9.]+)$", text, re.M)]


build_txt = read("build-run.txt")
test_txt = read("test-run.txt")
cli_txt = read("cli-matrix.txt")

build_reals = reals(build_txt)
test_reals = reals(test_txt)
if not build_reals:
    fail("build-run.txt 缺 real 计时")
if not test_reals:
    fail("test-run.txt 缺 real 计时")

results = re.findall(
    r"test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured;",
    test_txt,
)
if not results:
    fail("test-run.txt 缺 test result 行")
passed = sum(int(r[0]) for r in results)
failed = sum(int(r[1]) for r in results)
ignored = sum(int(r[2]) for r in results)
suites = len(results)

m = re.search(r"^diagram-parse (\d+)$", build_txt, re.M)
if not m:
    fail("build-run.txt 缺产物尺寸行")
bin_bytes = int(m.group(1))

formats: dict[str, dict] = {}
for fmt, rc, nbytes, sha in re.findall(
    r"^\$ diagram-parse --format (\w+) --compact \S+$\n\s+rc=(\d+)\s+bytes=(\d+)\s+"
    r"sha256=([0-9a-f]{64})",
    cli_txt,
    re.M,
):
    formats[fmt] = {
        "rc": int(rc),
        "ast_bytes": int(nbytes),
        "ast_sha256": sha,
        "ast_file": f"ast/{fmt}.json",
    }
if len(formats) != 7:
    fail(f"cli-matrix.txt 格式记录 {len(formats)} 条（期望 7）")

err_rc: dict[str, int] = {}
for key in ("d2", "wavedrom", "size", "badfmt"):
    val = read(f"err-{key}.rc").strip()
    if not val.startswith("rc="):
        fail(f"err-{key}.rc 内容异常: {val}")
    err_rc[key] = int(val[3:])

auto: dict[str, str] = {}
for fx, tag in re.findall(r"^auto\((\S+)\) -> (\w+)$", cli_txt, re.M):
    auto[fx] = tag
if len(auto) != 7:
    fail(f"auto 识别矩阵 {len(auto)} 条（期望 7）")

summary = {
    "kind": "one_shot_frozen_summary",
    "build_secs": build_reals[0],
    "test_secs": test_reals[0],
    "test_suites": suites,
    "tests_passed": passed,
    "tests_failed": failed,
    "tests_ignored": ignored,
    "binary_name": "diagram-parse",
    "binary_bytes": bin_bytes,
    "cli_formats": formats,
    "error_demos_rc": err_rc,
    "auto_matrix": auto,
}
out = os.path.join(STAGE, "build-summary.json")
with open(out, "w", encoding="utf-8") as fh:
    json.dump(summary, fh, ensure_ascii=False, indent=2, sort_keys=True)
    fh.write("\n")
print(
    f"parse_freeze: 构建 {build_reals[0]}s / 测试 {test_reals[0]}s / "
    f"{passed} 通过（{suites} 套件）/ 7 格式 sha 齐 / 4 错误演示 rc 齐"
)
