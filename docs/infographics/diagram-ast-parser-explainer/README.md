# diagram-ast-parser 技术长图交付树

对引擎仓 `diagram-ast-parser`（Rust 库 + CLI：把 DBML / WaveDrom / D2 / Structurizr DSL /
LikeC4 / nomnoml / Pikchr 七种文本图示语言解析为可序列化语法树）的可审计技术长图：
`index.html`（1200 CSS px · 中文 · 蓝色主系 · 零 JS · 零 CDN · 零外部请求 · 单文件自包含），
每个数字锚定 `data/` 下的冻结/重建证据，验证层见 `VERIFICATION.md`。

## 目录

| 路径 | 性质 | 说明 |
|---|---|---|
| `index.html` | 可重建 | 成品长图（内嵌全部 SVG 面板） |
| `data/frozen/` | **一次性冻结** | 真实构建/测试/CLI 转录的一次性记录（耗时/计数/尺寸/AST 原文与哈希），绝不重测覆盖 |
| `data/rebuild/` | 可重建（确定性） | 静态普查、行为探测（重建二进制重放 7 格式并逐字节比对冻结 AST）、管线/兼容数据、门禁与检查报告、声明对表 |
| `data/panels/` | 可重建 | 8 张面板的独立 SVG（与页面内嵌字节同源） |
| `render/` | 可重建 | full@2x.png（宽 2400 == 1200×2、高 == 页高×2 双硬断言）、灰度版、缩略图、分节裁剪图、layout.json |
| `tools/` | 源 | 全部生成脚本（冻结→导出→探测→面板→门禁→渲染→拼接→指纹→验证→真空） |
| `VERIFICATION.md` | 可重建 | 声明锚点注册表 + 门禁记录 + 渲染断言 + 指纹表 + 偏差披露（生成自 data/ 与 disclosures） |
| `data/fingerprints.json` | 可重建 | 产物指纹登记表（排除自身与 VERIFICATION.md，防自指） |

## 环境变量表（陌生 shell 凭此表可复跑）

### 必设（缺省即硬失败）

| 变量 | 含义 |
|---|---|
| `ENGINE_REPO` | 引擎仓绝对路径（**只读**；仅 `git archive`/`ls-files`/`rev-parse`/`show`/`status`） |
| `TREE_DIR` | 本交付树绝对路径（产物输出根；真空复跑时可以是 /tmp 拷贝） |
| `WORK_DIR` | /tmp 脚手架根：平面 python 拷贝、git archive 构建/探测、Chrome profile、真空 A/B |

### 可探测（有默认值）

| 变量 | 默认 | 说明 |
|---|---|---|
| `CARGO_BIN` | `cargo` | 重建二进制用（Homebrew cargo 即可，忽略仓库 stable 通道声明；AST 哈希不随编译器版本漂移） |
| `SVG_LINTER_BIN` | `svg-linter` | 面板静态检查（须在 PATH 或给绝对路径） |
| `CHROME_BIN` | playwright 缓存内 `chromium_headless_shell-1234/.../chrome-headless-shell` | CDP 截图用固定版 headless shell（不用随系统漂移的本机 Chrome） |
| `MAGICK_BIN` | `/opt/homebrew/bin/magick` | PNG 剥日期/时间等辅助块（舰队配方 `-strip`） |
| `RENDER_DPR` | `2` | 截图设备像素比（full@2x 高 == 页高 × dpr 硬断言） |
| `KEEP_VACUUM` | `0` | `1` 时保留真空 A/B 拷贝便于排查 |
| `PYTHONDONTWRITEBYTECODE` | 链内自行置 `1` | 交付树内不落 `.pyc`；外层 shell 也建议导出 |

### 复现需用哪个工作目录（重要）

- **python/node 一律在 `$WORK_DIR/py` 的平面拷贝中运行**（`rebuild_chain.sh` 自动做）；
  不要在交付树内直接执行任何 `.py`——树内不留 `.pyc`，脚本也不自定位树路径，全靠环境变量。
- **构建/探测一律在 `$WORK_DIR` 下 `git archive HEAD` 的拷贝中进行**，引擎仓对树外只读、
  不落任何产物（CARGO_TARGET_DIR 一律指向 WORK_DIR 下）。重建层产物不含任何绝对路径、
  时间戳或版本串（Rust 构建路径烧录约定：如未来需要嵌入路径/时间，必须以 `<WORK>`、
  `<build-dir>`、`<ts>` 占位符归一化；当前无任何字段需要，见披露 D01）。
