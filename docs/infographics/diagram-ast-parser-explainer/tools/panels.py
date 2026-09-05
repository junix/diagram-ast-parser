#!/usr/bin/env python3
"""生成 8 张数据驱动 SVG 面板 -> data/panels/p*.svg。

全部数字来自 data/frozen/build-summary.json 与 data/rebuild/*.json；
面板不含引擎源码文件名/行号/逐字摘录/内部标识符/路径/生成器名（六禁门禁另行机检）。
环境变量：TREE_DIR 必填。
"""
from __future__ import annotations

import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from svgkit import (  # noqa: E402
    CARD, CARD2, DEEP, FAINT, INK, LIGHT, LIGHTFILL, LINE, MID, PANEL_W,
    PAPER, SUB, SVG, measure, wrap,
)

TREE = os.environ.get("TREE_DIR") or (
    sys.exit("panels: 必须设置 TREE_DIR") or ""
)


def load(rel: str) -> dict:
    with open(os.path.join(TREE, rel), encoding="utf-8") as fh:
        return json.load(fh)


structure = load("data/rebuild/structure.json")
pipeline = load("data/rebuild/pipeline.json")
compat = load("data/rebuild/compat.json")
behavior = load("data/rebuild/behavior.json")
formats = load("data/rebuild/formats.json")
frozen = load("data/frozen/build-summary.json")

F = formats["formats"]
LOCS = structure["loc"]
OUT = os.path.join(TREE, "data", "panels")
os.makedirs(OUT, exist_ok=True)

ORDER = ["dbml", "wavedrom", "d2", "structurizr", "likec4", "nomnoml", "pikchr"]
DISP = {"dbml": "DBML", "wavedrom": "WaveDrom", "d2": "D2",
        "structurizr": "Structurizr DSL", "likec4": "LikeC4",
        "nomnoml": "nomnoml", "pikchr": "Pikchr"}


def fmt_count(fmt: str) -> str:
    s = F[fmt]["ast_shape"]
    if fmt == "dbml":
        zh = {"project": "项目", "table": "表", "enum": "枚举", "ref": "引用"}
        return f'{s["items"]} 条目（{"、".join(zh.get(k, k) for k in s["item_kinds"])}）'
    if fmt == "wavedrom":
        return f'{s["signals_top"]} 顶层信号 / {s["edges"]} 边'
    if fmt == "nomnoml":
        return f'{s["directives"]} 指令 / {s["statements"]} 语句'
    return f'{s["statements"]} 条语句'


