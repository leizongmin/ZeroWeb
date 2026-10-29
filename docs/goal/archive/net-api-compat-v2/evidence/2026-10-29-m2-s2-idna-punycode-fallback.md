# M2-S2 — url IDNA IgnoreInvalidPunycode 回退（2026-10-29）

**切片**: M1 归因档收敛排序第 2 位——url IDNA 簇 ~783 legs（IdnaTestV2 759 +
url-origin/a-element-origin 24）。

## 根因

`url 2.5.8 → idna 1.1.0` 域转 ASCII 在 `uts46.rs to_ascii_from_cow` 硬编码
`ErrorPolicy::FailFast`（该枚举仅 FailFast/MarkErrors 两档），`xn--` 标签
punycode 解码/验证失败即整域拒绝。WHATWG URL spec「domain to ASCII」非严格
路径自 2023 起 IgnoreInvalidPunycode=true（whatwg/url#737）——无效 punycode
标签**原样保留**、整域不失败，chromium/firefox/safari 全过。crate 无开关，
不满足 spec 语义。

## 实现（crates/engine/src/js_dom_bridge/url_idna.rs，新模块）

主解析失败后回退（`parse_url_to_json` Err 分支；正常路径零变化）：

1. **门槛**：仅绝对输入 + 特殊 scheme（http/https/ws/wss/ftp/file——host 走
   domain 解析 + IDNA；opaque host 面、相对解析、setter 面不涉）。
2. **逐标签甄别**：可回退 = 纯 ASCII 可打印（0x21..0x7F）+ 小写后 xn-- 前缀 +
   不含 spec forbidden domain code point（`% < > [ \ ] ^ | :`；`# / ? @` 由
   authority 拆分天然排除）+ **url crate 自身探测拒绝**（与生产同算法，不加
   依赖）。非 ASCII 混入与 `%`（forbidden domain 面）不参与——expected-null
   行维持 TypeError。
3. **占位重解析**：可回退标签换 `zwpcfb<n>` 占位重解析；仍失败 → 维持原失败。
4. **组件回写**：hostname/host/origin/href 的 host 部分按「占位形全标签 +
   计数恰等」回写原样（原 host 合法含同形标签 → 计数不符 → 放弃回退维持原
   失败，绝不错误回写）；pathname/search/hash/userinfo 无占位不跨组件。

一期遗留的 `serialize_url` panic 守卫（hostile username/password 面）随组件
提取迁入 `serialize_url_components`（catch_unwind → None），语义不变。

**迭代注记**：首轮实现 host 形态限定 `[a-z0-9-]`，IdnaTestV2 余 8 行含 `,`
标签失败——`,` 非 forbidden domain code point（spec 为非 fatal 验证错误，
IdnaTestV2 期望 PASS），crate deny list 过严；放宽为「可打印 ASCII −
forbidden domain 集」后收口。

## 验证

- 模块单测 6 项（回退/边界/碰撞放弃/userinfo+port/非特殊 scheme）+
  既有 url 单测 55 项零回归（`cargo test -p zero-engine --lib url_`）。
- 通道全量 A/B（`netapi-m2s2-r2.json` vs M2-S1 r3 基线，逐 subtest）：

| 域 | M2-S1 r3 | M2-S2 r2 | ΔP |
|---|---|---|---:|
| url | 4919/1146 | **5708/357** | **+789** |
| 其余六域 | — | — | ±0 |

- **+789 零回归**：IdnaTestV2 2671/2671 = 100%（759F→0）+ url-origin/
  a-element-origin 14 + IdnaTestV2-removed 面；url 通过率 81.1% → 94.1%，
  超出 M1 归因预测（~783）。
- url 残余 357F = setters-stripping 54 + USP BOM 28 + 单案甄别面，与 M1
  归因档预测一致。

## 边界与已知限制

- setter 路径（`__zw_set_url_part` hostname/host）未接回退——url-setters 语料
  无 IDNA setter 失败面（M1 归因），按精准修改原则不扩。
- 回写按字面 verbatim（小写化），与 UTS46 输出一致；含 `,` 等非 STD3 字符的
  域名序列化面（fetch/导航真实请求路径）未在本切片范围。
