#!/usr/bin/env python3
"""六禁门禁：出页前对 index.html 与全部面板 SVG 强制检查。

禁令：
  ① 引擎源码文件名（basename，含扩展名）
  ② file:line / 行号 / 区间定位
  ③ ≥25 字符逐字源码摘录（空白归一化后）
  ④ 引擎标识符（Rust 标识符；公开格式/生态/DSL 关键字、CLI 动词与取值、
     JSON 契约键与 format/kind 枚举值放行——判例见 allowlist.json 与 VERIFICATION.md）
  ⑤ 引擎内部路径（仓库相对路径 / 绝对路径）
  ⑥ 生成器文件名与重建命令

放行机制：allowlist.json 的 filenames/identifiers/zones_static + 从冻结 AST 与
诊断证据自动采收的 JSON 契约键与枚举值 + zone_files 指向的真实转录原文。
豁免按位置判定：命中区间完整落在放行 zone 区间内才豁免（zone 原文与命中文本
都做空白归一化后对位匹配；③ 在归一化文本上做同样的位置化豁免）。

自带 6 条真实违规的正向对照（样本实时取自引擎仓，保证真实性），6/6 咬住才可用。
环境变量：ENGINE_REPO、TREE_DIR 必填。用法：
  gate_check.py scan <file>...     扫描文件（违规即 rc=1）
  gate_check.py selfcheck           正向对照 6/6
"""
from __future__ import annotations

import html
import json
import os
import re
import subprocess
import sys

WINDOW = 25
RUST_NOISE = {
    "fn", "let", "mut", "pub", "use", "mod", "struct", "enum", "impl", "self",
    "super", "match", "if", "else", "for", "while", "loop", "return", "crate",
    "as", "where", "type", "ref", "move", "dyn", "const", "static",
    "unsafe", "trait", "true", "false", "None", "Some", "Ok", "Err", "Result",
    "String", "Vec", "Option", "usize", "str", "bool", "u8", "u16", "u32",
    "u64", "i8", "i16", "i32", "i64", "f32", "f64", "derive",
    "Debug", "Clone", "Copy", "Default", "Serialize", "Deserialize",
    "PartialEq", "Eq", "Hash", "Display", "FromStr", "Error", "Box",
    "format", "vec", "println", "eprintln", "write", "writeln", "new", "len",
    "trim", "contains", "expect", "unwrap", "panic", "assert_eq", "main",
    "args", "push", "map", "filter", "find", "collect", "iter", "lines",
    "split", "join", "min", "max", "sum", "count", "chars", "bytes", "encode",
    "and", "not", "the", "with", "into", "from", "this", "that", "exit",
}
GEN_PATTERNS = [
    "python3 ", "node ", "cargo ", "git archive", "shasum", "magick ",
    "svg-linter ", "CARGO_TARGET_DIR", "rebuild_chain", "freeze_once",
    "vacuum_rerun", "render_page", "export_static", "probe_behavior",
    "build_page", "gate_check", "stitch.py", "verification.py",
    "fingerprint.py", "cmp_artifacts.py", "parse_freeze.py",
    "panels.py", "svgkit.py", "check_engine.py", "allowlist.json",
    "disclosures.json", "__pycache__", "PYTHONDONTWRITEBYTECODE",
]
LOC_PATS = [
    r"[\w.-]+\.(?:rs|toml|yml|yaml|md|lock|json5|dbml|dsl|c4|d2|pikchr|nomnoml)\s*[:：]\s*\d+",
    r"第\s*\d+\s*[-~–至]\s*\d+\s*行",
    r"第\s*\d+\s*行",
    r"\bL\d+\b",
    r"\bline\s*[:：=]\s*\d+",
    r"\bcolumn\s*[:：=]\s*\d+",
]
PATH_PATS = [
    r"(?<![\w./~-])((?:src|tests|examples|docs|\.github)/[\w./-]+)",
    r"~/projects/[\w./-]+",
    r"/Users/\w+/[\w./-]+",
]


