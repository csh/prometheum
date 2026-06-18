pub mod unreal_parser;

slint::include_modules!();

fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let window = LithiumWindow::new().expect("failed to create main window");

    window.run().expect("failed to run application");
}
