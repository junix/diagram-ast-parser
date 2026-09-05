#!/usr/bin/env python3
"""SVG 生成底座：文本宽度估算、换行、卡片/标签/箭头等绘图助手与配色常量。

宽度估算按 PingFang SC + Helvetica Neue 度量近似（CJK 全宽=字号；ASCII 按字符类别），
换行取 1.08 安全系数；不追求亚像素精确，只为不溢出、不截断。
"""
from __future__ import annotations

import html

# ---- 配色（蓝色主系：系列色 3 档蓝；文本一律用文本色，不用系列色） ----
INK = "#182841"        # 主文本
SUB = "#44556E"        # 次文本
FAINT = "#75879D"      # 弱文本
LINE = "#D5DFEB"       # 分隔线/描边
PAPER = "#FFFFFF"
CARD = "#F2F7FD"       # 浅蓝底卡片
CARD2 = "#F7FAFE"
DEEP = "#14477E"       # 系列 1：深蓝
MID = "#2C6BC4"        # 系列 2：中蓝
LIGHT = "#8FB8EA"      # 系列 3：浅蓝
LIGHTFILL = "#E2EEFA"  # 浅蓝填充（系列 3 的淡背景，不算新系列）
OKINK = SUB            # “通过/一致”用文本色+符号表达，不引入绿色

FONT = "'PingFang SC','Hiragino Sans GB','Helvetica Neue','Arial',sans-serif"
MONO = "'SF Mono','Menlo','Consolas',monospace"

PANEL_W = 1140

_WIDE = set("０１２３３４５６７８９ＡＢＣＤＥＦＧＨＩＪＫＬＭＮＯＰＱＲＳＴＵＶＷＸＹＺａｂｃｄｅｆｇｈｉｊｋｌｍｎｏｐｑｒｓｔｕｖｗｘｙｚ（）【】《》「」『』——…·、。：；？！，")


def char_w(ch: str, size: float) -> float:
    o = ord(ch)
    if (
        0x2E80 <= o <= 0x9FFF
        or 0xF900 <= o <= 0xFAFF
        or 0xFF00 <= o <= 0xFF60
        or 0x3000 <= o <= 0x303F
        or ch in "—…·"
    ):
        return size
    if ch == " ":
        return size * 0.28
    if ch.isupper():
        return size * 0.68
    if ch.isdigit():
        return size * 0.56
    if ch.islower():
        return size * 0.53
    return size * 0.34


def measure(text: str, size: float) -> float:
    return sum(char_w(c, size) for c in text)


def wrap(text: str, size: float, max_w: float) -> list[str]:
    """按估算宽度换行（保守 1.08 安全系数）；不拆英文单词。"""
    limit = max_w / 1.08
    lines: list[str] = []
    for para in text.split("\n"):
        cur = ""
        for token in _tokens(para):
            trial = cur + token
            if cur and measure(trial.strip(), size) > limit:
                lines.append(cur.strip())
                cur = token.lstrip()
            else:
                cur = trial
        lines.append(cur.strip())
    return [l for l in lines] or [""]


_CJK_PUNCT = set("，。：；？！、）》」』】〕〉…—·％")
_ASCII_GLUE = set("_-./@")


def _tokens(text: str) -> list[str]:
    """切分断行单元：拉丁词整体（自带尾随空格），CJK 逐字（全宽标点黏前字），
    ASCII 连接符黏住两侧词不拆。空格只在词边界被丢弃，渲染出的多行文本按空格
    重连即可逐字复原原文——门禁的位置化豁免依赖这一点。"""
    toks: list[str] = []
    buf = ""
    mode = None  # None | cjk | latin
    for ch in text:
        if ch == " ":
            if buf:
                toks.append(buf + " ")
                buf = ""
                mode = None
            else:
                toks.append(" ")
            continue
        is_cjk = char_w(ch, 10) == 10
        if is_cjk:
            if ch in _CJK_PUNCT and buf and mode == "cjk":
                buf += ch  # 全宽标点跟随前字，不另起断行单元
            else:
                if buf:
                    toks.append(buf)
                buf = ch
                mode = "cjk"
            continue
        m = "latin" if (ch.isalnum() or ch in _ASCII_GLUE) else None
        if mode is None or m == mode:
            buf += ch
            mode = m or mode
        else:
            if buf:
                toks.append(buf)
            buf = ch
            mode = m
    if buf:
        toks.append(buf)
    return toks


