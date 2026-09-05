#!/usr/bin/env python3
"""确定性行为探测：在 /tmp 工作拷贝重建引擎二进制，对 7 格式 fixture 重放解析，
逐字节比对冻结层 AST（重建=冻结 硬门禁），并采集错误演示/auto 矩阵/双跑一致性
-> data/rebuild/behavior.json、data/rebuild/formats.json。

产物不含任何路径/时间/版本字符串（构建在 $WORK 下进行，输出只落 rc/字节/sha/JSON），
双跑逐字节相同。环境变量：ENGINE_REPO、TREE_DIR、WORK_DIR 必填。
"""
from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys


def fail(msg: str) -> None:
    print(f"probe_behavior: {msg}", file=sys.stderr)
    sys.exit(1)


REPO = os.environ.get("ENGINE_REPO") or fail("必须设置 ENGINE_REPO")
TREE = os.environ.get("TREE_DIR") or fail("必须设置 TREE_DIR")
WORK = os.environ.get("WORK_DIR") or fail("必须设置 WORK_DIR")
CARGO = os.environ.get("CARGO_BIN", "cargo")

FORMATS = [
    ("dbml", "schema.dbml"),
    ("wavedrom", "timing.json5"),
    ("d2", "architecture.d2"),
    ("structurizr", "workspace.dsl"),
    ("likec4", "model.c4"),
    ("nomnoml", "classes.nomnoml"),
    ("pikchr", "flow.pikchr"),
]

# 引擎门禁
subprocess.run(
    [sys.executable, os.path.join(TREE, "tools", "check_engine.py")],
    check=True,
    cwd=os.path.join(TREE, "tools"),
)

# 平面拷贝 + 重建（cargo 输出不入产物；失败即硬失败）
SRC = os.path.join(WORK, "probe", "src")
TARGET = os.path.join(WORK, "probe", "target")
subprocess.run(["rm", "-rf", os.path.join(WORK, "probe")], check=True)
os.makedirs(SRC)
archive = subprocess.run(
    ["git", "-C", REPO, "archive", "HEAD"], stdout=subprocess.PIPE
)
subprocess.run(["tar", "-x", "-C", SRC], input=archive.stdout, check=True)
build = subprocess.run(
    [CARGO, "build", "--release", "--offline"],
    cwd=SRC,
    env={**os.environ, "CARGO_TARGET_DIR": TARGET},
    capture_output=True,
    text=True,
)
if build.returncode != 0:
    print(build.stderr[-2000:], file=sys.stderr)
    fail("重建 cargo build --release --offline 失败")
BIN = os.path.join(TARGET, "release", "diagram-parse")
if not os.path.isfile(BIN):
    fail("重建二进制缺失")

frozen_dir = os.path.join(TREE, "data", "frozen")
with open(os.path.join(frozen_dir, "build-summary.json"), encoding="utf-8") as fh:
    frozen = json.load(fh)


def run_cli(args: list[str], stdin: str | None = None) -> tuple[int, str, str]:
    proc = subprocess.run(
        [BIN, *args], capture_output=True, text=True, cwd=SRC, input=stdin
    )
    return proc.returncode, proc.stdout, proc.stderr


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def ast_counts(fmt: str, doc: dict) -> dict:
    ast = doc["ast"]
    if fmt == "dbml":
        kinds = [i["node"]["kind"] for i in ast["items"]]
        return {"items": len(ast["items"]), "item_kinds": kinds}
    if fmt == "wavedrom":
        timing = ast.get("timing") or {}
        signals = timing.get("signal") or []
        edges = timing.get("edges") or []
        return {
            "signals_top": len(signals),
            "edges": len(edges),
            "has_register": ast.get("register") is not None,
        }
    if fmt == "nomnoml":
        return {
            "directives": len(ast.get("directives") or []),
            "statements": len(ast.get("statements") or []),
        }
    return {"statements": len(ast.get("statements") or [])}


formats_doc: dict[str, dict] = {}
rebuild_matches_frozen = {}
double_run_identical = {}
for fmt, fixture in FORMATS:
    rc1, out1, _ = run_cli(["--format", fmt, "--compact", f"examples/{fixture}"])
    rc2, out2, _ = run_cli(["--format", fmt, "--compact", f"examples/{fixture}"])
    if rc1 != 0 or rc2 != 0:
        fail(f"{fmt} 重放 rc={rc1}/{rc2}")
    double_run_identical[fmt] = out1 == out2
    with open(os.path.join(frozen_dir, "ast", f"{fmt}.json"), "rb") as fh:
        frozen_bytes = fh.read()
    repro = out1.encode() == frozen_bytes
    sha = sha256_bytes(out1.encode())
    frozen_sha = frozen["cli_formats"][fmt]["ast_sha256"]
    if sha != frozen_sha:
        fail(f"{fmt} 重建 AST sha 与冻结层不一致: {sha} != {frozen_sha}")
    if not repro:
        fail(f"{fmt} 重建 AST 与冻结文件不逐字节一致")
    rebuild_matches_frozen[fmt] = True
    doc = json.loads(out1)
    formats_doc[fmt] = {
        "fixture": fixture,
        "rc": rc1,
        "json_tag": doc["format"],
        "ast_bytes": frozen["cli_formats"][fmt]["ast_bytes"],
        "ast_sha256": sha,
        "ast_shape": ast_counts(fmt, doc),
    }

