use super::*;

include!("js_dom_bridge_tests/part01.rs");
include!("js_dom_bridge_tests/part02.rs");
include!("js_dom_bridge_tests/part03.rs");
include!("js_dom_bridge_tests/part04.rs");
include!("js_dom_bridge_tests/part05.rs");
include!("js_dom_bridge_tests/part06.rs");
include!("js_dom_bridge_tests/part07.rs");
include!("js_dom_bridge_tests/part08.rs");
include!("js_dom_bridge_tests/part09.rs");
include!("js_dom_bridge_tests/part10.rs");
include!("js_dom_bridge_tests/part11.rs");
include!("js_dom_bridge_tests/part12.rs");
include!("js_dom_bridge_tests/part13.rs");
include!("js_dom_bridge_tests/part14.rs");
include!("js_dom_bridge_tests/part15.rs");
include!("js_dom_bridge_tests/part16.rs");
include!("js_dom_bridge_tests/part17.rs");
include!("js_dom_bridge_tests/part18.rs");
include!("js_dom_bridge_tests/part19.rs");
include!("js_dom_bridge_tests/part20.rs");
include!("js_dom_bridge_tests/part21.rs");
include!("js_dom_bridge_tests/part22.rs");
include!("js_dom_bridge_tests/part23.rs");
include!("js_dom_bridge_tests/part24.rs");
include!("js_dom_bridge_tests/part25.rs");
include!("js_dom_bridge_tests/part26.rs");
include!("js_dom_bridge_tests/part27.rs");
include!("js_dom_bridge_tests/part28.rs");
include!("js_dom_bridge_tests/part29.rs");
include!("js_dom_bridge_tests/part30.rs");
include!("js_dom_bridge_tests/part31.rs");
include!("js_dom_bridge_tests/part32.rs");
include!("js_dom_bridge_tests/part33.rs");
include!("js_dom_bridge_tests/part34.rs");
include!("js_dom_bridge_tests/part35.rs");
include!("js_dom_bridge_tests/part36.rs");
include!("js_dom_bridge_tests/part37.rs");
include!("js_dom_bridge_tests/part38.rs");
include!("js_dom_bridge_tests/part39.rs");
include!("js_dom_bridge_tests/part40.rs");
include!("js_dom_bridge_tests/part41.rs");
include!("js_dom_bridge_tests/part42.rs");
include!("js_dom_bridge_tests/part43.rs");
include!("js_dom_bridge_tests/part44.rs");
include!("js_dom_bridge_tests/part45.rs");
include!("js_dom_bridge_tests/part46.rs");
include!("js_dom_bridge_tests/part47.rs");
include!("js_dom_bridge_tests/part48.rs");
include!("js_dom_bridge_tests/part49.rs");
include!("js_dom_bridge_tests/part50.rs");
include!("js_dom_bridge_tests/part51.rs");
include!("js_dom_bridge_tests/part52.rs");
include!("js_dom_bridge_tests/part56.rs");
include!("js_dom_bridge_tests/part53.rs");
include!("js_dom_bridge_tests/part57.rs");
include!("js_dom_bridge_tests/part58.rs");
include!("js_dom_bridge_tests/part59.rs");
include!("js_dom_bridge_tests/part60.rs");
include!("js_dom_bridge_tests/part61.rs");
include!("js_dom_bridge_tests/part62.rs");
include!("js_dom_bridge_tests/part63.rs");
include!("js_dom_bridge_tests/part64.rs");
include!("js_dom_bridge_tests/part65.rs");
include!("js_dom_bridge_tests/part66.rs");
include!("js_dom_bridge_tests/part67.rs");
include!("js_dom_bridge_tests/part68.rs");
include!("js_dom_bridge_tests/part69.rs");
include!("js_dom_bridge_tests/part70.rs");
include!("js_dom_bridge_tests/part71.rs");
include!("js_dom_bridge_tests/part72.rs");
include!("js_dom_bridge_tests/part73.rs");
include!("js_dom_bridge_tests/part74.rs");
include!("js_dom_bridge_tests/part75.rs");
include!("js_dom_bridge_tests/part76.rs");
include!("js_dom_bridge_tests/part77.rs");
include!("js_dom_bridge_tests/part78.rs");
include!("js_dom_bridge_tests/part79.rs");
include!("js_dom_bridge_tests/part80.rs");
include!("js_dom_bridge_tests/part81.rs");
include!("js_dom_bridge_tests/part82.rs");
include!("js_dom_bridge_tests/part83.rs");
include!("js_dom_bridge_tests/part84.rs");
include!("js_dom_bridge_tests/part85.rs");
include!("js_dom_bridge_tests/part86.rs");
include!("js_dom_bridge_tests/part87.rs");
include!("js_dom_bridge_tests/part88.rs");
include!("js_dom_bridge_tests/part89.rs");
include!("js_dom_bridge_tests/part90.rs");
include!("js_dom_bridge_tests/part91.rs");
include!("js_dom_bridge_tests/part92.rs");
include!("js_dom_bridge_tests/part93.rs");
include!("js_dom_bridge_tests/part94.rs");
include!("js_dom_bridge_tests/part95.rs");
include!("js_dom_bridge_tests/part96.rs");
// renderer 信任边界 ②C'：`__zw*` 特权 mutator counter-plugin 常驻回归（非 part 编号——
// 跨 part 面：删除策略 + 闭包化非回归 + 激活门不可旁路）。
include!("js_dom_bridge_tests/privilege_guard.rs");
// 45 号 shadow-dom-rendering M3：命令式面变异序列探针/回归（attachShadow 移动语义 +
// 分配表新鲜度）。
include!("js_dom_bridge_tests/part97_m3_shadow_probe.rs");

