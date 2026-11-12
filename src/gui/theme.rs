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

    visuals
}
