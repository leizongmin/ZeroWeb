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
    pub const TRANSPARENT: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };
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
}
