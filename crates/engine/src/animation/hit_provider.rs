//! M2-S3c 动画 transform 命中修正提供者槽（goal compositor-animation-continuity，RFC
//! `docs/goal/compositor-animation-continuity/m2-slice-state-input-consistency.md` §2.3）：
//! flag 开启且 transform 动画经 compositor 采样时，命中测试对已导出元素以「声明仿射
//! 的逆映射查询点」测基线布局矩形——与合成像素同一几何（compositor 像素 = 基线经
//! 全矩阵变换，S1c R3833 口径）。
//!
//! engine 不引 zero-protocol 依赖——提供者由 renderer 以 `Arc<dyn Fn>` 注册（输入
//! element_key，即导出声明同源 [`crate::hit_test::node_id_to_u64`] 编码；输出**含
//! transform-origin 折叠的**全仿射 `[a, b, c, d, tx, ty]` 列向量约定，像素实际经历的
//! 几何映射 T(o)·A·T(−o)，与 compositor 改写矩阵同式）。未注册（flag 关）或未命中
//! 返回 `None`，命中路径行为与现状逐字节一致（S2 FR-9 同款回滚纪律）。

use std::sync::{Arc, LazyLock, RwLock};

/// 命中修正提供者：element_key → 当刻全仿射（`None` = 未导出 / 采样无覆盖——延迟期
/// 无 backwards fill / 已结束无 fill，与 compositor「无覆盖 = 基线像素」镜像）。
type AnimationTransformFn = dyn Fn(u64) -> Option<[f64; 6]> + Send + Sync;

/// 槽内提供者的句柄类型（命中遍历构建时 Arc 克隆一次，逐节点只查表不解锁）。
pub(crate) type AnimationTransformProvider = Arc<AnimationTransformFn>;

static ANIMATION_TRANSFORM: LazyLock<RwLock<Option<AnimationTransformProvider>>> = LazyLock::new(|| RwLock::new(None));

/// renderer 侧装填/撤除命中修正提供者（`None` = 撤除：退出导出域、导航换代清表、
/// flag 关初始化）。进程级槽——renderer 线程写、命中测试线程读（点击主线程 +
/// elementFromPoint js worker 线程），与 gCS 查询覆盖槽同型跨线程可见模式。
pub fn set_animation_transform_provider(provider: Option<Arc<AnimationTransformFn>>) {
    if let Ok(mut slot) = ANIMATION_TRANSFORM.write() {
        *slot = provider;
    }
}

/// 命中遍历构建时读取当前提供者（`None` = 未注册，基线命中零干扰）。
pub(crate) fn animation_transform_provider() -> Option<AnimationTransformProvider> {
    ANIMATION_TRANSFORM.read().ok().and_then(|slot| slot.clone())
}

/// 仿射求逆（列向量约定 `[a, b, c, d, tx, ty]`：x' = a·x + c·y + tx）。行列式
/// 绝对值低于阈值视为奇异（`None`）——调用方跳过 provider 回落基线命中。
pub(crate) fn invert_affine(m: [f64; 6]) -> Option<[f64; 6]> {
    let [a, b, c, d, tx, ty] = m;
    let det = a * d - b * c;
    if det.abs() < 1e-12 {
        return None;
    }
    Some([
        d / det,
        -b / det,
        -c / det,
        a / det,
        (c * ty - d * tx) / det,
        (b * tx - a * ty) / det,
    ])
}

/// 仿射应用点变换（列向量约定，与 [`invert_affine`] 同序）。
pub(crate) fn apply_affine(m: [f64; 6], x: f64, y: f64) -> (f64, f64) {
    let [a, b, c, d, tx, ty] = m;
    (a * x + c * y + tx, b * x + d * y + ty)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_points_close(p: (f64, f64), q: (f64, f64)) {
        assert!((p.0 - q.0).abs() < 1e-9 && (p.1 - q.1).abs() < 1e-9, "{p:?} != {q:?}");
    }

    /// 逆映射往返：任意非奇异仿射下 apply(inv(m), apply(m, p)) == p。
    #[test]
    fn invert_round_trips_through_apply() {
        // rotate(37deg) ∘ translate(120, -40) ∘ scale(2, 0.5)（复合列表序）。
        let (sin, cos) = 37f64.to_radians().sin_cos();
        let rotate = [cos, sin, -sin, cos, 0.0, 0.0];
        let translate = [1.0, 0.0, 0.0, 1.0, 120.0, -40.0];
        let scale = [2.0, 0.0, 0.0, 0.5, 0.0, 0.0];
        let m = multiply(rotate, multiply(translate, scale));
        let inv = invert_affine(m).expect("非奇异");
        for p in [(0.0, 0.0), (100.0, 50.0), (-30.0, 80.0)] {
            let mapped = apply_affine(m, p.0, p.1);
            assert_points_close(apply_affine(inv, mapped.0, mapped.1), p);
        }
    }

    /// 奇异矩阵（线性部分退化）返回 None。
    #[test]
    fn singular_matrix_returns_none() {
        assert!(invert_affine([0.0, 0.0, 0.0, 0.0, 5.0, 5.0]).is_none());
        // scale(1, 0) 单轴压扁同样奇异。
        assert!(invert_affine([1.0, 0.0, 0.0, 0.0, 0.0, 0.0]).is_none());
    }

    /// origin 折叠全矩阵（T(o)·A·T(−o)，compositor 同式）的逆映射把「变换后视觉点」
    /// 送回基线坐标——绕中心 rotate(90deg)：基线盒左缘中点 (100, 150) 视觉上落在
    /// 盒上缘中点 (150, 100)，逆映射应把它送回 (100, 150)。
    #[test]
    fn origin_folded_rotation_inverse_maps_visual_point_to_baseline() {
        let (ox, oy) = (150.0_f64, 150.0_f64);
        let affine = [0.0, 1.0, -1.0, 0.0, 0.0, 0.0]; // rotate(90deg)
        // 全矩阵 = T(o) · A · T(−o)（multiply 先应用右参数，列表序组合）。
        let full = multiply(
            [1.0, 0.0, 0.0, 1.0, ox, oy],
            multiply(affine, [1.0, 0.0, 0.0, 1.0, -ox, -oy]),
        );
        let inv = invert_affine(full).expect("旋转非奇异");
        assert_points_close(apply_affine(inv, 150.0, 100.0), (100.0, 150.0));
    }

    /// 测试用仿射相乘（`a · b`，先应用 b 再应用 a；与 protocol `multiply_affine`
    /// 同式——engine 不引 protocol，测试内联同序实现）。
    fn multiply(a: [f64; 6], b: [f64; 6]) -> [f64; 6] {
        [
            a[0] * b[0] + a[2] * b[1],
            a[1] * b[0] + a[3] * b[1],
            a[0] * b[2] + a[2] * b[3],
            a[1] * b[2] + a[3] * b[3],
            a[0] * b[4] + a[2] * b[5] + a[4],
            a[1] * b[4] + a[3] * b[5] + a[5],
        ]
    }
}
