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
