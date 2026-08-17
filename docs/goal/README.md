# Goal 目录维护规则

本目录是 ZeroWeb 的目标执行契约区。每个 `*.md` 入口文档定义一个可长期无人值守执行
（`rally run docs/goal/<name>.md`）的目标；`<name>/` 子目录是其运行时控制面。本 README
固化 goal 全生命周期的维护规则——**每轮 rally 执行、人工归档、新 goal 立项都必须遵守**。

---

## 目录结构约定

```
docs/goal/
├── README.md              # 本文件——维护规则
├── <name>.md              # goal 入口文档（活跃目标；稳定，不每轮重写）
├── <name>/                # 运行时控制面（仅活跃/半归档 goal 有）
│   ├── master.md          # 唯一真实状态控制面板（持续演进）
│   ├── evidence/          # 验证证据（通过率报告、失败分析——持续追加）
│   └── archive/           # 里程碑级归档（只追加不修改）
├── zero-web.md            # 父目标（Mission 级总契约）
└── archive/               # 已完成 goal 的终态归档区
    ├── README.md          # 归档索引表（每个归档 goal 一行）
    ├── <name>.md          # 归档的入口文档
    └── <name>/            # 归档的控制面（master/evidence/archive 全树）
```

判别规则：
- **`docs/goal/` 根下的 `<name>.md` = 活跃目标**（含门控等待中的目标）。
- **`docs/goal/archive/` 下的 = 已完成目标**。根目录不应存在「入口已归档但状态仍
  Active」的悬空目录。
- 父目标 `zero-web.md` 永远在根目录，不归档（其 Done Criteria 是全局判据）。

---

## Goal 生命周期

```
立项（拆分/新建） → Active（rally 持续推进） → [门控等待] → DONE 判定 → 归档
```

### 1. 立项

- 从父目标或兄弟目标拆分时，入口文档必须包含：**拆分动机（含用户决策日期）**、
  **基线事实（实测，含代码路径佐证）**、Mission、Support Envelope（范围内/排除/依赖
  约束）、Done Criteria（DC-1~N，可验证）、活跃里程碑、Final Output Protocol、
  Document Control（参照 form-validation / canvas-2d 模板）。
- 同步创建 `<name>/master.md`（含缺口清单、里程碑状态表、碰撞管理记录）+
  `evidence/` + `archive/` 空目录。
- 必须声明**与兄弟 goal 的边界**和 **run-rules §9 碰撞管理**方式。

### 2. Active 期（每轮 rally）

- 进展只写 `<name>/master.md` 与 `evidence/`；**入口文档不在每轮重写**（仅在目标
  本身实质变化时修改）。
- master.md 各章节必须自洽：状态头、缺口清单、里程碑表、验证基线不得互相矛盾。
  发现历史残留旧数据（如中间态通过率）必须当轮修正——**不允许「顶部新、底部旧」**。
- 里程碑完成 → 过程与证据写入 `<name>/archive/`（只追加不修改）。

### 3. 门控等待（可选）

需要用户决策（RFC 批准、Mission 级单向门）或外部条件（物理环境、兄弟流 land）时：
- 入口文档状态行标注**启动门控**及解锁条件；master.md 建「待用户决策」表。
- 门控期间不停摆——转零碰撞面自主推进（WPT 导入、调研、Rust 侧、fixture）。
- **等审批不是 BLOCK**（Final Output Protocol 输出 CONTINUE）。

### 4. DONE 判定

同时满足才可判完成（入口文档 Done Criteria 逐项 + Final Output Protocol 的 DONE
允许条件）：
- 全部 DC 勾选有据（测试命令、报告路径、数字证据）；
- 验证基于上游真实 WPT 用例或如实标明的本地等价用例（无内建 inline 充数）；
- `cargo build` + `cargo test` + `cargo clippy` 全过；
- master.md 内部自洽、里程碑归档完成。

**判完成前的强制检查**：master.md 顶部终态与「验证基线」等历史章节逐节核对——
归档后只追加不可改，矛盾必须留在归档前解决（canvas-2d 归档时曾修正验证基线节
残留的中间态 oracle 数据）。

### 5. 归档

见下节。

---

## 归档规则

