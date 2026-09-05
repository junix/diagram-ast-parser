# 验证与审计（diagram-ast-parser 技术长图）

本文件是交付树的验证层：声明锚点注册表、门禁记录、渲染断言、产物指纹表、偏差披露。
复跑环境与判据见交付树 README；一次性冻结证据见 data/frozen/。

## 1. 对象与页面

- 引擎仓冻结提交：`8cfbfe572d5f…69f872`（完整值见 data/frozen/manifest.txt）
- 页面：1200 × 7310 CSS px；渲染 dpr 2；full@2x.png 为 2400 × 14620 px（宽 1200×2、高 页高×2 双硬断言通过）
- 渲染：8 片 CDP 截图，每片 scrollTo 后回读 scrollY 断言一致（全部通过）；零外部请求断言通过（总请求 1 项，全部 file://）
- 一次性实测：冷构建 11.16 s（release + 离线）；测试 10.11 s；12 通过 / 0 失败 / 0 跳过（4 个测试套件）；二进制 1,843,072 字节
- 重建复现：重建二进制重放 7 格式，7/7 逐字节复现冻结 AST；auto 识别 7/7 与冻结层一致；4 组错误演示退出码一致；同轮双跑 7/7 一致

## 2. 声明锚点注册表（页面声明 → 证据）

