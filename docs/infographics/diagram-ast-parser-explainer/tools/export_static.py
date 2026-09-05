#!/usr/bin/env python3
"""确定性静态导出：从引擎冻结提交导出结构普查 -> data/rebuild/structure.json，
并生成管线/兼容矩阵数据 -> data/rebuild/pipeline.json、data/rebuild/compat.json。

全部内容只读引擎仓 git 对象（git show HEAD:<path>），不碰工作区；
输出不含任何路径/时间/版本等环境字符串，双跑逐字节相同。
环境变量：ENGINE_REPO、TREE_DIR 必填（先过 check_engine.py）。
"""
from __future__ import annotations

import json
import os
import re
import subprocess
import sys


def fail(msg: str) -> None:
    print(f"export_static: {msg}", file=sys.stderr)
    sys.exit(1)


REPO = os.environ.get("ENGINE_REPO") or fail("必须设置 ENGINE_REPO")
TREE = os.environ.get("TREE_DIR") or fail("必须设置 TREE_DIR")


def git_show(path: str) -> str:
    proc = subprocess.run(
        ["git", "-C", REPO, "show", f"HEAD:{path}"], capture_output=True, text=True
    )
    if proc.returncode != 0:
        fail(f"git show HEAD:{path} 失败: {proc.stderr.strip()[:120]}")
    return proc.stdout


def git_ls() -> list[str]:
    proc = subprocess.run(
        ["git", "-C", REPO, "ls-files"], capture_output=True, text=True
    )
    if proc.returncode != 0:
        fail("git ls-files 失败")
    return [line for line in proc.stdout.splitlines() if line.strip()]


files = git_ls()
rust_src = sorted(p for p in files if p.startswith("src/") and p.endswith(".rs"))
rust_tests = sorted(p for p in files if p.startswith("tests/") and p.endswith(".rs"))
fixtures = sorted(p for p in files if p.startswith("examples/"))

groups: dict[str, list[str]] = {
    "core": [p for p in rust_src if "/" not in p[len("src/"):]],
    "ast": [p for p in rust_src if p.startswith("src/ast/")],
    "parser": [p for p in rust_src if p.startswith("src/parser/")],
}
if sum(len(v) for v in groups.values()) != len(rust_src):
    fail("src 分组不完整")

loc: dict[str, dict] = {}
for path in rust_src + rust_tests:
    text = git_show(path)
    loc[path] = {"lines": len(text.splitlines()), "bytes": len(text.encode())}

group_loc = {
    g: sum(loc[p]["lines"] for p in ps) for g, ps in groups.items()
}
test_loc = sum(loc[p]["lines"] for p in rust_tests)

tests_file = rust_tests[0] if len(rust_tests) == 1 else fail("期望恰一个集成测试文件")
tests_declared = len(re.findall(r"^#\[(?:tokio::)?test\]", git_show(tests_file), re.M))

lib_rs = git_show("src/lib.rs")
if "#![forbid(unsafe_code)]" not in lib_rs:
    fail("lib.rs 缺 forbid(unsafe_code) 声明（结构事实漂移）")

cargo_toml = git_show("Cargo.toml")
msrv_m = re.search(r'^rust-version\s*=\s*"([^"]+)"', cargo_toml, re.M)
msrv = msrv_m.group(1) if msrv_m else ""
deps: list[dict] = []
if "[dependencies]" in cargo_toml:
    deps_section = cargo_toml.split("[dependencies]", 1)[1]
    deps_section = deps_section.split("\n[", 1)[0]
    for m in re.finditer(
        r'^(\w[\w-]*)\s*=\s*(?:\{\s*version\s*=\s*)?"([^"]+)"', deps_section, re.M
    ):
        deps.append({"name": m.group(1), "version_spec": m.group(2)})
exact = [d for d in deps if d["version_spec"].startswith("=")]

format_modules = sorted(
    p for p in groups["ast"] if p != "src/ast/mod.rs"
)

structure = {
    "kind": "engine_census",
    "source": "git 对象（HEAD），非工作区",
    "rust_source_files": len(rust_src),
    "rust_test_files": len(rust_tests),
    "groups": {g: len(ps) for g, ps in groups.items()},
    "loc": {
        "core": group_loc["core"],
        "ast_types": group_loc["ast"],
        "parsers": group_loc["parser"],
        "integration_tests": test_loc,
        "src_total": sum(group_loc.values()),
        "total": sum(group_loc.values()) + test_loc,
    },
    "per_file_loc": {p: loc[p]["lines"] for p in sorted(loc)},
    "tests_declared": tests_declared,
    "forbid_unsafe": True,
    "msrv": msrv,
    "dependencies": deps,
    "dependencies_exact_pinned": len(exact),
    "format_modules": len(format_modules),
    "fixtures": [
        {"name": os.path.basename(p), "bytes": len(git_show(p).encode()),
         "lines": len(git_show(p).splitlines())}
        for p in fixtures
    ],
    "fixture_count": len(fixtures),
}