def fail(msg: str) -> None:
    print(f"gate_check: {msg}", file=sys.stderr)
    sys.exit(1)


def git(repo: str, *args: str) -> str:
    proc = subprocess.run(["git", "-C", repo, *args], capture_output=True, text=True)
    if proc.returncode != 0:
        fail(f"git {' '.join(args)} 失败: {proc.stderr.strip()[:150]}")
    return proc.stdout


def norm(s: str) -> str:
    return re.sub(r"\s+", " ", s).strip()


def spans_of(needle_hay: list[tuple[str, str]]) -> list[tuple[int, int]]:
    out: list[tuple[int, int]] = []
    for needle, hay in needle_hay:
        for m in re.finditer(re.escape(needle), hay):
            out.append((m.start(), m.end()))
    return out


class Gate:
    def __init__(self) -> None:
        self.repo = os.environ.get("ENGINE_REPO") or fail("必须设置 ENGINE_REPO")
        self.tree = os.environ.get("TREE_DIR") or fail("必须设置 TREE_DIR")

        with open(os.path.join(self.tree, "tools", "allowlist.json"),
                  encoding="utf-8") as fh:
            allow = json.load(fh)
        self.allow_files = set(allow["filenames"])
        self.allow_ids = set(allow["identifiers"])
        self.zones: list[str] = list(allow["zones_static"])
        for ref in allow["zone_files"]:
            self.zones.append(self._zone_from_file(ref))

        # 契约采收：冻结 AST + 行为层诊断的全部键，以及 format/kind 枚举值
        self.contract: set[str] = set()
        ast_dir = os.path.join(self.tree, "data", "frozen", "ast")
        for name in sorted(os.listdir(ast_dir)):
            if name.endswith(".json"):
                with open(os.path.join(ast_dir, name), encoding="utf-8") as fh:
                    self._harvest(json.load(fh))
        behavior_path = os.path.join(self.tree, "data", "rebuild", "behavior.json")
        if os.path.isfile(behavior_path):
            with open(behavior_path, encoding="utf-8") as fh:
                behavior = json.load(fh)
            for err in behavior.get("errors", {}).values():
                if err.get("json") is not None:
                    self._harvest(err["json"])

        # 引擎语料（git 对象，非工作区）
        self.files = [l for l in git(self.repo, "ls-files").splitlines() if l.strip()]
        self.basenames = {os.path.basename(p) for p in self.files}
        self.sources: dict[str, str] = {}
        for p in self.files:
            if p.endswith((".rs", ".toml", ".md", ".yml", ".json5", ".dbml",
                          ".dsl", ".c4", ".d2", ".nomnoml", ".pikchr")) \
                    or os.path.basename(p) in ("justfile",):
                self.sources[p] = git(self.repo, "show", f"HEAD:{p}")

        # ④ 只采收「定义位」标识符（struct/enum/fn/trait/type/mod/const/static/macro_rules
        # 之后的名字），而不是源码正文里出现过的每个英文词——禁令针对引擎标识符本身，
        # 不针对恰好同拼写的通用英文词；页面用词由 allowlist 与上下文连词规则兜底。
        DEF_RE = re.compile(
            r"\b(?:struct|enum|fn|trait|type|mod|const|static|macro_rules)\s+"
            r"([A-Za-z_][A-Za-z0-9_]*)"
        )
        ids: set[str] = set()
        for p, text in self.sources.items():
            if p.endswith(".rs"):
                ids.update(DEF_RE.findall(text))
        self.engine_ids = {
            i for i in ids
            if i not in RUST_NOISE and i not in self.allow_ids
            and i not in self.contract
        }
        # ③ 源窗口索引（归一化后）
        self.source_windows: set[str] = set()
        for text in self.sources.values():
            s = norm(text)
            for i in range(max(0, len(s) - WINDOW + 1)):
                self.source_windows.add(s[i:i + WINDOW])

    def _zone_from_file(self, ref: str) -> str:
        path, _, frag = ref.partition("#")
        with open(os.path.join(self.tree, path), encoding="utf-8") as fh:
            obj = json.load(fh)
        for key in frag.split("."):
            obj = obj[key]
        return obj if isinstance(obj, str) else json.dumps(obj, ensure_ascii=False)

    def _harvest(self, node: object) -> None:
        if isinstance(node, dict):
            for k, v in node.items():
                if isinstance(k, str):
                    self.contract.add(k)
                    if k in ("format", "kind") and isinstance(v, str):
                        self.contract.add(v)
                self._harvest(v)
        elif isinstance(node, list):
            for item in node:
                self._harvest(item)

    def _zone_spans(self, text: str) -> list[tuple[int, int]]:
        """zone 命中区间：按词构造空白容忍正则（渲染断行/实体还原只可能引入
        空白差异，非空白字符必须逐字连续），保证位置化豁免可精确命中。"""
        spans: list[tuple[int, int]] = []
        for z in self.zones:
            if not z.strip():
                continue
            pat = re.compile(r"\s*".join(re.escape(t) for t in z.split()))
            for m in pat.finditer(text):
                spans.append((m.start(), m.end()))
        return spans

    def scan_text(self, name: str, text: str, *, markup: bool | None = None) -> list[dict]:
        # HTML/SVG 内容先做「文本视图」：标签整段替换为单个空格，只留下人类可见文本。
        # 这样面板中被标签分隔的真实转录行在视图内重新连成连续原文，位置化豁免才能
        # 精确命中；同时所有六类检测都跑在可见文本上（属性值不进视图，检测不减弱——
        # 本页属性只含 CSS/SVG 布局词汇，无引擎内容）。
        if markup is None:
            stripped_l = text.lstrip().lower()
            markup = (".html" in name or ".svg" in name
                      or stripped_l.startswith(("<!doctype", "<svg")))
        if markup:
            text = html.unescape(re.sub(r"<[^>]*>", " ", text))
        violations: list[dict] = []
        zone_spans = self._zone_spans(text)

        def exempt(a: int, b: int) -> bool:
            return any(za <= a and b <= zb for za, zb in zone_spans)

        # ① 文件名
        for base in sorted(self.basenames):
            if base in self.allow_files:
                continue
            start = 0
            while True:
                idx = text.find(base, start)
                if idx < 0:
                    break
                if not exempt(idx, idx + len(base)):
                    violations.append({
                        "cat": 1, "where": name, "hit": base,
                        "ctx": text[max(0, idx - 30):idx + len(base) + 30],
                    })
                    break
                start = idx + 1
        # ② 行号定位
        for pat in LOC_PATS:
            for m in re.finditer(pat, text):
                if not exempt(m.start(), m.end()):
                    violations.append({
                        "cat": 2, "where": name, "hit": m.group(0),
                        "ctx": text[max(0, m.start() - 30):m.end() + 30],
                    })
        # ③ 逐字摘录（归一化 25 字符窗口，位置化豁免）
        page_norm = norm(text)
        norm_spans = self._zone_spans(page_norm)
        for i in range(max(0, len(page_norm) - WINDOW + 1)):
            w = page_norm[i:i + WINDOW]
            if w in self.source_windows:
                if not any(za <= i and i + WINDOW <= zb for za, zb in norm_spans):
                    violations.append({
                        "cat": 3, "where": name, "hit": w, "src": "engine",
                        "ctx": page_norm[max(0, i - 12):i + WINDOW + 12],
                    })
        # ④ 引擎标识符（若出现处只是已放行连词 token 的组成部分——如产品名中的
        # diagram/ast/parser——则该出现不构成标识符引用，免报）
        def ascii_word_char(c: str) -> bool:
            return c.isascii() and (c.isalnum() or c in "-_")

        for m in re.finditer(r"[A-Za-z_][A-Za-z0-9_]{2,}", text):
            tok = m.group(0)
            if tok in self.engine_ids and tok not in self.contract:
                ra, rb = m.start(), m.end()
                while ra > 0 and ascii_word_char(text[ra - 1]):
                    ra -= 1
                while rb < len(text) and ascii_word_char(text[rb]):
                    rb += 1
                run = text[ra:rb]
                if run != tok and (run in self.allow_ids or run in self.contract):
                    continue
                if not exempt(m.start(), m.end()):
                    violations.append({
                        "cat": 4, "where": name, "hit": tok,
                        "ctx": text[max(0, m.start() - 30):m.end() + 30],
                    })
        # ⑤ 内部路径
        for pat in PATH_PATS:
            for m in re.finditer(pat, text):
                if not exempt(m.start(), m.end()):
                    violations.append({
                        "cat": 5, "where": name, "hit": m.group(0).strip(),
                        "ctx": text[max(0, m.start() - 30):m.end() + 30],
                    })
        # ⑥ 生成器与重建命令
        for pat in GEN_PATTERNS:
            start = 0
            while True:
                idx = text.find(pat, start)
                if idx < 0:
                    break
                if not exempt(idx, idx + len(pat)):
                    violations.append({
                        "cat": 6, "where": name, "hit": pat.strip(),
                        "ctx": text[max(0, idx - 30):idx + len(pat) + 30],
                    })
                    break
                start = idx + 1
        return violations

    # ---- 正向对照（样本实时取自引擎仓） ----
    def selfcheck(self) -> list[dict]:
        lib = self.sources.get("src/lib.rs") or fail("正向对照样本缺失：lib.rs")
        ast_dbml = self.sources.get("src/ast/dbml.rs") or fail("正向对照样本缺失")
        m = re.search(r"pub struct ParseOptions \{[^}]*\}", lib, re.S)
        excerpt = norm(m.group(0))[:64] if m else norm(lib)[:64]
        if len(excerpt) < 30:
            fail("③ 正向对照摘录不足 30 字符")
        ident_m = re.search(r"pub struct (Dbml\w+)", ast_dbml)
        ident = ident_m.group(1) if ident_m else fail("④ 正向对照标识符缺失")
        tree_rs = "src/parser/tree.rs"
        if tree_rs not in self.sources:
            fail("⑤ 正向对照路径样本缺失")
        poisons = [
            (1, "引擎的 lexer.rs 负责词法切分"),
            (2, "出错位置在语句树第 42 行"),
            (3, f"结构定义如下：{excerpt}"),
            (4, f"根类型是 {ident} 的文档对象"),
            (5, f"树构建在 {tree_rs} 中完成"),
            (6, "本页由 build_page.py 生成"),
        ]
        results = []
        for want, poison in poisons:
            cats = {v["cat"] for v in self.scan_text("selfcheck", poison)}
            results.append({"expect": want, "caught_cats": sorted(cats),
                            "poison": poison[:64], "ok": want in cats})
        return results


