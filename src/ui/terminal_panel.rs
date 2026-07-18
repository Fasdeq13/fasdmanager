use crate::i18n::tr;
use gtk::prelude::*;
use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;
use vte4::prelude::*;

pub struct TerminalPanel {
    pub root_widget: gtk::Widget,
    pub terminal: vte4::Terminal,
    pub revealer: gtk::Revealer,
    shell_spawned: Cell<bool>,
}

fn detect_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
}

impl TerminalPanel {
    pub fn new() -> Rc<Self> {
        let terminal = vte4::Terminal::new();
        terminal.set_vexpand(true);
        terminal.set_hexpand(true);
        terminal.set_scrollback_lines(10_000);
        terminal.set_cursor_blink_mode(vte4::CursorBlinkMode::On);
        terminal.set_font_scale(1.0);

        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scrolled.set_child(Some(&terminal));
        scrolled.set_min_content_height(220);

        let header_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        header_box.set_margin_start(10);
        header_box.set_margin_end(6);
        header_box.set_margin_top(4);
        header_box.set_margin_bottom(4);

        let title_label = gtk::Label::new(Some(&tr("terminal.title")));
        title_label.add_css_class("caption-heading");
        title_label.set_halign(gtk::Align::Start);
        title_label.set_hexpand(true);
        header_box.append(&title_label);

        let close_button = gtk::Button::from_icon_name("window-close-symbolic");
        close_button.add_css_class("flat");
        close_button.set_tooltip_text(Some(&tr("terminal.close")));
        header_box.append(&close_button);

        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        content_box.add_css_class("fasdmanager-terminal-panel");
        content_box.append(&header_box);
        content_box.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        content_box.append(&scrolled);

        let revealer = gtk::Revealer::new();
        revealer.set_transition_type(gtk::RevealerTransitionType::SlideUp);
        revealer.set_transition_duration(180);
        revealer.set_child(Some(&content_box));
        revealer.set_reveal_child(false);

        let panel = Rc::new(TerminalPanel {
            root_widget: revealer.clone().upcast(),
            terminal,
            revealer,
            shell_spawned: Cell::new(false),
        });

        let revealer_for_close = panel.revealer.clone();
        close_button.connect_clicked(move |_| {
            revealer_for_close.set_reveal_child(false);
        });

        panel
    }

    pub fn toggle(self: &Rc<Self>) {
        let currently_visible = self.revealer.reveals_child();
        self.revealer.set_reveal_child(!currently_visible);
    }

    pub fn is_visible(self: &Rc<Self>) -> bool {
        self.revealer.reveals_child()
    }

    pub fn spawn_shell_in(self: &Rc<Self>, working_directory: PathBuf) {
        let shell = detect_shell();
        let dir_str = working_directory.to_string_lossy().to_string();

        self.shell_spawned.set(true);

        self.terminal.spawn_async(
            vte4::PtyFlags::DEFAULT,
            Some(&dir_str),
            &[&shell],
            &[],
            glib::SpawnFlags::DEFAULT,
            || {},
            -1,
            None::<&gio::Cancellable>,
            move |result| {
                if let Err(err) = result {
                    log::warn!("failed to spawn terminal shell: {err}");
                }
            },
        );
    }

    pub fn ensure_shell_spawned(self: &Rc<Self>, working_directory: PathBuf) {
        if !self.shell_spawned.get() {
            self.spawn_shell_in(working_directory);
        }
    }

    pub fn feed_command_text(self: &Rc<Self>, command_text: &str) {
        self.terminal.feed_child(command_text.as_bytes());
    }
}
