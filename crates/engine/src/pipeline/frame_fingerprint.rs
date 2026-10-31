//! S1 文档级 styled 帧输入指纹缓存（G1 失效模型首切片，设计卡 §2.0/§2.1/§4-S1）。
//!
//! 键范式沿用 T13 CascadeCache/T17 font_ids 系谱：内容寻址键完整刻画纯函数输入 +
//! 容量上限清空 + 独立 kill-switch。键为聚合指纹（FNV-1a 128 位双种子，哈希即键、
//! 无桶内完整比对——输入为整页 HTML/CSS 与环境快照，无法逐字节比对，碰撞概率
//! 2^-128 量级，判定为可接受）；与 CascadeCache 的「FNV 定桶 + 完整比对」不同，
//! 此处哈希碰撞即错误命中，故用 128 位宽度。
//!
//! OPTIMIZATION：热导航 = 同 URL 二次导航 = 全新 [`BudgetedRenderSession`]（新
//! NodeId 代际），NodeId 级缓存跨代不可用；同输入时 styled 帧重算（style+layout+
//! paint，cnblogs 实测 ~230-250ms）产出逐位相同的帧——指纹命中后整帧复用，四段
//! 全免。值跨代可复用的前提：ComputedStyle 零 NodeId 内嵌（PartialEq 契约）、
//! LayoutResult.root 已 Arc 共享。
//!
//! URL/域名不入键（内容寻址边界）：css 束在拼接期已按文档 base 绝对化
//!（`absolutize_css_urls`），base 差异被 css 内容哈希吸收；paint 侧 document_url
//! 的消费点是 `image_resource_key(u, document_url)` 族（img 元素/border-image/
//! effects/text-list/bg-clip-text 多处），`u` 来自已绝对化的束，且 image 四表键
//! 含 document_url（`build_img_intrinsic_all`）——跨 URL 同内容命中在新图片到达
//! 后键变 MISS 自愈，残余仅为到达前图元引用旧站站位图的瞬态。命中臂以
//! `Document::pristine_snapshot` 还原文档并重放 `set_url`/`set_referrer`（Parse
//! 步同值），还原文档不携带旧导航 URL/referrer。
//! canvas registry 为跨文档共享可变句柄且无版本号，位图变化不可完整刻画——按
//! **本文档代内活跃**判禁用：registry 长度对比导航起点基线
//!（`set_canvas_ctx_baseline`，webview 导航换代时下推；T13 S5 同址重入不清
//! registry，遗留死文档 context 不算活跃），代内有 getContext 即整体禁用短路，
//! 保守回退全量（diff 不确定一律回退，设计卡 §2.0）。

use std::collections::{HashMap, VecDeque};

use zero_css_parser::Stylesheet;
use zero_dom::{Document, NodeId};
use zero_style_system::ComputedStyle;

use super::RenderResult;
use crate::animation::compositor_export::ExportedAnimationSet;

/// 缓存条目：一次 styled 帧计算的完整输出（文档 + 样式 + 渲染结果 + 动画面）。
///
/// `stylesheets`/`export_set` 随帧保存：命中臂须复刻全量 Complete 的动画时钟注册
/// 与合成导出缓存副作用（全量在 Style/Paint 步内联完成，见 `advance_budgeted_render`）。
pub(crate) struct CachedFrame {
    /// 命中臂写回 `RenderPipeline::cached_doc` 的文档**纯净树快照**（自有深拷，
    /// 见 `Document::pristine_snapshot`）。DF-1 根修：不得以 Rc 与管线活文档共享
    /// ——回填后脚本突变路径原位修改 `cached_doc`，共享会把脚本效应写进缓存条
    /// 目，热航同键命中还原含突变树 + 脚本重执行 = 非幂等效应翻倍；命中臂再拷
    /// 一次入管线，缓存条目与活文档零共享，值恒为键输入的纯函数输出。
    pub(crate) doc: Document,
    /// 命中臂写回 `RenderPipeline::cached_styles` 的计算样式。
    pub(crate) styles: HashMap<NodeId, ComputedStyle>,
    /// 命中臂补跑 `AnimationClock::register_from_stylesheets` 所需的样式表集合
    ///（`collect_stylesheets` 纯函数，同键输入必同输出）。
    pub(crate) stylesheets: Vec<Stylesheet>,
    /// 合成动画导出集（compositor flag 开时非 None；与全量 export_set 同源）。
    pub(crate) export_set: Option<ExportedAnimationSet>,
    /// 整帧渲染结果（display list + layout + stats + canvas/动画产物）。
    pub(crate) result: RenderResult,
}

