# 提交后门禁复跑记录（指纹豁免）

本文件不参与指纹登记（`tools/fingerprint.py` EXCLUDED，见偏差披露 D08）：提交后再跑
门禁会产出新记录，若入指纹将迫使再提交、再复跑，无收敛点。每跑一次记一节。

## 2026-09-07（refine 批次 2026-09-06，gate-selfbite 修复当日实测）

背景：交付提交 `58181a5` 落地后，引擎主检出 HEAD 演进过冻结提交 `8cfbfe5`，修复前的
`check_engine.py` 对此一律硬失败，冻结/重建/渲染/真空四个链入口全部自咬（修复前实测
rc=1：「引擎 HEAD 已漂移: 58181a5290e2… != 冻结值 8cfbfe5…」）。修复为两态守卫后，
按下述配方实跑：

### 1) 冻结 worktree 配方（README「复跑步骤」第 0 步，实跑）

```text
$ WORK=$(mktemp -d /tmp/ign-dap-refine.XXXXXX)
$ git -C /Users/junix/projects/plot/diagram-ast-parser worktree add "$WORK/engine-frozen" 8cfbfe5
Preparing worktree (detached HEAD 8cfbfe5)
HEAD is now at 8cfbfe5 build: add standard justfile
```

门禁在冻结 worktree 上（ENGINE_REPO=该 worktree，TREE_DIR=真树）：

```text
check_engine: HEAD=8cfbfe572d5f 与冻结值一致；porcelain 0 条（全部为本交付树/治理 README）   rc=0
```

### 2) 「引擎已演进」路径（警告放行，实跑）

在干净检出且 HEAD 已演进的 worktree（交付提交 `58181a5`，porcelain 0 条）上：

```text
check_engine: 引擎已演进（警告，放行）: HEAD=58181a5290e2 != 冻结值 8cfbfe572d5f。
              …完全复现冻结证据请按 README「复跑步骤」开头的配方：
              git -C <引擎仓> worktree add <dir> 8cfbfe572d5f6e18a0bb6e45d30187816f69f872 …   rc=0
```

注：引擎主检出（含本 refine 未提交改动）上 porcelain 命中 ` M docs/…` 4 条仍硬失败
（rc=1）——这是 porcelain 白名单的正确行为：交付树内的未提交改动提交后即消失，
不构成链入口阻断。

### 3) 「证据漂移」毒丸（临时 /tmp 树拷贝上验证，真冻结层未动）

把树拷贝的 `data/frozen/manifest.txt` 中 `engine_head` 改为另一个合法 40 位 sha 后：

```text
check_engine: 证据漂移（硬失败）：冻结层记录的 HEAD 与本脚本常量不一致：
              e3c0ffdeadbeef… != 8cfbfe572d5f…   rc=1
```

### 收尾

- 两个 worktree 用毕已 `git worktree remove`，主检出 porcelain 复原为仅本树改动；
  /tmp 脚手架已删。
- 本 refine 未重跑重建链/真空复跑（`data/frozen/` 一次性守卫 + refine 约束禁引擎
  构建）；产物一致性以 `fingerprint.py check` 现场复验，结果见 VERIFICATION.md
  「2026-09-06 refine」一节。
