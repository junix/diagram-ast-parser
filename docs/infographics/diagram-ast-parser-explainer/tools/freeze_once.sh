#!/usr/bin/env bash
# 一次性冻结层：对引擎冻结提交做一次真实构建、真实测试与真实 CLI 转录，
# 全部产物先落在 /tmp 暂存，全部步骤成功后才原子移入 data/frozen/。
# data/frozen/manifest.txt 已存在即拒绝执行（绝不重测覆盖）。
#
# 冻结内容：
#   manifest.txt      引擎 HEAD/porcelain/工具链/时间（一次性环境事实）
#   build-run.txt     真实 cargo build --release --offline（冷 target，带耗时）
#   test-run.txt      真实 cargo test --offline（带耗时）
#   build-summary.json  上述日志的结构化摘要（耗时/计数/尺寸/sha）
#   cli-matrix.txt    7 格式真实 CLI 解析转录 + 错误/auto 演示（$ 前缀转录）
#   ast/<fmt>.json    7 份真实紧凑 JSON AST 输出（冻结证据本体）+ 1 份 pretty 样本
set -euo pipefail

fail() { echo "freeze_once: $*" >&2; exit 1; }

: "${ENGINE_REPO:?必须设置 ENGINE_REPO（引擎仓绝对路径，只读）}"
: "${TREE_DIR:?必须设置 TREE_DIR（交付树根目录）}"
: "${WORK_DIR:?必须设置 WORK_DIR（/tmp 脚手架目录）}"
CARGO_BIN="${CARGO_BIN:-cargo}"

TREE_DIR="$(cd "$TREE_DIR" && pwd)"
FROZEN="$TREE_DIR/data/frozen"
[[ -f "$FROZEN/manifest.txt" ]] && fail "一次性冻结证据已存在（$FROZEN/manifest.txt），绝不重测覆盖。"
command -v "$CARGO_BIN" >/dev/null 2>&1 || fail "找不到 cargo: $CARGO_BIN"

mkdir -p "$WORK_DIR/freeze"
STAGE="$WORK_DIR/freeze/staging"
rm -rf "$STAGE"
mkdir -p "$STAGE/ast"

# 引擎门禁 + 从冻结提交导出平面拷贝（引擎仓对树外只读）
export TREE_DIR
PYDIR="$(cd "$(dirname "$0")" && pwd)"
( cd "$PYDIR/.." && PYTHONDONTWRITEBYTECODE=1 python3 tools/check_engine.py ) \
  || fail "引擎门禁未通过（HEAD/porcelain 漂移）"

SRC="$WORK_DIR/freeze/src"
rm -rf "$SRC"
mkdir -p "$SRC"
git -C "$ENGINE_REPO" archive HEAD | tar -x -C "$SRC"

HEAD="$(git -C "$ENGINE_REPO" rev-parse HEAD)"
DIRTY="$(git -C "$ENGINE_REPO" status --porcelain | grep -c . || true)"

# ---------- 一次性真实构建（冷：全新 target 目录；--offline） ----------
TARGET="$WORK_DIR/freeze/target"
rm -rf "$TARGET"
export CARGO_TARGET_DIR="$TARGET"
BUILD_LOG="$STAGE/build-run.txt"
if ! ( cd "$SRC" && /usr/bin/time -p "$CARGO_BIN" build --release --offline ) \
     > "$BUILD_LOG" 2>&1; then
  mv "$BUILD_LOG" "$STAGE/build-run.failed.txt"
  tail -20 "$STAGE/build-run.failed.txt" || true
  fail "cargo build --release --offline 失败（失败留档 build-run.failed.txt，可排查后重试冻结）"
fi
BIN="$TARGET/release/diagram-parse"
[[ -x "$BIN" ]] || fail "构建产物缺失: $BIN"
BIN_BYTES="$(stat -f %z "$BIN")"
{
  echo
  echo "# 构建产物（release，字节）"
  echo "diagram-parse $BIN_BYTES"
} >> "$BUILD_LOG"