| 面板 | 声明 | 证据 | 期望 | 实测 |
|---|---|---|---|---|
| p1-hero | 7 种解析格式 | `data/rebuild/structure.json#format_modules` | 7 | 7 ✓ |
| p1-hero | 源文件 23+1 | `data/rebuild/structure.json#rust_source_files/rust_test_files` | "23+1" | "23+1" ✓ |
| p1-hero | 总行数 4,715 | `data/rebuild/structure.json#loc.total` | 4715 | 4715 ✓ |
| p1-hero | 测试通过数 = 集成测试声明数 | `frozen#tests_passed vs structure#tests_declared` | 12 | 12 ✓ |
| p1-hero | 最低 Rust 版本 1.85 | `structure#msrv` | "1.85" | "1.85" ✓ |
| p1-hero | 4 项依赖全部精确锁版 | `structure#dependencies_exact_pinned` | 4 | 4 ✓ |
| p2-formats | d2 退出码 0 | `formats#formats.d2.rc` | 0 | 0 ✓ |
| p2-formats | d2 JSON 字节数 | `frozen#cli_formats.d2.ast_bytes` | 1685 | 1685 ✓ |
| p2-formats | d2 JSON 标签 | `formats#formats.d2.json_tag` | "d2" | "d2" ✓ |
| p2-formats | d2 指纹前缀 | `frozen#cli_formats.d2.ast_sha256[:10]` | "c186f1bef1" | "c186f1bef1" ✓ |
| p2-formats | dbml 退出码 0 | `formats#formats.dbml.rc` | 0 | 0 ✓ |
| p2-formats | dbml JSON 字节数 | `frozen#cli_formats.dbml.ast_bytes` | 3007 | 3007 ✓ |
| p2-formats | dbml JSON 标签 | `formats#formats.dbml.json_tag` | "dbml" | "dbml" ✓ |
| p2-formats | dbml 指纹前缀 | `frozen#cli_formats.dbml.ast_sha256[:10]` | "3688963b22" | "3688963b22" ✓ |
| p2-formats | likec4 退出码 0 | `formats#formats.likec4.rc` | 0 | 0 ✓ |
| p2-formats | likec4 JSON 字节数 | `frozen#cli_formats.likec4.ast_bytes` | 2387 | 2387 ✓ |
| p2-formats | likec4 JSON 标签 | `formats#formats.likec4.json_tag` | "like_c4" | "like_c4" ✓ |
| p2-formats | likec4 指纹前缀 | `frozen#cli_formats.likec4.ast_sha256[:10]` | "79ad8aaece" | "79ad8aaece" ✓ |
| p2-formats | nomnoml 退出码 0 | `formats#formats.nomnoml.rc` | 0 | 0 ✓ |
| p2-formats | nomnoml JSON 字节数 | `frozen#cli_formats.nomnoml.ast_bytes` | 1130 | 1130 ✓ |
| p2-formats | nomnoml JSON 标签 | `formats#formats.nomnoml.json_tag` | "nomnoml" | "nomnoml" ✓ |
| p2-formats | nomnoml 指纹前缀 | `frozen#cli_formats.nomnoml.ast_sha256[:10]` | "5c41f6bce8" | "5c41f6bce8" ✓ |
| p2-formats | pikchr 退出码 0 | `formats#formats.pikchr.rc` | 0 | 0 ✓ |
| p2-formats | pikchr JSON 字节数 | `frozen#cli_formats.pikchr.ast_bytes` | 1538 | 1538 ✓ |
| p2-formats | pikchr JSON 标签 | `formats#formats.pikchr.json_tag` | "pikchr" | "pikchr" ✓ |
| p2-formats | pikchr 指纹前缀 | `frozen#cli_formats.pikchr.ast_sha256[:10]` | "676a1a3a30" | "676a1a3a30" ✓ |
| p2-formats | structurizr 退出码 0 | `formats#formats.structurizr.rc` | 0 | 0 ✓ |
| p2-formats | structurizr JSON 字节数 | `frozen#cli_formats.structurizr.ast_bytes` | 2926 | 2926 ✓ |
| p2-formats | structurizr JSON 标签 | `formats#formats.structurizr.json_tag` | "structurizr" | "structurizr" ✓ |
| p2-formats | structurizr 指纹前缀 | `frozen#cli_formats.structurizr.ast_sha256[:10]` | "88203ea62f" | "88203ea62f" ✓ |
| p2-formats | wavedrom 退出码 0 | `formats#formats.wavedrom.rc` | 0 | 0 ✓ |
| p2-formats | wavedrom JSON 字节数 | `frozen#cli_formats.wavedrom.ast_bytes` | 676 | 676 ✓ |
| p2-formats | wavedrom JSON 标签 | `formats#formats.wavedrom.json_tag` | "wave_drom" | "wave_drom" ✓ |
| p2-formats | wavedrom 指纹前缀 | `frozen#cli_formats.wavedrom.ast_sha256[:10]` | "c222d9d16b" | "c222d9d16b" ✓ |
| p2-formats | 重建 7/7 逐字节复现 | `formats#rebuild_reproduce_all` | true | true ✓ |
| p2-formats | 双跑 7/7 一致 | `formats#double_run_all_identical` | true | true ✓ |
| p4-span-error | 未闭合块演示退出码 1 | `behavior#errors.d2_unterminated.rc` | 1 | 1 ✓ |
| p4-span-error | 诊断字段 = 契约五键 | `behavior#errors.d2_unterminated.json vs pipeline#error_json_keys` | ["column", "format", "line", "message", "span"] | ["column", "format", "line", "message", "span"] ✓ |
| p5-auto | auto 矩阵 7/7 一致 | `behavior#auto_matrix_all_match` | true | true ✓ |
| p6-limits | 超限演示退出码 1 | `behavior#errors.input_size_limit.rc` | 1 | 1 ✓ |
| p6-limits | 超限信息含「超出配置上限」语义 | `behavior#errors.input_size_limit.raw` | true | true ✓ |
| p6-limits | 退出码 0/1/2 均实测 | `pipeline#exit_codes[0..2].observed` | true | true ✓ |
| p8-verify | 构建耗时来自冻结层 | `frozen#build_secs` | 11.16 | 11.16 ✓ |
| p8-verify | 测试耗时来自冻结层 | `frozen#test_secs` | 10.11 | 10.11 ✓ |

共 44 条声明，全部机检对表通过（True）。

## 3. 六禁门禁

六类禁止（① 源码文件名 ② 行号定位 ③ ≥25 字符归一化逐字摘录 ④ 引擎标识符 ⑤ 内部路径 ⑥ 生成器名与重建命令）在出页前对 index.html 与全部面板强制检查，违规即拒绝出页。正向对照自检：六条各含一处真实违规（样本实时取自引擎仓），结果 6/6 咬住；页面与面板违规 0 处。

| 对照 | 投毒样本（前 40 字符） | 命中类别 | 结果 |
|---|---|---|---|
| 1 | `引擎的 lexer.rs 负责词法切分` | [1, 4] | 咬住 |
| 2 | `出错位置在语句树第 42 行` | [2] | 咬住 |
| 3 | `结构定义如下：pub struct ParseOptions { pub max` | [3, 4] | 咬住 |
| 4 | `根类型是 DbmlDocument 的文档对象` | [4] | 咬住 |
| 5 | `树构建在 src/parser/tree.rs 中完成` | [1, 4, 5] | 咬住 |
| 6 | `本页由 build_page.py 生成` | [6] | 咬住 |

