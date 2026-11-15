mod app;
mod assembler;
mod cli;
mod debugger;
mod gui;
mod machine;
mod program;
mod utilities;

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if !args.is_empty() {
        std::process::exit(cli::main(&args));
    }

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([854.0, 480.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Von Neumann Machine Simulator",
        options,
        Box::new(|cc| Ok(Box::new(app::VnmApp::new(cc)))),
    )
}