def esc(s: str) -> str:
    return html.escape(s, quote=True)


class SVG:
    def __init__(self, height: int, title: str):
        self.h = height
        self.title = title
        self.parts: list[str] = []

    def add(self, s: str) -> None:
        self.parts.append(s)

    # ---- 基元 ----
    def rect(self, x: float, y: float, w: float, h: float, *, fill: str = PAPER,
             stroke: str | None = LINE, sw: float = 1, rx: float = 10,
             dash: str | None = None, opacity: float | None = None) -> None:
        extra = f' stroke-dasharray="{dash}"' if dash else ""
        if opacity is not None:
            extra += f' opacity="{opacity}"'
        stroke_attr = f' stroke="{stroke}" stroke-width="{sw}"' if stroke else ""
        self.add(
            f'<rect x="{x:.1f}" y="{y:.1f}" width="{w:.1f}" height="{h:.1f}" '
            f'rx="{rx}" fill="{fill}"{stroke_attr}{extra}/>'
        )

    def line(self, x1: float, y1: float, x2: float, y2: float, *,
             stroke: str = LINE, sw: float = 1, dash: str | None = None) -> None:
        extra = f' stroke-dasharray="{dash}"' if dash else ""
        self.add(
            f'<line x1="{x1:.1f}" y1="{y1:.1f}" x2="{x2:.1f}" y2="{y2:.1f}" '
            f'stroke="{stroke}" stroke-width="{sw}"{extra}/>'
        )

    def text(self, x: float, y: float, s: str, *, size: float = 14, fill: str = INK,
             weight: int = 400, anchor: str = "start", mono: bool = False,
             spacing: float | None = None) -> float:
        fam = MONO if mono else FONT
        ls = f' letter-spacing="{spacing}"' if spacing else ""
        self.add(
            f'<text x="{x:.1f}" y="{y:.1f}" font-family="{fam}" font-size="{size}" '
            f'fill="{fill}" font-weight="{weight}" text-anchor="{anchor}"{ls}>'
            f"{esc(s)}</text>"
        )
        return measure(s, size)

    def text_block(self, x: float, y: float, s: str, *, size: float = 14,
                   fill: str = INK, weight: int = 400, max_w: float = 400,
                   lh: float = 1.55, mono: bool = False) -> float:
        """多行文本块；返回占用高度。"""
        lines = s.split("\n") if mono else wrap(s, size, max_w)
        for i, ln in enumerate(lines):
            self.text(x, y + i * size * lh, ln, size=size, fill=fill,
                      weight=weight, mono=mono)
        return len(lines) * size * lh

    def chip(self, x: float, y: float, s: str, *, size: float = 13,
             fill: str = LIGHTFILL, stroke: str = MID, ink: str = DEEP,
             weight: int = 600, h: float = 26, pad: float = 11) -> float:
        w = measure(s, size) + pad * 2
        self.rect(x, y, w, h, fill=fill, stroke=stroke, sw=1, rx=h / 2)
        self.text(x + w / 2, y + h / 2 + size * 0.36, s, size=size, fill=ink,
                  weight=weight, anchor="middle")
        return w

    def arrow(self, x1: float, y1: float, x2: float, y2: float, *,
              stroke: str = MID, sw: float = 1.6) -> None:
        import math

        ang = math.atan2(y2 - y1, x2 - x1)
        L, spread = 7.5, math.radians(26)
        a1 = (x2 - L * math.cos(ang - spread), y2 - L * math.sin(ang - spread))
        a2 = (x2 - L * math.cos(ang + spread), y2 - L * math.sin(ang + spread))
        self.add(
            f'<path d="M {x1:.1f} {y1:.1f} L {x2:.1f} {y2:.1f}" stroke="{stroke}" '
            f'stroke-width="{sw}" fill="none"/>'
        )
        self.add(
            f'<path d="M {x2:.1f} {y2:.1f} L {a1[0]:.1f} {a1[1]:.1f} '
            f'L {a2[0]:.1f} {a2[1]:.1f} Z" fill="{stroke}"/>'
        )

    def render(self) -> str:
        head = (
            f'<svg xmlns="http://www.w3.org/2000/svg" width="{PANEL_W}" '
            f'height="{self.h}" viewBox="0 0 {PANEL_W} {self.h}" '
            f'role="img" aria-label="{esc(self.title)}">'
        )
        body = "\n".join(self.parts)
        return f"{head}\n{body}\n</svg>\n"