// 46 号 s363 mixed eager/defer 同 URL 记录共享：defer 面命中 eager 缓存零再求值钉。
include!("js_dom_bridge_tests/part98_s363_mixed_eager_defer.rs");

// 47 号 s397 module parse 失败缓存面：tag 5 同对象复抛 + tag 3 逐次新对象对照钉。
include!("js_dom_bridge_tests/part99_s397_module_parse_failure.rs");

// 48 号 s398 OBJECT contentWindow 嵌套浏览上下文面：text/html 物化 + name 快照
// + 四面 null 分类 + R5274 恒 null 面保持钉。
include!("js_dom_bridge_tests/part100_s398_object_content_window.rs");

// 49 号 s399 object 动态加载链：插入钩子 load/error 事件面 + data IDL setter
// 重启 + about:blank 嵌套上下文 + win.Window interface + image/detached 边界钉。
include!("js_dom_bridge_tests/part101_s399_object_dynamic_load.rs");

// 50 号 s400 object scheme 取回面 + window.length：data:/blob:/bare-origin 解码
// 七面 + LegacyUnforgeable 计数（iframe/object entry + destroy/移除递减）钉。
include!("js_dom_bridge_tests/part102_s400_object_schemes_window_length.rs");

// P39（siteopt github r1）：tagName/namespaceURI stamp 失配窗口永久章快道回归。
include!("js_dom_bridge_tests/part99_p39_tag_ns_stamp_memo.rs");

// 51 号 s403 object 嵌套文档脚本通道：parser 静态启航（静默）+ R206 脚本移植
// （parent 回调链）+ 嵌套导航重物化 + named access morph + replaceState 槽钉。
include!("js_dom_bridge_tests/part103_s403_object_nested_scripts.rs");

// 52 号 s404 object window[N] 索引 + javascript: 导航：WindowProperties 索引层
// （own accessor 物化序近似树序 + descriptor/越界/移除重排）+ data 属性静默
// fallback + 完成值求值通道（percent-decode → 重物化 → load 重派/静默中止）钉。
include!("js_dom_bridge_tests/part104_s404_object_window_index_javascript_nav.rs");

// 53 号 s405 embed 动态加载链：插入钩子启航（共享加载核 src 属性读）+ 失败终态
// 映射 load + javascript: 全静默 + src IDL 重启臂 + window[N] 活跃集合计入钉。
include!("js_dom_bridge_tests/part105_s405_embed_dynamic_load.rs");

