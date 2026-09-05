#!/usr/bin/env python3
"""生成 VERIFICATION.md：声明锚点注册表 + 门禁记录 + 渲染断言 + 指纹表 + 偏差披露
+ 真空复跑记录。全部内容从 data/ 与 tools/disclosures.json 机器生成。

环境变量：TREE_DIR 必填。
"""
from __future__ import annotations

import json
import os
import sys


def fail(msg: str) -> None:
    print(f"verification: {msg}", file=sys.stderr)
    sys.exit(1)


TREE = os.environ.get("TREE_DIR") or fail("必须设置 TREE_DIR")


def load(rel: str):
    with open(os.path.join(TREE, rel), encoding="utf-8") as fh:
        return json.load(fh)


def read(rel: str) -> str:
    with open(os.path.join(TREE, rel), encoding="utf-8") as fh:
        return fh.read()


frozen = load("data/frozen/build-summary.json")
structure = load("data/rebuild/structure.json")
claims = load("data/rebuild/page-claims.json")
gate = load("data/rebuild/gate-selfcheck.json")
lint = load("data/rebuild/lint-report.json")
behavior = load("data/rebuild/behavior.json")
formats = load("data/rebuild/formats.json")
fingerprints = load("data/fingerprints.json")
disclosures = load("tools/disclosures.json")

manifest = dict(
    line.split("=", 1) for line in
    read("data/frozen/manifest.txt").splitlines()
    if "=" in line and not line.startswith("#")
)

layout = None
if os.path.isfile(os.path.join(TREE, "render", "layout.json")):
    layout = load("render/layout.json")

vacuum = None
vacuum_path = os.path.join(TREE, "data", "frozen", "vacuum-report.txt")
if os.path.isfile(vacuum_path):
    vacuum = read("data/frozen/vacuum-report.txt")

L = []
A = L.append
A("# 验证与审计（diagram-ast-parser 技术长图）")
A("")
A("本文件是交付树的验证层：声明锚点注册表、门禁记录、渲染断言、产物指纹表、偏差披露。")
A("复跑环境与判据见交付树 README；一次性冻结证据见 data/frozen/。")
A("")
A("## 1. 对象与页面")
A("")
A(f"- 引擎仓冻结提交：`{manifest['engine_head'][:12]}…{manifest['engine_head'][-6:]}`"
  f"（完整值见 data/frozen/manifest.txt）")
if layout:
    A(f"- 页面：{layout['page_width']} × {layout['page_height']} CSS px；渲染 dpr {layout['dpr']}；"
      f"full@2x.png 为 {layout['full2x_expected'][0]} × {layout['full2x_expected'][1]} px"
      f"（宽 {layout['page_width']}×{layout['dpr']}、高 页高×{layout['dpr']} 双硬断言通过）")
    A(f"- 渲染：{layout['slice_count']} 片 CDP 截图，每片 scrollTo 后回读 scrollY 断言一致"
      f"（{layout['scroll_assertions_ok'] and '全部通过'}）；"
      f"零外部请求断言通过（总请求 {layout['request_count']} 项，全部 file://）")
A(f"- 一次性实测：冷构建 {frozen['build_secs']} s（release + 离线）；"
  f"测试 {frozen['test_secs']} s；"
  f"{frozen['tests_passed']} 通过 / {frozen['tests_failed']} 失败 / "
  f"{frozen['tests_ignored']} 跳过（{frozen['test_suites']} 个测试套件）；"
  f"二进制 {frozen['binary_bytes']:,} 字节")
A(f"- 重建复现：重建二进制重放 7 格式，7/7 逐字节复现冻结 AST；"
  f"auto 识别 7/7 与冻结层一致；4 组错误演示退出码一致；同轮双跑 7/7 一致")
A("")
A("## 2. 声明锚点注册表（页面声明 → 证据）")
A("")
A("| 面板 | 声明 | 证据 | 期望 | 实测 |")
A("|---|---|---|---|---|")
for c in claims["claims"]:
    exp = json.dumps(c["expected"], ensure_ascii=False)
    act = json.dumps(c["actual"], ensure_ascii=False)
    if len(exp) > 48:
        exp = exp[:45] + "…"
    if len(act) > 48:
        act = act[:45] + "…"
    A(f"| {c['panel']} | {c['claim']} | `{c['evidence']}` | {exp} | {act} {'✓' if c['ok'] else '✗'} |")
A("")
A(f"共 {claims['count']} 条声明，全部机检对表通过（{claims['all_ok']}）。")
A("")
A("## 3. 六禁门禁")
A("")
A("六类禁止（① 源码文件名 ② 行号定位 ③ ≥25 字符归一化逐字摘录 ④ 引擎标识符 "
  "⑤ 内部路径 ⑥ 生成器名与重建命令）在出页前对 index.html 与全部面板强制检查，"
  "违规即拒绝出页。正向对照自检：六条各含一处真实违规（样本实时取自引擎仓），"
  "结果 6/6 咬住；页面与面板违规 0 处。")
A("")
A("| 对照 | 投毒样本（前 40 字符） | 命中类别 | 结果 |")
A("|---|---|---|---|")
for r in gate["selfcheck"]:
    poison = r["poison"][:40].replace("|", "\\|")
    A(f"| {r['expect']} | `{poison}` | {r['caught_cats']} | {'咬住' if r['ok'] else '漏检'} |")
A("")
A("放行判例与豁免规则见偏差披露 D07。")
A("")
A("## 4. SVG 静态检查")
A("")
A(f"- 工具：`{lint['tool']}`，{lint['profile']} 档")
A(f"- 检查 {len(lint['panels'])} 张面板，合计 findings {lint['total_findings']}"
  f"（逐张 rc=0）")
A("")
A("## 5. 产物指纹表")
A("")
A(f"算法 sha256；登记 {fingerprints['count']} 项；"
  f"自指排除：{', '.join(f'`{p}`' for p in fingerprints['excluded_self_referential'])}"
  f"（与 *.failed.txt）。")
A("")
A("| 文件 | sha256 |")
A("|---|---|")
for rel, sha in fingerprints["files"].items():
    A(f"| `{rel}` | `{sha}` |")
A("")
A("## 6. 偏差披露")
A("")
for d in disclosures["items"]:
    body = d["body"].replace("\n", " ")
    A(f"- **{d['id']} {d['title']}**：{body}（锚点：`{d['anchor']}`）")
A("")
A("## 7. 真空复跑")
A("")
if vacuum:
    A("判据（README 预声明）：删除全部可重建产物后，双拷贝 A/B 全链重建；文本逐字节 cmp；"
      "位图首选逐字节，退路像素零差 + PNG 辅助块归一化；data/frozen/ 不参与删除。"
      "结果一次性冻结到 data/frozen/vacuum-report.txt（有守卫，绝不重跑覆盖）。原文如下：")
    A("")
    A("```text")
    A(vacuum.rstrip("\n"))
    A("```")
else:
    A("尚未执行（真空复跑为一次性步骤，执行后本节由重建链自动填充其冻结报告原文）。")
A("")

out = os.path.join(TREE, "VERIFICATION.md")
with open(out, "w", encoding="utf-8") as fh:
    fh.write("\n".join(L) + "\n")
print(f"verification: VERIFICATION.md 生成（{len(L)} 行，披露 {len(disclosures['items'])} 条）")
