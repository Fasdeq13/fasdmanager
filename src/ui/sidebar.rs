use crate::fs::devices::{self, DeviceEntry, UnmountProgressEvent};
use crate::fs::settings::{FavoriteItem, Settings, SidebarSection};
use crate::fs::xdg_dirs::{resolve_xdg_user_dir, XdgUserDir};
use crate::i18n::tr;
use crate::ui::icon_widget::build_icon_image;
use crate::util::format::format_size;
use gtk::gio;
use gtk::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

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
struct SidebarEntryDescriptor {
    label_key: &'static str,
    icon_name: &'static str,
    target: SidebarTarget,
    favorite_item: Option<FavoriteItem>,
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
            favorite_item: Some(FavoriteItem::Home),
        },
        SidebarEntryDescriptor {
            label_key: "sidebar.desktop",
            icon_name: "user-desktop",
            target: SidebarTarget::Path(resolve_xdg_user_dir(XdgUserDir::Desktop)),
            favorite_item: Some(FavoriteItem::Desktop),
        },
        SidebarEntryDescriptor {
            label_key: "sidebar.documents",
            icon_name: "folder-documents",
            target: SidebarTarget::Path(resolve_xdg_user_dir(XdgUserDir::Documents)),
            favorite_item: Some(FavoriteItem::Documents),
        },
        SidebarEntryDescriptor {
            label_key: "sidebar.downloads",
            icon_name: "folder-download",
            target: SidebarTarget::Path(resolve_xdg_user_dir(XdgUserDir::Download)),
            favorite_item: Some(FavoriteItem::Downloads),
        },
        SidebarEntryDescriptor {
            label_key: "sidebar.pictures",
            icon_name: "folder-pictures",
            target: SidebarTarget::Path(resolve_xdg_user_dir(XdgUserDir::Pictures)),
            favorite_item: Some(FavoriteItem::Pictures),
        },
        SidebarEntryDescriptor {
            label_key: "sidebar.music",
            icon_name: "folder-music",
            target: SidebarTarget::Path(resolve_xdg_user_dir(XdgUserDir::Music)),
            favorite_item: Some(FavoriteItem::Music),
        },
        SidebarEntryDescriptor {
            label_key: "sidebar.videos",
            icon_name: "folder-videos",
            target: SidebarTarget::Path(resolve_xdg_user_dir(XdgUserDir::Videos)),
            favorite_item: Some(FavoriteItem::Videos),
        },
        SidebarEntryDescriptor {
            label_key: "sidebar.applications",
            icon_name: "view-app-grid-symbolic",
            target: SidebarTarget::Applications,
            favorite_item: Some(FavoriteItem::Applications),
        },
    ]
}