// 54 号 s406 frames named access 扩 embed/object + 嵌套导航失败终态：R177 帧代理
// named face 收口（embed-network-error 子测 3/4 driving）+ 失败三臂收拢（embed
// 重派 load / object 静默保持）钉。
include!("js_dom_bridge_tests/part106_s406_frames_named_embed_nav_fail.rs");

// 55 号 s407 静态 embed 链 + fallback 子内容激活：parser 扫描扩 EMBED（src 启航
// + media/loaded-ancestor 双门）+ object 祖先门 + data: 不可渲染 essence
// 'fallback' 态（子内容激活 + 渲染失败事件映射）钉。
include!("js_dom_bridge_tests/part107_s407_embed_static_fallback.rs");

// 56 号 s408 detached 文档 style 元素 LinkStyle sheet + headEl 本地树三件套：
// non-parser-created-doc.html 红根修复（headEl appendChild 链 + plain 产物
// `sheet` 访问器 + _makeStyleSheet 门 plain owner 臂）钉。
include!("js_dom_bridge_tests/part108_s408_detached_style_sheet.rs");

// 57 号 s409 script-created link stylesheet 不阻塞 script：s163 延迟仿真相对
// URL 归一（发题侧）+ 阻塞计数门控到 parser-inserted 面（script-created-link
// 双案红根修复）钉。
include!("js_dom_bridge_tests/part109_s409_script_created_link_stylesheet.rs");

// 58 号 s411 maxlength 适用态门：maxlength 仅六文本态适用（number 不限长
// maxlength-number.html 红根修复）+ text 既有绿面 + 非法 type 值 Text 态
// invalid default 对照钉。
include!("js_dom_bridge_tests/part110_s411_input_maxlength_number.rs");

// s421 pending 视图 handle rect：createElement 产物裸 handle 身份在 mutation
// pending 窗口内的真几何（embed-allowed-schemas 5F 根因——load 同 turn 派发先于
// apply，快照无 rect 恒 0）。
include!("js_dom_bridge_tests/part111_s421_pending_handle_rect.rs");

// s422 constraint validation candidate 门：object/fieldset/output 非 candidate
// （object-constraint-validation 1F 根因——setCustomValidity 后 checkValidity
// 恒应 true，candidacy 门缺失 customError 落穿）。
include!("js_dom_bridge_tests/part112_s422_object_constraint_validation.rs");

// 编辑命令兼容 Form A 小刀：splitText 头截断欠发射面双向钉测（D2 v1 形复现，
// 红=欠发射复发按流归因 / 绿=已被 B1/D1/E1 吸收转常驻正期望）。
include!("js_dom_bridge_tests/part111_form_a_splittrim_probe.rs");

// 编辑命令兼容 ihtml 轴二行内壳刀：inserthtml 宿主直子文本门对选区锚在行内壳
// /rawtext 容器内的静默 bail 修复双向钉测（styled 壳溶解 / rawtext 壳内拼接 /
// pre 块载荷解包 / xmp markup 载荷 parser 事实钉）。
include!("js_dom_bridge_tests/part112_ihtml_shell_sel_probe.rs");

// 编辑命令兼容 ip 206 M1 主桶探针：insertParagraph 拆分边界 inline formatting
// element 复制/丢弃语义三形钉测（b 壳/span 壳 caret 拆分 + 壳全含选区空壳丢弃）。
include!("js_dom_bridge_tests/part113_ip_inline_shell_split_probe.rs");

// s423 canvas 导出编码格式面：toBlob/toDataURL type 参数接通（toBlob.jpeg 1F
// 根因——type 忽略恒产 png Blob；png+jpeg 白名单，其余 PNG fallback）。
include!("js_dom_bridge_tests/part113_s423_canvas_export_format.rs");

// T68 sel/handle 双族注册表 sync 常驻钉：t67-minrepro 复现页核心四案例语义
//（class 形态 QSA 归并 / removeChild 序列化空串 / drain 后 identity 反查 /
// fragment 视图 handle 族子宿主 wire 双臂）。
include!("js_dom_bridge_tests/part114_t68_selhandle_registry_sync.rs");

