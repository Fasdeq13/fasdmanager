use crate::fs::progress_copy::{start_copy_with_progress, CancelToken, CopyEvent};
use crate::i18n::tr;
use crate::util::format::format_size;
use adw::prelude::*;
use gtk::prelude::*;
use std::path::PathBuf;

pub fn spawn_copy_with_dialog(
    parent_window: &impl IsA<gtk::Window>,
    src: PathBuf,
    dst_dir: PathBuf,
    on_finished: impl Fn(PathBuf) + 'static,
) {
    let source_name = src
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| src.to_string_lossy().to_string());

    let title_label = gtk::Label::new(Some(&format!("{}: {}", tr("toolbar.new_tab"), source_name)));
    title_label.set_wrap(true);
    title_label.set_xalign(0.0);
    title_label.add_css_class("title-4");

    let current_file_label = gtk::Label::new(Some(""));
    current_file_label.set_xalign(0.0);
    current_file_label.set_ellipsize(pango::EllipsizeMode::Middle);
    current_file_label.add_css_class("dim-label");

    let progress_bar = gtk::ProgressBar::new();
    progress_bar.set_show_text(true);
    progress_bar.set_fraction(0.0);

    let stats_label = gtk::Label::new(Some(""));
    stats_label.set_xalign(0.0);
    stats_label.add_css_class("caption");
    stats_label.add_css_class("dim-label");

    let cancel_button = gtk::Button::with_label(&tr("dialog.cancel"));
    cancel_button.add_css_class("destructive-action");
    cancel_button.set_halign(gtk::Align::End);

    let content_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(20);
    content_box.set_margin_end(20);
    content_box.append(&title_label);
    content_box.append(&current_file_label);
    content_box.append(&progress_bar);
    content_box.append(&stats_label);
    content_box.append(&cancel_button);

    let dialog = adw::Dialog::builder()
        .content_width(440)
        .content_height(-1)
        .child(&content_box)
        .can_close(false)
        .build();

    dialog.present(Some(parent_window.upcast_ref::<gtk4::Widget>()));

    let (mut rx, cancel_token) = start_copy_with_progress(src, dst_dir);

    let cancel_token_for_button: CancelToken = cancel_token.clone();
    cancel_button.connect_clicked(move |_| {
        cancel_token_for_button.cancel();
    });

    let dialog_for_task = dialog.clone();

    glib::spawn_future_local(async move {
        while let Some(event) = rx.recv().await {
            match event {
                CopyEvent::Planning => {
                    stats_label.set_text(&tr("status.loading"));
                }
                CopyEvent::Started {
                    total_bytes: tb,
                    total_files: tf,
                } => {
                    stats_label.set_text(&format!("0 / {} • 0 / {}", tf, format_size(tb)));
                }
                CopyEvent::FileStarted { name } => {
                    current_file_label.set_text(&name);
                }
                CopyEvent::Progress {
                    bytes_done,
                    total_bytes: tb,
                    files_done,
                    total_files: tf,
                    bytes_per_second,
                } => {
                    let fraction = if tb > 0 {
                        bytes_done as f64 / tb as f64
                    } else {
                        0.0
                    };
                    progress_bar.set_fraction(fraction.clamp(0.0, 1.0));
                    let speed_text = format!("{}/s", format_size(bytes_per_second.max(0.0) as u64));
                    stats_label.set_text(&format!(
                        "{} / {} • {} / {} • {}",
                        files_done,
                        tf,
                        format_size(bytes_done),
                        format_size(tb),
                        speed_text
                    ));
                }
                CopyEvent::FileFinished => {}
                CopyEvent::Finished { destination } => {
                    dialog_for_task.force_close();
                    on_finished(destination);
                    break;
                }
                CopyEvent::Cancelled => {
                    dialog_for_task.force_close();
                    break;
                }
                CopyEvent::Failed { message } => {
                    current_file_label.set_text(&message);
                    progress_bar.add_css_class("error");
                    cancel_button.set_label(&tr("dialog.close"));
                    let dialog_to_close = dialog_for_task.clone();
                    cancel_button.connect_clicked(move |_| {
                        dialog_to_close.force_close();
                    });
                    break;
                }
            }
        }
    });
}
