use crate::fs::devices::{self, DeviceEntry};
use crate::i18n::tr;
use crate::ui::icon_widget::build_icon_image;
use gtk::gio;
use gtk::prelude::*;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum SidebarTarget {
    Path(PathBuf),
    Applications,
    Trash,
    Device(PathBuf),
}

impl PartialEq for SidebarTarget {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SidebarTarget::Path(a), SidebarTarget::Path(b)) => a == b,
            (SidebarTarget::Applications, SidebarTarget::Applications) => true,
            (SidebarTarget::Trash, SidebarTarget::Trash) => true,
            (SidebarTarget::Device(a), SidebarTarget::Device(b)) => a == b,
            _ => false,
        }
    }
}

impl SidebarTarget {
    pub fn writable_path(&self) -> Option<PathBuf> {
        match self {
            SidebarTarget::Path(p) => Some(p.clone()),
            SidebarTarget::Device(p) => Some(p.clone()),
            SidebarTarget::Applications | SidebarTarget::Trash => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SidebarEntryDescriptor {
    pub label_key: &'static str,
    pub icon_name: &'static str,
    pub target: SidebarTarget,
}

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
}

fn favorites_entries() -> Vec<SidebarEntryDescriptor> {
    let home = home_dir();
    vec![
        SidebarEntryDescriptor {
            label_key: "sidebar.home",
            icon_name: "user-home",
            target: SidebarTarget::Path(home.clone()),
        },
        SidebarEntryDescriptor {
            label_key: "sidebar.desktop",
            icon_name: "user-desktop",
            target: SidebarTarget::Path(home.join("Desktop")),
        },
        SidebarEntryDescriptor {
            label_key: "sidebar.documents",
            icon_name: "folder-documents",
            target: SidebarTarget::Path(home.join("Documents")),
        },
        SidebarEntryDescriptor {
            label_key: "sidebar.downloads",
            icon_name: "folder-download",
            target: SidebarTarget::Path(home.join("Downloads")),
        },
        SidebarEntryDescriptor {
            label_key: "sidebar.pictures",
            icon_name: "folder-pictures",
            target: SidebarTarget::Path(home.join("Pictures")),
        },
        SidebarEntryDescriptor {
            label_key: "sidebar.music",
            icon_name: "folder-music",
            target: SidebarTarget::Path(home.join("Music")),
        },
        SidebarEntryDescriptor {
            label_key: "sidebar.videos",
            icon_name: "folder-videos",
            target: SidebarTarget::Path(home.join("Videos")),
        },
        SidebarEntryDescriptor {
            label_key: "sidebar.applications",
            icon_name: "applications-system",
            target: SidebarTarget::Applications,
        },
    ]
}

fn locations_entries() -> Vec<SidebarEntryDescriptor> {
    vec![
        SidebarEntryDescriptor {
            label_key: "sidebar.filesystem",
            icon_name: "drive-harddisk",
            target: SidebarTarget::Path(PathBuf::from("/")),
        },
        SidebarEntryDescriptor {
            label_key: "sidebar.trash",
            icon_name: "user-trash",
            target: SidebarTarget::Trash,
        },
    ]
}

fn build_section(
    title_key: &str,
    entries: Vec<SidebarEntryDescriptor>,
    on_select: impl Fn(SidebarTarget) + 'static,
) -> gtk::Box {
    let section_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    section_box.set_margin_top(8);

    let title_label = gtk::Label::new(Some(&tr(title_key)));
    title_label.set_halign(gtk::Align::Start);
    title_label.set_margin_start(12);
    title_label.set_margin_bottom(2);
    title_label.add_css_class("caption-heading");
    title_label.add_css_class("dim-label");
    section_box.append(&title_label);

    let list_box = gtk::ListBox::new();
    list_box.set_selection_mode(gtk::SelectionMode::Single);
    list_box.add_css_class("boxed-list");
    list_box.set_margin_start(8);
    list_box.set_margin_end(8);

    for entry in entries {
        let row = gtk::ListBoxRow::new();
        let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        row_box.set_margin_top(6);
        row_box.set_margin_bottom(6);
        row_box.set_margin_start(8);
        row_box.set_margin_end(8);

        let icon = build_icon_image(entry.icon_name, 18);
        row_box.append(&icon);

        let label = gtk::Label::new(Some(&tr(entry.label_key)));
        label.set_halign(gtk::Align::Start);
        row_box.append(&label);

        row.set_child(Some(&row_box));

        let target = entry.target.clone();
        row.set_widget_name(&sidebar_target_to_widget_name(&target));
        list_box.append(&row);
    }

    list_box.connect_row_activated(move |_list, row| {
        let name = row.widget_name();
        if let Some(target) = widget_name_to_sidebar_target(&name) {
            on_select(target);
        }
    });

    section_box.append(&list_box);
    section_box
}

fn sidebar_target_to_widget_name(target: &SidebarTarget) -> String {
    match target {
        SidebarTarget::Path(p) => format!("path::{}", p.to_string_lossy()),
        SidebarTarget::Applications => "applications".to_string(),
        SidebarTarget::Trash => "trash".to_string(),
        SidebarTarget::Device(p) => format!("device::{}", p.to_string_lossy()),
    }
}

fn widget_name_to_sidebar_target(name: &str) -> Option<SidebarTarget> {
    if name == "applications" {
        return Some(SidebarTarget::Applications);
    }
    if name == "trash" {
        return Some(SidebarTarget::Trash);
    }
    if let Some(rest) = name.strip_prefix("device::") {
        return Some(SidebarTarget::Device(PathBuf::from(rest)));
    }
    if let Some(rest) = name.strip_prefix("path::") {
        return Some(SidebarTarget::Path(PathBuf::from(rest)));
    }
    None
}

fn build_devices_section(
    on_select: std::rc::Rc<dyn Fn(SidebarTarget)>,
) -> (gtk::Box, gio::VolumeMonitor) {
    let section_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    section_box.set_margin_top(8);

    let title_label = gtk::Label::new(Some(&tr("sidebar.devices")));
    title_label.set_halign(gtk::Align::Start);
    title_label.set_margin_start(12);
    title_label.set_margin_bottom(2);
    title_label.add_css_class("caption-heading");
    title_label.add_css_class("dim-label");
    section_box.append(&title_label);

    let list_box = gtk::ListBox::new();
    list_box.set_selection_mode(gtk::SelectionMode::None);
    list_box.add_css_class("boxed-list");
    list_box.set_margin_start(8);
    list_box.set_margin_end(8);
    list_box.set_visible(false);

    section_box.append(&list_box);

    let list_box_for_refresh = list_box.clone();
    let section_box_for_refresh = section_box.clone();
    let refresh = std::rc::Rc::new(move || {
        while let Some(child) = list_box_for_refresh.first_child() {
            list_box_for_refresh.remove(&child);
        }

        let device_list: Vec<DeviceEntry> = devices::list_devices();
        section_box_for_refresh.set_visible(!device_list.is_empty());
        list_box_for_refresh.set_visible(!device_list.is_empty());

        for device in device_list {
            let row = gtk::ListBoxRow::new();
            row.set_selectable(false);
            row.set_activatable(false);

            let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
            row_box.set_margin_top(6);
            row_box.set_margin_bottom(6);
            row_box.set_margin_start(8);
            row_box.set_margin_end(8);

            let icon = build_icon_image("drive-removable-media", 18);
            row_box.append(&icon);

            let label = gtk::Label::new(Some(&device.display_name));
            label.set_halign(gtk::Align::Start);
            label.set_hexpand(true);
            label.set_ellipsize(pango::EllipsizeMode::End);
            row_box.append(&label);

            if device.is_mounted {
                if let Some(mount_path) = device.mount_path.clone() {
                    let click_gesture = gtk::GestureClick::new();
                    let on_select_for_click = on_select.clone();
                    let mount_path_for_click = mount_path.clone();
                    click_gesture.connect_released(move |_gesture, _n, _x, _y| {
                        on_select_for_click(SidebarTarget::Device(mount_path_for_click.clone()));
                    });
                    row_box.add_controller(click_gesture);
                }

                if device.can_eject {
                    let eject_button = gtk::Button::from_icon_name("media-eject-symbolic");
                    eject_button.add_css_class("flat");
                    eject_button.set_valign(gtk::Align::Center);
                    let volume_for_eject = device.volume.clone();
                    eject_button.connect_clicked(move |_| {
                        devices::eject_volume(&volume_for_eject, |result| {
                            if let Err(err) = result {
                                log::warn!("failed to eject volume: {err}");
                            }
                        });
                    });
                    row_box.append(&eject_button);
                }
            } else {
                let mount_button = gtk::Button::from_icon_name("media-playback-start-symbolic");
                mount_button.add_css_class("flat");
                mount_button.set_valign(gtk::Align::Center);
                mount_button.set_tooltip_text(Some(&tr("toolbar.reload")));
                let volume_for_mount = device.volume.clone();
                mount_button.connect_clicked(move |_| {
                    devices::mount_volume(&volume_for_mount, |result| {
                        if let Err(err) = result {
                            log::warn!("failed to mount volume: {err}");
                        }
                    });
                });
                row_box.append(&mount_button);
            }

            row.set_child(Some(&row_box));
            list_box_for_refresh.append(&row);
        }
    });

    refresh();

    let refresh_for_monitor = refresh.clone();
    let monitor = devices::watch_device_changes(move || {
        refresh_for_monitor();
    });

    (section_box, monitor)
}

pub fn build_sidebar_content(
    on_select: impl Fn(SidebarTarget) + 'static,
) -> (gtk::Widget, gio::VolumeMonitor) {
    let on_select: std::rc::Rc<dyn Fn(SidebarTarget)> = std::rc::Rc::new(on_select);

    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scrolled.set_hexpand(true);
    scrolled.set_vexpand(true);

    let outer_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    outer_box.set_margin_bottom(16);

    let favorites_on_select = on_select.clone();
    let favorites_section = build_section("sidebar.favorites", favorites_entries(), move |target| {
        favorites_on_select(target);
    });
    outer_box.append(&favorites_section);

    let devices_on_select = on_select.clone();
    let (devices_section, volume_monitor) = build_devices_section(devices_on_select);
    outer_box.append(&devices_section);

    let locations_on_select = on_select.clone();
    let locations_section = build_section("sidebar.locations", locations_entries(), move |target| {
        locations_on_select(target);
    });
    outer_box.append(&locations_section);

    scrolled.set_child(Some(&outer_box));
    (scrolled.upcast(), volume_monitor)
}


