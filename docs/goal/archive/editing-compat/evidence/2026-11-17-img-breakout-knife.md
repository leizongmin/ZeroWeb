# img 出壳刀收口（IMG-BREAKOUT/IMG-SWCSS-GUARD，2026-11-17）＝insertimage 语料 23F 全清收官

刀体 b2851753e（`fix(editing): insertimage 出壳刀——折叠插入点提离 inline
壳链＋跨块载体 swcss 保真`；part01g.js IMG-BREAKOUT 臂＋part06.js
IMG-SWCSS-GUARD 守卫＋part132 钉测 16 形，一提交常驻）＋门禁批 i7i1
ALL-DONE（head=b2851753e，bin cebb16870ed34df3616144ec7777b640，2026-11-17
11:16 完走）。F2P 恰 23 零新增红，**img 语料全清 962P/0F/0T**——
insertimage 普查残红（ilb 收官后基线在册）全量划除。

## 基线

刀前基线＝census-insertimage.log（fresh runner 单文件
`ZW_SCRIPT_TIMEOUT_MS=300000 ./target/test-guard --per-proc-mem 4
--total-mem 8 --time-limit 1800 -- ./target/release/zero-wpt-runner
testharness-editing-run 'insertimage.html'`，939P/23F；当时二进制未含刀体
——现盘 release 旧字节特征串 0 复核，i7i1 L0 复建后 ksig 双串各 2 兑现）。
23 键两机制：R1 壳内落位 21（空壳内 caret/腰点分裂/壳界边界/整壳选删/跨壳
界/双空壳链/裸 span 等形）＋R2 跨块值载体 2（swcss true→span、false→font
双态）。mt world 对照＝ihe6-mt 基线 mt 面 img 键恒绿（judge/normalize/
windowing 口径差异），mt 门维持残红恰 X37 双键恒一不动。

## 探针（五段取证，临时测试读数后删除不入提交）

- 数据写同步探针：证伪「data 写未同步宿主」假说——两壳形的 `node.data`
  写均生效。
- JS 树 vs getter 视图分叉探针：`_zwVaSplitText` 是 JS 树拆分，对 M 家族
  （part03 `_zwAttachCharacterDataMethods` 挂载 splitText 的）节点后续
  索引寻址 mutation（RemoveChildAt 等）失配宿主视图——mid-shell 双 B
  根因链（JS 侧 `<B>#A#B</B><IMG/><B>#B</B>`，getter 视图尾 B 滞留 b1）。
- TEMP-DBG（part01g 内，读数后删）：拆前无 delete（cont=#t:AB@1
  st=function），拆分确来自本刀 text 臂而非删除路径。
- appendChild 语义探针：**plain 家族对 createElement 产物壳的 appendChild
  是序列化拷贝**（副本入壳、原节点滞留源 childNodes、parentNode 已指向
  壳——簿记不一致态）；innerHTML 镜像元素目标则真移动（源 childNodes
  清空）。parentNode 判据在此态失效，摘原须按 indexOf 位置判定。

## 根因与修复面（两机制，一提交；vendored insertImage 主体零改动）

1. **IMG-BREAKOUT 出壳落位**（part01g insertimage 臂，对齐 Chrome
   break-out delta＝对象插入时溶解 inline 壳链）：折叠插入点逐层提离
   inline 壳链顶（提离目标＝壳链顶非宿主/块级父——v1 误作止步条件致空壳
   形/块内形零提升、钉测首轮 12/13 红，v2 修正；同宿主门防跨宿主越界）；
   腰点右半壳分裂（文本臂优先节点自带 splitText——M 家族全同步 getter
   视图，数据截断＋尾插入均发 mutation，宿主侧按索引 mutation 不失配；
   无该法的族〔root-text 直子，zz-probe-fs4 实证〕回落 `_zwVaSplitText`
   JS-local 拆分，靠后续宿主直子级插入的子树重序列化承载）；mid-branch
   摘原按 indexOf 位置判定（拷贝态在位即摘、真移动态缺席自然跳过——
   parentNode 判据对序列化拷贝态失效）；插入后邻接空 inline 壳摘除；
   stripWrappers:false 留壳＝语料口径（deleteSelection 不溶壳）。
2. **IMG-SWCSS-GUARD 跨块值载体保真**（part06 `_zwSync` F1'-B 清除位点）：
   `globalThis.__zwImgDelSelInternal` 环绕 deleteSelection（try/finally），
   img apply 的跨块删选不清 swcss → restoreValues 载体按模式选形（true→
   span 载体、false→font 载体），img 落载体壳前块层。

## 钉测（part132，同提交常驻）

`part132_img_breakout_pins.rs` 16 形 1 fn：R1 11 壳形（字节断言，exp 全取
wpt-data insertimage.js 语料录形）＋R2 font 载体字节断言＋span 载体结构
断言（裸 innerHTML color 串形随写入路径波动，字节恒一由语料腿 normalize
口径承载）＋控制 3（根层直插/壳间插入点/sole-br 步现役绿形恒一）。
全库 4126P/0F（4126＝i7h1 面 4118＋part132 钉 1＋intervening css 刀测试
7——L3b 期望用下限门 4119＋`test_img_breakout_pins` 编入在册双证，规避
i7h0 型考古失配）。定向 clippy `-p zero-engine --lib -D warnings` 干净。

## 门禁批（i7i1，首发 ALL-DONE）

| 腿 | 门 | 结果 |
|---|---|---|
| L0 runner 复建 | bin 锚＋刀体双特征串 | bin=cebb1687… ksig IMG-BREAKOUT=2 IMG-SWCSS-GUARD=2 head=b2851753e |
| L1/L2 img 双腿 | 962P/0F/0T | 双腿同数（wall 20s/17s；939+23 翻清算术对齐） |
| L3 键集 judge | F2P 恰 census 残 23 键＋零新增红 | F2P-EXACT-23（特征键 mid-shell-span＋boundary 双中） |
| L3b enginelib strict | F=0＋P≥4119＋钉编入在册 | P=4126 F=0 pin-in=1 |
| L4 mt 腿 | 残红恰 X37 双键恒一（P 留痕） | P=10090 F=2 T=0 wall=2504s，set-identity 恒一 |

批工件 `.acceptance/dps/i7i1-*`（census/img-a/img-b/lib/mt 日志＋键集/
f2p/newred＋批体/发射器脚本＋build.log）；发射器 md5 断言
0501eab9b3ed77a2b630b99b95ccbd38；systemd unit zw-i7i1（批 pid 2808126，
批体脚本 chmod 400 冻结）；混合树注记＝style-system
registry.rs/shorthand/mod.rs 两 foreign fmt 脏文件（css 流域孤儿，未入
批门白名单、未入提交；编辑语料断言 DOM 层 innerHTML，与 style-system
渲染域正交）。

## 账本注记（本刀不动，后续刀候选）

- `_zwFbqInsertAtRangeStart`（part06 :973）raw `sn.splitText` 对非 img
  调用方维持裸拆——非 M 家族节点同型 JS-local 欠发射面潜在（img 路径
  已由本刀 text 臂 splitText 优先修复；其余调用方在 fb apply 面）。
- img-into-carry-font 发射分叉（img 走 deleteSelection apply 臂与
  insertImage 直插臂的 restoreValues 差）已因 img 落块层（块级元素不入
  inline 载体重发路径）而规避，无残留。