- `TREE_DIR` 本身可以是真树或 /tmp 拷贝（真空复跑即这么做），产物不嵌入树路径。
- `freeze_once.sh` 与 `vacuum_rerun.sh` 有一次性守卫：目标证据已存在即拒绝执行。
- 每次链入口先过引擎门禁：HEAD 必须等于冻结提交，porcelain 只允许本交付树
  （与外部治理 README），否则硬失败。

## 复跑步骤

```bash
export ENGINE_REPO=/Users/junix/projects/plot/diagram-ast-parser   # 引擎仓（只读）
export TREE_DIR=$ENGINE_REPO/docs/infographics/diagram-ast-parser-explainer
export WORK_DIR=$(mktemp -d /tmp/ign-dap.XXXXXX)                   # 用完即删
export PYTHONDONTWRITEBYTECODE=1

# 1) 一次性冻结（仅 data/frozen/manifest.txt 缺失时；随树分发后禁止重跑）
bash "$TREE_DIR/tools/freeze_once.sh"

# 2) 确定性重建链：导出→探测（重建+逐字节比对）→面板→页面(对表+六禁门禁+SVG 检查)
#    →渲染→拼接→指纹→验证
bash "$TREE_DIR/tools/rebuild_chain.sh"

# 3) 真空复跑（一次性；判据见下）
bash "$TREE_DIR/tools/vacuum_rerun.sh"

# 4) 真空后收尾：把一次性真空报告收进登记表，再生成验证文档并终检
( cd "$WORK_DIR/py" && python3 fingerprint.py write \
  && python3 verification.py \
  && python3 fingerprint.py check )   # 以登记表为准，无未登记产物
```

依赖：python3（含 PIL）、node（≥21，全局 WebSocket）、cargo + 离线注册表缓存
（Cargo.lock 全量锁版，`--offline` 构建）、svg-linter、固定版 chrome-headless-shell、magick。

## 真空复跑判据（预声明）

1. 删除全部可重建产物（`index.html`、`data/rebuild/`、`data/panels/`、`render/*`、
   `data/fingerprints.json`、`VERIFICATION.md`）；**`data/frozen/` 不参与删除**。
2. 双拷贝 A/B（tools + frozen + README）各自全链重建。
3. A vs B：文本类逐字节 cmp；PNG 首选逐字节，不一致退路为像素零差 + PNG 辅助块
   （tIME/tEXt 等）归一化后一致。
4. 真树再重建一次并与 A 比对（证明真树产物即确定性产物）。
5. 结果一次性冻结到 `data/frozen/vacuum-report.txt`（有守卫，绝不重跑覆盖）。
6. 页面不引用任何「冻结层文件数」之类的流程敏感计数，真空报告落盘不影响页面字节。

## 六禁门禁（build 期强制）

① 引擎源码文件名 ② file:line/行号/区间 ③ 逐字源码摘录（归一化 ≥25 字符）
④ 引擎标识符（放行：公开格式/生态/DSL 关键字、CLI 动词与取值、产品/CLI 名、
JSON 契约键与 format/kind 枚举值——后者由门禁构建期从冻结 AST 与诊断证据自动采收；
判例逐条登记于 VERIFICATION.md 披露 D07）⑤ 引擎内部路径 ⑥ 生成器文件名与重建命令。
出页前对 `index.html` 与全部面板强制检查，违规即拒绝出页；门禁自带六条**真实违规**
正向对照（样本实时取自引擎仓并断言真实性），6/6 咬住才可用。

## 已知依赖与限制

- 截图用固定版 chrome-headless-shell（playwright 缓存 `chromium_headless_shell-1234`，
  舰队同机共用，逐字节稳定），文本度量依赖系统字体（PingFang SC 等）；换机器或换 shell
  版本渲染像素可能不同，真空判据的位图退路（像素零差）仅在同机同版本成立（披露 D09）。
- `svg-linter` 以默认 recommended 档判定 0 findings（披露 D10）。
- 兼容矩阵条目为引擎 README 的中文意译（披露 D04）；auto 触发条件为源码逻辑的中文概括，
  判定矩阵为实测（披露 D05）；错误演示输入为本方合成、输出为真实观测（披露 D14）。
- 一次性冻结的耗时/尺寸只对冻结环境成立（Homebrew cargo 1.98.0 / macOS arm64，
  披露 D03）。
