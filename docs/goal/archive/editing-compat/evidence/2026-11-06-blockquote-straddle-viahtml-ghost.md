# blockquote straddle ×4 归因与修复：ViaHtml setter 重建幽灵链 + 主循环重定位臂

日期：2026-11-06。基线 = rtl 刀后 fail 面（iol 8F / iul 12F，`batch-rtl-{iol,iul}-post.log`）。目标族 = blockquote compare ×4（iol :513/:533 + iul :523/:543 镜像，前档静态归因见 `2026-11-06-nested-qstate-blockquote-attribution.md`）。

## 归因：静态推演两度矛盾，DBG 插桩三轮定谳

静态归因预判「straddle 时块未收集」与探针 round1 实测不符：`[foo<blockquote>bar]</blockquote>baz` exec 后 bar 块完全未转换（非「转换了但壳不解包」）；而 G3（`<ol><li>foo</ol><blockquote>[bar]</blockquote>baz` 选区全在 bq 内）反而产出 lift+合并树。两形状同涉 blockquote 容器、行为相反，静态推演不可判，转 DBG 插桩（`zz-probe-bqdbg.html` dump 式探针 + part01e/part01c 五点临时插桩）。

### DBG 铁证链（round2/round3）

nodeList 构建 = `[text foo(host 直子), text bar(bq 直子)]`——块收集本身正常，bar 在列。主循环逐员换型，分歧全在 wrap 落点：

1. **ITER1**：wrap `[text foo]` 命中 `_zwJusWrapViaHtml` flat 模型边界臂（part01c：成员含 editing host 直子文本 → `host.innerHTML = html` 整段重写）。DBG `VIAHTML done kids=3` 后 host 子链全为新建对象（expando 身份标记全失，`kids=[LI,BLOCKQUOTE,#text]` 无标记）。
2. **幽灵链**：旧 text bar 的 parentNode 字段仍指旧 BLOCKQUOTE、旧 BLOCKQUOTE.parentNode 仍指 host（`gp=DIV:isHost=true`）——爬升可达致 `_zwListNodeInDocument` 误真（`inDoc=true`）；但不在 host `firstChild/nextSibling` 链上（子链遍历无旧标记），且幽灵 BLOCKQUOTE `previousSibling=null`（兄弟链断）。
3. **ITER2 落幽灵**：bar 的 li wrap、final wrap 全在幽灵链上做；indent-merge 臂四条件判 `pE=true sI=true ps=null`——ps=null 因幽灵无兄弟链，合并臂 miss → `fresh=true`，新 OL 建在幽灵 BLOCKQUOTE 内，对真实树不可见。
4. **G7 对照（绿）**：nodeList 成员是 P 元素（无 host 直子文本）→ ViaHtml 不触发 → 树全程活引用 → ITER2 merge 臂 `ps=OL:e=true:t=ol` → `fresh=false shellP=OL` 归并 → 正确树。

### 机制定谳

ViaHtml 是 justify 簇引入的 flat 模型 splice 路径（commit `b171ddbf9`：innerHTML getter 走 pending 模型，host 直子文本直接 DOM 操作会与序列化失同步，setter 重写是必要契约，不可简单绕过）。其正确性边界 = 单次 wrap 调用自身（内部有 open-tag 串匹配重定位与端点偏移映射）；跨调用面（toggle 主循环的 nodeList 残余成员）不在其修复域内——残余成员的旧对象引用被 setter 重建孤儿化，后续迭代全落幽灵。

## 修复：主循环迭代头幽灵重定位臂

`_zwListRelocateWrapGhosts(host, nodeList)`（part01e，模块级新函数）+ enable 主循环迭代头调用 + `bqHost` 宿主锚（活动选区端点爬升 editing host，爬升失败恒 null 臂自动禁用）：

- **活引用判定**：宿主 `firstChild/nextSibling` 链身份可达即未重建，跳过（零扰动直过）。
- **幽灵重锚**：成员爬升到宿主直下记下行索引路径（旧链内部 childNodes 一致，索引可靠）；顶层按 `outerHTML` 精确匹配新树同位对象（FIXME 注明同序列化兄弟歧义取首中，语料现存形无歧义面）；沿旧路径 `childNodes[idx]` 下行换新引用。
- **活树路径产物**：重定位后 ITER2 merge 臂 `ps=OL` 命中归并 + wrap 清理尾移除腾空 blockquote——产出即语料期望树，无新增转换逻辑。

共 62 行（函数 50 + 调用 5 + 宿主锚 7）。临时 DBG 插桩（part01e 五点 + part01c 三点 + expando 身份标记）判读后全撤，`grep DBG-bq|__zwBqDbg|__zwBqHost|_zwBqMark` 双文件 0 命中；临时探针 `zz-probe-bqdbg.html` 已删。

## 探针验证（zz-probe-bq.html，10 案）

| 案 | 语料 | 结果 |
| --- | --- | --- |
| iol-F1 :513 | straddle 跨 bq | got=exp ✓（修复前 bq 内无转换）|
| iol-F2 :533 | straddle + 边界 br + 尾段 | got=exp ✓（br 消费 + 尾段 split bq 随活树路径自动正确）|
| iol-G2/G5 nolift | 选区全在 bq 内 | 绿面保持 ✓ |
| iol-G3/G6 lift | 前置同型列表 + bq | 绿面保持 ✓（trace 逐字一致）|
| iol-G7 pform | p 形 straddle | 绿面保持 ✓ |
| iol-F3 pform+尾 | 尾 p 残留 bq | 绿面保持 ✓ |
| iul-F1/F2 :523/:543 | 镜像 | got=exp ✓ |

F2 预判的「边界 br 消费 + 尾段 split」无需新臂：幽灵链阻断解除后，引擎在活树上已有的 G6 同型机制（br 消费 + split blockquote）直接产出正确形状。

## 零扰动探针

zz-probe-br（6 案 exp=got 全一致 + 选区恢复位正常）、zz-probe-rtl（6 案 got 落 alts 备选树）、zz-probe-dps（11 案）、zz-probe-qv5（10 案）全过。

## 门禁批（回填）

systemd detached `batch-bq-legs.sh`（flock 单实例、marker `_zwListRelocateWrapGhosts` 4 命中，19:53:26→20:26:14）：iol 腿 1753→**1755P**/8→**6F**、iul 腿 2053→**2055P**/12→**10F**，judge F2P 2+2（恰 = bq 族 4 行 :513/:533/:523/:543 全翻）/ P2F 0 / post-only 0 / lost 0，双腿 VERDICT OK；enginelib **3,773P/5F 五红名册恒一 ROSTER OK**；mt 全文件腿键级恒等（rows 10,092→10,092，P 9,676→9,676 / F 416→416，F2P 0 / P2F 0 / post-only 0 / lost 0；日志行计数 19,331P/837F）VERDICT OK。**BATCH-VERDICT OK**。

## 残留

- part01c.js 2335 行超 2000 行限为既有状态，本轮未增内容，拆分另行挂账。
- ViaHtml 重锚的 outerHTML 首中策略在同序列化兄弟形（如双 `<blockquote>bar</blockquote>`）有理论歧义面，语料现存形无此形；歧义红行出现时补二段锚。
