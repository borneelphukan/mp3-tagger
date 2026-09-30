use std::cell::RefCell;
use std::rc::Rc;

/// Width the View > Theme submenu should end up with.
const THEME_MENU_WIDTH: f32 = 150.0;

// A menu is drawn in an Area sized to its content, so measuring the content
// width here is what the submenu window will be.
fn measure(fix: bool) -> f32 {
    let ctx = egui::Context::default();
    let captured = Rc::new(RefCell::new(0.0f32));
    let sink = captured.clone();

    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(400.0, 300.0),
            )),
            ..Default::default()
        },
        |ctx| {
            egui::Area::new(egui::Id::new("menu")).show(ctx, |ui| {
                egui::Frame::menu(&ctx.style()).show(ui, |ui| {
                    for label in ["Default", "Dark", "Light"] {
                        _ = ui.selectable_label(false, label);
                    }
                    if fix {
                        let left = ui.min_rect().left();
                        ui.expand_to_include_x(left + THEME_MENU_WIDTH);
                    }
                    *sink.borrow_mut() = ui.min_rect().width();
                });
            });
        },
    );

    let width = *captured.borrow();
    width
}

#[test]
fn theme_submenu_is_wider() {
    let before = measure(false);
    let after = measure(true);
    println!("menu content width: before={before:.1} after={after:.1}");
    assert!(after >= THEME_MENU_WIDTH, "too narrow: {after}");
    assert!(after > before, "not wider: {before} -> {after}");
}