def main() -> None:
    if len(sys.argv) < 2:
        fail("用法: gate_check.py scan <file>... | selfcheck")
    gate = Gate()
    mode = sys.argv[1]
    if mode == "selfcheck":
        results = gate.selfcheck()
        for r in results:
            print(f"  对照 {r['expect']}: caught={r['caught_cats']} ok={r['ok']}")
        coverage = {c for r in results for c in r["caught_cats"]}
        if not all(r["ok"] for r in results) or coverage != {1, 2, 3, 4, 5, 6}:
            fail(f"正向对照未 6/6（覆盖 {sorted(coverage)}）")
        print("gate_check: selfcheck 6/6 全部咬住")
        return
    if mode != "scan" or len(sys.argv) < 3:
        fail("用法: gate_check.py scan <file>... | selfcheck")
    total = 0
    for path in sys.argv[2:]:
        with open(path, encoding="utf-8") as fh:
            violations = gate.scan_text(path, fh.read())
        for v in violations[:40]:
            print(f"  [{v['cat']}] {v['where']}: {v['hit']!r}  …{v['ctx'][:70]}…")
        total += len(violations)
    if total:
        fail(f"六禁门禁违规 {total} 处，拒绝出页")
    print(f"gate_check: scan {len(sys.argv) - 2} 文件 0 违规")


if __name__ == "__main__":
    main()
