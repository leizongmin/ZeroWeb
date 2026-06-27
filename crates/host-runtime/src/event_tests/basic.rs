use super::*;

#[test]
fn test_app_event_resized_values() {
    let event = AppEvent::Resized {
        width: 1920,
        height: 1080,
    };
    if let AppEvent::Resized { width, height } = event {
        assert_eq!(width, 1920);
        assert_eq!(height, 1080);
    } else {
        panic!("Expected Resized variant");
    }
}

#[test]
fn test_app_event_scale_factor_changed_value() {
    let event = AppEvent::ScaleFactorChanged { scale_factor: 2.0 };
    if let AppEvent::ScaleFactorChanged { scale_factor } = event {
        assert!((scale_factor - 2.0).abs() < f64::EPSILON);
    } else {
        panic!("Expected ScaleFactorChanged variant");
    }
}

#[test]
fn test_app_event_keyboard_input_values() {
    let pressed = AppEvent::KeyboardInput {
        key: "A".to_string(),
        text: None,
        pressed: true,
    };
    if let AppEvent::KeyboardInput { key, pressed: p, .. } = &pressed {
        assert_eq!(key, "A");
        assert!(p);
    } else {
        panic!("Expected KeyboardInput variant");
    }

    let released = AppEvent::KeyboardInput {
        key: "Escape".to_string(),
        text: None,
        pressed: false,
    };
    if let AppEvent::KeyboardInput { pressed: p, .. } = &released {
        assert!(!p);
    } else {
        panic!("Expected KeyboardInput variant");
    }
}

#[test]
fn test_app_event_debug_format() {
    assert!(!format!("{:?}", AppEvent::RedrawRequested).is_empty());
    assert!(!format!("{:?}", AppEvent::CloseRequested).is_empty());
    assert!(!format!("{:?}", AppEvent::Focused).is_empty());
    assert!(!format!("{:?}", AppEvent::Unfocused).is_empty());
    assert!(
        format!(
            "{:?}",
            AppEvent::Resized {
                width: 100,
                height: 200
            }
        )
        .contains("100")
    );
    assert!(
        format!(
            "{:?}",
            AppEvent::KeyboardInput {
                key: "X".into(),
                text: None,
                pressed: true
            }
        )
        .contains("X")
    );
}

#[test]
fn test_app_event_focused_unfocused_distinct() {
    let focused = format!("{:?}", AppEvent::Focused);
    let unfocused = format!("{:?}", AppEvent::Unfocused);
    assert_ne!(focused, unfocused);
}

// --- ???????? ---

#[test]
fn test_mouse_button_variants() {
    assert_eq!(format!("{:?}", MouseButton::Left), "Left");
    assert_eq!(format!("{:?}", MouseButton::Right), "Right");
    assert_eq!(format!("{:?}", MouseButton::Middle), "Middle");
    assert_eq!(format!("{:?}", MouseButton::Back), "Back");
    assert_eq!(format!("{:?}", MouseButton::Forward), "Forward");
    assert_eq!(format!("{:?}", MouseButton::Other(8)), "Other(8)");
}

#[test]
fn test_mouse_button_equality() {
    assert_eq!(MouseButton::Left, MouseButton::Left);
    assert_ne!(MouseButton::Left, MouseButton::Right);
    assert_eq!(MouseButton::Other(3), MouseButton::Other(3));
    assert_ne!(MouseButton::Other(3), MouseButton::Other(4));
}

#[test]
fn test_mouse_scroll_delta_pixel() {
    let delta = MouseScrollDelta::PixelDelta(10.0, 20.0);
    assert_eq!(delta, MouseScrollDelta::PixelDelta(10.0, 20.0));
}

#[test]
fn test_mouse_scroll_delta_line() {
    let delta = MouseScrollDelta::LineDelta(3.0, -1.0);
    assert_eq!(delta, MouseScrollDelta::LineDelta(3.0, -1.0));
}

#[test]
fn test_ime_event_enabled() {
    let e = ImeEvent::Enabled;
    assert!(format!("{:?}", e).contains("Enabled"));
}