fn locations_entries() -> Vec<SidebarEntryDescriptor> {
    vec![
        SidebarEntryDescriptor {
            label_key: "sidebar.filesystem",
            icon_name: "drive-harddisk",
            target: SidebarTarget::Path(PathBuf::from("/")),
            favorite_item: None,
        },
        SidebarEntryDescriptor {
            label_key: "sidebar.trash",
            icon_name: "user-trash",
            target: SidebarTarget::Trash,
            favorite_item: None,
        },
    ]
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

fn attach_hide_row_menu(
    row_widget: &impl IsA<gtk::Widget>,
    item: FavoriteItem,
    settings: Rc<RefCell<Settings>>,
    on_changed: Rc<dyn Fn()>,
) {
    let click_gesture = gtk::GestureClick::new();
    click_gesture.set_button(3);

    let row_widget_for_menu = row_widget.clone().upcast::<gtk::Widget>();
    click_gesture.connect_pressed(move |_gesture, _n_press, x, y| {
        let menu = gio::Menu::new();
        menu.append(Some(&tr("sidebar.hide_item")), Some("sidebar-row.hide"));

        let action_group = gio::SimpleActionGroup::new();
        let settings = settings.clone();
        let on_changed = on_changed.clone();
        let hide_action = gio::SimpleAction::new("hide", None);
        hide_action.connect_activate(move |_, _| {
            settings.borrow_mut().set_favorite_visible(item, false);
            let _ = settings.borrow().save();
            on_changed();
        });
        action_group.add_action(&hide_action);

        row_widget_for_menu.insert_action_group("sidebar-row", Some(&action_group));

        let popover = gtk::PopoverMenu::from_model(Some(&menu));
        popover.set_parent(&row_widget_for_menu);
        popover.set_pointing_to(Some(&gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
        popover.popup();
    });

    row_widget.add_controller(click_gesture);
}

fn build_row(entry: &SidebarEntryDescriptor) -> (gtk::ListBoxRow, gtk::Box) {
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
    row.set_widget_name(&sidebar_target_to_widget_name(&entry.target));

    (row, row_box)
}

fn build_section(
    title_key: &str,
    entries: Vec<SidebarEntryDescriptor>,
    settings: Rc<RefCell<Settings>>,
    on_select: Rc<dyn Fn(SidebarTarget)>,
    on_settings_changed: Rc<dyn Fn()>,
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

    let mut visible_count = 0;

    for entry in &entries {
        if let Some(item) = entry.favorite_item {
            if !settings.borrow().is_favorite_visible(item) {
                continue;
            }
        }

        let (row, row_box) = build_row(entry);

        if let Some(item) = entry.favorite_item {
            attach_hide_row_menu(&row_box, item, settings.clone(), on_settings_changed.clone());
        }

        list_box.append(&row);
        visible_count += 1;
    }

    section_box.set_visible(visible_count > 0);

    let on_select_for_activate = on_select.clone();
    list_box.connect_row_activated(move |_list, row| {
        let name = row.widget_name();
        if let Some(target) = widget_name_to_sidebar_target(&name) {
            on_select_for_activate(target);
        }
    });

    section_box.append(&list_box);
    section_box
}

fn unmount_status_widget() -> (gtk::Box, gtk::Spinner, gtk::Label) {
    let status_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let spinner = gtk::Spinner::new();
    spinner.set_spinning(true);
    let label = gtk::Label::new(Some(&tr("devices.unmounting")));
    label.add_css_class("caption");
    label.add_css_class("dim-label");
    label.set_ellipsize(pango::EllipsizeMode::End);
    status_box.append(&spinner);
    status_box.append(&label);
    (status_box, spinner, label)
}

fn build_devices_section(
    on_select: Rc<dyn Fn(SidebarTarget)>,
    settings: Rc<RefCell<Settings>>,
) -> (gtk::Box, gio::VolumeMonitor) {
    let section_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    section_box.set_margin_top(8);
    section_box.set_visible(settings.borrow().is_sidebar_section_visible(SidebarSection::Devices));

    let title_label = gtk::Label::new(Some(&tr("sidebar.devices")));
    title_label.set_halign(gtk::Align::Start);
    title_label.set_margin_start(12);
    title_label.set_margin_bottom(2);
    title_label.add_css_class("caption-heading");
    title_label.add_css_class("dim-label");
    section_box.append(&title_label);

    {
        let click_gesture = gtk::GestureClick::new();
        click_gesture.set_button(3);
        let title_label_for_menu = title_label.clone().upcast::<gtk::Widget>();
        let settings_for_hide = settings.clone();
        let section_box_for_hide = section_box.clone();
        click_gesture.connect_pressed(move |_gesture, _n_press, x, y| {
            let menu = gio::Menu::new();
            menu.append(Some(&tr("sidebar.hide_section")), Some("sidebar-section.hide"));

            let action_group = gio::SimpleActionGroup::new();
            let settings_for_action = settings_for_hide.clone();
            let section_box_for_action = section_box_for_hide.clone();
            let hide_action = gio::SimpleAction::new("hide", None);
            hide_action.connect_activate(move |_, _| {
                settings_for_action
                    .borrow_mut()
                    .set_sidebar_section_visible(SidebarSection::Devices, false);
                let _ = settings_for_action.borrow().save();
                section_box_for_action.set_visible(false);
            });
            action_group.add_action(&hide_action);

            title_label_for_menu.insert_action_group("sidebar-section", Some(&action_group));

            let popover = gtk::PopoverMenu::from_model(Some(&menu));
            popover.set_parent(&title_label_for_menu);
            popover.set_pointing_to(Some(&gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
            popover.popup();
        });
        title_label.add_controller(click_gesture);
    }

    let list_box = gtk::ListBox::new();
    list_box.set_selection_mode(gtk::SelectionMode::None);
    list_box.add_css_class("boxed-list");
    list_box.set_margin_start(8);
    list_box.set_margin_end(8);
    list_box.set_visible(false);

    section_box.append(&list_box);

    let list_box_for_refresh = list_box.clone();
    let section_box_for_refresh = section_box.clone();
    let settings_for_refresh = settings.clone();
    let refresh = Rc::new(move || {
        while let Some(child) = list_box_for_refresh.first_child() {
            list_box_for_refresh.remove(&child);
        }

        let section_visible =
            settings_for_refresh.borrow().is_sidebar_section_visible(SidebarSection::Devices);

        let device_list: Vec<DeviceEntry> = devices::list_devices();
        section_box_for_refresh.set_visible(section_visible && !device_list.is_empty());
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

            let action_slot = gtk::Box::new(gtk::Orientation::Horizontal, 4);
            row_box.append(&action_slot);

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

                if device.can_unmount {
                    let unmount_button = gtk::Button::from_icon_name("media-eject-symbolic");
                    unmount_button.add_css_class("flat");
                    unmount_button.set_valign(gtk::Align::Center);
                    unmount_button.set_tooltip_text(Some(&tr("devices.unmount")));

                    let volume_for_unmount = device.volume.clone();
                    let display_name_for_unmount = device.display_name.clone();
                    let action_slot_for_unmount = action_slot.clone();
                    let unmount_button_for_click = unmount_button.clone();

                    unmount_button.connect_clicked(move |_| {
                        let (status_box, _spinner, status_label) = unmount_status_widget();
                        action_slot_for_unmount.remove(&unmount_button_for_click);
                        action_slot_for_unmount.append(&status_box);

                        let status_label_for_progress = status_label.clone();
                        let display_name_for_progress = display_name_for_unmount.clone();

                        devices::unmount_volume_with_progress(
                            &volume_for_unmount,
                            move |event| match event {
                                UnmountProgressEvent::Started => {
                                    status_label_for_progress
                                        .set_text(&tr("devices.unmounting"));
                                }
                                UnmountProgressEvent::Flushing { bytes_left } => {
                                    if bytes_left > 0 {
                                        status_label_for_progress.set_text(
                                            &crate::i18n::tr_with(
                                                "devices.unmount_flushing_bytes",
                                                &format_size(bytes_left as u64),
                                            ),
                                        );
                                    } else {
                                        status_label_for_progress
                                            .set_text(&tr("devices.unmount_flushing"));
                                    }
                                }
                                UnmountProgressEvent::Finished(result) => match result {
                                    Ok(()) => {
                                        log::info!(
                                            "unmounted {display_name_for_progress} successfully"
                                        );
                                    }
                                    Err(err) => {
                                        log::warn!(
                                            "failed to unmount {display_name_for_progress}: {err}"
                                        );
                                    }
                                },
                            },
                        );
                    });

                    action_slot.append(&unmount_button);
                }

                if device.can_eject {
                    let eject_button = gtk::Button::from_icon_name("media-eject-symbolic");
                    eject_button.set_visible(!device.can_unmount);
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
                    action_slot.append(&eject_button);
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
                action_slot.append(&mount_button);
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
    let on_select: Rc<dyn Fn(SidebarTarget)> = Rc::new(on_select);
    let settings: Rc<RefCell<Settings>> = Rc::new(RefCell::new(Settings::load()));

    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scrolled.set_hexpand(true);
    scrolled.set_vexpand(true);

    let outer_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    outer_box.set_margin_bottom(16);
    scrolled.set_child(Some(&outer_box));

    let favorites_slot = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let (devices_section, volume_monitor) =
        build_devices_section(on_select.clone(), settings.clone());
    let locations_slot = gtk::Box::new(gtk::Orientation::Vertical, 0);

    outer_box.append(&favorites_slot);
    outer_box.append(&devices_section);
    outer_box.append(&locations_slot);

    let rebuild_favorites_locations: Rc<RefCell<Option<Rc<dyn Fn()>>>> =
        Rc::new(RefCell::new(None));
    let rebuild_for_closure = rebuild_favorites_locations.clone();

    let favorites_slot_for_rebuild = favorites_slot.clone();
    let locations_slot_for_rebuild = locations_slot.clone();
    let on_select_for_rebuild = on_select.clone();
    let settings_for_rebuild = settings.clone();

    let rebuild_fn: Rc<dyn Fn()> = Rc::new(move || {
        while let Some(child) = favorites_slot_for_rebuild.first_child() {
            favorites_slot_for_rebuild.remove(&child);
        }
        while let Some(child) = locations_slot_for_rebuild.first_child() {
            locations_slot_for_rebuild.remove(&child);
        }

        let on_changed = rebuild_for_closure
            .borrow()
            .clone()
            .expect("rebuild closure must be set before first use");

        let favorites_section = build_section(
            "sidebar.favorites",
            favorites_entries(),
            settings_for_rebuild.clone(),
            on_select_for_rebuild.clone(),
            on_changed.clone(),
        );
        favorites_slot_for_rebuild.append(&favorites_section);

        let locations_section = build_section(
            "sidebar.locations",
            locations_entries(),
            settings_for_rebuild.clone(),
            on_select_for_rebuild.clone(),
            on_changed,
        );
        locations_slot_for_rebuild.append(&locations_section);
    });

    *rebuild_favorites_locations.borrow_mut() = Some(rebuild_fn.clone());
    rebuild_fn();

    (scrolled.upcast(), volume_monitor)
}
