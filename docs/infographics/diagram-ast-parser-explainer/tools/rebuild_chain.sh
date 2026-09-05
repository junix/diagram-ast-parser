#!/usr/bin/env bash
# 确定性重建链：引擎门禁 -> 平面 python 拷贝 -> 静态导出 -> 行为探测（含重建二进制
# 与冻结 AST 逐字节比对）-> 面板 -> 页面（声明对表 + 六禁门禁 + SVG 检查）
# -> CDP 渲染 -> 拼接 -> 指纹登记 -> 验证文档。
# 引擎仓对树外只读；构建/探测在 WORK_DIR 的 git archive 拷贝中进行。
set -euo pipefail

fail() { echo "rebuild_chain: $*" >&2; exit 1; }

: "${ENGINE_REPO:?必须设置 ENGINE_REPO}"
: "${TREE_DIR:?必须设置 TREE_DIR}"
: "${WORK_DIR:?必须设置 WORK_DIR}"
CARGO_BIN="${CARGO_BIN:-cargo}"
SVG_LINTER_BIN="${SVG_LINTER_BIN:-svg-linter}"
CHROME_BIN="${CHROME_BIN:-$HOME/Library/Caches/ms-playwright/chromium_headless_shell-1234/chrome-headless-shell-mac-arm64/chrome-headless-shell}"
MAGICK_BIN="${MAGICK_BIN:-/opt/homebrew/bin/magick}"
RENDER_DPR="${RENDER_DPR:-2}"
export CARGO_BIN SVG_LINTER_BIN CHROME_BIN MAGICK_BIN RENDER_DPR ENGINE_REPO TREE_DIR WORK_DIR
export PYTHONDONTWRITEBYTECODE=1

[[ -d "$TREE_DIR/tools" ]] || fail "TREE_DIR 缺 tools/：$TREE_DIR"
[[ -d "$TREE_DIR/data/frozen" ]] || fail "TREE_DIR 缺 data/frozen/（一次性冻结层必须先行）"

# 引擎门禁（HEAD + porcelain）
( cd "$TREE_DIR" && python3 tools/check_engine.py ) || fail "引擎门禁未通过"

# 平面 python 拷贝（树内不留 .pyc，脚本不自定位树路径，全靠环境变量）
PY="$WORK_DIR/py"
rm -rf "$PY"
mkdir -p "$PY"
cp "$TREE_DIR"/tools/*.py "$TREE_DIR"/tools/*.json "$PY"/
cp "$TREE_DIR"/tools/*.mjs "$PY"/ 2>/dev/null || true
cd "$PY"

python3 export_static.py
python3 probe_behavior.py
python3 panels.py
python3 build_page.py
node render_page.mjs
python3 stitch.py
python3 fingerprint.py write
python3 verification.py
echo "rebuild_chain: 完成 -> $TREE_DIR"
