use super::ahem_uses_embox_position;

/// R841：line-height ≈ font-size（half-leading≈0）启用 em-box 位（修 ifc-008/line-height-121）。
#[test]
fn r841_embox_gate_half_leading_zero() {
    // lh:1（含 1em、Ahem lh:normal=1.0）→ half-leading=0 → em-box 位
    assert!(ahem_uses_embox_position(40.0, 40.0), "lh:1 应启用 em-box 位");
    assert!(
        ahem_uses_embox_position(100.0, 100.0),
        "lh:1em（100px）应启用（ifc-008）"
    );
    // 极小数值误差仍视为 lh≈fs
    assert!(ahem_uses_embox_position(40.0 + 0.1, 40.0), "亚像素偏差应仍启用");
}

/// R841：line-height:0（行盒塌缩）与 line-height>1（含 leading）保留 R817 位。
#[test]
fn r841_embox_gate_leading_present() {
    // lh:0（line-height:0px 测试簇）→ half-leading=-fs/2 → 不启用（避免 27 用例越过 1%）
    assert!(!ahem_uses_embox_position(0.0, 20.0), "lh:0 不应启用");
    // lh>1（va-117a 等）→ 含正 half-leading → 不启用（R839 妥协位）
    assert!(!ahem_uses_embox_position(130.0, 40.0), "lh>1 不应启用");
    assert!(!ahem_uses_embox_position(80.0, 40.0), "lh:2 不应启用");
    // lh:0.5（<fs）也不启用
    assert!(!ahem_uses_embox_position(10.0, 20.0), "lh:0.5 不应启用");
}
