#!/usr/bin/env bash
# 真空复跑：删除全部可重建产物 -> 双拷贝 A/B 全链重建 -> 文本逐字节 cmp（位图逐字节，
# 退路像素零差+PNG 辅助块归一化）-> 真树重建并与 A 比对 -> 一次性报告冻结到
# data/frozen/vacuum-report.txt。data/frozen/ 不参与删除（真空报告自身除外的既有文件不动）。
# 注意：页面不引用任何「冻结层文件数」，故真空报告落盘后页面无需变化，
# 真树重跑链可与 A 逐字节一致（cc-hooks-rs 树的历史教训已在本树规避）。
set -euo pipefail

fail() { echo "vacuum_rerun: $*" >&2; exit 1; }

: "${ENGINE_REPO:?必须设置 ENGINE_REPO}"
: "${TREE_DIR:?必须设置 TREE_DIR}"
: "${WORK_DIR:?必须设置 WORK_DIR}"
KEEP_VACUUM="${KEEP_VACUUM:-0}"
REPORT="$TREE_DIR/data/frozen/vacuum-report.txt"
[[ -f "$REPORT" ]] && fail "真空报告已存在（一次性冻结，绝不重跑覆盖）"
CHAIN="$(cd "$(dirname "$0")" && pwd)/rebuild_chain.sh"

VAC="$WORK_DIR/vacuum"
rm -rf "$VAC"
mkdir -p "$VAC"

# ---------- 1. 删除全部可重建产物（保留 tools/ README data/frozen/ 与目录骨架） ----------
cd "$TREE_DIR"
rm -f index.html VERIFICATION.md data/fingerprints.json
rm -rf data/rebuild data/panels render/sections render/slices
mkdir -p data/rebuild data/panels render/sections
find render -maxdepth 1 -type f \( -name '*.png' -o -name '*.json' \) -delete 2>/dev/null || true

# ---------- 2. 双拷贝 A/B 全链重建 ----------
for SIDE in A B; do
  T="$VAC/$SIDE/tree"; W="$VAC/$SIDE/work"
  mkdir -p "$T/data/frozen" "$T/tools" "$T/render"
  cp -R "$TREE_DIR/tools/." "$T/tools/"
  cp -R "$TREE_DIR/data/frozen/." "$T/data/frozen/"
  cp "$TREE_DIR/README.md" "$T/README.md" 2>/dev/null || true
  ENGINE_REPO="$ENGINE_REPO" TREE_DIR="$T" WORK_DIR="$W" bash "$CHAIN"
done

# ---------- 3. A/B 比对 ----------
CMP="$VAC/cmp"
mkdir -p "$CMP"
cp "$TREE_DIR"/tools/cmp_artifacts.py "$CMP/"
( cd "$CMP" && PYTHONDONTWRITEBYTECODE=1 TREE_DIR="$VAC/A/tree" \
    python3 cmp_artifacts.py "$VAC/A/tree" "$VAC/B/tree" ) | tee "$VAC/ab-cmp.txt"

# ---------- 4. 真树重建并与 A 比对（证明真树产物即确定性产物） ----------
ENGINE_REPO="$ENGINE_REPO" TREE_DIR="$TREE_DIR" WORK_DIR="$VAC/real" bash "$CHAIN"
( cd "$CMP" && PYTHONDONTWRITEBYTECODE=1 TREE_DIR="$TREE_DIR" \
    python3 cmp_artifacts.py "$VAC/A/tree" "$TREE_DIR" ) | tee "$VAC/real-cmp.txt"

# ---------- 5. 一次性报告 ----------
{
  echo "# 真空复跑一次性报告（绝不重跑覆盖）"
  echo "engine_head=$(git -C "$ENGINE_REPO" rev-parse HEAD)"
  echo "criteria: 删除可重建产物 -> A/B 双拷贝全链重建 -> 文本逐字节 cmp；位图逐字节优先，"
  echo "          退路像素零差 + PNG 辅助块归一化；data/frozen/ 不参与删除"
  echo "---- A/B 比对 ----"
  cat "$VAC/ab-cmp.txt"
  echo "---- 真树 vs A 比对 ----"
  cat "$VAC/real-cmp.txt"
  echo "---- 指纹终检 ----"
  echo "真空报告自身须先被登记：真空后执行 fingerprint.py write -> verification.py -> check（见 README）"
} > "$REPORT"

if [[ "$KEEP_VACUUM" != "1" ]]; then
  rm -rf "$VAC"
fi
echo "vacuum_rerun: 完成 -> $REPORT"