#[test]
fn test_ime_event_preedit() {
    let e = ImeEvent::Preedit {
        text: "??".to_string(),
        cursor: Some((0, 2)),
    };
    if let ImeEvent::Preedit { text, cursor } = e {
        assert_eq!(text, "??");
        assert_eq!(cursor, Some((0, 2)));
    } else {
        panic!("Expected Preedit");
    }
}

#[test]
fn test_ime_event_preedit_no_cursor() {
    let e = ImeEvent::Preedit {
        text: "abc".to_string(),
        cursor: None,
    };
    if let ImeEvent::Preedit { text, cursor } = e {
        assert_eq!(text, "abc");
        assert!(cursor.is_none());
    } else {
        panic!("Expected Preedit");
    }
}

#[test]
fn test_ime_event_commit() {
    let e = ImeEvent::Commit("????".to_string());
    if let ImeEvent::Commit(text) = e {
        assert_eq!(text, "????");
    } else {
        panic!("Expected Commit");
    }
}

#[test]
fn test_ime_event_disabled() {
    let e = ImeEvent::Disabled;
    assert!(format!("{:?}", e).contains("Disabled"));
}

#[test]
fn test_ime_event_equality() {
    assert_eq!(ImeEvent::Enabled, ImeEvent::Enabled);
    assert_ne!(ImeEvent::Enabled, ImeEvent::Disabled);
    assert_eq!(ImeEvent::Commit("a".to_string()), ImeEvent::Commit("a".to_string()));
}

#[test]
fn test_touch_phase_variants() {
    assert_eq!(TouchPhase::Started, TouchPhase::Started);
    assert_ne!(TouchPhase::Started, TouchPhase::Ended);
}

#[test]
fn test_touch_event_fields() {
    let te = TouchEvent {
        id: 42,
        phase: TouchPhase::Moved,
        x: 100.0,
        y: 200.0,
    };
    assert_eq!(te.id, 42);
    assert_eq!(te.phase, TouchPhase::Moved);
    assert!((te.x - 100.0).abs() < f64::EPSILON);
    assert!((te.y - 200.0).abs() < f64::EPSILON);
}

#[test]
fn test_app_event_mouse_moved() {
    let e = AppEvent::MouseMoved { x: 50.0, y: 75.0 };
    if let AppEvent::MouseMoved { x, y } = e {
        assert!((x - 50.0).abs() < f64::EPSILON);
        assert!((y - 75.0).abs() < f64::EPSILON);
    } else {
        panic!("Expected MouseMoved");
    }
}

#[test]
fn test_app_event_mouse_input() {
    let e = AppEvent::MouseInput {
        button: MouseButton::Left,
        pressed: true,
        x: 0.0,
        y: 0.0,
    };
    if let AppEvent::MouseInput { button, pressed, .. } = e {
        assert_eq!(button, MouseButton::Left);
        assert!(pressed);
    } else {
        panic!("Expected MouseInput");
    }
}

#[test]
fn test_app_event_mouse_wheel() {
    let e = AppEvent::MouseWheel {
        delta: MouseScrollDelta::LineDelta(1.0, 0.0),
        x: 0.0,
        y: 0.0,
    };
    if let AppEvent::MouseWheel { delta, .. } = &e {
        assert_eq!(*delta, MouseScrollDelta::LineDelta(1.0, 0.0));
    } else {
        panic!("Expected MouseWheel");
    }
}

#[test]
fn test_app_event_touch() {
    let e = AppEvent::Touch(TouchEvent {
        id: 0,
        phase: TouchPhase::Started,
        x: 10.0,
        y: 20.0,
    });
    if let AppEvent::Touch(te) = &e {
        assert_eq!(te.id, 0);
        assert_eq!(te.phase, TouchPhase::Started);
    } else {
        panic!("Expected Touch");
    }
}

#[test]
fn test_app_event_ime() {
    let e = AppEvent::Ime(ImeEvent::Commit("abc".to_string()));
    if let AppEvent::Ime(ImeEvent::Commit(s)) = &e {
        assert_eq!(s, "abc");
    } else {
        panic!("Expected Ime(Commit)");
    }
}

// --- ?????? ---