pipeline = {
    "kind": "parse_pipeline",
    "paths": [
        {
            "id": "json5",
            "formats": ["wavedrom"],
            "stages": ["JSON5 输入", "JSON5 解析器", "类型化时序/寄存器 AST"],
            "note": "信号组、数据道、节点/相位/边、寄存器字段；未知字段原样保留",
        },
        {
            "id": "braced",
            "formats": ["dbml", "d2", "structurizr", "likec4", "pikchr"],
            "stages": [
                "源文本",
                "可配置词法器",
                "花括号语句树",
                "格式分类器 + 类型化 AST 构建器",
            ],
            "note": "五种语言共享词法与树骨架，各自落到专用 AST",
        },
        {
            "id": "nomnoml",
            "formats": ["nomnoml"],
            "stages": ["源文本", "平衡分类器扫描器", "分类器/关系 AST"],
            "note": "# 指令行与 [分类器] 记法按括号平衡切分",
        },
    ],
    "lexer_capabilities": [
        "单引号 / 双引号 / 三引号 / DBML 反引号字符串（按语言启用）",
        "行注释 / 块注释按语言选择",
        "拒绝不闭合的字符串与块定界符",
        "花括号系解析器受嵌套深度上限约束",
        "JSON5 嵌套与 nomnoml 嵌套分类文本当前不受深度上限约束（引擎如实声明）",
    ],
    "limits": {
        "max_input_bytes_default": 8388608,
        "max_input_bytes_display": "8 MiB",
        "max_nesting_depth_default": 128,
        "cli_flags": ["--max-input-bytes", "--max-depth"],
    },
    "auto_detect_order": [
        {"format": "wave_drom", "trigger": "以 { 开头且出现 signal: 或 reg: 键"},
        {"format": "like_c4", "trigger": "出现 specification 且伴随 model / views"},
        {"format": "structurizr", "trigger": "出现 workspace 且伴随 softwaresystem / systemcontext / container"},
        {"format": "dbml", "trigger": "出现 table 表语句 / ref: / enum 之一"},
        {"format": "nomnoml", "trigger": "存在 # 或 [ 开头的行且文本含方括号"},
        {"format": "pikchr", "trigger": "行首词命中几何对象类型（box / circle / arrow 等）"},
        {"format": "d2", "trigger": "以上都不命中时的兜底"},
    ],
    "exit_codes": [
        {"code": 0, "meaning": "解析成功，stdout 输出 JSON AST", "observed": True},
        {"code": 1, "meaning": "解析失败（诊断走 stderr，可要求 JSON 形态）", "observed": True},
        {"code": 2, "meaning": "用法或输入输出错误（非法 format 值 / 文件不可读）", "observed": True},
        {"code": 3, "meaning": "AST 序列化失败（源码中预留的路径，演示未触发）", "observed": False},
    ],
    "error_json_keys": ["format", "message", "span", "line", "column"],
    "span_model": {
        "fields": ["start", "end"],
        "unit": "UTF-8 字节偏移",
        "wrapper": "每个语句级节点都携带定位区间",
        "line_column": "一行一列起算（错误报告用）",
    },
    "document_envelope": {"tag": "format", "content": "ast", "case": "snake_case"},
}

compat = {
    "kind": "compatibility_matrix",
    "source": "引擎 README 兼容性矩阵的中文意译（见偏差披露）",
    "rows": [
        {
            "format": "DBML",
            "implemented": "项目、表、表局部、列与设置、索引、检查约束、枚举、引用、表组、注记",
            "deferred": "模块导入解析、数据血缘扩展、SQL 生成、语义重复/类型检查",
        },
        {
            "format": "WaveDrom",
            "implemented": "JSON5 输入、嵌套信号组、lanes、data、节点、相位/周期、边、头尾、寄存器字段、未知字段保留",
            "deferred": "波形符号校验、边语法校验、渲染语义",
        },
        {
            "format": "D2",
            "implemented": "条目/映射、标量标签、边链、边属性映射、常规/展开导入",
            "deferred": "替换、变量、通配、类、导入装载、形状/属性校验、board/layer/scenario 语义",
        },
        {
            "format": "Structurizr DSL",
            "implemented": "workspace、模型层级、公共元素、关系、指令、通用块/属性、视图即块",
            "deferred": "include/script/plugin、表达式求值、标识符解析、隐含关系、原型展开、视图计算",
        },
        {
            "format": "LikeC4",
            "implemented": "specification 种类、模型层级、普通/带种类关系、标签、视图、extend、部署节点与实例引用",
            "deferred": "多文件合并、词法作用域解析、谓词、继承、部署展开、视图计算、样式校验",
        },
        {
            "format": "nomnoml",
            "implemented": "指令/自定义样式、分类器类型/属性、隔间、二元关系与端点标签",
            "deferred": "超过两个顶层分类器的关系链、布局/配置校验、嵌套图语义分析",
        },
        {
            "format": "Pikchr",
            "implemented": "对象、标签、方向、赋值、define 块、print、assert、命名位置；属性以 token 保留",
            "deferred": "表达式优先级 AST、对象/位置引用解析、宏展开、几何求值",
        },
    ],
    "note": "解析成功只意味着源在本项目 AST 中结构可表示，不代表上游渲染器会语义接受",
}

out_dir = os.path.join(TREE, "data", "rebuild")
os.makedirs(out_dir, exist_ok=True)
for name, doc in (
    ("structure.json", structure),
    ("pipeline.json", pipeline),
    ("compat.json", compat),
):
    with open(os.path.join(out_dir, name), "w", encoding="utf-8") as fh:
        json.dump(doc, fh, ensure_ascii=False, indent=2, sort_keys=True)
        fh.write("\n")
print(
    f"export_static: 源文件 {structure['rust_source_files']}+{structure['rust_test_files']}"
    f" / LOC {structure['loc']['total']} / 测试声明 {tests_declared} / 依赖 {len(deps)}"
)
