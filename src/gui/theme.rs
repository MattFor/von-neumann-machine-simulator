use egui::{Color32, Context, CornerRadius, Stroke, TextStyle, Theme, Visuals, vec2};

// Background, card, widget, hovered widget, line, text, accent, warning.
const DARK: [u32; 8] = [
    0x14161a, 0x1c1f25, 0x2a2e37, 0x363b47, 0x2f343d, 0xd4d7dd, 0x3d7bea, 0xffb454,
];
const LIGHT: [u32; 8] = [
    0xeef0f3, 0xffffff, 0xe4e7ec, 0xd5d9e0, 0xd3d7dd, 0x22262b, 0x2f6fed, 0xb45f06,
];

pub fn apply(ctx: &Context) {
    ctx.set_visuals_of(Theme::Dark, visuals(Visuals::dark(), DARK));
    ctx.set_visuals_of(Theme::Light, visuals(Visuals::light(), LIGHT));

    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = vec2(8.0, 4.0);
        style.spacing.button_padding = vec2(8.0, 3.0);
        style.spacing.interact_size.y = 22.0;

        for (text_style, size) in [
            (TextStyle::Small, 11.0),
            (TextStyle::Body, 14.0),
            (TextStyle::Button, 14.0),
            (TextStyle::Monospace, 13.0),
            (TextStyle::Heading, 19.0),
        ] {
            if let Some(font) = style.text_styles.get_mut(&text_style) {
                font.size = size;
            }
        }
    });
}

fn visuals(mut visuals: Visuals, palette: [u32; 8]) -> Visuals {
    let [
        background,
        card,
        widget,
        hovered,
        line,
        text,
        accent,
        warning,
    ] = palette.map(|rgb| Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8));
    let radius = CornerRadius::same(6);

    visuals.panel_fill = background;
    visuals.window_fill = card;
    visuals.window_stroke = Stroke::new(1.0, line);
    visuals.window_corner_radius = radius;
    visuals.menu_corner_radius = radius;
    visuals.extreme_bg_color = background;
    visuals.faint_bg_color = card.lerp_to_gamma(text, 0.05);
    visuals.selection.bg_fill = accent;
    visuals.selection.stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.hyperlink_color = accent;
    visuals.warn_fg_color = warning;
    visuals.slider_trailing_fill = true;

    let widgets = &mut visuals.widgets;

    widgets.noninteractive.bg_stroke = Stroke::new(1.0, line);
    widgets.noninteractive.fg_stroke.color = text;
    widgets.noninteractive.corner_radius = radius;

    for (state, fill) in [
        (&mut widgets.inactive, widget),
        (&mut widgets.hovered, hovered),
        (&mut widgets.active, hovered),
        (&mut widgets.open, hovered),
    ] {
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.fg_stroke.color = text;
        state.corner_radius = radius;
    }

    // The fill of a checkbox's box, a radio button and a slider's rail, but not of buttons; in the
    // widget colour an unticked box all but vanished into the cards.
    widgets.inactive.bg_fill = if visuals.dark_mode {
        Color32::from_rgb(0x4f, 0x55, 0x63)
    } else {
        Color32::from_rgb(0xb4, 0xba, 0xc4)
    };

    // Hovering still brightens that box in dark, where the hover colour is darker than it.
    if visuals.dark_mode {
        widgets.hovered.bg_fill = Color32::from_rgb(0x60, 0x67, 0x76);
        widgets.active.bg_fill = Color32::from_rgb(0x60, 0x67, 0x76);
    }

    visuals
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unticked_box_stands_out_in_both_themes() {
        let dark = visuals(Visuals::dark(), DARK);
        let sum = |c: Color32| c.r() as u32 + c.g() as u32 + c.b() as u32;
        assert!(sum(dark.widgets.hovered.bg_fill) > sum(dark.widgets.inactive.bg_fill));
        assert_ne!(dark.widgets.hovered.weak_bg_fill, dark.widgets.hovered.bg_fill);

        let apart = |a: Color32, b: Color32| {
            (0..3)
                .map(|channel| a.to_array()[channel].abs_diff(b.to_array()[channel]))
                .max()
                .unwrap()
        };
        let ctx = Context::default();
        apply(&ctx);

        for (theme, visuals, fill, button) in [
            (
                Theme::Light,
                visuals(Visuals::light(), LIGHT),
                Color32::from_rgb(0xb4, 0xba, 0xc4),
                Color32::from_rgb(0xe4, 0xe7, 0xec),
            ),
            (
                Theme::Dark,
                visuals(Visuals::dark(), DARK),
                Color32::from_rgb(0x4f, 0x55, 0x63),
                Color32::from_rgb(0x2a, 0x2e, 0x37),
            ),
        ] {
            assert_eq!(visuals.widgets.inactive.bg_fill, fill, "{theme:?}");
            assert!(apart(fill, visuals.window_fill) >= 48, "{theme:?}");
            assert!(apart(fill, visuals.panel_fill) >= 48, "{theme:?}");

            // Buttons keep the widget colour.
            assert_eq!(visuals.widgets.inactive.weak_bg_fill, button, "{theme:?}");

            // That is the fill egui paints the box with.
            ctx.set_theme(theme);

            let mut output = ctx.run_ui(Default::default(), |ui| {
                ui.checkbox(&mut false, "box");
            });
            output.textures_delta.clear();

            assert!(
                output.shapes.iter().any(
                    |clipped| matches!(&clipped.shape, egui::Shape::Rect(rect) if rect.fill == fill)
                ),
                "{theme:?}"
            );
        }
    }
}