#[test]
fn test_convert_mouse_button_all_variants() {
    assert_eq!(convert_mouse_button(winit::event::MouseButton::Left), MouseButton::Left);
    assert_eq!(
        convert_mouse_button(winit::event::MouseButton::Right),
        MouseButton::Right
    );
    assert_eq!(
        convert_mouse_button(winit::event::MouseButton::Middle),
        MouseButton::Middle
    );
    assert_eq!(convert_mouse_button(winit::event::MouseButton::Back), MouseButton::Back);
    assert_eq!(
        convert_mouse_button(winit::event::MouseButton::Forward),
        MouseButton::Forward
    );
    assert_eq!(
        convert_mouse_button(winit::event::MouseButton::Other(9)),
        MouseButton::Other(9)
    );
}

#[test]
fn test_convert_scroll_delta_pixel() {
    let pos = winit::dpi::PhysicalPosition::new(5.0, -3.0);
    let delta = winit::event::MouseScrollDelta::PixelDelta(pos);
    let result = convert_scroll_delta(delta);
    assert_eq!(result, MouseScrollDelta::PixelDelta(5.0, -3.0));
}

#[test]
fn test_convert_scroll_delta_line() {
    let delta = winit::event::MouseScrollDelta::LineDelta(2.0, -1.0);
    let result = convert_scroll_delta(delta);
    assert_eq!(result, MouseScrollDelta::LineDelta(2.0, -1.0));
}

#[test]
fn test_convert_ime_enabled() {
    let result = convert_ime(winit::event::Ime::Enabled);
    assert_eq!(result, ImeEvent::Enabled);
}

#[test]
fn test_convert_ime_preedit() {
    let result = convert_ime(winit::event::Ime::Preedit("abc".to_string(), Some((0, 1))));
    assert_eq!(
        result,
        ImeEvent::Preedit {
            text: "abc".to_string(),
            cursor: Some((0, 1))
        }
    );
}

#[test]
fn test_convert_ime_preedit_no_cursor() {
    let result = convert_ime(winit::event::Ime::Preedit(String::new(), None));
    assert_eq!(
        result,
        ImeEvent::Preedit {
            text: String::new(),
            cursor: None
        }
    );
}

#[test]
fn test_convert_ime_commit() {
    let result = convert_ime(winit::event::Ime::Commit("hello".to_string()));
    assert_eq!(result, ImeEvent::Commit("hello".to_string()));
}

#[test]
fn test_convert_ime_disabled() {
    let result = convert_ime(winit::event::Ime::Disabled);
    assert_eq!(result, ImeEvent::Disabled);
}

#[test]
fn test_keyboard_input_text_field_for_unidentified_key() {
    let event = AppEvent::KeyboardInput {
        key: "Unidentified".to_string(),
        text: Some("a".to_string()),
        pressed: true,
    };
    if let AppEvent::KeyboardInput { key, text, pressed } = event {
        assert_eq!(key, "Unidentified");
        assert_eq!(text.as_deref(), Some("a"));
        assert!(pressed);
    } else {
        panic!("Expected KeyboardInput variant");
    }
}

#[test]
fn test_convert_touch_phase_all() {
    assert_eq!(
        convert_touch_phase(winit::event::TouchPhase::Started),
        TouchPhase::Started
    );
    assert_eq!(convert_touch_phase(winit::event::TouchPhase::Moved), TouchPhase::Moved);
    assert_eq!(convert_touch_phase(winit::event::TouchPhase::Ended), TouchPhase::Ended);
    assert_eq!(
        convert_touch_phase(winit::event::TouchPhase::Cancelled),
        TouchPhase::Cancelled
    );
}

#[test]
fn test_element_state_to_pressed() {
    assert!(element_state_to_pressed(winit::event::ElementState::Pressed));
    assert!(!element_state_to_pressed(winit::event::ElementState::Released));
}

#[test]
fn test_app_event_new_variants_debug() {
    assert!(!format!("{:?}", AppEvent::MouseMoved { x: 0.0, y: 0.0 }).is_empty());
    assert!(
        !format!(
            "{:?}",
            AppEvent::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                x: 0.0,
                y: 0.0,
            }
        )
        .is_empty()
    );
    assert!(
        !format!(
            "{:?}",
            AppEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(1.0, 0.0),
                x: 0.0,
                y: 0.0,
            }
        )
        .is_empty()
    );
    assert!(
        !format!(
            "{:?}",
            AppEvent::Touch(TouchEvent {
                id: 0,
                phase: TouchPhase::Started,
                x: 0.0,
                y: 0.0
            })
        )
        .is_empty()
    );
    assert!(!format!("{:?}", AppEvent::Ime(ImeEvent::Enabled)).is_empty());
}