// 编辑命令兼容 ip 残 99F M3-A 探针：可编辑根层非首行裸文本 caret insertParagraph
// 拆分左段裸留语义五形钉测（行中尾位/词中切片/行首补 br/首行 wrap guard/dPS p）。
include!("js_dom_bridge_tests/part114_ip_root_bare_text_split_probe.rs");

// s423 pending 视图 selector rect：gBCR selector 身份在 mutation pending 窗口内
// 走有效视图现算布局（CSSOM View update the style and layout steps）——testharness
// 单脚本 turn 内 style 写 → offsetWidth/offsetLeft 同 turn 断言面（position-area
// 族 ~190F 根因）双向钉测（写读同值 + drain 后快照旧行为）。
include!("js_dom_bridge_tests/part115_s423_pending_selector_rect.rs");

// 编辑命令兼容 ip 残 99F M3-C C4 刀钉测：块边界拆分块级后随兄弟止步不入新块 +
// bare-br 行界双空块物化（表跨选区/bare-br 尾·首侧·尾随块级/行中窄门 guard/
// 整块选删 host 级 caret，dPS div·p 双面）。
include!("js_dom_bridge_tests/part116_c4_block_boundary_pins.rs");

// 编辑命令兼容 ip 残 59F M3-B B 族刀钉测：拆点块界空白/nbsp 归一（② sep2 同型
// 块拆臂行界折叠空格翻 `&nbsp;`/whitespace-only 段退化占位/单侧退化跳过对侧 +
// ⑦ 宿主元素边界臂尾侧 whitespace-only 留宿主、首文本行界翻 `&nbsp;`）。
include!("js_dom_bridge_tests/part117_b_split_boundary_ws_pins.rs");
include!("js_dom_bridge_tests/part118_shell_carry_pins.rs");
// T72（site-optimizer perf-mem T72/P42）：iframe win.navigator 平面对象语义常驻钉
//（part05 _zwMakeIframeWin 工厂面——原型链/品牌/Illegal constructor 双臂/
// serviceWorker 槽恒一/走链吸收五断言面；T66 主窗面钉在 part95.rs）。
include!("js_dom_bridge_tests/part119_t72_iframe_navigator_pins.rs");
include!("js_dom_bridge_tests/part119_c1_order_pins.rs");
include!("js_dom_bridge_tests/part120_ip_atomic_rawtext_pins.rs");

// ip C4 容器链垫/劈裂刀钉测（2026-11-15）：垫底 br 持行豁免——空 dt 拆分垫
// `<dt><br></dt>` 保全 + blockquote/ul 容器链尾新块物化 + 控制键恒一。
include!("js_dom_bridge_tests/part121_ip_c4_container_pad_pins.rs");

// ip C5 根层裸 br break-out 刀钉测（2026-11-15）：宿主唯一子 div 选区删净面
// div 解包（`<div>[abc]</div>` exp `<br><div><br></div>`）+ 标签 p/宿主多块
// 双控制键现役绿形恒一。
include!("js_dom_bridge_tests/part122_ip_c5_root_bare_br_pins.rs");