# ---------- 一次性真实测试 ----------
if ! ( cd "$SRC" && /usr/bin/time -p "$CARGO_BIN" test --offline ) \
     > "$STAGE/test-run.txt" 2>&1; then
  mv "$STAGE/test-run.txt" "$STAGE/test-run.failed.txt"
  tail -20 "$STAGE/test-run.failed.txt" || true
  fail "cargo test 失败（失败留档 test-run.failed.txt，可排查后重试冻结）"
fi

# ---------- 一次性真实 CLI 转录（7 格式 + 演示） ----------
CLI="$STAGE/cli-matrix.txt"
FORMATS="dbml:schema.dbml wavedrom:timing.json5 d2:architecture.d2 structurizr:workspace.dsl likec4:model.c4 nomnoml:classes.nomnoml pikchr:flow.pikchr"

{
  echo "# 一次性真实 CLI 转录（引擎冻结提交；命令以 \$ 前缀记录，输出落 ast/ 与本文件）"
  echo "engine_head=$HEAD"
  echo
} > "$CLI"

for pair in $FORMATS; do
  f="${pair%%:*}"; fx="${pair##*:}"
  set +e
  ( cd "$SRC" && "$BIN" --format "$f" --compact "examples/$fx" ) \
    > "$STAGE/ast/$f.json" 2>/dev/null
  rc=$?
  set -e
  [[ "$rc" == 0 ]] || fail "格式 $f 解析 rc=${rc}（期望 0），冻结中止"
  bytes=$(wc -c < "$STAGE/ast/$f.json" | tr -d ' ')
  sha=$(shasum -a 256 "$STAGE/ast/$f.json" | cut -d' ' -f1)
  echo "\$ diagram-parse --format $f --compact $fx" >> "$CLI"
  echo "  rc=$rc  bytes=$bytes  sha256=$sha  -> ast/$f.json" >> "$CLI"
done

set +e
( cd "$SRC" && "$BIN" --format dbml "examples/schema.dbml" ) \
  > "$STAGE/ast/dbml.pretty.json" 2>/dev/null
rc=$?
set -e
[[ "$rc" == 0 ]] || fail "dbml pretty 解析失败"
{
  echo "\$ diagram-parse --format dbml schema.dbml   -> rc=${rc}（pretty 形态样本 -> ast/dbml.pretty.json）"
  echo
  echo "# 错误与行为演示（stdin 输入；诊断 JSON 走 stderr，此处合并转录）"
} >> "$CLI"

# 错误演示：未闭合块（D2）
printf 'a: {\n  b: c\n' > "$WORK_DIR/freeze/err-d2.txt"
set +e
"$BIN" --format d2 --diagnostic-json - < "$WORK_DIR/freeze/err-d2.txt" \
  > "$STAGE/err-d2.json" 2>&1
rc=$?
set -e
echo "rc=$rc" > "$STAGE/err-d2.rc"
echo "\$ printf 'a: {\\n  b: c\\n' | diagram-parse --format d2 --diagnostic-json -   -> rc=$rc" >> "$CLI"
cat "$STAGE/err-d2.json" >> "$CLI"

# 错误演示：WaveDrom 负数寄存器位宽
printf "{ reg: [{ bits: -1, name: 'reserved' }] }" > "$WORK_DIR/freeze/err-wavedrom.txt"
set +e
"$BIN" --format wavedrom --diagnostic-json - < "$WORK_DIR/freeze/err-wavedrom.txt" \
  > "$STAGE/err-wavedrom.json" 2>&1
rc=$?
set -e
echo "rc=$rc" > "$STAGE/err-wavedrom.rc"
echo "\$ printf \"{ reg: [{ bits: -1, name: 'reserved' }] }\" | diagram-parse --format wavedrom --diagnostic-json -   -> rc=$rc" >> "$CLI"
cat "$STAGE/err-wavedrom.json" >> "$CLI"

