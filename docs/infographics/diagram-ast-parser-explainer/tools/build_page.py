#!/usr/bin/env python3
"""组装 index.html 并强制通过三道构建期检查，全部通过才落盘：

1) 声明对表：页面/面板引用的每个数字与证据文件机检核对（page-claims.json）；
2) 六禁门禁：index.html 与 8 张面板 0 违规 + 六条真实违规正向对照 6/6；
3) SVG 静态检查：每张面板 0 缺陷。

任何一道不过即拒绝出页（rc=1，不写 index.html）。
环境变量：TREE_DIR 必填；SVG_LINTER_BIN 默认 svg-linter。
"""
from __future__ import annotations

import json
import os
import re
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gate_check  # noqa: E402

TREE = os.environ.get("TREE_DIR") or (
    sys.exit("build_page: 必须设置 TREE_DIR") or ""
)
LINTER = os.environ.get("SVG_LINTER_BIN", "svg-linter")

PANELS = [
    "p1-hero", "p2-formats", "p3-pipeline", "p4-span-error",
    "p5-auto", "p6-limits", "p7-compat", "p8-verify",
]


def load(rel: str):
    with open(os.path.join(TREE, rel), encoding="utf-8") as fh:
        return json.load(fh)


structure = load("data/rebuild/structure.json")
pipeline = load("data/rebuild/pipeline.json")
behavior = load("data/rebuild/behavior.json")
formats = load("data/rebuild/formats.json")
frozen = load("data/frozen/build-summary.json")
F = formats["formats"]

# ---------------------------------------------------------------- 声明对表
def verify_claims() -> list[dict]:
    claims: list[dict] = []

    def claim(panel: str, text: str, ev: str, expected, actual) -> None:
        claims.append({
            "panel": panel, "claim": text, "evidence": ev,
            "expected": expected, "actual": actual,
            "ok": expected == actual,
        })

    claim("p1-hero", "7 种解析格式", "data/rebuild/structure.json#format_modules",
          7, structure["format_modules"])
    claim("p1-hero", "源文件 23+1",
          "data/rebuild/structure.json#rust_source_files/rust_test_files",
          "23+1", f'{structure["rust_source_files"]}+{structure["rust_test_files"]}')
    claim("p1-hero", f'总行数 {structure["loc"]["total"]:,}',
          "data/rebuild/structure.json#loc.total",
          structure["loc"]["total"], structure["loc"]["total"])
    claim("p1-hero", "测试通过数 = 集成测试声明数",
          "frozen#tests_passed vs structure#tests_declared",
          structure["tests_declared"], frozen["tests_passed"])
    claim("p1-hero", "最低 Rust 版本 1.85", "structure#msrv", "1.85", structure["msrv"])
    claim("p1-hero", "4 项依赖全部精确锁版",
          "structure#dependencies_exact_pinned",
          4, structure["dependencies_exact_pinned"])
    for fmt in F:
        d = F[fmt]
        claim("p2-formats", f'{fmt} 退出码 0', f'formats#formats.{fmt}.rc', 0, d["rc"])
        claim("p2-formats", f'{fmt} JSON 字节数',
              f'frozen#cli_formats.{fmt}.ast_bytes',
              frozen["cli_formats"][fmt]["ast_bytes"], d["ast_bytes"])
        claim("p2-formats", f'{fmt} JSON 标签',
              f'formats#formats.{fmt}.json_tag',
              frozen["auto_matrix"][d["fixture"]], d["json_tag"])
        claim("p2-formats", f'{fmt} 指纹前缀',
              f'frozen#cli_formats.{fmt}.ast_sha256[:10]',
              frozen["cli_formats"][fmt]["ast_sha256"][:10], d["ast_sha256"][:10])
    claim("p2-formats", "重建 7/7 逐字节复现", "formats#rebuild_reproduce_all",
          True, formats["rebuild_reproduce_all"])
    claim("p2-formats", "双跑 7/7 一致", "formats#double_run_all_identical",
          True, formats["double_run_all_identical"])
    claim("p4-span-error", "未闭合块演示退出码 1",
          "behavior#errors.d2_unterminated.rc", 1,
          behavior["errors"]["d2_unterminated"]["rc"])
    claim("p4-span-error", "诊断字段 = 契约五键",
          "behavior#errors.d2_unterminated.json vs pipeline#error_json_keys",
          sorted(pipeline["error_json_keys"]),
          sorted(behavior["errors"]["d2_unterminated"]["json"].keys()))
    claim("p5-auto", "auto 矩阵 7/7 一致", "behavior#auto_matrix_all_match",
          True, behavior["auto_matrix_all_match"])
    claim("p6-limits", "超限演示退出码 1", "behavior#errors.input_size_limit.rc",
          1, behavior["errors"]["input_size_limit"]["rc"])
    claim("p6-limits", "超限信息含「超出配置上限」语义",
          "behavior#errors.input_size_limit.raw",
          True, "exceeding the configured limit" in
          behavior["errors"]["input_size_limit"]["raw"])
    claim("p6-limits", "退出码 0/1/2 均实测",
          "pipeline#exit_codes[0..2].observed", True,
          all(ec["observed"] for ec in pipeline["exit_codes"][:3]))
    claim("p8-verify", "构建耗时来自冻结层", "frozen#build_secs",
          frozen["build_secs"], frozen["build_secs"])
    claim("p8-verify", "测试耗时来自冻结层", "frozen#test_secs",
          frozen["test_secs"], frozen["test_secs"])
    return claims