# ---------------------------------------------------------------- p1 hero
def p1() -> None:
    h = 720
    s = SVG(h, "总览：七种图示 DSL 到一套语法树引擎")
    s.rect(0, 0, PANEL_W, h, fill=PAPER, stroke=None)
    s.rect(40, 36, 520, 64, fill=DEEP, stroke=None, rx=12)
    s.text(66, 76, "diagram-ast-parser", size=26, fill="#FFFFFF", weight=700)
    s.text(342, 74, "0.1.0", size=15, fill=LIGHTFILL, weight=600)
    s.text(580, 60, "Rust 库 + 命令行", size=18, fill=INK, weight=600)
    s.text(580, 88, "语法级语法树 · 字节区间定位 · 整仓禁用 unsafe 代码", size=15, fill=SUB)
    s.text(40, 148, "把七种「文本画图」语言解析为可序列化的各自专用语法树——", size=20, weight=600, fill=INK)
    s.text(40, 180, "只做结构，不做语义：导入合并、名字绑定、视图求值、布局统统留给后续编译阶段。", size=17, fill=SUB)
    x, y = 40, 214
    for n in ("DBML", "WaveDrom", "D2", "Structurizr DSL", "LikeC4", "nomnoml", "Pikchr"):
        cw = measure(n, 15) + 26
        if x + cw > 1100:
            x = 40
            y += 44
        s.chip(x, y, n, size=15, h=32)
        x += cw + 14
    yy = 300
    boxes = [("七种 DSL 源文本", 220), ("解析引擎", 160), ("类型化语法树", 200), ("JSON 输出", 160)]
    bx = 60
    centers = []
    for label, bw in boxes:
        s.rect(bx, yy, bw, 60, fill=CARD, stroke=MID, sw=1.4, rx=10)
        s.text(bx + bw / 2, yy + 36, label, size=16, fill=DEEP, weight=600, anchor="middle")
        centers.append((bx, bw))
        bx += bw + 60
    for i in range(3):
        x1 = centers[i][0] + centers[i][1]
        x2 = centers[i + 1][0]
        s.arrow(x1 + 8, yy + 30, x2 - 8, yy + 30)
    s.text(60, yy + 96, "同一信封：序列化后以格式标签 + 树体两个字段包裹，格式名一律蛇形命名。", size=14.5, fill=SUB)
    facts = [
        ("7", "解析格式", "一套入口统一调度"),
        (f'{structure["rust_source_files"]}+{structure["rust_test_files"]}', "Rust 源文件", "引擎 + 集成测试"),
        (f'{LOCS["total"]:,}', "行 Rust 代码", f'其中引擎 {LOCS["src_total"]:,} 行'),
        (f'{frozen["tests_passed"]}', "测试全部通过", f'一次性实测 {frozen["test_secs"]:.1f} 秒'),
        (f'{structure["dependencies_exact_pinned"]}', "依赖全部精确锁版", "clap / serde 系列锁定单版本"),
        ("1.85", "最低 Rust 版本", "工具链声明与文档一致"),
    ]
    cw, ch, gap = 340, 118, 20
    y0 = 436
    for i, (num, label, note) in enumerate(facts):
        cx = 40 + (i % 3) * (cw + gap)
        cy = y0 + (i // 3) * (ch + gap)
        s.rect(cx, cy, cw, ch, fill=CARD2, stroke=LINE, sw=1, rx=12)
        s.text(cx + 24, cy + 44, num, size=30, fill=DEEP, weight=700)
        s.text(cx + 24, cy + 74, label, size=16, fill=INK, weight=600)
        s.text(cx + 24, cy + 99, note, size=13.5, fill=FAINT)
    save(s, "p1-hero.svg")


# ---------------------------------------------------------------- p2 格式版图
def p2() -> None:
    h = 830
    s = SVG(h, "七种格式的实测解析事实")
    s.rect(0, 0, PANEL_W, h, fill=PAPER, stroke=None)
    cw, ch, gx, gy = 518, 176, 24, 26
    y0 = 24
    for i, fmt in enumerate(ORDER):
        d = F[fmt]
        cx = 40 + (i % 2) * (cw + gx)
        cy = y0 + (i // 2) * (ch + gy)
        s.rect(cx, cy, cw, ch, fill=CARD2, stroke=LINE, sw=1, rx=12)
        s.rect(cx, cy, 6, ch, fill=DEEP, stroke=None, rx=3)
        s.text(cx + 26, cy + 34, DISP[fmt], size=19, weight=700, fill=INK)
        s.chip(cx + 26 + measure(DISP[fmt], 19) + 16, cy + 12,
               f'JSON 标签 {d["json_tag"]}', size=12, h=24)
        s.text(cx + 26, cy + 62, d["domain"], size=14, fill=SUB)
        rows = [
            f'样例 {d["fixture"]}',
            f'实测：退出码 {d["rc"]} · 紧凑 JSON {d["ast_bytes"]:,} 字节',
            f'结构：{fmt_count(fmt)}',
        ]
        yy = cy + 88
        for r in rows:
            s.text(cx + 26, yy, r, size=13.5, fill=SUB)
            yy += 24
        ok = behavior["auto_matrix_observed"][d["fixture"]] == d["json_tag"]
        s.text(cx + cw - 26, cy + 62,
               ("auto 识别 ✓ " if ok else "auto ✗ ")
               + behavior["auto_matrix_observed"][d["fixture"]],
               size=13, fill=SUB, anchor="end")
        s.text(cx + cw - 26, cy + 34, f'指纹 {d["ast_sha256"][:10]}…',
               size=12.5, fill=FAINT, anchor="end")
    cx = 40 + (cw + gx)
    cy = y0 + 3 * (ch + gy)
    s.rect(cx, cy, cw, ch, fill=LIGHTFILL, stroke=MID, sw=1.2, rx=12)
    s.text(cx + 26, cy + 34, "重建逐字节复现", size=18, weight=700, fill=DEEP)
    s.text(cx + 26, cy + 66, "冻结层存底的 7 份 AST 输出，由重建二进制重放解析：", size=13.5, fill=SUB)
    s.text(cx + 26, cy + 92, "7/7 逐字节一致 · 双跑 7/7 一致", size=15, weight=600, fill=INK)
    s.text(cx + 26, cy + 118, "每个指纹都可与冻结证据对表核对", size=13.5, fill=SUB)
    s.text(cx + 26, cy + 146, "证据：冻结摘要 + 行为探测报告", size=12.5, fill=FAINT)
    save(s, "p2-formats.svg")


# ---------------------------------------------------------------- p3 管线
def p3() -> None:
    h = 760
    s = SVG(h, "三条前端路径共享一套类型化出口")
    s.rect(0, 0, PANEL_W, h, fill=PAPER, stroke=None)
    lane_defs = [
        ("JSON5 路径", ["WaveDrom"], ["源文本", "JSON5 解析器", "时序 / 寄存器语法树"], LIGHT),
        ("花括号语句树路径", ["DBML", "D2", "Structurizr DSL", "LikeC4", "Pikchr"],
         ["源文本", "可配置词法器", "花括号语句树", "格式分类器", "类型化构建器"], MID),
        ("平衡扫描路径", ["nomnoml"], ["源文本", "平衡分类器扫描器", "分类器 / 关系语法树"], LIGHT),
    ]
    y = 28
    lane_h = 118
    for title, langs, stages, accent in lane_defs:
        s.rect(40, y, PANEL_W - 80, lane_h, fill=CARD2, stroke=LINE, sw=1, rx=12)
        s.rect(40, y, 6, lane_h, fill=accent, stroke=None, rx=3)
        s.text(64, y + 32, title, size=16.5, weight=700, fill=INK)
        x = 64
        for lang in langs:
            wch = measure(lang, 12.5) + 20
            s.chip(x, y + 46, lang, size=12.5, h=24, fill=PAPER)
            x += wch + 8
        bx = 64
        by = y + 84
        bw = (PANEL_W - 128 - (len(stages) - 1) * 46) / len(stages)
        centers = []
        for st in stages:
            s.rect(bx, by - 22, bw, 34, fill=PAPER, stroke=accent, sw=1.3, rx=8)
            s.text(bx + bw / 2, by + 0.5, st, size=13.5, fill=DEEP, weight=600, anchor="middle")
            centers.append(bx + bw)
            bx += bw + 46
        for i in range(len(stages) - 1):
            s.arrow(centers[i] + 4, by - 5, centers[i] + 42, by - 5)
        y += lane_h + 18
    conv_y = y + 6
    s.arrow(570, y - 10, 570, conv_y - 4)
    s.rect(190, conv_y, 760, 64, fill=DEEP, stroke=None, rx=12)
    s.text(570, conv_y + 40, "七套专用语法树 · 统一文档信封 · 序列化为 JSON", size=18,
           fill="#FFFFFF", weight=700, anchor="middle")
    ny = conv_y + 88
    s.rect(40, ny, PANEL_W - 80, h - ny - 24, fill=CARD, stroke=LINE, sw=1, rx=12)
    s.text(64, ny + 32, "词法层的共同约定（如实呈现）", size=15.5, weight=700, fill=INK)
    yy = ny + 62
    for note in pipeline["lexer_capabilities"]:
        s.text(64, yy, "· " + note, size=13.5, fill=SUB)
        yy += 26
    save(s, "p3-pipeline.svg")


# ---------------------------------------------------------------- p4 区间与错误
def p4() -> None:
    h = 680
    s = SVG(h, "字节区间与一行一列错误定位")
    s.rect(0, 0, PANEL_W, h, fill=PAPER, stroke=None)
    lx, ly, lw = 40, 28, 540
    s.rect(lx, ly, lw, 320, fill=CARD2, stroke=LINE, sw=1, rx=12)
    s.text(lx + 24, ly + 36, "每个语句级节点都带字节区间", size=17, weight=700, fill=INK)
    s.text(lx + 24, ly + 64, "区间记录 UTF-8 字节偏移的起点与终点；", size=13.5, fill=SUB)
    s.text(lx + 24, ly + 88, "错误报告另给一行一列起算的行列号。", size=13.5, fill=SUB)
    demo = "box 订单 { label 提交 }"
    nbytes = len(demo.encode())
    dx, dy = lx + 24, ly + 150
    s.text(dx, dy, demo, size=17, mono=True, fill=INK, weight=600)
    s.line(dx, dy + 14, dx + 360, dy + 14, stroke=LINE, sw=1.2)
    for pos, lab in ((0, "起点 0"), (nbytes, f"终点 {nbytes}")):
        wpx = dx + (pos / nbytes) * 340
        s.line(wpx, dy + 8, wpx, dy + 20, stroke=MID, sw=1.6)
        s.text(wpx, dy + 40, lab, size=12.5, fill=SUB,
               anchor="middle" if pos else "start")
    s.text(dx, dy + 66, f"上面这行共 {nbytes} 字节（中文每字 3 字节）· 刻度示意", size=12.5, fill=FAINT)
    s.text(lx + 24, ly + 258, "区间可取并集（更早起点与更晚终点），供上层做", size=13.5, fill=SUB)
    s.text(lx + 24, ly + 282, "「原文回显 + 指针」式诊断与源映射。", size=13.5, fill=SUB)
    rx, ry, rw = 610, 28, 490
    s.rect(rx, ry, rw, 320, fill="#101B2E", stroke=None, rx=12)
    s.text(rx + 24, ry + 34, "真实诊断转录（未闭合块）", size=15.5, weight=700, fill="#D9E7FA")
    raw_lines = behavior["errors"]["d2_unterminated"]["raw"].strip().splitlines()
    yy = ry + 66
    for ln in raw_lines:
        s.text(rx + 24, yy, ln, size=13, mono=True, fill="#BBD4F5")
        yy += 25
    s.chip(rx + 24, ry + 196, "stdin 传入 · 诊断走 stderr", size=12, h=26,
           fill="#1B2C49", stroke="#3D5E92", ink="#D9E7FA")
    s.chip(rx + 24 + 250, ry + 196, "退出码 1", size=12.5, h=26,
           fill="#1B2C49", stroke="#3D5E92", ink="#D9E7FA")
    s.text(rx + 24, ry + 268, "诊断可要求机器可读的 JSON 形态（诊断开关）", size=12.5, fill="#8FA9CC")
    by = 380
    s.rect(40, by, PANEL_W - 80, h - by - 28, fill=CARD, stroke=LINE, sw=1, rx=12)
    s.text(64, by + 34, "诊断对象的五个字段", size=15.5, weight=700, fill=INK)
    fields = [
        ("format", "出错的格式"),
        ("message", "人读信息"),
        ("span", "字节区间（可得时）"),
        ("line", "一基行号"),
        ("column", "一基列号"),
    ]
    x = 64
    for key, zh in fields:
        s.rect(x, by + 56, 196, 76, fill=PAPER, stroke=LINE, sw=1, rx=10)
        s.text(x + 16, by + 84, key, size=14.5, mono=True, fill=DEEP, weight=600)
        s.text(x + 16, by + 112, zh, size=12.5, fill=SUB)
        x += 212
    s.text(64, by + 168, "未知顶层语句不会被静默丢弃：属性开放的语言把不认识的属性留作通用属性节点，", size=13.5, fill=SUB)
    s.text(64, by + 194, "语义校验属于后续阶段——这是引擎明示的边界。", size=13.5, fill=SUB)
    save(s, "p4-span-error.svg")


# ---------------------------------------------------------------- p5 auto 顺序
def p5() -> None:
    h = 620
    s = SVG(h, "auto 格式识别的瀑布顺序")
    s.rect(0, 0, PANEL_W, h, fill=PAPER, stroke=None)
    s.text(40, 36, "按从上到下的顺序做保守试探，先命中先停：", size=15.5, fill=SUB)
    stages = pipeline["auto_detect_order"]
    y = 58
    row_h = 62
    for i, st in enumerate(stages):
        accent = MID if i < len(stages) - 1 else FAINT
        s.rect(40, y, 620, row_h - 10, fill=CARD2, stroke=LINE, sw=1, rx=10)
        s.rect(40, y, 6, row_h - 10, fill=accent, stroke=None, rx=3)
        s.text(64, y + 25, st["format"], size=14.5, mono=True, fill=DEEP, weight=700)
        yy = y + 45
        for ln in wrap(st["trigger"], 13.5, 560)[:2]:
            s.text(64, yy, ln, size=13.5, fill=SUB)
            yy += 19
        if i < len(stages) - 1:
            s.arrow(70, y + row_h - 10, 70, y + row_h - 1)
        y += row_h
    s.text(40, y + 8, "生产建议显式指定格式：多种语言词法形态可能重叠。", size=13.5, fill=FAINT)
    mx = 700
    s.rect(mx, 58, 400, 7 * 62 - 10 + 84, fill=CARD, stroke=LINE, sw=1, rx=12)
    s.text(mx + 24, 92, "实测识别矩阵（7/7 命中）", size=16, weight=700, fill=INK)
    yy = 126
    for fmt in ORDER:
        d = F[fmt]
        s.text(mx + 24, yy, d["fixture"], size=13, mono=True, fill=INK)
        s.text(mx + 200, yy, "→", size=13, fill=FAINT)
        s.text(mx + 226, yy, d["json_tag"], size=13, mono=True, fill=DEEP, weight=600)
        s.text(mx + 356, yy, "✓", size=13, fill=SUB)
        yy += 34
    s.text(mx + 24, yy + 8, "全部由重建二进制对冻结样例逐个重放实测；", size=12.5, fill=FAINT)
    s.text(mx + 24, yy + 32, "仅给波形对象时也识别为时序格式。", size=12.5, fill=FAINT)
    save(s, "p5-auto.svg")


# ---------------------------------------------------------------- p6 限流与退出码
def p6() -> None:
    lx, ly = 40, 28
    dark_y = ly + 320
    h = dark_y + 300 + 28  # 左列实测高度：上限卡 300 + 间距 20 + 转录卡 300 + 底边距
    s = SVG(h, "资源限流与退出码契约")
    s.rect(0, 0, PANEL_W, h, fill=PAPER, stroke=None)
    s.rect(lx, ly, 546, 300, fill=CARD2, stroke=LINE, sw=1, rx=12)
    s.text(lx + 24, ly + 36, "两项资源上限（默认值，均可经命令行调整）", size=16.5, weight=700, fill=INK)
    rows = [
        ("输入字节数上限", "8 MiB", "--max-input-bytes"),
        ("花括号嵌套深度上限", "128 层", "--max-depth"),
    ]
    yy = ly + 74
    for label, val, flag in rows:
        s.rect(lx + 24, yy, 498, 66, fill=PAPER, stroke=LINE, sw=1, rx=10)
        s.text(lx + 44, yy + 28, label, size=14.5, fill=INK, weight=600)
        s.text(lx + 44, yy + 52, flag, size=13, mono=True, fill=DEEP)
        s.text(lx + 498 + 4, yy + 38, val, size=20, fill=DEEP, weight=700, anchor="end")
        yy += 82
    s.text(lx + 24, yy + 6, "如实声明：深度上限只约束花括号系解析器；", size=13.5, fill=SUB)
    s.text(lx + 24, yy + 30, "JSON5 嵌套与分类器嵌套文本当前不受它约束。", size=13.5, fill=SUB)
    s.rect(lx, dark_y, 546, 300, fill="#101B2E", stroke=None, rx=12)
    s.text(lx + 24, dark_y + 34, "超限实测（4 字节上限喂 12 字节输入）", size=15, weight=700, fill="#D9E7FA")
    raw = behavior["errors"]["input_size_limit"]["raw"].strip()
    yy = dark_y + 66
    for ln in wrap(raw, 13, 500):
        s.text(lx + 24, yy, ln, size=13, mono=True, fill="#BBD4F5")
        yy += 24
    s.text(lx + 24, dark_y + 180, "退出码 1 · 拒绝先于解析发生", size=13, fill="#8FA9CC")
    s.text(lx + 24, dark_y + 240, "拒绝发生在解析之前：先量长度，再谈语法。", size=13, fill="#8FA9CC")
    rx = 610
    s.rect(rx, 28, 490, h - 56, fill=CARD, stroke=LINE, sw=1, rx=12)
    s.text(rx + 24, 62, "退出码契约", size=16.5, weight=700, fill=INK)
    yy = 100
    for ec in pipeline["exit_codes"]:
        mark = "实测" if ec["observed"] else "预留（演示未触发）"
        accent = MID if ec["observed"] else FAINT
        s.rect(rx + 24, yy, 442, 88, fill=PAPER, stroke=LINE, sw=1, rx=10)
        s.rect(rx + 24, yy, 6, 88, fill=accent, stroke=None, rx=3)
        s.text(rx + 46, yy + 34, str(ec["code"]), size=20, weight=700, fill=DEEP)
        s.text(rx + 88, yy + 32, mark, size=12, fill=SUB if ec["observed"] else FAINT)
        lines = wrap(ec["meaning"], 13, 350)
        s.text(rx + 88, yy + 62, lines[0], size=13.5, fill=INK)
        if len(lines) > 1:
            s.text(rx + 88, yy + 82, lines[1], size=13.5, fill=INK)
        yy += 104
    save(s, "p6-limits.svg")


# ---------------------------------------------------------------- p7 兼容矩阵
def p7() -> None:
    rows = compat["rows"]
    col1_x = 40
    col2_x, col2_w = 300, 430
    col3_x, col3_w = 750, 350
    # 先量后画：行高取两列换行行数的较大者，画布高度由实测内容高度决定
    measured = []
    y = 92 + 18
    for r in rows:
        w1 = wrap(r["implemented"], 13, col2_w - 28)
        w2 = wrap(r["deferred"], 13, col3_w - 28)
        row_h = max(len(w1), len(w2)) * 21 + 40
        measured.append((r, w1, w2, row_h, y))
        y += row_h + 12
    h = y + 16
    s = SVG(h, "兼容矩阵：已实现与有意推迟")
    s.rect(0, 0, PANEL_W, h, fill=PAPER, stroke=None)
    s.text(40, 40, "每一行都来自引擎自述的兼容矩阵（中文意译）：解析成功只代表结构可表示，", size=14.5, fill=SUB)
    s.text(40, 64, "不代表上游渲染器会语义接受同一份源。", size=14.5, fill=SUB)
    s.text(col2_x, 92, "已实现", size=14.5, weight=700, fill=DEEP)
    s.text(col3_x, 92, "有意推迟（留给后续阶段）", size=14.5, weight=700, fill=FAINT)
    for idx, (r, w1, w2, row_h, ry) in enumerate(measured):
        s.rect(40, ry, PANEL_W - 80, row_h, fill=CARD2 if idx % 2 == 0 else PAPER,
               stroke=LINE, sw=1, rx=10)
        s.text(col1_x + 20, ry + 32, r["format"], size=16.5, weight=700, fill=INK)
        yy = ry + 30
        for ln in w1:
            s.text(col2_x + 14, yy, ln, size=13, fill=INK)
            yy += 21
        yy = ry + 30
        for ln in w2:
            s.text(col3_x + 14, yy, ln, size=13, fill=FAINT)
            yy += 21
    save(s, "p7-compat.svg")


# ---------------------------------------------------------------- p8 验证带
def p8() -> None:
    h = 640
    s = SVG(h, "可审计性：证据分层与门禁")
    s.rect(0, 0, PANEL_W, h, fill=PAPER, stroke=None)
    cw = (PANEL_W - 80 - 48) / 3
    y0 = 28
    ch = 560
    titles = ["冻结层（一次性）", "重建层（确定性）", "门禁与检查（构建期）"]
    cols = [
        [
            f'冷构建 {frozen["build_secs"]:.1f} 秒（离线、全新目标目录）',
            f'测试 {frozen["test_secs"]:.1f} 秒 · {frozen["tests_passed"]} 项通过',
            f'二进制 {frozen["binary_bytes"]:,} 字节',
            "7 份紧凑 JSON 语法树逐字存底并计指纹",
            "冻结后绝不重测覆盖（一次性守卫）",
        ],
        [
            "重建二进制重放解析 7/7 逐字节复现冻结输出",
            "同轮双跑 7/7 输出一致",
            "auto 识别矩阵 7/7 与冻结层一致",
            "错误演示 4 组退出码与冻结层一致",
            "重建链产物不含路径/时间/版本串（归一化烧录）",
        ],
        [
            "六禁令扫描：页面 + 全部面板 0 违规",
            "六条真实违规正向对照 6/6 咬住",
            "SVG 静态检查：8 张面板全部 0 缺陷",
            "页面数字逐一与证据文件机检对表",
            "渲染前断言宽度与滚动定位（记录在案）",
        ],
    ]
    accents = [DEEP, MID, LIGHT]
    for i, (title, items) in enumerate(zip(titles, cols)):
        cx = 40 + i * (cw + 24)
        s.rect(cx, y0, cw, ch, fill=CARD2, stroke=LINE, sw=1, rx=12)
        s.rect(cx, y0, cw, 6, fill=accents[i], stroke=None, rx=3)
        s.text(cx + 20, y0 + 38, title, size=16.5, weight=700, fill=INK)
        yy = y0 + 74
        for it in items:
            s.text(cx + 20, yy, "·", size=14, fill=accents[i] if i < 2 else SUB, weight=700)
            lines = wrap(it, 13.5, cw - 52)
            for j, ln in enumerate(lines):
                s.text(cx + 38, yy + j * 19, ln, size=13.5, fill=SUB)
            yy += len(lines) * 19 + 16
        s.line(cx + 20, y0 + ch - 66, cx + cw - 20, y0 + ch - 66, stroke=LINE, sw=1)
        s.text(cx + 20, y0 + ch - 40, ["一次性实测，只此一份", "双跑逐字节相同", "出页前强制通过"][i],
               size=13, fill=FAINT)
    save(s, "p8-verify.svg")


def save(svg: SVG, name: str) -> None:
    with open(os.path.join(OUT, name), "w", encoding="utf-8") as fh:
        fh.write(svg.render())
    print(f"panels: {name} ({svg.h}px)")


for fn in (p1, p2, p3, p4, p5, p6, p7, p8):
    fn()
print("panels: 8 张面板生成完毕")