**触发**：goal 判定 DONE 后的下一个整理动作（可由该 goal 的 rally 流自己做，也可由
维护轮统一做）。

**两种模式**（二选一，必须把选定模式登记进 `archive/README.md` 索引表）：

### 模式 A：整体归档（canvas-2d 模式，默认推荐）

```
git mv docs/goal/<name>.md  docs/goal/archive/<name>.md
git mv docs/goal/<name>/    docs/goal/archive/<name>/
```

- 整树（入口 + master + evidence + archive）进顶层归档区；goal 根目录彻底清出。
- 适用：引用面已收敛、或愿意同步更新全部外部引用的目标。
- **必须同步**（见「归档检查清单」）。

### 模式 B：入口自归档（form-validation 模式）

```
git mv docs/goal/<name>.md  docs/goal/<name>/archive/<name>-goal-v<N>-<日期>.md
```

- 仅入口文档移入**自身**的 `archive/` 子目录并加归档头注；`master.md`/`evidence/`
  原地保留（master 状态头改为 ✅ 已完成）。
- 适用：外部路径引用面大（如 spec-rfc 权威位置指向 `<name>/test-matrix.md`）、
  整树搬移会破坏大量链接的目标。
- **遗留要求**：`docs/goal/` 根不得残留该 goal 的入口 `.md`；master.md 状态头
  必须已是「✅ 已完成」；顶层 `archive/README.md` 照样登记（注明模式 B）。

### 归档检查清单（两模式通用）

1. [ ] DONE 判定证据齐全（DC 逐项、里程碑归档、evidence 持久化）
2. [ ] master.md 自洽（状态头 = 里程碑表 = 验证基线；无中间态残留数据）
3. [ ] 全仓引用扫描：`grep -rn "goal/<name>" --include="*.md" --include="*.rs" --include="*.toml" .`
   - 父目标（zero-web.md）赛道指针更新为「已完成 + 归档地址」
   - 子 goal 的「父目标」字段更新
   - spec-rfc / research 中的权威路径更新（模式 A）或确认不受影响（模式 B）
4. [ ] `archive/README.md` 索引表登记（文档链接 + 归档日期 + 完成状态摘要）
5. [ ] `git diff --check` + pre-commit-guard PASS 后提交（纯 `.md` 走豁免门禁）

---

## 状态头规范

每个入口文档头部必须有 `**状态**` 行，取值与含义：

| 状态 | 含义 |
|---|---|
| `Active` | 活跃推进中 |
| `Active（启动门控——…）` | 活跃但源码改动被门控；括号内写解锁条件 |
| `✅ Completed（日期——…）` | 已完成（归档前/模式 B 归档后入口或 master 的终态标记） |

**滞后即 bug**：入口文档的阶段描述（如「当前阶段：M0」）落后 master.md 实际进度
（M4 完成）视为文档缺陷，发现即修。html-compat 曾因此残留根目录未归档——入口状态
行停在 M0，与 master/completion-audit 的「M0-M4 全部完成」矛盾，导致完成状态不可见。

---

## 立项/归档的联动更新

- **新 goal 立项时**：若从已归档目标拆分，父目标字段直接指向 `docs/goal/archive/`
  路径（不要指向根目录不存在的人口）。
- **归档时**：当天新立的子 goal 若引用了被归档目标，同批更新（同一次提交内完成，
  避免中间态断链）。
- **并行流协调**（run-rules §10）：各流只写自己的 goal 控制面；跨流整理（统一归档、
  README 登记）由维护轮做或经用户确认后做。

---

## 历史问题与教训（为什么不写这些规则）

| 案例 | 问题 | 本规则对策 |
|---|---|---|
| canvas-2d master.md 验证基线节残留中间态 oracle 数据（真通过 9/不一致 27 vs 终态 41/41、0） | 归档前未做全章节自洽核对 | DONE 判定前的强制检查（§4） |
| form-validation 入口已归档但 master 状态头仍 Active、顶层 README 未登记 | 半归档中间态；流只做了入口移动 | 模式 B 遗留要求 + 索引表强制登记（§5） |
| html-compat M0-M4 完成但入口状态行停在「当前阶段：M0」 | 状态滞后掩盖完成事实，长期未归档 | 状态头规范「滞后即 bug」（§状态头） |