# auto 识别矩阵（与冻结层比对）
auto_observed: dict[str, str] = {}
for fmt, fixture in FORMATS:
    rc, out, _ = run_cli(["--format", "auto", "--compact", f"examples/{fixture}"])
    if rc != 0:
        fail(f"auto({fixture}) rc={rc}")
    tag = json.loads(out)["format"]
    if tag != frozen["auto_matrix"][fixture]:
        fail(f"auto({fixture}) 识别 {tag} 与冻结层 {frozen['auto_matrix'][fixture]} 不一致")
    auto_observed[fixture] = tag

# 错误演示（stdin；输出与 rc 原样采集，无路径入产物）
def err_demo(key: str, args: list[str], stdin: str) -> dict:
    rc, out, err = run_cli(args, stdin=stdin)
    frozen_rc = frozen["error_demos_rc"][key]
    if rc != frozen_rc:
        fail(f"错误演示 {key} rc={rc} 与冻结层 {frozen_rc} 不一致")
    payload = out if out.strip() else err
    try:
        parsed = json.loads(payload)
    except json.JSONDecodeError:
        parsed = None
    return {"rc": rc, "json": parsed, "raw": payload.strip()}


errors = {
    "d2_unterminated": err_demo(
        "d2", ["--format", "d2", "--diagnostic-json", "-"], "a: {\n  b: c\n"
    ),
    "wavedrom_negative_reg_width": err_demo(
        "wavedrom",
        ["--format", "wavedrom", "--diagnostic-json", "-"],
        "{ reg: [{ bits: -1, name: 'reserved' }] }",
    ),
    "input_size_limit": err_demo(
        "size", ["--format", "d2", "--max-input-bytes", "4", "-"], "a: {\n  b: c\n"
    ),
    "bad_format_value": err_demo(
        "badfmt", ["--format", "nope", "-"], "a: {\n  b: c\n"
    ),
}
# auto 波形样本：成功演示（rc=0），不属于错误演示，不做错误 rc 比对
rc_aw, out_aw, _ = run_cli(
    ["--format", "auto", "--compact", "-"],
    '{ signal: [{ name: "clk", wave: "p..." }] }',
)
if rc_aw != 0:
    fail(f"auto 波形样本 rc={rc_aw} 非 0")
tag_aw = json.loads(out_aw)["format"]
if tag_aw != "wave_drom":
    fail(f"auto 波形样本识别为 {tag_aw} != wave_drom")
with open(os.path.join(frozen_dir, "auto-wave.json"), "rb") as fh:
    if out_aw.encode() != fh.read():
        fail("auto 波形样本输出与冻结层 auto-wave.json 不逐字节一致")

# 带诊断开关的两组必须产出 JSON；超限/非法 format 两组按冻结层事实为纯文本（拒绝先于解析）
for key in ("d2_unterminated", "wavedrom_negative_reg_width"):
    if errors[key]["json"] is None:
        fail(f"错误演示 {key} 未产出 JSON 诊断")
for key in ("input_size_limit", "bad_format_value"):
    if errors[key]["json"] is not None:
        fail(f"错误演示 {key} 与冻结层事实不符（应为纯文本诊断）")

behavior = {
    "kind": "behavior_probe",
    "rebuilt_binary_reproduces_frozen_ast": rebuild_matches_frozen,
    "double_run_identical": double_run_identical,
    "auto_matrix_observed": auto_observed,
    "auto_matrix_expected": frozen["auto_matrix"],
    "auto_matrix_all_match": auto_observed == frozen["auto_matrix"],
    "errors": errors,
    "auto_wavejson_sample": {
        "rc": rc_aw,
        "json_tag": json.loads(out_aw)["format"],
    },
}

domain = {
    "dbml": "数据库建模",
    "wavedrom": "时序波形 / 寄存器",
    "d2": "声明式地图与边",
    "structurizr": "架构模型 + 视图",
    "likec4": "架构模型 + 规约",
    "nomnoml": "分类器与关系图",
    "pikchr": "过程式几何绘图",
}
formats_out = {
    "kind": "format_facts",
    "formats": {
        fmt: {**data, "domain": domain[fmt]} for fmt, data in formats_doc.items()
    },
    "rebuild_reproduce_all": all(rebuild_matches_frozen.values()),
    "double_run_all_identical": all(double_run_identical.values()),
}

out_dir = os.path.join(TREE, "data", "rebuild")
os.makedirs(out_dir, exist_ok=True)
for name, doc in (("behavior.json", behavior), ("formats.json", formats_out)):
    with open(os.path.join(out_dir, name), "w", encoding="utf-8") as fh:
        json.dump(doc, fh, ensure_ascii=False, indent=2, sort_keys=True)
        fh.write("\n")
print(
    f"probe_behavior: 重建二进制 7/7 逐字节复现冻结 AST；"
    f"auto 矩阵 {len(auto_observed)}/7 一致；错误演示 4 组 rc 一致"
)
