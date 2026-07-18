mod app;
mod fs;
mod i18n;
mod ui;
mod util;

fn main() -> glib::ExitCode {
    app::build_and_run()
}
