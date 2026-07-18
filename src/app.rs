use crate::i18n;
use crate::ui::window::FasdManagerWindow;
use adw::prelude::*;
use gio::ApplicationFlags;
use once_cell::sync::OnceCell;

pub const APP_ID: &str = "org.fasd.manager";

static TOKIO_RUNTIME: OnceCell<tokio::runtime::Runtime> = OnceCell::new();

pub fn build_and_run() -> glib::ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    i18n::init();

    let cpu_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2);

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(cpu_count.clamp(2, 8))
        .max_blocking_threads(32)
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");
    TOKIO_RUNTIME
        .set(runtime)
        .unwrap_or_else(|_| panic!("tokio runtime already initialized"));
    let stored_runtime = TOKIO_RUNTIME.get().expect("runtime just set");
    let enter_guard = stored_runtime.enter();
    std::mem::forget(enter_guard);

    let application = adw::Application::builder()
        .application_id(APP_ID)
        .flags(ApplicationFlags::default())
        .build();

    application.connect_activate(move |app| {
        let window = FasdManagerWindow::new(app);
        window.present();
    });

    application.run()
}