// ip I5 pre 换行归一刀钉测（2026-11-16）：pre 劈裂空右块垫 br 臂认左块尾字面
// \n 持行已足（免视守卫 white-space 感知）→ 单 br 垫新空行免三行形 + 裸尾
// 双 br/nxt 存在四控制键现役绿形恒一。
include!("js_dom_bridge_tests/part123_ip_i5_pre_lf_pins.rs");
include!("js_dom_bridge_tests/part124_ihtml_block_pins.rs");
include!("js_dom_bridge_tests/part125_ihtml_residual_pins.rs");
// ilb C+F 外迁族刀钉测（2026-11-17）：dt/dd/pre 元素 caret 行断 br 落容器
// 内不外迁＋li 现役绿形控制键。
include!("js_dom_bridge_tests/part126_ilb_dt_dd_pre_pins.rs");
// ilb A 壳尾计数族刀钉测（2026-11-17）：壳尾可见性补断 v2 落位＋现役绿形
// 控制键。
include!("js_dom_bridge_tests/part127_ilb_shell_tail_vispad_pins.rs");
// ilb H+J 归一域刀钉测（2026-11-17）：行断面空白归一三则落位＋退化头/
// nbsp-first 控制键。
include!("js_dom_bridge_tests/part128_ilb_wscanon_pins.rs");
// ilb I flex/grid br 元素面刀钉测（2026-11-17）：声明 pre 系坍缩行断 <br>
// 占位落位＋非坍缩行断控制键。
include!("js_dom_bridge_tests/part129_ilb_flexbr_pins.rs");
// ilb B+G+E 选区删除域刀钉测（2026-11-17）：同节点元素容器下钻＋后删元素
// caret 入壳＋块界补断细化＋rawtext no-op 守卫＋现役绿形控制键。
include!("js_dom_bridge_tests/part130_ilb_seldel_pins.rs");
include!("js_dom_bridge_tests/part131_ilb_seltbl_pins.rs");
include!("js_dom_bridge_tests/part132_img_breakout_pins.rs");
include!("js_dom_bridge_tests/part133_evt_reject_pins.rs");
include!("js_dom_bridge_tests/part134_r13_cefalse_wrap_pins.rs");
include!("js_dom_bridge_tests/part129_kd_rule_inset_inline_pins.rs");
include!("js_dom_bridge_tests/part135_fs_br_pins.rs");
// indent br 边界刀钉测（2026-11-17）：块边界相邻唯一行界 br 消费（尾侧摘除
// 删／首侧直形·行内壳尾链删，run=1 判据）＋run4/caret 前 br 三守卫现绿恒一。
include!("js_dom_bridge_tests/part136_indent_br_pins.rs");
// indent 邻居合并去重刀钉测（2026-11-18 A 族刀）：ViaHtml 邻居复用臂
// prefix/suffix 复本缺陷修复＋post-rebuild 次邻合并臂（三形）。
include!("js_dom_bridge_tests/part137_indent_merge_dedup_pins.rs");
include!("js_dom_bridge_tests/part138_indent_container_hoist_pins.rs");
include!("js_dom_bridge_tests/part139_indent_ws_merge_pins.rs");
include!("js_dom_bridge_tests/part140_indent_ol_tail_pins.rs");
include!("js_dom_bridge_tests/part141_indent_table_tbody_pins.rs");
// indent F 族 li 尾 br 尾位持行刀钉测（2026-11-18）：extraneous 判定尾位
// 豁免后 li 内三形选区整 li 包裹双 br 全留＋无尾 br 基线控制键。
include!("js_dom_bridge_tests/part142_indent_li_tail_br_pins.rs");
// ip I4 族 CE-island host 本体 li/dd 拆分钉测（2026-11-18）：div 双块内嵌
// 目标键×2＋h1/dt/span 行断退化＋ul 宿主兄弟拆分＋dps 未设 'p' 面控制键。
include!("js_dom_bridge_tests/part143_ip_island_li_dd_pins.rs");
// exec2/exec3 刀钉测（2026-11-19）：unknown-command 门表外 ret=false 零事件
// 零 DOM＋递归重入门嵌套拒绝＋深度清零＋bold 已知命令控制键。
include!("js_dom_bridge_tests/part144_exec_gate_pins.rs");

// 刀 ATP 恢复道钉测（2026-11-19）：CSS 源 @property 注册喂入（R2
// note_stylesheets）＋同名重声明后者胜＋无效规则不注册＋string 值 canonical
// 双引号重串化（R1 strict 门）＋universal 无 initial 兜底。
include!("js_dom_bridge_tests/part145_atp_recovery_pins.rs");

// MATHFN 刀钉测：sign/abs 端到端链（IDL 写面 + gCS 读，WPT _test_math 同款
// 通道）。
include!("js_dom_bridge_tests/mathfn_pins.rs");