// --- Additional coverage tests ---

#[test]
fn test_mouse_button_copy() {
    let btn = MouseButton::Left;
    let btn2 = btn;
    assert_eq!(btn, btn2);
}

#[test]
fn test_mouse_button_other_copy() {
    let btn = MouseButton::Other(16);
    let btn2 = btn;
    assert_eq!(btn, btn2);
}

#[test]
fn test_mouse_scroll_delta_copy() {
    let delta = MouseScrollDelta::PixelDelta(100.0, 200.0);
    let delta2 = delta;
    assert_eq!(delta, delta2);
}

#[test]
fn test_mouse_scroll_delta_line_copy() {
    let delta = MouseScrollDelta::LineDelta(-5.0, 3.0);
    let delta2 = delta;
    assert_eq!(delta, delta2);
}

#[test]
fn test_touch_phase_copy() {
    let phase = TouchPhase::Moved;
    let phase2 = phase;
    assert_eq!(phase, phase2);
}

#[test]
fn test_touch_phase_all_variants_distinct() {
    let phases = [
        TouchPhase::Started,
        TouchPhase::Moved,
        TouchPhase::Ended,
        TouchPhase::Cancelled,
    ];
    for i in 0..phases.len() {
        for j in 0..phases.len() {
            if i == j {
                assert_eq!(phases[i], phases[j]);
            } else {
                assert_ne!(phases[i], phases[j]);
            }
        }
    }
}

#[test]
fn test_touch_event_clone() {
    let te = TouchEvent {
        id: 99,
        phase: TouchPhase::Ended,
        x: 500.0,
        y: -10.0,
    };
    let te2 = te.clone();
    assert_eq!(te.id, te2.id);
    assert_eq!(te.phase, te2.phase);
    assert!((te.x - te2.x).abs() < f64::EPSILON);
    assert!((te.y - te2.y).abs() < f64::EPSILON);
}

#[test]
fn test_ime_event_preedit_equality() {
    let a = ImeEvent::Preedit {
        text: "x".to_string(),
        cursor: Some((0, 1)),
    };
    let b = ImeEvent::Preedit {
        text: "x".to_string(),
        cursor: Some((0, 1)),
    };
    assert_eq!(a, b);
}

#[test]
fn test_ime_event_preedit_inequality() {
    let a = ImeEvent::Preedit {
        text: "a".to_string(),
        cursor: None,
    };
    let b = ImeEvent::Preedit {
        text: "b".to_string(),
        cursor: None,
    };
    assert_ne!(a, b);
}

#[test]
fn test_ime_event_commit_empty_string() {
    let e = ImeEvent::Commit(String::new());
    if let ImeEvent::Commit(s) = e {
        assert!(s.is_empty());
    } else {
        panic!("Expected Commit");
    }
}

#[test]
fn test_ime_event_preedit_empty_text_with_cursor() {
    let e = ImeEvent::Preedit {
        text: String::new(),
        cursor: Some((0, 0)),
    };
    if let ImeEvent::Preedit { text, cursor } = e {
        assert!(text.is_empty());
        assert_eq!(cursor, Some((0, 0)));
    } else {
        panic!("Expected Preedit");
    }
}

#[test]
fn test_app_event_resized_large_values() {
    let event = AppEvent::Resized {
        width: u32::MAX,
        height: u32::MAX,
    };
    if let AppEvent::Resized { width, height } = event {
        assert_eq!(width, u32::MAX);
        assert_eq!(height, u32::MAX);
    } else {
        panic!("Expected Resized");
    }
}

#[test]
fn test_app_event_mouse_moved_zero() {
    let e = AppEvent::MouseMoved { x: 0.0, y: 0.0 };
    if let AppEvent::MouseMoved { x, y } = e {
        assert!((x - 0.0).abs() < f64::EPSILON);
        assert!((y - 0.0).abs() < f64::EPSILON);
    } else {
        panic!("Expected MouseMoved");
    }
}

