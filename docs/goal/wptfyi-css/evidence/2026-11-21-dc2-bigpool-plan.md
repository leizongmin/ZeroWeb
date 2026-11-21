# DC-2 大池规划输入——weekly run 44668230656 census 逐套件签名聚类（2026-11-21 22:1x，续开轮产出）

数据源：accept-44668230656/new/*（本流 accept-weekly-run.sh 产物）。
全量 F=171,128−103,813=67,315。逐套件签名聚类（消息归一 N=数字）。

## 套件 F 榜（前 12）

| suite | F | 主签名 |
|---|---|---|
| css-grid | 8,007 | 264 "N N N "→"none "＋240 'from' unsupported＋96 repeat→none＋布局 offsetLeft/Top |
| css-values | 4,930 | 1308 'to'+1043 'from' unsupported＝插值登记门＋478 数值差 |
| css-align | 3,926 | 286 数值差＋baseline expanded "→""＋justify/align 序列化 |
| css-transforms | 3,629 | 339+220 数值差＋142 from 门＋123 createDiv harness 错＋90 matrix |
| css-flexbox | 3,329 | 纯布局几何（offset/scroll 宽高断言） |
| css-text | 2,850 | （未聚类） |
| css-sizing | 2,808 | 275 数值＋246 auto→N＋198/112 max/min-content→N＋126 from 门 |
| css-backgrounds | 2,687 | （未聚类） |
| css-motion | 2,416 | 396 from+186 to 门＋86 ray()＋50 path() 序列化 |
| selectors | 2,034 | （未聚类） |
| cssom | 1,503 | （未聚类） |
| cssom-view | 1,432 | （未聚类） |

## 机制主题（按期望产出排序）

1. **插值 consult 通道逐属性族登记**（~3,200F：values 2351＋motion 582＋
   transforms 142＋sizing 126）——`interpolate_property_value_strict`
   返 None→条目不登记→'from/to value should be supported'。S2
   size-lerp 刀（对端已认领，~845F size keyword 切片）是本机制首刀；
   余下族（color/position/angle/filter/transform 列表族）无主。每族
   预注册需先读 transition_registry.rs 登记门与端点解析通道。
2. **尺寸/对齐 keyword computed 表示**（~1,500F：sizing max/min-content
   +auto 家族＋align baseline expanded "→""）——famA gCS 空串族同根
   （读通道无 keyword 表示）。S1b/S1c/G1a 注册域延伸；which-value 判别
   归 S2-symbolic 挂账不变。
3. **grid computed 串化族**（~1,000F：grid-template-*/repeat()/named
   lines→"none "）——属性未入 computed 串化通道（KA2 同型：存储+串化
   最小闭环；布局几何不依赖本刀）。grid-no-interpolation 112F＝离散
   翻转动画语义，独立小刀。
4. **grid/flex 布局几何**（~3,500F：offset/scroll 断言）——深布局机器
   （track sizing/RTL/intrinsic/flex 算法），DC-2 后段主力，需专刀
   序列逐面推进。
5. **transform matrix 精度/串化＋motion ray/path canonical**（~700F）
   ——数值精度与 canonical 序串化，刀型小产出稳。
6. **runners harness 错**（transform 123 createDiv undefined＋grid 76
   testharness pending）——页面级 shim/运行器缺口，ops 债域，逐案
   三角定位后可批处理。

## DC-2 里程碑对位

- 共享 CSSOM 读写（已大头收账）→ 插值通道族（主题 1）＋keyword 表示
  （主题 2）＋grid 串化（主题 3）＝DC-2 剩余主轴，刀型产出可预估
  ~5,700F。
- 布局几何（主题 4）+transform 精度（主题 5）跨入 DC-3 收敛段。
- fill-stroke 371F 全灭（0P）＝SVG 呈现面整域缺口，单列评估是否
  DC-2 范围（依赖 render-foundation SVG 管线，疑似超范围）。
- dead pages 49/25 suites＝ops 债清单（runner 崩溃/超时），挂账批处理。

## 认领态（本 note 时点）

- S2 size-lerp：对端 16:2x fork 已认领（~845F）。
- rshare：对端 KA2 fork 已认领（values random 面）。
- 主题 1 余族/主题 2/主题 3：无主候选。本流下一刀候选（cwcv 清偿后）：
  主题 3 grid computed 串化族（与在飞 face 域不相交：part04.js 属
  cwcv/shim 面，grid 串化属 engine bridge＋style-system 存储，需先核
  对端 G1a 排水）。