// G1b 刀钉测（2026-11-21）：handle 通道 gCS width/height used-value echo——
// auto 高度动态元素 used px 回填＋显式 px 零漂移＋display:none 门臂＋drain 后
// 跨批 handle 回落（CSSOM-1 §resolved value，selector 通道 S1 刀的 handle 臂镜像）。
include!("js_dom_bridge_tests/part146_g1b_handle_used_echo.rs");

// s10-registries 刀② adopt 重钉钉测（2026-11-24）：frag 展平直子补章（跨文档
// appendChild(frag) 域限定）＋SR 经 host.shadowRoot 入口续走 global-own 重钉＋
// scoped-own 持久＋null-own 保持＋同文档搬移零移五面。
include!("js_dom_bridge_tests/part147_s10_adopt_restamp_pins.rs");

// s10-registries 刀②续 DSR contents 常驻钉测（2026-11-24）：主文档模板 contents
// 的声明式 shadow root 视图三面（doc-parse 物化 / cloneNode 转登 / 入文档语义）。
include!("js_dom_bridge_tests/part148_s10k2_dsr_contents_pins.rs");

// s10-registries 刀③ 常驻钉测（2026-11-24）：no-src iframe R365 槽接线 / DSR-attr
// transfer 印记 / 跨文档 adopt 矩阵 / 主文档 hydrate r 键四面。
include!("js_dom_bridge_tests/part149_s10k3_iframe_reg_pins.rs");
include!("js_dom_bridge_tests/part150_s10k4_dsr_mirror_parent_pins.rs");
// s10-registries 刀⑤ 常驻钉测（2026-11-24）：setHTMLUnsafe DSR 直通面 / template
// contents 无 BC 文档域 null registry 克隆链两面。
include!("js_dom_bridge_tests/part151_s10k5_unsafe_dsr_nobc_pins.rs");
// s10-registries 刀⑥ 常驻钉测（2026-11-24）：define() 参数校验矩阵 / IsConstructor
// 序门 / prototype-lifecycle 校验 / cbi extends 排除集 / 正向升级面。
include!("js_dom_bridge_tests/part152_s10k6_define_validation_pins.rs");
// s10-registries 刀⑦ 常驻钉测（2026-11-24）：define() spec 步序重构＋Get/转换
// rethrow 矩阵＋per-registry running flag＋whenDefined promise 恒等＋shadow-including
// 升级序。
include!("js_dom_bridge_tests/part153_s10k7_registries_knife7_pins.rs");
// s10-registries 刀⑧ 常驻钉测（2026-11-24）：registry 创建时点 own-stamp（createElement
// 文档域印＋移动不重钉＋initialize 子树盖印＋DSR clonable 根克隆传播）。
include!("js_dom_bridge_tests/part154_s10k8_registries_knife8_pins.rs");
// s10-domparsing 刀⑨ 常驻钉测（2026-11-24）：IAH 兄弟链/校验/NMA＋ccf isEqualNode/
// appendChild 展平执行＋html context 剥壳（壳不落文档）。
include!("js_dom_bridge_tests/part155_s10k9_domparsing_knife9_pins.rs");

// s10-sanitizer 刀⑩ 常驻钉测（2026-11-24）：default policy createParserOptions 链/
// 返回值校验/回调副本隔离/TrustedParserOptions sanitizer 方法形/ccf runScripts 章。
include!("js_dom_bridge_tests/part156_s10k10_parseroptions_pins.rs");
include!("js_dom_bridge_tests/part157_s10k11_config_modifiers_pins.rs");
// s10-sanitizer 刀⑫ 常驻钉测（2026-11-25）：setHTML 写链视图恒重建（'<' 捷径
// 清空）/comments 双路径/dataAttributes:false 显式名单放行/detached doc write
// 物化快照遍历。
include!("js_dom_bridge_tests/part158_s10k12_writechain_pins.rs");

