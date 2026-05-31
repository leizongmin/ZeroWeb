//! 颜色类型 — RGBA 颜色表示与操作

/// RGBA 颜色（每个通道 0-255）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    /// 红色通道
    pub r: u8,
    /// 绿色通道
    pub g: u8,
    /// 蓝色通道
    pub b: u8,
    /// 透明度通道
    pub a: u8,
}

impl Color {
    /// 透明色
    pub const TRANSPARENT: Self = Self { r: 0, g: 0, b: 0, a: 0 };
    /// 黑色
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    /// 白色
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    /// 红色
    pub const RED: Self = Self {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    /// 绿色
    pub const GREEN: Self = Self {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };
    /// 蓝色
    pub const BLUE: Self = Self {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };

    /// 创建 RGBA 颜色
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// 创建 RGB 颜色（不透明）
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// 转换为 f32 数组（每个通道 0.0-1.0）
    pub fn to_f32_array(&self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }

    /// 转换为线性颜色空间（近似 sRGB → linear）
    pub fn to_linear_f32(&self) -> [f32; 4] {
        let to_linear = |v: u8| {
            let s = v as f32 / 255.0;
            if s <= 0.04045 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        [
            to_linear(self.r),
            to_linear(self.g),
            to_linear(self.b),
            self.a as f32 / 255.0,
        ]
    }

    /// 从十六进制字符串解析颜色（支持 #RGB, #RRGGBB, #RRGGBBAA）
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix('#')?;
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                Some(Self::rgb(r, g, b))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::rgb(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::rgba(r, g, b, a))
            }
            _ => None,
        }
    }

    /// 预乘 alpha
    pub fn premultiplied(&self) -> [f32; 4] {
        let a = self.a as f32 / 255.0;
        [
            self.r as f32 / 255.0 * a,
            self.g as f32 / 255.0 * a,
            self.b as f32 / 255.0 * a,
            a,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_constants() {
        assert_eq!(Color::BLACK.r, 0);
        assert_eq!(Color::BLACK.a, 255);
        assert_eq!(Color::WHITE.r, 255);
        assert_eq!(Color::TRANSPARENT.a, 0);
    }

    #[test]
    fn test_color_to_f32() {
        let c = Color::WHITE;
        let f = c.to_f32_array();
        assert!((f[0] - 1.0).abs() < f32::EPSILON);
        assert!((f[3] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_color_from_hex() {
        assert_eq!(Color::from_hex("#fff"), Some(Color::WHITE));
        assert_eq!(Color::from_hex("#000000"), Some(Color::BLACK));
        assert_eq!(Color::from_hex("#ff0000"), Some(Color::RED));
        assert_eq!(Color::from_hex("#00ff0000"), Some(Color::rgba(0, 255, 0, 0)));
        assert_eq!(Color::from_hex("invalid"), None);
    }

    #[test]
    fn test_color_premultiplied() {
        let c = Color::rgba(255, 128, 0, 128);
        let p = c.premultiplied();
        assert!((p[0] - 0.5).abs() < 0.01); // 255/255 * 128/255 ≈ 0.5
        assert!((p[3] - 128.0 / 255.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_color_linear() {
        let c = Color::WHITE;
        let l = c.to_linear_f32();
        assert!((l[0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_color_from_hex_short() {
        // #RGB shorthand
        let c = Color::from_hex("#f00").unwrap();
        assert_eq!(c, Color::RED);
        let c2 = Color::from_hex("#0f0").unwrap();
        assert_eq!(c2, Color::GREEN);
        let c3 = Color::from_hex("#00f").unwrap();
        assert_eq!(c3, Color::BLUE);
    }

    #[test]
    fn test_color_from_hex_with_alpha() {
        // #RRGGBBAA
        let c = Color::from_hex("#ff000080").unwrap();
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
        assert_eq!(c.a, 128);
    }

    #[test]
    fn test_color_from_hex_invalid() {
        assert!(Color::from_hex("").is_none());
        assert!(Color::from_hex("#").is_none());
        assert!(Color::from_hex("#12").is_none());
        assert!(Color::from_hex("ffffff").is_none()); // missing #
        assert!(Color::from_hex("#gggggg").is_none()); // invalid hex digits
    }

    #[test]
    fn test_color_srgb_to_linear_black() {
        let c = Color::BLACK;
        let l = c.to_linear_f32();
        assert!(l[0].abs() < 0.001);
        assert!(l[1].abs() < 0.001);
        assert!(l[2].abs() < 0.001);
        assert!((l[3] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_color_srgb_to_linear_mid_gray() {
        // 128/255 ~ 0.502 in sRGB
        let c = Color::rgb(128, 128, 128);
        let l = c.to_linear_f32();
        // linear value should be ~0.216 (less than 0.5 due to gamma)
        assert!(l[0] > 0.2 && l[0] < 0.25);
        assert!((l[0] - l[1]).abs() < f32::EPSILON);
    }

    #[test]
    fn test_color_premultiplied_opaque() {
        let c = Color::rgb(200, 100, 50);
        let p = c.premultiplied();
        assert!((p[0] - 200.0 / 255.0).abs() < 0.001);
        assert!((p[1] - 100.0 / 255.0).abs() < 0.001);
        assert!((p[2] - 50.0 / 255.0).abs() < 0.001);
        assert!((p[3] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_color_premultiplied_transparent() {
        let c = Color::rgba(255, 255, 255, 0);
        let p = c.premultiplied();
        assert!(p[0].abs() < f32::EPSILON);
        assert!(p[1].abs() < f32::EPSILON);
        assert!(p[2].abs() < f32::EPSILON);
        assert!(p[3].abs() < f32::EPSILON);
    }

    #[test]
    fn test_color_equality_and_copy() {
        let c1 = Color::rgba(10, 20, 30, 40);
        let c2 = c1;
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_color_rgb_is_opaque() {
        let c = Color::rgb(100, 150, 200);
        assert_eq!(c.r, 100);
        assert_eq!(c.g, 150);
        assert_eq!(c.b, 200);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn test_color_rgba_custom_alpha() {
        let c = Color::rgba(255, 128, 64, 32);
        assert_eq!(c.a, 32);
    }

    #[test]
    fn test_color_transparent_is_fully_transparent() {
        let c = Color::TRANSPARENT;
        assert_eq!(c.a, 0);
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
    }

    #[test]
    fn test_color_to_f32_array_channels() {
        let c = Color::rgb(0, 128, 255);
        let f = c.to_f32_array();
        assert!(f[0].abs() < f32::EPSILON);
        assert!((f[1] - 128.0 / 255.0).abs() < f32::EPSILON);
        assert!((f[2] - 1.0).abs() < f32::EPSILON);
        assert!((f[3] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_color_to_linear_f32_dark_values() {
        // Values below threshold (<= 0.04045) use linear division
        let c = Color::rgb(10, 10, 10);
        let l = c.to_linear_f32();
        let expected = 10.0 / 255.0 / 12.92;
        assert!((l[0] - expected).abs() < 0.0001);
    }

    #[test]
    fn test_color_from_hex_long_with_full_alpha() {
        let c = Color::from_hex("#ffffffff").unwrap();
        assert_eq!(c, Color::WHITE);
    }

    #[test]
    fn test_color_hash_consistency() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Color::RED);
        set.insert(Color::RED);
        set.insert(Color::BLUE);
        assert_eq!(set.len(), 2);
    }

    /// 测试 sRGB→linear 转换在阈值边界（v=10, s≈0.0392）使用线性分支
    #[test]
    fn test_color_to_linear_f32_threshold_boundary() {
        // s = 10/255 ≈ 0.03922 ≤ 0.04045 → 使用 s/12.92 分支
        let c_low = Color::rgb(10, 10, 10);
        let l_low = c_low.to_linear_f32();
        let expected_low = 10.0_f32 / 255.0 / 12.92;
        assert!((l_low[0] - expected_low).abs() < 0.0001);

        // s = 11/255 ≈ 0.04314 > 0.04045 → 使用幂函数分支
        let c_high = Color::rgb(11, 11, 11);
        let l_high = c_high.to_linear_f32();
        let s_high = 11.0_f32 / 255.0;
        let expected_high = ((s_high + 0.055) / 1.055).powf(2.4);
        assert!((l_high[0] - expected_high).abs() < 0.0001);

        // 确保两个分支的结果不同
        assert!((l_low[0] - l_high[0]).abs() > 0.00001);
    }

    /// 测试 alpha=0 时 linear 转换：RGB 通道应正常转换，alpha 通道为 0.0
    #[test]
    fn test_color_to_linear_alpha_zero_with_nonzero_rgb() {
        // alpha=0 但 RGB 非零 — linear 转换中 alpha 独立于 RGB
        let c = Color::rgba(255, 128, 64, 0);
        let l = c.to_linear_f32();
        // RGB 通道应正常转换
        assert!((l[0] - 1.0).abs() < 0.01, "R 通道应正常转换");
        assert!(l[1] > 0.1 && l[1] < 0.3, "G 通道应正常转换");
        assert!(l[2] > 0.01 && l[2] < 0.1, "B 通道应正常转换");
        // alpha 通道应为 0.0
        assert!(l[3].abs() < f32::EPSILON, "alpha=0 应转换为 0.0");
    }

    /// 测试 premultiplied alpha 在中等 alpha 值下的精度
    #[test]
    fn test_color_premultiplied_mid_alpha_precision() {
        let c = Color::rgba(200, 150, 100, 128);
        let p = c.premultiplied();
        let a = 128.0_f32 / 255.0;
        assert!((p[0] - 200.0 / 255.0 * a).abs() < 0.001);
        assert!((p[1] - 150.0 / 255.0 * a).abs() < 0.001);
        assert!((p[2] - 100.0 / 255.0 * a).abs() < 0.001);
        assert!((p[3] - a).abs() < f32::EPSILON);
    }

    /// 测试 to_f32_array 在全边界值下的正确性（0 和 255）
    #[test]
    fn test_color_to_f32_array_boundary_values() {
        // 全零
        let c_zero = Color::rgba(0, 0, 0, 0);
        let f_zero = c_zero.to_f32_array();
        assert!(f_zero.iter().all(|&v| v.abs() < f32::EPSILON));

        // 全 255
        let c_max = Color::rgba(255, 255, 255, 255);
        let f_max = c_max.to_f32_array();
        assert!(f_max.iter().all(|&v| (v - 1.0).abs() < f32::EPSILON));
    }
}