#[test]
fn test_app_event_mouse_moved_negative_coords() {
    let e = AppEvent::MouseMoved { x: -999.5, y: -0.1 };
    if let AppEvent::MouseMoved { x, y } = e {
        assert!((x - (-999.5)).abs() < 1e-10);
        assert!((y - (-0.1)).abs() < 1e-10);
    } else {
        panic!("Expected MouseMoved");
    }
}

#[test]
fn test_app_event_mouse_input_right_released() {
    let e = AppEvent::MouseInput {
        button: MouseButton::Right,
        pressed: false,
        x: 0.0,
        y: 0.0,
    };
    if let AppEvent::MouseInput { button, pressed, .. } = e {
        assert_eq!(button, MouseButton::Right);
        assert!(!pressed);
    } else {
        panic!("Expected MouseInput");
    }
}

#[test]
fn test_app_event_mouse_input_middle_pressed() {
    let e = AppEvent::MouseInput {
        button: MouseButton::Middle,
        pressed: true,
        x: 0.0,
        y: 0.0,
    };
    if let AppEvent::MouseInput { button, pressed, .. } = e {
        assert_eq!(button, MouseButton::Middle);
        assert!(pressed);
    } else {
        panic!("Expected MouseInput");
    }
}

#[test]
fn test_app_event_mouse_input_back_forward() {
    let back = AppEvent::MouseInput {
        button: MouseButton::Back,
        pressed: true,
        x: 0.0,
        y: 0.0,
    };
    let fwd = AppEvent::MouseInput {
        button: MouseButton::Forward,
        pressed: true,
        x: 0.0,
        y: 0.0,
    };
    if let AppEvent::MouseInput { button, .. } = &back {
        assert_eq!(*button, MouseButton::Back);
    } else {
        panic!("Expected MouseInput");
    }
    if let AppEvent::MouseInput { button, .. } = &fwd {
        assert_eq!(*button, MouseButton::Forward);
    } else {
        panic!("Expected MouseInput");
    }
}

#[test]
fn test_app_event_mouse_wheel_pixel_delta_large() {
    let e = AppEvent::MouseWheel {
        delta: MouseScrollDelta::PixelDelta(100000.0, -99999.0),
        x: 0.0,
        y: 0.0,
    };
    if let AppEvent::MouseWheel { delta, .. } = &e {
        assert_eq!(*delta, MouseScrollDelta::PixelDelta(100000.0, -99999.0));
    } else {
        panic!("Expected MouseWheel");
    }
}

#[test]
fn test_app_event_touch_started_at_origin() {
    let e = AppEvent::Touch(TouchEvent {
        id: 0,
        phase: TouchPhase::Started,
        x: 0.0,
        y: 0.0,
    });
    if let AppEvent::Touch(te) = &e {
        assert_eq!(te.id, 0);
        assert_eq!(te.phase, TouchPhase::Started);
    } else {
        panic!("Expected Touch");
    }
}

#[test]
fn test_app_event_ime_preedit_dispatch() {
    let e = AppEvent::Ime(ImeEvent::Preedit {
        text: "abc".to_string(),
        cursor: Some((0, 3)),
    });
    if let AppEvent::Ime(ImeEvent::Preedit { text, cursor }) = &e {
        assert_eq!(text, "abc");
        assert_eq!(*cursor, Some((0, 3)));
    } else {
        panic!("Expected Ime(Preedit)");
    }
}

#[test]
fn test_mouse_button_all_debug_roundtrip() {
    let variants: Vec<MouseButton> = vec![
        MouseButton::Left,
        MouseButton::Right,
        MouseButton::Middle,
        MouseButton::Back,
        MouseButton::Forward,
        MouseButton::Other(0),
        MouseButton::Other(u16::MAX),
    ];
    for v in &variants {
        let debug = format!("{:?}", v);
        assert!(!debug.is_empty());
    }
}

#[test]
fn test_mouse_scroll_delta_pixel_inequality() {
    let a = MouseScrollDelta::PixelDelta(1.0, 2.0);
    let b = MouseScrollDelta::PixelDelta(1.0, 3.0);
    assert_ne!(a, b);
}