/// 指纹缓存：文档级键 → 完整帧输出。容量上限 + **LRU-by-lookup** 驱逐（命中
/// 提升至驱逐序尾；同键覆盖不驱逐、不计新插入）。
///
/// 容量 4：一轮导航典型产生 2-4 个 budget session（FirstPaint 轮、styled 轮、
/// 图片批轮、字体重绘轮），容量覆盖单轮导航全部帧形态。驱逐序语义对「热导航
/// 首轮（FirstPaint/styled，环境初态键）必须仍驻留」的目标等价或优于纯 FIFO：
/// 初态键正是每轮热导航的命中目标，命中即提升保活，不会被同页突发异键轮挤出；
/// 未被命中的条目按最久未活跃逐出，顺序与 FIFO 相同（最旧先出）。
pub(crate) struct FrameFingerprintCache {
    entries: HashMap<u128, CachedFrame>,
    /// 驱逐序（LRU-by-lookup：尾 = 最近插入或最近命中，首 = 最久未活跃）。
    order: VecDeque<u128>,
    capacity: usize,
}

impl FrameFingerprintCache {
    pub(crate) fn new(capacity: usize) -> Self {
        Self {
            entries: HashMap::new(),
            order: VecDeque::new(),
            capacity: capacity.max(1),
        }
    }

    /// 查键并将命中条目移至驱逐序尾部（活跃条目保活）。
    pub(crate) fn lookup(&mut self, key: u128) -> Option<&CachedFrame> {
        if self.entries.contains_key(&key)
            && let Some(pos) = self.order.iter().position(|k| *k == key)
        {
            let k = self.order.remove(pos).expect("pos checked");
            self.order.push_back(k);
        }
        self.entries.get(&key)
    }

    /// 插入/覆盖条目；超容量按驱逐序逐出序首（最久未活跃）异键条目。
    pub(crate) fn insert(&mut self, key: u128, frame: CachedFrame) {
        if !self.entries.contains_key(&key) {
            while self.entries.len() >= self.capacity {
                if let Some(oldest) = self.order.pop_front() {
                    self.entries.remove(&oldest);
                } else {
                    break;
                }
            }
            self.order.push_back(key);
        }
        self.entries.insert(key, frame);
    }

    /// 条目数（测试）。
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }
}

/// kill-switch：`ZW_STYLE_FINGERPRINT_CACHE`（缺省开，仅 `0` 关——与
/// `ZW_CASCADE_CACHE` 判定语义一致）。关 = 不查不写，逐字节回旧全量路径。
pub(crate) fn fingerprint_cache_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_STYLE_FINGERPRINT_CACHE").as_deref() != Ok("0"))
}

/// FNV-1a 128 位（双种子并行两路 64 位，见 §模块注记碰撞论证）。
pub(crate) struct Fnv128 {
    hi: u64,
    lo: u64,
}

impl Fnv128 {
    pub(crate) fn new() -> Self {
        Self {
            hi: 0xcbf29ce484222325,
            lo: 0x9e3779b97f4a7c15,
        }
    }

    pub(crate) fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.hi ^= u64::from(*b);
            self.hi = self.hi.wrapping_mul(0x100000001b3);
            self.lo ^= u64::from(*b);
            self.lo = self.lo.wrapping_mul(0x100000001b3).rotate_left(27);
        }
    }

    pub(crate) fn finish(self) -> u128 {
        (u128::from(self.hi) << 64) | u128::from(self.lo)
    }
}

/// 对整型写入字节序稳定的哈希输入（跨进程/平台一致，便于测量配对）。
macro_rules! hash_int {
    ($h:expr, $v:expr) => {
        $h.write(&$v.to_le_bytes())
    };
}

/// 对 `HashMap` 键排序后哈希（HashMap 迭代序不定——text.rs:1526 先例，排序消序）。
fn hash_sorted_u64_map<V>(h: &mut Fnv128, map: &HashMap<u64, V>, value_bytes: impl Fn(&V) -> Vec<u8>) {
    let mut keys: Vec<u64> = map.keys().copied().collect();
    keys.sort_unstable();
    hash_int!(h, keys.len() as u64);
    for k in keys {
        hash_int!(h, k);
        h.write(&value_bytes(&map[&k]));
    }
}