// s10-sanitizer 刀⑬ 常驻钉测（2026-11-25）：foreign ns 端到端保真（元素/属性
// 三元组、xmlns:foo null-ns 形、isEqualNode 配对、消毒链保真）。
include!("js_dom_bridge_tests/part159_s10k13_foreignns_pins.rs");
include!("js_dom_bridge_tests/part160_s10k14_parsehtml_skeleton_pins.rs");
include!("js_dom_bridge_tests/part161_s10k15_variadic_mutation_pins.rs");
include!("js_dom_bridge_tests/part162_s10k16_activation_gecn_pins.rs");
// s10-k17 刀⑰ 常驻钉测（2026-11-25）：激活链 MEl 区域查询并查（R517 GECN/qsa
// handle 域静态结果并入 fusion 子树 MEl 区域匹配集；R518 A/AREA href getter
// 解析绝对 URL）＋corpus 同构 stamp/filter/href/click 相位。
include!("js_dom_bridge_tests/part163_s10k17_mel_region_query_pins.rs");
// s10-k19 刀⑲ 常驻钉测（2026-11-25）：TT script 身份门（R521——声明 sink
// accessor 快照盖章 + prepare 时身份救援 + Node/HTMLElement 通用 descriptor
// bypass 判据面）＋corpus script-enforcement-001/002 同构 trust/drop/rescue 相位。
include!("js_dom_bridge_tests/part164_s10k19_tt_script_provenance_pins.rs");
include!("js_dom_bridge_tests/part165_s10k20_tt_parser_svg_pins.rs");
include!("js_dom_bridge_tests/part166_s10k20b_tt_csp_latch_pins.rs");
include!("js_dom_bridge_tests/part167_s10k21_tt_ctor_pins.rs");
// wptfyi-css k114 刀 常驻钉测（2026-11-25）：@function 定义文法 CSSOM 双通道
//（构造表 replaceSync 空体保留门 + insertRule 写面 SyntaxError——css-mixins-1
// §2.1.1 prelude 严格文法 host 单源回调）。
include!("js_dom_bridge_tests/part167_k114_fn_prelude_cssom_pins.rs");
// wptfyi-css k115 刀 常驻钉测（2026-11-25）：@function CSSOM 结构化面
//（CSSFunctionRule/CSSFunctionDeclarations/CSSFunctionDescriptors 接口面 +
// canonical cssText 序列化域——css-mixins-1 §3；回调缺席 type-1 回落臂）。
include!("js_dom_bridge_tests/part168_k115_fn_cssom_pins.rs");
// wptfyi-css k117 刀 常驻钉测（2026-11-25）：@mixin CSSOM 结构化面
//（CSSMixinRule/CSSResultRule 接口面 + canonical cssText 序列化 +
// parsing test_child/@layer 丢弃深形 + wire 通道——css-mixins-1 §cssom）。
include!("js_dom_bridge_tests/part169_k117_mixin_cssom_pins.rs");
// wptfyi-css k131 刀 常驻钉测（2026-11-26）：shadow defs——gCS 判读通道 shadow
// scopes 全集（外链条目 + 内联 `<style>`）＋shadow 表携文档级 `@mixin` 聚合
// parse（zwsh 寻址面级联缺位修复——css-mixins-1 × css-scoping）。
include!("js_dom_bridge_tests/kc5r131_shadow_defs_pins.rs");
// wptfyi-css k133 刀 常驻钉测（2026-11-26）：part-match——`::part(<ident-list>)`
// 解析＋匹配＋级联面端到端＋gCS 伪元素存储槽投影（css-shadow-parts §3.2.2）。
include!("js_dom_bridge_tests/kc5r133_part_match_pins.rs");
// wptfyi-css k118 刀 常驻钉测（2026-11-25）：`<dashed-function>` 调用实参
// 文法写面（元素 style/cssText/样式表三通道拒收矩阵 + custom property 豁免
// + 零漂移对照——css-mixins-1 §function-rule + csswg-drafts#11749）。
include!("js_dom_bridge_tests/part170_k118_dashed_fn_pins.rs");
// s10k22 刀㉒ 常驻钉测（2026-11-25）：TT secondary-document MEl plain 世界
//（createHTMLDocument 产物——setAttribute(NS) ns 印记 + node 绑定族 IDL
// String 化＋绑定域复检 + innerHTML/srcdoc/text 三名/setHTMLUnsafe/IAH/
// execCommand 双向 + AccBypass 委托让位防双门）＋无 meta 负面页。
include!("js_dom_bridge_tests/part171_s10k22_secondary_doc_pins.rs");
// s10k23 刀㉓ 常驻钉测（2026-11-25）：TT fromLiteral 静态方法（三数据接口——
// IsTemplateStringObject 近似 throws 保信 + TrustedHTML parse-then-serialize
// 归一化域 + bound method + 品牌 sink 保真）＋无 meta 负面页。
include!("js_dom_bridge_tests/part172_s10k23_from_literal_pins.rs");
// s10k24 刀㉔ 常驻钉测（2026-11-25）：Document.write 写流 body 空清重置
//（Base/buffer 重放复活钉面 + 多参拼接 + 非 body 清空不重置对照）。
include!("js_dom_bridge_tests/part173_s10k24_write_clear_reset_pins.rs");

