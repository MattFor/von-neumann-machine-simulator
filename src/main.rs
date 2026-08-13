mod gui;
mod app;
mod program;
mod machine;
mod debugger;
mod utilities;

fn main() /*-> eframe::Result<()> */ {
	let options = eframe::NativeOptions::default();

	eframe::run_native(
		"Von Neumann Machine Simulator",
		options,
		Box::new(|cc| {
			Ok(Box::new(app::VnmApp::new(cc)))
		})
	).expect("TODO: error message")
}