放行判例与豁免规则见偏差披露 D07。

## 4. SVG 静态检查

- 工具：`svg-linter`，recommended(默认) 档
- 检查 8 张面板，合计 findings 0（逐张 rc=0）

## 5. 产物指纹表

算法 sha256；登记 71 项；自指排除：`VERIFICATION.md`, `data/fingerprints.json`（与 *.failed.txt）。

| 文件 | sha256 |
|---|---|
| `README.md` | `67b1030e35b0df087631571faeda19b31ad46b86f3e1f496ed337722ef27509c` |
| `data/frozen/ast/d2.json` | `c186f1bef1fbb0e7f5c31697cf91fefdd7b7d9f056bd24313841c78fb27ca954` |
| `data/frozen/ast/dbml.json` | `3688963b227f8a9ada25c29ea13dc9192e4e51a9fdf696a54c9ce701ac72bf1f` |
| `data/frozen/ast/dbml.pretty.json` | `21a9fe074dd9d842e5ae2a625691287cf07ceae781880ef20bd55a7bdff0d7a5` |
| `data/frozen/ast/likec4.json` | `79ad8aaece299611456499c4f7c7b335378481a57436c474b096cdb8d276dd26` |
| `data/frozen/ast/nomnoml.json` | `5c41f6bce84286526f4230d6d1fa8625f92b74e25e88dce14969544cc0027bf0` |
| `data/frozen/ast/pikchr.json` | `676a1a3a30c200d2c8937d9168d4959adaafe98e9c1d8b9156ae1b62cabfbaa8` |
| `data/frozen/ast/structurizr.json` | `88203ea62f53e14c8c8adecb21536d2556524e446f6ebb354e50c39ae5bd1f37` |
| `data/frozen/ast/wavedrom.json` | `c222d9d16b8d417d10d1ae809444ed44d49ab90c3b2de9884c3f85f4e1428e05` |
| `data/frozen/auto-wave.json` | `ea9f9682009880ad3e98bb18e17bb2d1cc55c9a0fcfc55fc9c0ac28c0dfc7663` |
| `data/frozen/build-run.txt` | `873e228df06109b352d690ee0c75eeba3267e5affa4a7d8a1654076b55414eda` |
| `data/frozen/build-summary.json` | `cd17547279030b9ada23fe2eed376b6b24f7f90ee20a15c653f15c973ab28698` |
| `data/frozen/cli-matrix.txt` | `ddda0e697cdb34b3dc79dda1aae4a4596165d7c882b003398e0fa5df31b0b98b` |
| `data/frozen/err-badfmt.rc` | `9830ce9d409097e8147e1ce53d55c9b0539a7344c241bd2caad75ca441f08355` |
| `data/frozen/err-badfmt.txt` | `06932331f99583b09e6becfeb5895e7ff54d05d48d82d329a1fad13bd1a0b166` |
| `data/frozen/err-d2.json` | `31d81d84e7f103dcf0b94a0f430b1249d66be818b837c5908edc6e051be98059` |
| `data/frozen/err-d2.rc` | `91d957f8f2748a950526d4d21e0d5cd3bf5518e6c60b1beb470a13be1f978b1d` |
| `data/frozen/err-size.json` | `6f1f363b2fb03f66d836374a920af643e1b77e4382b3c60b3b76e64ece4ab4cd` |
| `data/frozen/err-size.rc` | `91d957f8f2748a950526d4d21e0d5cd3bf5518e6c60b1beb470a13be1f978b1d` |
| `data/frozen/err-wavedrom.json` | `8b11d28ad94f06432244c209e6ecd873bcc466e204c84223938c2278b317ad4a` |
| `data/frozen/err-wavedrom.rc` | `91d957f8f2748a950526d4d21e0d5cd3bf5518e6c60b1beb470a13be1f978b1d` |
| `data/frozen/manifest.txt` | `21dcdf5ead48b50dce87f4aca4f091569e2d607c4c6852484df15ee177c5f6ee` |
| `data/frozen/test-run.txt` | `fcd11dc11a41d31d81adf34d193c20e31b4989c2d141e27cb93f1de0658886e1` |
| `data/frozen/vacuum-report.txt` | `aa216035ebb9153bc13335860891c08e877f4e5acdf071a2e566c2930c4d1895` |
| `data/panels/p1-hero.svg` | `b0ed96a962bd6b8a97e583a555e468ae2e425c0afd92452bd84421fe7765ece5` |
| `data/panels/p2-formats.svg` | `9d5c9869b4550d1a811edf32b10444bd6c8610eacfba633cfd6f8649cc133bad` |
| `data/panels/p3-pipeline.svg` | `7338f01723306698ebc2f364afd353c4bcfb3135142393aa2f605605a15616f6` |
| `data/panels/p4-span-error.svg` | `b8ea4466ca3dbd894b41841fcc983b814515ef4ad8b8e7ca0ad0f39cb6986807` |
| `data/panels/p5-auto.svg` | `29990213e0b88403a7611f61ddc543894192126b67b7a750b8ecc74210268e99` |
| `data/panels/p6-limits.svg` | `1e260d6574d6246112cdc47f9c75040d32eebfd33628090311843691c1dace20` |
| `data/panels/p7-compat.svg` | `3c0e5028847769a60000fe1aad4682145ceab1ae380f901914972281dbff49db` |
| `data/panels/p8-verify.svg` | `3c521c01b2ec967196d0c50f066df55d6516806b827debead9669ed2e7cd5082` |
| `data/rebuild/behavior.json` | `624696f0aa93ddcacaeb5be208a001dead97b7a8efd16e9dcc7614c624163477` |
| `data/rebuild/compat.json` | `3a346a1bdcc6a6c5a306e4fd58bdbd3b6197dd9bb2025aee951f43a07d842de0` |
| `data/rebuild/formats.json` | `3e62c8e515d1ba6fd6389003da0b1cd1b51e53b3cdca1dc2b1c0d08abf5ed38b` |
| `data/rebuild/gate-selfcheck.json` | `4b4b0eccdbe94817f7de361da7df37e1bb84d3d522e88d25d29428877712649e` |
| `data/rebuild/lint-report.json` | `03d09418e51e520e5c9cae514a172dee0bb78e66be77427c3fd3be97a72e20d3` |
| `data/rebuild/page-claims.json` | `36043503dfea596a6ea26d4daeaa02cc5d09df09053c760ad67daad80de2ce53` |
| `data/rebuild/pipeline.json` | `8a5f49d6d5d7492ff41bee93798b19d147279125cafb8b2e0c446d542f435aed` |
| `data/rebuild/structure.json` | `701f79f4e155d0e5ab2062545e6210b6906c79a3400bff5698d67ff34e15c889` |
| `index.html` | `a1a63150fb6f5b349808b9a83cba2947f4188bc3ac98cb40fc31a822fb69faeb` |
| `render/full@2x.gray.png` | `cba6177e2dcac930621320862a854ace3d2518c14349e8760eddcb5a38b62c8f` |
| `render/full@2x.png` | `063f249b31e0d7493aeaa85b1875a454e3e1844c67a21bcd3f3f70062d17d2d9` |
| `render/layout.json` | `9c9f7480e8ec3707571fe2a75017c78f9621625fb839bf2632ed04dd68c972d2` |
| `render/sections/01-p1-hero.png` | `349f81590408bda603e2f3bb7a33a9298c6e525f6b4873974630d6814db7a4ef` |
| `render/sections/02-p2-formats.png` | `dd0a3f468a969479af0b5c0261c908262a114b9f32454950a00a9b39d39878dc` |
| `render/sections/03-p3-pipeline.png` | `3c66e2e7a859fd5cdf8bd3e527e8d5c0d345d295edd3033ee735812616b8248e` |
| `render/sections/04-p4-span-error.png` | `b042af0e4589860a5d8f7e15e91cadfb4a06d0935d35511b8bf100661c0a6bec` |
| `render/sections/05-p5-auto.png` | `dfb7f9076af40de55293e3a52edefe90385ac800a84efe8f7def4da71b81fb11` |
| `render/sections/06-p6-limits.png` | `67ddd0c0e51200119b43fcc1844fd4eeb577566d9ff630a2dbb9cab1a82ed125` |
| `render/sections/07-p7-compat.png` | `c708294f4ed3952abd8fa9e15a814e712c00defa6c0ca296357e03cf6a7adfd0` |
| `render/sections/08-p8-verify.png` | `84560d10f210612cea60b4f96e0726a77f9b20fe5b91e83924cf5d0ada9262f9` |
| `render/thumb.png` | `d619f8f07f93f1747062420f05f7b5bdc8b34b0c0d6430da6a4b0ffa235fafbd` |
| `tools/allowlist.json` | `47836582936eed88d10ff8c4389550e1f5fe996aea2cff162b0cf9ca836f26d9` |
| `tools/build_page.py` | `844eac3ebb8ec1f10efb02b2e4578a3c3bde6b17a5505ff232e556034b1eff59` |
| `tools/check_engine.py` | `0df1b905505565990d5a313c115806a8be597972edcd1b75e93561f14a586041` |
| `tools/cmp_artifacts.py` | `9732ba57a386a47b3b02eaf61c0e8cdc2874e58c987efc125fda50d918735aaf` |
| `tools/disclosures.json` | `007ff5d12dd45de5574aaff8715de13cf40cc4ad508c2f15af097f30f867aa3d` |
| `tools/export_static.py` | `549f4b2648611c008db0d4b812be0f9e8b10fca1ae2500ddfd0fec67b05c0079` |
| `tools/fingerprint.py` | `927035a8b985df663066efc292fedff169cff269cb10d0af641c20c9adb061dc` |
| `tools/freeze_once.sh` | `83d1e2a8cc76f7e13a690b1727e7e990d7e230d3928614b39b13169e14387e82` |
| `tools/gate_check.py` | `fee9defe2e7cd9176f64d62fc8700462684b0277611cc14ea4d804c775d54819` |
| `tools/panels.py` | `51f970b633412c20fc48624e4f08eded2e45cc8112278158589dbf1541194f5b` |
| `tools/parse_freeze.py` | `9642c2776f77c10215e1fdf783c1c50e01e5653821311722e16febd886cc07ff` |
| `tools/probe_behavior.py` | `f136895b886522d10018a9cda82ef4dadf0c91b00528cac4f45c58b752a7e30b` |
| `tools/rebuild_chain.sh` | `b5ec28f5c11031c50b0f6fec111de286a58d4ccd3d07304987aa57da7076772e` |
| `tools/render_page.mjs` | `580bc1a97ad04a91167b2f2eb9dca51abae9f05bd7d88f756d3232b454d85c47` |
| `tools/stitch.py` | `504c5f5dc9c58df7c6ef3deac5fdec344940fc8c0667322705529437a98ef9b7` |
| `tools/svgkit.py` | `3a21b95c62f03772cc4205f2290ca623e49de40a9e0acc93d681b6a6238567cd` |
| `tools/vacuum_rerun.sh` | `b736cd015e7c9154725f15376a711f8e907d5a4eb0aa874060068c4ed5797ea7` |
| `tools/verification.py` | `1128e744a6ccebd98e95994a7601b395eeb66e01e741bd31460d81b3043537c5` |

