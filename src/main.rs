mod app;
mod debugger;
mod gui;
mod machine;
mod program;
mod utilities;

fn main() /*-> eframe::Result<()> */
{
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_min_inner_size([854.0, 480.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Von Neumann Machine Simulator",
        options,
        Box::new(|cc| Ok(Box::new(app::VnmApp::new(cc)))),
    )
    .expect("TODO: error message")
}