// s10k25 刀㉕ 常驻钉测（2026-11-26）：DOMParser 产物文档 write/writeln
//（方法面 + TT 管道逐参拼接 + 隐式 open 替换 + 非 enforced 直入对照）。
include!("js_dom_bridge_tests/part174_s10k25_parsed_doc_write_pins.rs");

// s10k27 刀㉖ 常驻钉测（2026-11-26）：Report-Only CSP monitoring——RO sink 面
//（透传/变形/违规字段）、enforced 面（TypeError/违规/抛错原样）、直调包装
// 语义、WebSocket connect-src 先于宿主缺失臂。
include!("js_dom_bridge_tests/part175_s10k27_ro_sink_pins.rs");

// s10k28 刀㉗ 常驻钉测（2026-11-26）：TT 违规 sample 形——sink 面截断
//（40 单元＋高代理延至 41）与 createPolicy 名单 block 违规（directive
// 'trusted-types'＋originalPolicy=名单原文＋sample 名截断无前缀）。
include!("js_dom_bridge_tests/part176_s10k28_sample_clip_pins.rs");
include!("js_dom_bridge_tests/part177_s10k29_eval_gate_pins.rs");

// wptfyi-css k134 刀 常驻钉测（2026-11-26）：gCS 第二参 handle 通道透传——
// 动态元素伪元素读形投影（k133 selector 通道的 handle 臂镜像）＋静态 part
// 五槽 selector 通道 shim 级端到端＋槽 miss 宽容面（CSSOM-1）。
include!("js_dom_bridge_tests/kc5r134_pseudo_handle_pins.rs");

// wptfyi-css k135 刀 常驻钉测（2026-11-26）：innerHTML 物化 shadow 子元素
// gCS 读路径（单跳 shadow 地址随读上传＋视图槽解析）＋存储槽 background
// 判据扩面 shim 级端到端＋无地址零漂移对照（CSSOM-1＋css-shadow-parts）。
include!("js_dom_bridge_tests/kc5r135_slot_gate_pins.rs");

// wptfyi-css k136 刀 常驻钉测（2026-11-26）：异种伪元素槽表 gCS 读面＋
// first-line/first-letter part 通道读形＋highlight 实参归一（CSSOM-1＋
// css-pseudo-4）。
include!("js_dom_bridge_tests/kc5r136_other_slots_pins.rs");

// wptfyi-css k149 刀 常驻钉测（2026-11-28）：@mixin body CSSOM decl 写父链
// 重算（嵌套/合成 child 写后 @result/@mixin cssText 逐层重组，物化↔重组恒等
// ＋params 剥/保形头恒等）。
include!("js_dom_bridge_tests/part178_k149_mixin_invalidation_pins.rs");

// wptfyi-css k150 刀 常驻钉测（2026-11-28）：CSSStyleRule.insertRule 裸声明
// 追加（@apply at-decl 臂＋hasApply 分流重建＋错误臂＋index 头插＋at 条目
// rebuild 保形）。
include!("js_dom_bridge_tests/part179_k150_style_rule_insert_rule_pins.rs");
