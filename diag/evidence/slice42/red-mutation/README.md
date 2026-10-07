# slice42 变异 RED 归档

- `natural-red-prefix-s42.log` — 修复前自然红（4 个新测试 + 2 个既有钉翻转面，TREE_SHA=d1e2a3aae 尾部记录）。
- `mutation-only-delta-s42.diff` — 修复态→变异态纯 delta；变异位点行号见 hunk @@（修复态行号）：
  - M-A L10834 morph 产物回收臂 `if (elsM.length === 0)` → `if (false)`
  - M-B L10852-10854 批删回收原语 `__zwNADelete` 双面清除 → plain delete（wired no-op 复现）
  - M-C1 L10466 NA 树序排序臂 `if (inDoc)` → `if (false)`
  - M-C2 L10879-10881 attr 并入排序 `out32.sort(...)` → 移除
- `mutation-red-s42.log` — 变异态定向红：21P/6F，6 红全数命中三修复面覆盖测试（尾部 TREE_SHA）。
- 逐字节还原自证：修复态 sha256 `79a654cc4a71df608c0eccb8264e3940d58be319c344bd3f2270dd7c7e324a7f`，还原后实测同值（RESTORE_BYTE_EXACT_OK）。
- 行号锚：修复态 = slice42 提交 `79cd8b98a`（origin/main 28784ba23 merge 之前）。merge 在本文件 :12556（PopStateEvent，upstream 独占）插入 21 行，四变异位点（L10466-10881）全部位于其前，行号在新锚（merge 后）仍有效。