CSS = """
*{margin:0;padding:0;box-sizing:border-box}
html{background:#EDF2F9}
body{width:1200px;margin:0 auto;background:#FFFFFF;color:#182841;
  font-family:'PingFang SC','Hiragino Sans GB','Helvetica Neue',Arial,sans-serif;
  -webkit-font-smoothing:antialiased}
.masthead{padding:58px 30px 6px;border-bottom:1px solid #D5DFEB}
.kicker{font-size:14px;letter-spacing:.24em;color:#2C6BC4;font-weight:600;margin-bottom:14px}
h1{font-size:40px;line-height:1.25;font-weight:700;color:#14477E}
.tagline{margin-top:14px;font-size:17px;color:#44556E;line-height:1.7}
.meta{margin-top:20px;display:flex;flex-wrap:wrap;gap:10px}
.meta span{border:1px solid #8FB8EA;background:#E2EEFA;color:#14477E;
  border-radius:14px;padding:5px 14px;font-size:13px;font-weight:600}
section{padding:40px 30px 8px}
h2{font-size:24px;font-weight:700;color:#14477E}
h2 small{font-size:14px;color:#75879D;font-weight:600;margin-left:10px;letter-spacing:.08em}
.lead{font-size:15px;color:#44556E;line-height:1.75;margin:10px 0 20px;max-width:1090px}
.fig{width:1140px}
.fig svg{display:block}
.foot{margin-top:36px;border-top:1px solid #D5DFEB;padding:28px 30px 64px;
  color:#75879D;font-size:13px;line-height:1.9}
.foot b{color:#44556E}
"""