#[test]
fn test_mouse_scroll_delta_cross_variant_inequality() {
    let pixel = MouseScrollDelta::PixelDelta(1.0, 2.0);
    let line = MouseScrollDelta::LineDelta(1.0, 2.0);
    assert_ne!(pixel, line);
}

// --- ?????????? ---

/// ????? resize ???????????????????????
#[test]
fn test_resize_event_carries_correct_dimensions() {
    let cases: Vec<(u32, u32)> = vec![
        (1920, 1080), // Full HD
        (2560, 1440), // QHD
        (1366, 768),  // ?????
        (1, 1),       // ???
    ];
    for (w, h) in cases {
        let event = AppEvent::Resized { width: w, height: h };
        if let AppEvent::Resized { width, height } = event {
            assert_eq!(width, w, "resize width ???: ?? {w}, ?? {width}");
            assert_eq!(height, h, "resize height ???: ?? {h}, ?? {height}");
        } else {
            panic!("Expected Resized variant");
        }
    }
}

/// ?????????????????????????
#[test]
fn test_mouse_move_coordinates_precision() {
    let cases: Vec<(f64, f64)> = vec![
        (0.0, 0.0),
        (1920.5, 1080.25), // ????
        (-100.0, -200.0),  // ???
    ];
    for (x, y) in cases {
        let event = AppEvent::MouseMoved { x, y };
        if let AppEvent::MouseMoved { x: ex, y: ey } = event {
            assert!((ex - x).abs() < f64::EPSILON, "x ?????: ?? {x}, ?? {ex}");
            assert!((ey - y).abs() < f64::EPSILON, "y ?????: ?? {y}, ?? {ey}");
        } else {
            panic!("Expected MouseMoved variant");
        }
    }
}

/// ???IME ???????? ? Enabled ? ?? Preedit ? Commit ? Disabled
#[test]
fn test_ime_composition_full_lifecycle() {
    // ??????"?"?????
    let enabled = ImeEvent::Enabled;
    let preedit1 = ImeEvent::Preedit {
        text: "z".to_string(),
        cursor: Some((0, 1)),
    };
    let preedit2 = ImeEvent::Preedit {
        text: "zh".to_string(),
        cursor: Some((0, 2)),
    };
    let preedit3 = ImeEvent::Preedit {
        text: "zhon".to_string(),
        cursor: Some((0, 4)),
    };
    let preedit4 = ImeEvent::Preedit {
        text: "zhong".to_string(),
        cursor: Some((0, 5)),
    };
    let commit = ImeEvent::Commit("?".to_string());
    let disabled = ImeEvent::Disabled;

    // ??????????????
    assert!(matches!(enabled, ImeEvent::Enabled));
    assert_eq!(
        preedit3,
        ImeEvent::Preedit {
            text: "zhon".to_string(),
            cursor: Some((0, 4))
        }
    );
    assert_eq!(commit, ImeEvent::Commit("?".to_string()));
    assert!(matches!(disabled, ImeEvent::Disabled));

    // ??????????
    let lifecycle: Vec<ImeEvent> = vec![enabled, preedit1, preedit2, preedit3, preedit4, commit, disabled];
    assert_eq!(lifecycle.len(), 7);

    // ?? commit ??
    if let ImeEvent::Commit(text) = &lifecycle[5] {
        assert!(!text.is_empty(), "IME commit ??????");
        assert_eq!(text, "?");
    } else {
        panic!("? 6 ????? Commit");
    }

    // ?? Preedit ??????
    let texts: Vec<&str> = lifecycle[1..=4]
        .iter()
        .map(|e| {
            if let ImeEvent::Preedit { text, .. } = e {
                text.as_str()
            } else {
                ""
            }
        })
        .collect();
    assert_eq!(texts, vec!["z", "zh", "zhon", "zhong"]);
}