/// 计算当前管线环境 + 会话输入的文档级指纹键。
///
/// 返回 `None` = 禁用短路（kill-switch 关、canvas 活跃页），调用方走全量路径。
pub(crate) fn frame_fingerprint_key(pipeline: &super::RenderPipeline, html: &str, css: &str) -> Option<u128> {
    if !fingerprint_cache_enabled() {
        return None;
    }
    // canvas 本文档代内活跃保守回退：registry 位图无版本号，键不可完整刻画
    //（§模块注记）。基线 = 导航起点 registry 长度（webview 换代时下推）；当前
    // 长度偏离基线即本文档发生过 getContext——canvas 像素可被脚本随时改写，帧
    // 不再是 (html, css, 环境) 的纯函数，整体禁用。锁失败同样保守回退。
    if let Some(registry) = &pipeline.canvas_registry {
        let canvas_active = match registry.lock() {
            Ok(reg) => reg.contexts.len() != pipeline.canvas_ctx_baseline,
            Err(_) => true,
        };
        if canvas_active {
            return None;
        }
    }
    let mut h = Fnv128::new();
    // 域分隔前缀：键空间与探针/未来其他指纹键隔离。
    h.write(b"zw-s1-frame-v1\x00");
    h.write(html.as_bytes());
    h.write(&[0]);
    h.write(css.as_bytes());
    h.write(&[0]);
    hash_int!(h, pipeline.viewport_width.to_bits());
    hash_int!(h, pipeline.viewport_height.to_bits());
    // 样式上下文面：版本号 + 枚举值（版本防 getter 遗漏、枚举值防跨实例版本不可比）。
    hash_int!(h, pipeline.style_system.context_version());
    h.write(format!("{:?}\x00", pipeline.style_system.prefers_color_scheme()).as_bytes());
    h.write(format!("{:?}\x00", pipeline.style_system.media_type()).as_bytes());
    h.write(format!("{:?}\x00", pipeline.print_page_setup).as_bytes());
    hash_int!(h, u64::from(pipeline.skip_indicators));
    hash_int!(h, u64::from(pipeline.compositor_animations));
    hash_sorted_u64_map(&mut h, &pipeline.image_sizes, |v| {
        let (w, r) = v;
        [w.to_bits().to_le_bytes(), r.to_bits().to_le_bytes()].concat()
    });
    hash_sorted_u64_map(&mut h, &pipeline.image_ratios, |v| v.to_bits().to_le_bytes().to_vec());
    hash_sorted_u64_map(&mut h, &pipeline.image_no_ratio, |v| {
        let (w, r) = v;
        // DF-5：Option 加判别前缀字节——裸 map_or(0, to_bits) 使 None 与
        // Some(0.0) 同哈希输入，「固有尺寸未知→解码出 0×0」状态迁移键不变，
        // 陈旧帧命中（两态布局路径不同：占位/宽高比回退 vs 零盒）。
        let mut bytes = Vec::with_capacity(10);
        match w {
            None => bytes.push(0u8),
            Some(w) => {
                bytes.push(1);
                bytes.extend_from_slice(&w.to_bits().to_le_bytes());
            }
        }
        match r {
            None => bytes.push(0u8),
            Some(r) => {
                bytes.push(1);
                bytes.extend_from_slice(&r.to_bits().to_le_bytes());
            }
        }
        bytes
    });
    hash_sorted_u64_map(&mut h, &pipeline.image_solid_colors, |v| v.to_vec());
    // 字体面：family → font_id 映射整体入键（字体晚到触发 request_rerender 重绘时
    // resolver 变化 → 键变 → MISS 全量，防止热导航用 fallback 字体缓存命中真字体帧）。
    {
        let mut families: Vec<&String> = pipeline.font_resolver.keys().collect();
        families.sort();
        hash_int!(h, families.len() as u64);
        for family in families {
            h.write(family.as_bytes());
            h.write(&[0]);
            hash_int!(h, pipeline.font_resolver[family]);
        }
    }
    Some(h.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_frame(tag: u8) -> CachedFrame {
        let html = format!("<html><body><p>frame-{tag}</p></body></html>");
        let doc = zero_dom::parse_html(&html);
        // 最小 RenderResult：空图元 + 计时 total_ms=tag 供覆盖断言。
        let result = crate::pipeline::make_render_result(
            zero_render_foundation::primitive::RenderPrimitives::default(),
            Vec::new(),
            zero_layout_engine::LayoutResult {
                root: std::sync::Arc::new(zero_layout_engine::LayoutBox::default()),
                viewport_width: 100.0,
                viewport_height: 100.0,
                paint_skip_node_ids: std::collections::HashSet::new(),
            },
            crate::pipeline::PipelineTimings {
                parse_ms: 0.0,
                style_ms: 0.0,
                layout_ms: 0.0,
                paint_ms: 0.0,
                commit_ms: 0.0,
                total_ms: f64::from(tag),
                parse_count: 0,
                style_count: 0,
                layout_count: 0,
                paint_count: 0,
            },
            zero_render_foundation::primitive::RenderStats::default(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        CachedFrame {
            doc,
            styles: HashMap::new(),
            stylesheets: Vec::new(),
            export_set: None,
            result,
        }
    }

    #[test]
    fn cache_capacity_evicts_oldest_and_keeps_reactivated() {
        let mut cache = FrameFingerprintCache::new(2);
        cache.insert(1, sample_frame(1));
        cache.insert(2, sample_frame(2));
        assert_eq!(cache.len(), 2);
        // 命中 1 → 移至驱逐序尾；插入 3 驱逐 2（最旧）。
        assert!(cache.lookup(1).is_some());
        cache.insert(3, sample_frame(3));
        assert!(cache.lookup(1).is_some(), "活跃条目不得被逐出");
        assert!(cache.lookup(2).is_none(), "最旧未活跃条目被 FIFO 驱逐");
        assert!(cache.lookup(3).is_some());
        assert_eq!(cache.len(), 2, "容量上限保持");
    }

    #[test]
    fn cache_same_key_overwrites_without_growth() {
        let mut cache = FrameFingerprintCache::new(2);
        cache.insert(1, sample_frame(1));
        cache.insert(1, sample_frame(2));
        assert_eq!(cache.len(), 1, "同键覆盖不增长不驱逐");
        assert_eq!(
            cache.lookup(1).expect("覆盖后可查").result.timings.total_ms,
            2.0,
            "覆盖后读到新值"
        );
    }

    #[test]
    fn fnv128_key_length_stable_and_input_sensitive() {
        let mut a = Fnv128::new();
        a.write(b"abc");
        let ka = a.finish();
        let mut b = Fnv128::new();
        b.write(b"abd");
        let kb = b.finish();
        assert_ne!(ka, kb, "一字符差必须判异");
        let mut c = Fnv128::new();
        c.write(b"abc");
        assert_eq!(ka, c.finish(), "同输入同键");
        // 域分隔：拼接边界注入 0 分隔，"ab"+"c" 与 "a"+"bc" 不同键。
        let mut d = Fnv128::new();
        d.write(b"ab");
        d.write(&[0]);
        d.write(b"c");
        let mut e = Fnv128::new();
        e.write(b"a");
        e.write(&[0]);
        e.write(b"bc");
        assert_ne!(d.finish(), e.finish(), "拼接歧义由分隔字节消除");
    }

    #[test]
    fn kill_switch_defaults_enabled_and_zero_disables() {
        // 缺省（env 未设）开；"0" 关；其余值开（与 ZW_CASCADE_CACHE 判定语义一致）。
        // env 只读进程级快照不可在单测内安全切换——钉语义恒真式 + 独立集成验证。
        assert!(fingerprint_cache_enabled() || !fingerprint_cache_enabled());
        let unset = std::env::var("ZW_STYLE_FINGERPRINT_CACHE").is_err();
        if unset {
            assert!(fingerprint_cache_enabled(), "env 未设时缺省开启");
        }
    }

    #[test]
    fn fingerprint_key_image_no_ratio_none_vs_zero_decode_differ() {
        // DF-5：「固有尺寸未知」（None）与「成功解码 0×0」（Some(0.0)）必须判异——
        // 裸 map_or(0, to_bits) 折叠使两态同哈希输入，「未知→零尺寸解码」状态迁移
        // 键不变，陈旧帧命中（布局路径不同：占位/宽高比回退 vs 零盒）。
        let mut unknown = super::super::RenderPipeline::new(100.0, 100.0);
        let mut table_unknown = std::collections::HashMap::new();
        table_unknown.insert(9u64, (None, None));
        unknown.set_image_no_ratio(table_unknown);
        let mut zero = super::super::RenderPipeline::new(100.0, 100.0);
        let mut table_zero = std::collections::HashMap::new();
        table_zero.insert(9u64, (Some(0.0f32), Some(0.0f32)));
        zero.set_image_no_ratio(table_zero);
        let k_unknown = frame_fingerprint_key(&unknown, "<img src=x>", "");
        let k_zero = frame_fingerprint_key(&zero, "<img src=x>", "");
        assert_ne!(
            k_unknown, k_zero,
            "None 与 Some(0.0) 必须判异（状态迁移不得命中陈旧帧）"
        );
    }

    #[test]
    fn fingerprint_key_excludes_document_url_includes_content_and_env() {
        // URL/域名不入键（正确性边界）：同内容不同 URL 必须同键。
        let mut a = super::super::RenderPipeline::new(100.0, 100.0);
        a.set_document_url(Some("https://a.example/page"));
        let mut b = super::super::RenderPipeline::new(100.0, 100.0);
        b.set_document_url(Some("https://b.example/other"));
        let ka = frame_fingerprint_key(&a, "<p>same</p>", "p{}");
        let kb = frame_fingerprint_key(&b, "<p>same</p>", "p{}");
        assert_eq!(ka, kb, "同内容不同 URL 必须同键（内容寻址）");
        // 同 URL 不同内容必须判异。
        let kc = frame_fingerprint_key(&a, "<p>other</p>", "p{}");
        assert_ne!(ka, kc, "不同内容同 URL 必须判异");
        // 内容同、css 异必须判异。
        let kd = frame_fingerprint_key(&a, "<p>same</p>", "p{color:red}");
        assert_ne!(ka, kd, "css 异必须判异");
        // 内容同、视口异必须判异。
        let mut c = super::super::RenderPipeline::new(200.0, 100.0);
        let ke = frame_fingerprint_key(&c, "<p>same</p>", "p{}");
        assert_ne!(ka, ke, "视口异必须判异");
        // 图片表状态异必须判异（热导航图片批重绘轮不被旧帧污染）。
        let mut d = super::super::RenderPipeline::new(100.0, 100.0);
        let mut sizes = std::collections::HashMap::new();
        sizes.insert(7u64, (10.0f32, 20.0f32));
        d.set_image_sizes(sizes);
        let kf = frame_fingerprint_key(&d, "<p>same</p>", "p{}");
        assert_ne!(ka, kf, "图片表异必须判异");
        // 字体面异必须判异（字体晚到重绘轮不被 fallback 帧命中）。
        let mut e = super::super::RenderPipeline::new(100.0, 100.0);
        let mut resolver = std::collections::HashMap::new();
        resolver.insert("Test".to_string(), 3u32);
        e.set_font_resolver(resolver);
        let kg = frame_fingerprint_key(&e, "<p>same</p>", "p{}");
        assert_ne!(ka, kg, "font resolver 异必须判异");
    }

    #[test]
    fn fingerprint_key_disabled_for_active_canvas() {
        // canvas 本文档代内活跃（registry 长度偏离导航起点基线）→ None 保守回退全量；
        // 基线对齐后遗留 context 不禁用（T13 S5 同址重入不清 registry，死文档遗留
        // context 不是本文档活跃——S1 命中不被前代遗留误禁）。
        let mut a = super::super::RenderPipeline::new(100.0, 100.0);
        let registry = std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
        a.set_canvas_registry(Some(registry));
        let k = frame_fingerprint_key(&a, "<p>x</p>", "p{}");
        assert!(k.is_some(), "空 canvas registry（无 context）不禁用");
        // 注册一个 context（构造先例：canvas.rs t13 测试同款）→ 偏离基线 0 → 禁用。
        if let Some(reg) = &a.canvas_registry {
            let mut reg = reg.lock().expect("lock");
            let id = reg.next_ctx_id;
            reg.contexts.insert(id, zero_canvas::CanvasContext::new(4, 4));
            reg.next_ctx_id += 1;
        }
        let k2 = frame_fingerprint_key(&a, "<p>x</p>", "p{}");
        assert!(k2.is_none(), "本文档发生 getContext 必须禁用短路");
        // 导航换代下推基线 = 当前长度（遗留 context 计入基线）→ 重新允许短路。
        let len = a
            .canvas_registry
            .as_ref()
            .map(|r| r.lock().expect("lock").contexts.len())
            .unwrap_or(0);
        a.set_canvas_ctx_baseline(len);
        let k3 = frame_fingerprint_key(&a, "<p>x</p>", "p{}");
        assert!(k3.is_some(), "基线对齐后遗留死文档 context 不得禁用短路");
        // 本文档再新增 context → 再次偏离基线 → 禁用。
        if let Some(reg) = &a.canvas_registry {
            let mut reg = reg.lock().expect("lock");
            let id = reg.next_ctx_id;
            reg.contexts.insert(id, zero_canvas::CanvasContext::new(4, 4));
            reg.next_ctx_id += 1;
        }
        let k4 = frame_fingerprint_key(&a, "<p>x</p>", "p{}");
        assert!(k4.is_none(), "基线后新增 context 必须禁用短路");
    }
}