## 6. 偏差披露

- **D01 重建层的环境归一化（路径烧录）**：重建链在 /tmp 工作拷贝中构建与探测（引擎仓对树外只读）。重建层产物（data/rebuild/*.json、面板、页面）不含任何绝对路径、时间戳或工具版本串：探测只落 rc/字节/哈希/结构化 JSON；页面与面板数字全部来自冻结/重建证据。若未来某证据字段需要携带路径，必须以 <WORK>/<build-dir>/<ts> 占位符归一化后在 README 写明——当前没有任何字段需要。（锚点：`tools/probe_behavior.py + data/rebuild/behavior.json`）
- **D02 一次性冻结不重测**：冷构建耗时、测试耗时、测试计数、二进制尺寸、7 份 AST 输出及其哈希、错误演示退出码均为一次性实测，冻结后绝不重测覆盖（freeze_once.sh 有一次性守卫）。真空复跑不重建 data/frozen/；页面引用这些数字时以冻结层为准。（锚点：`data/frozen/build-summary.json + tools/freeze_once.sh`）
- **D03 构建器与工具链事实**：冻结与重建使用 Homebrew cargo/rustc 1.98.0（macOS arm64），忽略仓库的 stable 通道声明文件；仓库声明的最低 Rust 版本为 1.85。依赖在 Cargo.lock 全量锁版且走 --offline 离线构建（注册表缓存命中，无需联网）。若换机器重跑，工具链版本可能不同：AST 输出为确定性的结构序列化，哈希不随编译器版本漂移，但冻结层的耗时/尺寸数字只对冻结环境成立。（锚点：`data/frozen/manifest.txt`）
- **D04 兼容矩阵为引擎 README 的中文意译**：p7 面板的「已实现/有意推迟」条目是引擎 README 兼容性矩阵的中文意译（属引擎自述事实，非本方实测）；措辞经意译以避免逐字摘录，语义忠实原表。实测部分（计数/退出码/指纹）与该矩阵分开呈现。（锚点：`data/rebuild/compat.json + tools/export_static.py`）
- **D05 auto 触发条件为源码逻辑的中文概括**：p5 面板各级触发条件是引擎格式识别函数逻辑的中文概括（关键词为公开 DSL 关键字）；判定矩阵（fixture→识别结果）为重建二进制真实重放实测，7/7 与冻结层一致。（锚点：`data/rebuild/pipeline.json#auto_detect_order + behavior.json#auto_matrix_observed`）
- **D06 p4 区间刻度为示意、字节总数为真值**：p4 面板左侧的示例 DSL 行与字节刻度为示意教学图（该行非引擎仓内容）；展示的字节总数按同一字符串实际 UTF-8 编码长度计算（真值），刻度线位置按等宽近似绘制，已在图内标注「刻度示意」。右侧诊断 JSON 为真实命令行输出的逐字转录。（锚点：`tools/panels.py p4 + data/rebuild/behavior.json#errors.d2_unterminated.raw`）
- **D07 门禁放行判例（逐条）**：④标识符禁按判例放行：产品/CLI 名（diagram-ast-parser、diagram-parse）；CLI 动词与 --format 取值（auto/dbml/wavedrom/d2/structurizr/likec4/nomnoml/pikchr 及别名 wavejson/structurizr-dsl/c4/pic）；公开格式与生态名（DBML/WaveDrom/D2/Structurizr/LikeC4/C4/nomnoml/Pikchr/JSON/JSON5/Rust/cargo/clap/serde 系）；公开 DSL 关键字（signal/reg/wave/table/ref/enum/workspace/softwaresystem/systemcontext/container/specification/model/views/box/circle/arrow 等，见 allowlist.json 全表）；JSON 契约键与 format/kind 枚举值由门禁在构建期从冻结 AST 与诊断证据自动采收放行；少量通用英文词（input/output/parse/source/span/line/column/message/format 等）按通用语放行并在 allowlist 登记理由。①文件名禁放行：7 个 fixture 名与 Cargo.toml/Cargo.lock（生态 manifest 名）。②行号禁对真实转录 zone 内的 line/column 字段值豁免（契约值）。③摘录禁对命中区间完整落在放行 zone（真实 CLI 转录原文、CLI flag、AST 信封示例）内豁免。（锚点：`tools/allowlist.json + tools/gate_check.py`）
- **D08 指纹登记的自指排除**：指纹登记表排除 data/fingerprints.json（登记表自身，防自指漂移）与 VERIFICATION.md（内嵌指纹表）。除此两件与 *.failed.txt（冻结失败留档，非证据）外全树登记。（锚点：`tools/fingerprint.py EXCLUDED`）
- **D09 渲染依赖固定版 chrome-headless-shell 与系统字体**：截图用固定版 chrome-headless-shell（playwright 缓存 chromium_headless_shell-1234，舰队同机共用；去 --headless=new，加 --disable-gpu + srgb 色彩剖面固定光栅路径）与系统字体（PingFang SC 等）。真空复跑位图比对首选逐字节；PNG 编码差异时退路为像素零差 + PNG 辅助块归一化后一致（本树 PNG 均经 magick 剥除日期/时间块，双跑逐字节一致）。换机器或换 shell 版本渲染像素可能不同。（锚点：`tools/render_page.mjs + tools/stitch.py + render/layout.json`）
- **D10 SVG 静态检查使用默认 recommended 档**：逐张 rc=0 且 0 缺陷以 svg-linter 默认 recommended profile 判定（稳定规则集）；预览类规则不计入该声明。文本布局按字体度量近似估算并保守换行。（锚点：`data/rebuild/lint-report.json`）
- **D11 渲染切片为瞬态中间产物**：CDP 切片在拼接完成后即删除（render/slices/ 不留档）；layout.json 保留每片 scrollTo/scrollY 断言记录与零外部请求断言供复核。（锚点：`render/layout.json scrolls`）
- **D12 退出码 3 未在演示中触发**：退出码 0/1/2 由真实演示观测（成功解析、解析失败、非法格式值）；退出码 3（AST 序列化失败）为引擎源码预留路径，正常输入无法触发，页面如实标注「预留（演示未触发）」。（锚点：`data/rebuild/pipeline.json#exit_codes + behavior.json#errors`）
- **D13 引擎仓 porcelain 状态披露**：冻结与重建期间引擎仓 HEAD 恒为冻结值，porcelain 仅含本交付树（未跟踪的 docs/ 目录以最外层目录显示）。若 docs/infographics/ 根出现外部治理进程放置的 README.md，非本树产物，本树不增不删不改，仅在验证文档披露。（锚点：`tools/check_engine.py`）
- **D14 错误演示输入为本方合成**：四组错误演示的 stdin 输入为本方合成（未闭合块、负数寄存器位宽、超限输入、非法格式值），输出与退出码为真实命令行观测；引擎自带测试对这些场景另有断言（测试计数在冻结层）。合成输入不冒充引擎样本。（锚点：`data/rebuild/behavior.json#errors + tools/probe_behavior.py`）
- **D15 首次冻结尝试中途中止（未落地任何冻结产物）**：一次性冻结脚本首跑在真实构建与测试完成之后、CLI 转录环节因两处脚本缺陷中止（变量名后紧跟多字节标点致 set -u 误析；CLI 调用未切到导出拷贝目录致 rc=2）。冻结采用「全部产物先落 /tmp 暂存、最后原子移入」设计，中止时 data/frozen/ 尚不存在，无任何产物落地；修复脚本后重跑的冻结才是本树唯一冻结层。已落地的 build-run/test-run 耗时数字来自第二次（成功）运行，首次运行的构建/测试计时未进入任何证据。（锚点：`tools/freeze_once.sh（暂存-原子移入设计）`）
- **D16 构建期工具缺陷的自修复记录（均发生在任何指纹登记之前）**：出页前的迭代中修复了本方工具缺陷：依赖普查正则漏配无括号精确锁版写法（曾计 2，应为 4，已加修复并用对表声明固定）；行为探测曾把 auto 成功演示误比错误退出码；文本换行器曾把连续中文切成不可断整串导致两处面板出界（svg-linter 咬住后改为逐字可断）；门禁标识符采集曾误收源码注释/字符串里的普通英文词（改为只认定义位标识符），HTML/SVG 扫描改为先剥标签取可见文本视图再做位置化豁免（空白容忍、实体还原）。以上全部发生在冻结层落地之后但任何指纹登记/真空复跑之前；冻结层证据不受影响，最终门禁以修复后的工具通过（含六条真实违规正向对照 6/6）。（锚点：`tools/export_static.py + tools/probe_behavior.py + tools/svgkit.py + tools/gate_check.py`）

## 7. 真空复跑

判据（README 预声明）：删除全部可重建产物后，双拷贝 A/B 全链重建；文本逐字节 cmp；位图首选逐字节，退路像素零差 + PNG 辅助块归一化；data/frozen/ 不参与删除。结果一次性冻结到 data/frozen/vacuum-report.txt（有守卫，绝不重跑覆盖）。原文如下：

```text
# 真空复跑一次性报告（绝不重跑覆盖）
engine_head=8cfbfe572d5f6e18a0bb6e45d30187816f69f872
criteria: 删除可重建产物 -> A/B 双拷贝全链重建 -> 文本逐字节 cmp；位图逐字节优先，
          退路像素零差 + PNG 辅助块归一化；data/frozen/ 不参与删除
---- A/B 比对 ----
比对文件 70 项：文本逐字节一致 59，位图逐字节一致 11，位图退路一致 0
独有文件：A 0 / B 0（登记表与验证文档自指文件已跳过）
cmp_artifacts: 两树产物等价（判据内一致）
---- 真树 vs A 比对 ----
比对文件 70 项：文本逐字节一致 59，位图逐字节一致 11，位图退路一致 0
独有文件：A 0 / B 0（登记表与验证文档自指文件已跳过）
cmp_artifacts: 两树产物等价（判据内一致）
---- 指纹终检 ----
真空报告自身须先被登记：真空后执行 fingerprint.py write -> verification.py -> check（见 README）
```