/// ???????????? element_state_to_pressed ????
/// ?Ctrl/Shift ??? pressed=true???? pressed=false?
#[test]
fn test_keyboard_modifier_state_conversion() {
    // ?? Ctrl+Shift ?????????
    // 1. Ctrl ??
    let ctrl_pressed = element_state_to_pressed(winit::event::ElementState::Pressed);
    assert!(ctrl_pressed, "Ctrl ??? pressed ?? true");

    // 2. Shift ????? Ctrl ????
    let shift_pressed = element_state_to_pressed(winit::event::ElementState::Pressed);
    assert!(shift_pressed, "Shift ??? pressed ?? true");

    // 3. ??????Ctrl+Shift+A?
    let char_pressed = element_state_to_pressed(winit::event::ElementState::Pressed);
    assert!(char_pressed, "?????? pressed ?? true");

    // 4. ?????
    let char_released = element_state_to_pressed(winit::event::ElementState::Released);
    assert!(!char_released, "?????? pressed ?? false");

    // 5. Shift ??
    let shift_released = element_state_to_pressed(winit::event::ElementState::Released);
    assert!(!shift_released, "Shift ??? pressed ?? false");

    // 6. Ctrl ??
    let ctrl_released = element_state_to_pressed(winit::event::ElementState::Released);
    assert!(!ctrl_released, "Ctrl ??? pressed ?? false");

    // ?? AppEvent::KeyboardInput ?????????/????
    let ctrl_down_event = AppEvent::KeyboardInput {
        key: "Control".to_string(),
        text: None,
        pressed: ctrl_pressed,
    };
    let shift_down_event = AppEvent::KeyboardInput {
        key: "Shift".to_string(),
        text: None,
        pressed: shift_pressed,
    };
    if let AppEvent::KeyboardInput { key, pressed, .. } = ctrl_down_event {
        assert_eq!(key, "Control");
        assert!(pressed);
    } else {
        panic!("Expected KeyboardInput");
    }
    if let AppEvent::KeyboardInput { key, pressed, .. } = shift_down_event {
        assert_eq!(key, "Shift");
        assert!(pressed);
    } else {
        panic!("Expected KeyboardInput");
    }

    // ??????
    let ctrl_up_event = AppEvent::KeyboardInput {
        key: "Control".to_string(),
        text: None,
        pressed: ctrl_released,
    };
    if let AppEvent::KeyboardInput { pressed, .. } = ctrl_up_event {
        assert!(!pressed, "Ctrl ???? pressed ?? false");
    } else {
        panic!("Expected KeyboardInput");
    }
}

/// ???AppEvent::KeyboardInput ? key ? pressed ???????????
/// ?? winit ? KeyEvent ????????????
/// ?????? AppEvent::KeyboardInput ???????
#[test]
fn test_keyboard_input_event_construction() {
    // ????
    let press = AppEvent::KeyboardInput {
        key: "A".into(),
        text: None,
        pressed: true,
    };
    if let AppEvent::KeyboardInput { key, pressed, .. } = press {
        assert_eq!(key, "A", "????? key ?? 'A'");
        assert!(pressed, "???? pressed ?? true");
    } else {
        panic!("Expected KeyboardInput variant");
    }

    // ????
    let release = AppEvent::KeyboardInput {
        key: "A".into(),
        text: None,
        pressed: false,
    };
    if let AppEvent::KeyboardInput { key, pressed, .. } = release {
        assert_eq!(key, "A");
        assert!(!pressed, "???? pressed ?? false");
    } else {
        panic!("Expected KeyboardInput variant");
    }

    // ???
    let enter_press = AppEvent::KeyboardInput {
        key: "Enter".into(),
        text: None,
        pressed: true,
    };
    if let AppEvent::KeyboardInput { key, pressed, .. } = enter_press {
        assert_eq!(key, "Enter");
        assert!(pressed);
    } else {
        panic!("Expected KeyboardInput variant");
    }

    // ?????????????
    let empty_key = AppEvent::KeyboardInput {
        key: String::new(),
        text: None,
        pressed: true,
    };
    if let AppEvent::KeyboardInput { key, pressed, .. } = empty_key {
        assert_eq!(key, "", "??????????");
        assert!(pressed);
    } else {
        panic!("Expected KeyboardInput variant");
    }
}