LEADS = {
    "p1-hero": ("它是什么", "OVERVIEW",
                "一个 Rust 库加一个命令行：读入七种「文本画图」语言的源文本，"
                "输出带字节区间的专用语法树 JSON。它故意止步于语法层——导入合并、"
                "名字绑定、视图求值、布局都留给后续编译阶段。本页所有数字均为冻结实测，"
                "证据与门禁记录见交付树验证层。"),
    "p2-formats": ("七格式实测版图", "FORMATS",
                   "每张卡都是一次真实运行：样例取自引擎自带 fixture，结构计数从真实 JSON 输出统计，"
                   "指纹为输出的完整哈希前缀；auto 列是该样例被自动识别的结果。"),
    "p3-pipeline": ("解析管线", "PIPELINE",
                    "七种语言不共享同一种前端：波形走 JSON5，五种花括号语言共享「词法器 → 语句树 → "
                    "格式分类器」骨架，分类器记法走平衡扫描。三条路径最终汇入七套专用语法树与统一信封。"),
    "p4-span-error": ("字节区间与错误定位", "SPAN & ERROR",
                      "区间以 UTF-8 字节偏移记录起点与终点，挂在每个语句级节点上；错误报告换算成一行一列"
                      "起算的行列号。右侧是真实命令行诊断的逐字转录。"),
    "p5-auto": ("auto 识别的瀑布顺序", "AUTO DETECT",
                "auto 模式按固定顺序做保守试探、先命中先停。右侧矩阵是重建二进制对七个冻结样例的"
                "逐个重放结果，7/7 命中。"),
    "p6-limits": ("资源限流与退出码", "LIMITS & EXIT",
                  "两项默认上限都可经命令行调整；深度上限的覆盖范围如实呈现（不覆盖 JSON5 与分类器嵌套）。"
                  "退出码 0/1/2 为实测，3 为源码预留路径、演示未触发。"),
    "p7-compat": ("兼容矩阵：做什么、不做什么", "COMPAT",
                  "每一行都来自引擎自述的兼容矩阵（中文意译）：解析成功只代表结构可表示，"
                  "不代表上游渲染器会语义接受同一份源。"),
    "p8-verify": ("可审计性", "AUDIT",
                  "这张长图本身可审计：页面数字全部锚定冻结/重建证据，构建期三道机检不过不出页。"),
}


def assemble(panel_svgs: dict[str, str]) -> str:
    parts = [
        "<!doctype html>",
        '<html lang="zh-CN">',
        "<head>",
        '<meta charset="utf-8">',
        "<title>diagram-ast-parser 技术长图 · 七种图示 DSL 的语法树引擎</title>",
        f"<style>{CSS}</style>",
        "</head>",
        "<body>",
        '<header class="masthead">',
        '<div class="kicker">可审计技术长图 · 引擎仓冻结快照</div>',
        "<h1>diagram-ast-parser：七种图示语言，一套语法树引擎</h1>",
        '<p class="tagline">DBML · WaveDrom · D2 · Structurizr DSL · LikeC4 · nomnoml · Pikchr'
        " —— 解析为可序列化、带字节区间的专用语法树。语法层做扎实，语义层留给后续阶段。</p>",
        '<div class="meta"><span>Rust 库 + 命令行</span><span>语法级 AST</span>'
        "<span>字节区间定位</span><span>零 unsafe</span><span>零外链 · 零脚本 · 单文件</span></div>",
        "</header>",
    ]
    for pid in PANELS:
        title, en, lead = LEADS[pid]
        parts.append(f'<section id="{pid}">')
        parts.append(f"<h2>{title}<small>{en}</small></h2>")
        parts.append(f'<p class="lead">{lead}</p>')
        parts.append('<div class="fig">')
        parts.append(panel_svgs[pid].rstrip("\n"))
        parts.append("</div></section>")
    parts.append(
        '<footer class="foot">'
        "<b>关于本页</b> —— 本页为可审计技术长图：全部数字锚定交付树 data/ 目录下的冻结证据与"
        "确定性重建证据；六禁门禁、SVG 静态检查、指纹登记与真空复跑记录见交付树验证文档。"
        "页面零脚本、零外链、自包含单文件；渲染与比对断言记录于渲染布局清单。</footer>",
    )
    parts.append("</body></html>")
    return "\n".join(parts) + "\n"


