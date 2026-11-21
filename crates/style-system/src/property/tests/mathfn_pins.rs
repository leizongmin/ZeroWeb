// sign()/abs() 写面钉测（MATHFN 刀，signs-abs-computed 语料 INVALID 80F 桶
// 驱动）：z-index <integer> 槽 20 单位族 × 四形态全受理 + time 原子经型别擦除
// 函数实参放行 + rotate 角度槽 sign 型别擦除 + IEEE 负零钳形 + 负钉（abs 型
// 别保留、time 树/混型拒）。
use crate::property::ComputedStyle;
use crate::property::apply_property_value;

/// signs-abs-computed 语料 z-index 全形：20 单位族 × {plain, sign(-N),
/// calc(sign(N)), calc(sign(-N)), clamp(sign(sign(N))), clamp(sign(sign(-N)))}
/// + 混合 px-em（写面 validate 门全放行）。
#[test]
fn sign_abs_z_index_slot_forms_accepted() {
    let units = ["em", "ex", "ch", "rem", "vh", "vmax", "vmin", "vw", "ms", "s"];
    let mut forms: Vec<String> = Vec::new();
    for u in units {
        for form in [
            format!("sign(3{u})"),
            format!("sign(-3{u})"),
            format!("calc(sign(3{u}))"),
            format!("calc(sign(-3{u}))"),
            format!("clamp(-1, calc( 1 / sign(sign(3{u}))), 1)"),
            format!("clamp(-1, calc( 1 / sign(sign(-3{u}))), 1)"),
        ] {
            forms.push(form);
        }
    }
    forms.push("sign(42px - 2em)".to_string());
    forms.push("calc(3 + sign(42px - 2em))".to_string());
    for f in &forms {
        assert!(crate::property::validate_decl_value("z-index", f), "{f} 须受理");
    }
}

#[test]
fn sign_abs_z_index_slot_negative_pins() {
    // abs() 型别保留（§10.5 abs(A)→随 A）：<length>/<time> 产出入 <integer>
    // 槽拒；time 树（非擦除形）拒；number−time 混型拒。
    assert!(!crate::property::validate_decl_value("z-index", "abs(3px)"));
    assert!(!crate::property::validate_decl_value("z-index", "abs(1s)"));
    assert!(!crate::property::validate_decl_value("z-index", "calc(1s * 3)"));
    assert!(!crate::property::validate_decl_value("z-index", "min(1s, 2s)"));
    // 纯 number 形受理对照（signs-abs-computed 既有 P 面）。
    assert!(crate::property::validate_decl_value("z-index", "sign(-3)"));
}

#[test]
fn sign_abs_z_index_apply_rounding() {
    // apply 面：sign 结果 number → <integer> 四舍五入；混合 px-em 依默认
    // 字体基准（16px：42 - 32 = 10 > 0 → 1）；负零钳形 IEEE -1。
    let cases = [
        ("sign(1em)", "Integer(1)"),
        ("sign(1s)", "Integer(1)"),
        ("sign(-3)", "Integer(-1)"),
        ("sign(42px - 2em)", "Integer(1)"),
        ("clamp(-1, calc( 1 / sign(sign(-0px))), 1)", "Integer(-1)"),
        ("clamp(-1, calc( 1 / sign(sign(0px))), 1)", "Integer(1)"),
    ];
    for (v, want) in cases {
        let mut style = ComputedStyle::default();
        assert!(apply_property_value(&mut style, "z-index", v), "{v} 须受理");
        assert_eq!(format!("{:?}", style.z_index), want, "{v}");
    }
}

#[test]
fn sign_abs_time_slot_transition_delay() {
    // parse_time strip_suffix 陷阱钉：函数形以 s/ms 结尾不再误入裸数字分支
    //（Time 叶保族负号重写后 calc_expr_is_pure_time 认树）。
    assert_eq!(zero_css_parser::values::parse_time("abs(-1s)"), Some(1.0));
    assert_eq!(zero_css_parser::values::parse_time("abs(-1ms)"), Some(0.001));
    assert_eq!(zero_css_parser::values::parse_time("calc(1s + abs(-2s))"), Some(3.0));
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "transition-delay", "abs(-1s)"));
}

#[test]
fn sign_abs_rotate_slot_type_erasure() {
    // rotate 角度槽：sign 型别擦除（calc_expr_has_length 豁免 Sign 子树），
    // 混合 px-em 依默认字体基准（42 - 32 = 10 → 1 → 90 + 30 = 120deg，
    // 语料期望 120deg 同值）。
    assert!(crate::property::validate_decl_value(
        "rotate",
        "calc(90deg + 30deg * sign(42px - 2em))"
    ));
    // abs 型别保留负钉。
    assert!(!crate::property::validate_decl_value(
        "rotate",
        "calc(90deg + abs(2px))"
    ));
}

#[test]
fn animation_duration_plain_negative_literal_rejected() {
    // animation-duration-invalid `-3s` should-not-set（css-animations-1
    // §propdef `<time [0s,∞]>` 范围锁）：裸负字面超文法界，非 math 形不走
    // §10.12 computed 期 clamp 面（MATHFN 刀 b1' 电池 −1 回归根因钉——
    // validate 放行原文存储，oracle parse_non_negative_time_list 链须拒）。
    assert!(
        !crate::property::validate_decl_value("animation-duration", "-3s"),
        "裸负字面须拒收"
    );
    let mut style = ComputedStyle::default();
    assert!(
        !crate::property::apply_property_value(&mut style, "animation-duration", "-3s"),
        "apply 链须拒收"
    );
    // 对照：非负裸字面与 math 形负值（§10.12）照常受理。
    assert!(crate::property::validate_decl_value("animation-duration", "3s"));
    assert!(crate::property::validate_decl_value(
        "animation-duration",
        "calc(1s * -infinity)"
    ));
}

#[test]
fn sign_abs_scale_slot_static_acceptance() {
    // scale-animation-math-functions-tentative 'to' 受理面（MATHFN 刀电池 b1
    // 26→0 全崩根因）：transition 端点 `calc(sign(20rem - 20px) * 180)`
    // validate + apply 双链须受理——parse_finite_number_math eval 至纯 number
    // 端点（20rem-20px=300px→sign=1→180），插值通道拿数值端点真插值。
    let v = "calc(sign(20rem - 20px) * 180)";
    assert!(crate::property::validate_decl_value("scale", v), "{v} 须受理");
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "scale", v), "{v} apply 须受理");
    // number 槽 % 叶拒（percentage-without-context 语料）：sign(50%) 擦除形。
    assert!(!crate::property::validate_decl_value("scale", "calc(sign(50%) * 180)"));
}