# 错误演示：输入超限（--max-input-bytes）
set +e
"$BIN" --format d2 --max-input-bytes 4 - < "$WORK_DIR/freeze/err-d2.txt" \
  > "$STAGE/err-size.json" 2>&1
rc=$?
set -e
echo "rc=$rc" > "$STAGE/err-size.rc"
echo "\$ printf 'a: {\\n  b: c\\n' | diagram-parse --format d2 --max-input-bytes 4 -   -> rc=$rc" >> "$CLI"
cat "$STAGE/err-size.json" >> "$CLI"

# 错误演示：非法 format 值（用法错误）
set +e
"$BIN" --format nope - < "$WORK_DIR/freeze/err-d2.txt" > "$STAGE/err-badfmt.txt" 2>&1
rc=$?
set -e
echo "rc=$rc" > "$STAGE/err-badfmt.rc"
echo "\$ printf 'a: {\\n  b: c\\n' | diagram-parse --format nope -   -> rc=$rc" >> "$CLI"
cat "$STAGE/err-badfmt.txt" >> "$CLI"

# stdin 正常模式 + auto 识别矩阵
printf '{ signal: [{ name: "clk", wave: "p..." }] }' > "$WORK_DIR/freeze/auto-wave.txt"
set +e
"$BIN" --format auto --compact - < "$WORK_DIR/freeze/auto-wave.txt" > "$STAGE/auto-wave.json" 2>/dev/null
rc=$?
set -e
echo "\$ printf '{ signal: [{ name: \"clk\", wave: \"p...\" }] }' | diagram-parse --format auto --compact -   -> rc=${rc}（识别为 wave_drom）" >> "$CLI"
echo >> "$CLI"
echo "# auto 识别矩阵（逐 fixture 真实运行，读取输出中的 format 标签）" >> "$CLI"
for pair in $FORMATS; do
  fx="${pair##*:}"
  set +e
  tag=$( ( cd "$SRC" && "$BIN" --format auto --compact "examples/$fx" 2>/dev/null) \
    | python3 -c 'import json,sys; print(json.load(sys.stdin)["format"])')
  rc=$?
  set -e
  [[ -n "$tag" ]] || fail "auto 识别 $fx 失败"
  echo "auto($fx) -> $tag" >> "$CLI"
done

# ---------- 结构化摘要（python 解析一次性日志） ----------
mkdir -p "$WORK_DIR/py"
cp "$PYDIR/parse_freeze.py" "$WORK_DIR/py/"
( cd "$WORK_DIR/py" && PYTHONDONTWRITEBYTECODE=1 \
    FROZEN_STAGING="$STAGE" TREE_DIR="$TREE_DIR" python3 parse_freeze.py )

# ---------- manifest 最后写入并原子移入冻结层 ----------
{
  echo "# 一次性冻结证据（绝不重测覆盖）"
  echo "# 产生方式: git archive HEAD -> /tmp 平面拷贝 -> 真实 cargo build/test + 真实 CLI 转录"
  echo "engine_head=$HEAD"
  echo "engine_porcelain_dirty_entries=$DIRTY"
  echo "archive_files=$(git -C "$ENGINE_REPO" archive HEAD | tar -t - | wc -l | tr -d ' ')"
  echo "frozen_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "cargo=$("$CARGO_BIN" --version | head -1)"
  echo "rustc=$(rustc --version | head -1)"
  echo "os=$(uname -s) $(uname -m) / $(sw_vers -productVersion 2>/dev/null || true)"
  echo "build_mode=--release --offline（冷 target，一次性计时）"
} > "$STAGE/manifest.txt"

mkdir -p "$FROZEN"
rm -f "$STAGE/build-run.failed.txt" "$STAGE/test-run.failed.txt"
mv "$STAGE"/* "$FROZEN/"
rmdir "$STAGE" 2>/dev/null || true
echo "freeze_once: 完成 -> $FROZEN"