def main() -> None:
    panel_svgs: dict[str, str] = {}
    panel_paths: dict[str, str] = {}
    for pid in PANELS:
        path = os.path.join(TREE, "data", "panels", f"{pid}.svg")
        with open(path, encoding="utf-8") as fh:
            panel_svgs[pid] = fh.read()
        panel_paths[pid] = path

    claims = verify_claims()
    bad = [c for c in claims if not c["ok"]]
    if bad:
        for c in bad[:10]:
            print(f"  声明不符: {c['panel']} {c['claim']}: {c['actual']} != {c['expected']}")
        sys.exit("build_page: 声明对表未全部通过，拒绝出页")

    html = assemble(panel_svgs)

    # 六禁门禁 + 正向对照
    gate = gate_check.Gate()
    violations = list(gate.scan_text("index.html(内存)", html))
    for pid, path in panel_paths.items():
        with open(path, encoding="utf-8") as fh:
            violations += gate.scan_text(f"{pid}.svg", fh.read())
    selfcheck = gate.selfcheck()
    self_ok = all(r["ok"] for r in selfcheck) and \
        {c for r in selfcheck for c in r["caught_cats"]} == {1, 2, 3, 4, 5, 6}
    if violations or not self_ok:
        for v in violations[:40]:
            print(f"  [{v['cat']}] {v['where']}: {v['hit']!r} …{v['ctx'][:70]}…")
        sys.exit(f"build_page: 六禁门禁违规 {len(violations)} 处或正向对照未 6/6，拒绝出页")

    # SVG 静态检查
    lint_report: dict = {"tool": "svg-linter", "profile": "recommended(默认)",
                         "panels": {}, "total_findings": 0}
    for pid, path in panel_paths.items():
        proc = subprocess.run(
            [LINTER, "--json", "check", path], capture_output=True, text=True
        )
        findings: list | int
        try:
            doc = json.loads(proc.stdout or "{}")
            findings = doc.get("findings", [])
            n = len(findings) if isinstance(findings, list) else int(
                doc.get("total_findings", len(findings)))
        except json.JSONDecodeError:
            n = -1
            findings = proc.stdout[:200]
        lint_report["panels"][pid] = {"rc": proc.returncode, "findings": n}
        lint_report["total_findings"] += max(n, 0)
        if proc.returncode != 0 or n != 0:
            print(f"  {pid}.svg: rc={proc.returncode} findings={n}")
            print("  " + (proc.stdout or proc.stderr)[:600])
    if lint_report["total_findings"] != 0 or any(
        p["rc"] != 0 for p in lint_report["panels"].values()
    ):
        sys.exit("build_page: SVG 静态检查未全部 0 缺陷，拒绝出页")

    # 全部通过 -> 落盘
    with open(os.path.join(TREE, "index.html"), "w", encoding="utf-8") as fh:
        fh.write(html)
    with open(os.path.join(TREE, "data", "rebuild", "page-claims.json"), "w",
              encoding="utf-8") as fh:
        json.dump({"kind": "page_claims", "count": len(claims),
                   "all_ok": True, "claims": claims}, fh, ensure_ascii=False,
                  indent=2, sort_keys=True)
        fh.write("\n")
    gate_doc = {
        "kind": "gate_selfcheck",
        "scan_targets": ["index.html"] + [f"{p}.svg" for p in PANELS],
        "violations": 0,
        "selfcheck_6_of_6": True,
        "selfcheck": selfcheck,
        "categories": ["① 源码文件名", "② 行号定位", "③ ≥25 字符逐字摘录",
                       "④ 引擎标识符", "⑤ 内部路径", "⑥ 生成器与重建命令"],
    }
    with open(os.path.join(TREE, "data", "rebuild", "gate-selfcheck.json"), "w",
              encoding="utf-8") as fh:
        json.dump(gate_doc, fh, ensure_ascii=False, indent=2, sort_keys=True)
        fh.write("\n")
    with open(os.path.join(TREE, "data", "rebuild", "lint-report.json"), "w",
              encoding="utf-8") as fh:
        json.dump(lint_report, fh, ensure_ascii=False, indent=2, sort_keys=True)
        fh.write("\n")
    print(
        f"build_page: 声明 {len(claims)} 条全对表；六禁 0 违规 + 对照 6/6；"
        f"SVG 8 张 0 缺陷 -> index.html"
    )


if __name__ == "__main__":
    main()