/// ????????????????????????
/// ?? BasicApp ?????? convert_mouse_button ? AppEvent::MouseInput ?????
#[test]
fn test_mouse_event_button_values() {
    let cases: Vec<(winit::event::MouseButton, MouseButton)> = vec![
        (winit::event::MouseButton::Left, MouseButton::Left),
        (winit::event::MouseButton::Right, MouseButton::Right),
        (winit::event::MouseButton::Middle, MouseButton::Middle),
    ];

    for (winit_btn, expected_btn) in cases {
        // ?? convert_mouse_button ????
        assert_eq!(
            convert_mouse_button(winit_btn),
            expected_btn,
            "convert_mouse_button ?? {expected_btn:?} ??"
        );

        // ???? AppEvent ????
        let press_event = AppEvent::MouseInput {
            button: expected_btn,
            pressed: true,
            x: 0.0,
            y: 0.0,
        };
        if let AppEvent::MouseInput { button, pressed, .. } = press_event {
            assert_eq!(button, expected_btn, "??????: ?? {expected_btn:?}");
            assert!(pressed);
        } else {
            panic!("Expected MouseInput variant");
        }

        let release_event = AppEvent::MouseInput {
            button: expected_btn,
            pressed: false,
            x: 0.0,
            y: 0.0,
        };
        if let AppEvent::MouseInput { button, pressed, .. } = release_event {
            assert_eq!(button, expected_btn, "??????????: ?? {expected_btn:?}");
            assert!(!pressed);
        } else {
            panic!("Expected MouseInput variant");
        }
    }
}

/// ???resize ??? width=0?height=0 ?????? panic?
/// ??????????????? (0, 0) ???
#[test]
fn test_window_resize_zero_size() {
    // ???? AppEvent::Resized
    let event = AppEvent::Resized { width: 0, height: 0 };
    if let AppEvent::Resized { width, height } = event {
        assert_eq!(width, 0, "??? resize ??? width ?? 0");
        assert_eq!(height, 0, "??? resize ??? height ?? 0");
    } else {
        panic!("Expected Resized variant");
    }

    // ?? winit PhysicalSize ????
    let size = winit::dpi::PhysicalSize::new(0u32, 0u32);
    assert_eq!(size.width, 0);
    assert_eq!(size.height, 0);

    // ?? Debug ???? panic
    let debug_str = format!("{:?}", event);
    assert!(!debug_str.is_empty(), "Debug ???????????");
}

/// ???IME ????????????????????
/// ????????????????????
#[test]
fn test_ime_composition_empty_string() {
    // ????? Commit ??
    let commit_empty = ImeEvent::Commit(String::new());
    if let ImeEvent::Commit(text) = &commit_empty {
        assert!(text.is_empty(), "???? commit ? text ???");
    } else {
        panic!("Expected Commit variant");
    }

    // ????? Preedit ??
    let preedit_empty = ImeEvent::Preedit {
        text: String::new(),
        cursor: None,
    };
    if let ImeEvent::Preedit { text, cursor } = &preedit_empty {
        assert!(text.is_empty(), "???? preedit ? text ???");
        assert!(cursor.is_none(), "? preedit ? cursor ?? None");
    } else {
        panic!("Expected Preedit variant");
    }

    // ???? Preedit ???
    let preedit_empty_with_cursor = ImeEvent::Preedit {
        text: String::new(),
        cursor: Some((0, 0)),
    };
    if let ImeEvent::Preedit { text, cursor } = &preedit_empty_with_cursor {
        assert!(text.is_empty());
        assert_eq!(*cursor, Some((0, 0)));
    } else {
        panic!("Expected Preedit variant");
    }

    // ?? winit ????
    assert_eq!(
        convert_ime(winit::event::Ime::Commit(String::new())),
        ImeEvent::Commit(String::new()),
        "winit ? commit ?????"
    );
    assert_eq!(
        convert_ime(winit::event::Ime::Preedit(String::new(), None)),
        ImeEvent::Preedit {
            text: String::new(),
            cursor: None,
        },
        "winit ? preedit ?????"
    );

    // ?????
    assert_eq!(commit_empty, ImeEvent::Commit(String::new()));
    assert_eq!(
        preedit_empty,
        ImeEvent::Preedit {
            text: String::new(),
            cursor: None,
        }
    );
}
