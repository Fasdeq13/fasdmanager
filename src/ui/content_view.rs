use crate::fs::applications::{launch_application, load_applications_listing};
use crate::fs::distro_detect::detect_package_manager;
use crate::fs::entry::{sort_entries, EntryKind, FileEntry};
use crate::fs::package_owner::find_owning_package_async;
use crate::fs::package_uninstall::{build_uninstall_command, is_running_as_root, run_uninstall_async};
use crate::fs::reader::read_dir_async;
use crate::fs::{ops, trash_ops};
use crate::i18n::tr;
use crate::ui::app_state::SharedAppState;
use crate::ui::copy_progress_dialog::spawn_copy_with_dialog;
use crate::ui::file_entry_object::FileEntryObject;
use crate::ui::icon_widget::build_icon_image_for_widget_scale;
use crate::ui::sidebar::SidebarTarget;
use adw::prelude::*;
use gtk::prelude::*;
use gtk::{gdk, gio, glib};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

const GRID_ICON_SIZE: i32 = 48;
const LIST_ICON_SIZE: i32 = 22;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ContentSource {
    Directory,
    Applications,
    Trash,
}

pub struct ContentPane {
    pub root_widget: gtk::Widget,
    pub toast_overlay: adw::ToastOverlay,
    pub list_store: gio::ListStore,
    pub selection_model: gtk::MultiSelection,
    pub current_target: Rc<RefCell<SidebarTarget>>,
    pub current_source: Rc<RefCell<ContentSource>>,
    pub path_bar_label: gtk::Label,
    pub status_label: gtk::Label,
    pub back_history: Rc<RefCell<Vec<SidebarTarget>>>,
    pub forward_history: Rc<RefCell<Vec<SidebarTarget>>>,
    pub nav_back_button: gtk::Button,
    pub nav_forward_button: gtk::Button,
    pub empty_trash_button: gtk::Button,
    pub view_stack: gtk::Stack,
    app_state: SharedAppState,
    window: gtk::Window,
    terminal_panel: Rc<crate::ui::terminal_panel::TerminalPanel>,
}

fn target_to_content_source(target: &SidebarTarget) -> ContentSource {
    match target {
        SidebarTarget::Path(_) => ContentSource::Directory,
        SidebarTarget::Device(_) => ContentSource::Directory,
        SidebarTarget::Applications => ContentSource::Applications,
        SidebarTarget::Trash => ContentSource::Trash,
    }
}

fn resolve_real_binary_path(desktop_entry_path: &std::path::Path) -> PathBuf {
    let Some(entry) = crate::fs::desktop_entry::parse_desktop_file(
        desktop_entry_path,
        crate::i18n::current_lang().code(),
    ) else {
        return desktop_entry_path.to_path_buf();
    };

    let Some(exec) = entry.exec else {
        return desktop_entry_path.to_path_buf();
    };

    let Some(binary_name) = exec.split_whitespace().next() else {
        return desktop_entry_path.to_path_buf();
    };

    if binary_name.starts_with('/') {
        return PathBuf::from(binary_name);
    }

    let path_var = std::env::var("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(binary_name);
        if candidate.is_file() {
            return candidate;
        }
    }

    desktop_entry_path.to_path_buf()
}

fn build_grid_item_widget() -> (gtk::Box, gtk::Image, gtk::Label) {
    let item_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
    item_box.set_margin_top(8);
    item_box.set_margin_bottom(8);
    item_box.set_margin_start(6);
    item_box.set_margin_end(6);
    item_box.set_halign(gtk::Align::Center);

    let icon = gtk::Image::new();
    icon.set_pixel_size(GRID_ICON_SIZE);
    icon.set_halign(gtk::Align::Center);
    item_box.append(&icon);

    let label = gtk::Label::new(None);
    label.set_wrap(true);
    label.set_wrap_mode(pango::WrapMode::WordChar);
    label.set_justify(gtk::Justification::Center);
    label.set_lines(2);
    label.set_ellipsize(pango::EllipsizeMode::End);
    label.set_max_width_chars(14);
    label.set_halign(gtk::Align::Center);
    item_box.append(&label);

    (item_box, icon, label)
}

impl ContentPane {
    pub fn new(
        app_state: SharedAppState,
        window: gtk::Window,
        terminal_panel: Rc<crate::ui::terminal_panel::TerminalPanel>,
    ) -> Rc<Self> {
        let list_store = gio::ListStore::new::<FileEntryObject>();
        let selection_model = gtk::MultiSelection::new(Some(list_store.clone()));

        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(move |_factory, list_item| {
            let list_item = list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("expected ListItem");
            let (item_box, _icon, _label) = build_grid_item_widget();
            list_item.set_child(Some(&item_box));
        });

        factory.connect_bind(move |_factory, list_item| {
            let list_item = list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("expected ListItem");
            let Some(item) = list_item.item() else {
                return;
            };
            let Ok(file_object) = item.downcast::<FileEntryObject>() else {
                return;
            };
            let Some(item_box) = list_item.child().and_then(|w| w.downcast::<gtk::Box>().ok())
            else {
                return;
            };

            let icon_widget = item_box
                .first_child()
                .and_then(|w| w.downcast::<gtk::Image>().ok());
            let label_widget = icon_widget
                .as_ref()
                .and_then(|icon| icon.next_sibling())
                .and_then(|w| w.downcast::<gtk::Label>().ok());

            if let Some(icon) = icon_widget {
                let scale_factor = icon.scale_factor();
                let rendered = build_icon_image_for_widget_scale(
                    &file_object.icon_name(),
                    GRID_ICON_SIZE,
                    scale_factor,
                );
                if let Some(paintable) = rendered.paintable() {
                    icon.set_paintable(Some(&paintable));
                } else {
                    icon.set_icon_name(Some(&file_object.icon_name()));
                }
            }
            if let Some(label) = label_widget {
                label.set_text(&file_object.display_name());
            }
        });

        let grid_view = gtk::GridView::new(Some(selection_model.clone()), Some(factory));
        grid_view.set_max_columns(16);
        grid_view.set_min_columns(2);
        grid_view.add_css_class("fasdmanager-grid");

        let list_factory = gtk::SignalListItemFactory::new();
        list_factory.connect_setup(move |_factory, list_item| {
            let list_item = list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("expected ListItem");

            let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            row_box.set_margin_top(4);
            row_box.set_margin_bottom(4);
            row_box.set_margin_start(8);
            row_box.set_margin_end(8);

            let icon = gtk::Image::new();
            icon.set_pixel_size(LIST_ICON_SIZE);
            row_box.append(&icon);

            let label = gtk::Label::new(None);
            label.set_halign(gtk::Align::Start);
            label.set_hexpand(true);
            label.set_ellipsize(pango::EllipsizeMode::Middle);
            row_box.append(&label);

            let size_label = gtk::Label::new(None);
            size_label.set_halign(gtk::Align::End);
            size_label.add_css_class("dim-label");
            size_label.add_css_class("caption");
            size_label.set_width_chars(10);
            row_box.append(&size_label);

            list_item.set_child(Some(&row_box));
        });

        list_factory.connect_bind(move |_factory, list_item| {
            let list_item = list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("expected ListItem");
            let Some(item) = list_item.item() else {
                return;
            };
            let Ok(file_object) = item.downcast::<FileEntryObject>() else {
                return;
            };
            let Some(row_box) = list_item.child().and_then(|w| w.downcast::<gtk::Box>().ok())
            else {
                return;
            };

            let icon_widget = row_box
                .first_child()
                .and_then(|w| w.downcast::<gtk::Image>().ok());
            let label_widget = icon_widget
                .as_ref()
                .and_then(|icon| icon.next_sibling())
                .and_then(|w| w.downcast::<gtk::Label>().ok());
            let size_widget = label_widget
                .as_ref()
                .and_then(|label| label.next_sibling())
                .and_then(|w| w.downcast::<gtk::Label>().ok());

            if let Some(icon) = icon_widget {
                let scale_factor = icon.scale_factor();
                let rendered = build_icon_image_for_widget_scale(
                    &file_object.icon_name(),
                    LIST_ICON_SIZE,
                    scale_factor,
                );
                if let Some(paintable) = rendered.paintable() {
                    icon.set_paintable(Some(&paintable));
                } else {
                    icon.set_icon_name(Some(&file_object.icon_name()));
                }
            }
            if let Some(label) = label_widget {
                label.set_text(&file_object.display_name());
            }
            if let Some(size_label) = size_widget {
                if file_object.kind() == EntryKind::Directory {
                    size_label.set_text("");
                } else {
                    size_label.set_text(&crate::util::format::format_size(file_object.size()));
                }
            }
        });

        let list_view = gtk::ListView::new(Some(selection_model.clone()), Some(list_factory));
        list_view.add_css_class("fasdmanager-list");

        let view_stack = gtk::Stack::new();
        view_stack.set_vexpand(true);
        view_stack.set_hexpand(true);
        view_stack.add_named(&grid_view, Some("grid"));
        view_stack.add_named(&list_view, Some("list"));
        view_stack.set_visible_child_name("grid");

        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scrolled.set_hexpand(true);
        scrolled.set_vexpand(true);
        scrolled.set_child(Some(&view_stack));

        let path_bar_label = gtk::Label::new(Some(""));
        path_bar_label.set_halign(gtk::Align::Start);
        path_bar_label.set_ellipsize(pango::EllipsizeMode::Middle);
        path_bar_label.add_css_class("heading");
        path_bar_label.set_margin_start(4);
        path_bar_label.set_margin_top(0);
        path_bar_label.set_hexpand(true);

        let status_label = gtk::Label::new(Some(""));
        status_label.set_halign(gtk::Align::Start);
        status_label.add_css_class("dim-label");
        status_label.add_css_class("caption");
        status_label.set_margin_start(12);
        status_label.set_margin_bottom(6);
        status_label.set_margin_top(2);

        let nav_back_button = gtk::Button::from_icon_name("go-previous-symbolic");
        nav_back_button.set_tooltip_text(Some(&tr("toolbar.back")));
        nav_back_button.add_css_class("flat");
        nav_back_button.set_sensitive(false);

        let nav_forward_button = gtk::Button::from_icon_name("go-next-symbolic");
        nav_forward_button.set_tooltip_text(Some(&tr("toolbar.forward")));
        nav_forward_button.add_css_class("flat");
        nav_forward_button.set_sensitive(false);

        let nav_up_button = gtk::Button::from_icon_name("go-up-symbolic");
        nav_up_button.set_tooltip_text(Some(&tr("toolbar.up")));
        nav_up_button.add_css_class("flat");

        let reload_button = gtk::Button::from_icon_name("view-refresh-symbolic");
        reload_button.set_tooltip_text(Some(&tr("toolbar.reload")));
        reload_button.add_css_class("flat");

        let new_folder_button = gtk::Button::from_icon_name("folder-new-symbolic");
        new_folder_button.set_tooltip_text(Some(&tr("toolbar.new_folder")));
        new_folder_button.add_css_class("flat");

        let search_entry = gtk::SearchEntry::new();
        search_entry.set_placeholder_text(Some(&tr("status.search_placeholder")));
        search_entry.set_hexpand(true);
        search_entry.set_max_width_chars(28);

        let view_toggle_button = gtk::Button::from_icon_name("view-list-symbolic");
        view_toggle_button.set_tooltip_text(Some(&tr("toolbar.view_list")));
        view_toggle_button.add_css_class("flat");

        let empty_trash_button = gtk::Button::from_icon_name("user-trash-symbolic");
        empty_trash_button.set_tooltip_text(Some(&tr("menu.empty_trash")));
        empty_trash_button.add_css_class("flat");
        empty_trash_button.add_css_class("destructive-action");
        empty_trash_button.set_visible(false);

        let toolbar_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        toolbar_box.set_margin_start(8);
        toolbar_box.set_margin_end(8);
        toolbar_box.set_margin_top(6);
        toolbar_box.set_margin_bottom(4);
        toolbar_box.append(&nav_back_button);
        toolbar_box.append(&nav_forward_button);
        toolbar_box.append(&nav_up_button);
        toolbar_box.append(&reload_button);
        toolbar_box.append(&new_folder_button);
        toolbar_box.append(&empty_trash_button);
        toolbar_box.append(&path_bar_label);
        toolbar_box.append(&search_entry);
        toolbar_box.append(&view_toggle_button);

        let outer_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer_box.append(&toolbar_box);
        outer_box.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        outer_box.append(&scrolled);
        outer_box.append(&status_label);

        let toast_overlay = adw::ToastOverlay::new();
        toast_overlay.set_child(Some(&outer_box));

        let pane = Rc::new(ContentPane {
            root_widget: toast_overlay.clone().upcast(),
            toast_overlay,
            list_store,
            selection_model,
            current_target: Rc::new(RefCell::new(SidebarTarget::Path(
                dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
            ))),
            current_source: Rc::new(RefCell::new(ContentSource::Directory)),
            path_bar_label,
            status_label,
            back_history: Rc::new(RefCell::new(Vec::new())),
            forward_history: Rc::new(RefCell::new(Vec::new())),
            nav_back_button: nav_back_button.clone(),
            nav_forward_button: nav_forward_button.clone(),
            empty_trash_button: empty_trash_button.clone(),
            view_stack: view_stack.clone(),
            app_state,
            window,
            terminal_panel,
        });

        pane.setup_activation(&grid_view, &list_view);
        pane.setup_context_menu();
        pane.setup_keyboard_shortcuts();
        pane.setup_toolbar_actions(
            &nav_back_button,
            &nav_forward_button,
            &nav_up_button,
            &reload_button,
            &new_folder_button,
            &search_entry,
            &view_toggle_button,
            &empty_trash_button,
        );

        pane
    }

    fn setup_activation(self: &Rc<Self>, grid_view: &gtk::GridView, list_view: &gtk::ListView) {
        let pane = self.clone();
        self.selection_model
            .connect_selection_changed(move |model, _, _| {
                let count = model.selection().size();
                if count > 0 {
                    pane.status_label.set_text(&crate::i18n::tr_with(
                        "status.items_selected",
                        &count.to_string(),
                    ));
                } else {
                    pane.refresh_status_label_idle();
                }
            });

        let pane = self.clone();
        grid_view.connect_activate(move |_view, position| {
            if let Some(obj) = pane.list_store.item(position) {
                if let Ok(file_object) = obj.downcast::<FileEntryObject>() {
                    pane.activate_entry(&file_object.to_file_entry());
                }
            }
        });

        let pane = self.clone();
        list_view.connect_activate(move |_view, position| {
            if let Some(obj) = pane.list_store.item(position) {
                if let Ok(file_object) = obj.downcast::<FileEntryObject>() {
                    pane.activate_entry(&file_object.to_file_entry());
                }
            }
        });
    }

    fn refresh_status_label_idle(self: &Rc<Self>) {
        let count = self.list_store.n_items();
        if count == 0 {
            self.status_label.set_text(&tr("status.empty_folder"));
        } else {
            self.status_label
                .set_text(&crate::i18n::tr_with("status.items_total", &count.to_string()));
        }
    }

    fn show_error_toast(self: &Rc<Self>, message: String) {
        let toast = adw::Toast::builder().title(message).timeout(4).build();
        self.toast_overlay.add_toast(toast);
    }

    fn setup_context_menu(self: &Rc<Self>) {
        let click_gesture = gtk::GestureClick::new();
        click_gesture.set_button(3);
        let pane = self.clone();
        click_gesture.connect_pressed(move |_gesture, _n_press, x, y| {
            pane.show_context_menu(x, y);
        });
        self.root_widget.add_controller(click_gesture);
    }

    fn setup_keyboard_shortcuts(self: &Rc<Self>) {
        let controller = gtk::EventControllerKey::new();
        let pane = self.clone();

        controller.connect_key_pressed(move |_ctrl, keyval, _keycode, modifier_state| {
            let ctrl_pressed = modifier_state.contains(gdk::ModifierType::CONTROL_MASK);
            let is_trash_view = matches!(*pane.current_source.borrow(), ContentSource::Trash);

            if ctrl_pressed && keyval == gdk::Key::c {
                pane.copy_selection_to_clipboard(false);
                return glib::Propagation::Stop;
            }

            if ctrl_pressed && keyval == gdk::Key::x && !is_trash_view {
                pane.copy_selection_to_clipboard(true);
                return glib::Propagation::Stop;
            }

            if ctrl_pressed && keyval == gdk::Key::v && !is_trash_view {
                pane.paste_clipboard();
                return glib::Propagation::Stop;
            }

            if keyval == gdk::Key::Delete {
                let shift_pressed = modifier_state.contains(gdk::ModifierType::SHIFT_MASK);
                if is_trash_view {
                    pane.restore_selected_from_trash();
                } else if shift_pressed {
                    pane.prompt_delete_permanently();
                } else {
                    pane.move_selection_to_trash();
                }
                return glib::Propagation::Stop;
            }

            if keyval == gdk::Key::F2 && !is_trash_view {
                pane.prompt_rename_selected();
                return glib::Propagation::Stop;
            }

            glib::Propagation::Proceed
        });

        self.root_widget.add_controller(controller);
    }

    #[allow(clippy::too_many_arguments)]
    fn setup_toolbar_actions(
        self: &Rc<Self>,
        nav_back_button: &gtk::Button,
        nav_forward_button: &gtk::Button,
        nav_up_button: &gtk::Button,
        reload_button: &gtk::Button,
        new_folder_button: &gtk::Button,
        search_entry: &gtk::SearchEntry,
        view_toggle_button: &gtk::Button,
        empty_trash_button: &gtk::Button,
    ) {
        let pane = self.clone();
        nav_back_button.connect_clicked(move |_| {
            pane.navigate_back();
        });

        let pane = self.clone();
        nav_forward_button.connect_clicked(move |_| {
            pane.navigate_forward();
        });

        let pane = self.clone();
        empty_trash_button.connect_clicked(move |_| {
            pane.prompt_empty_trash();
        });

        let pane = self.clone();
        nav_up_button.connect_clicked(move |_| {
            pane.navigate_up();
        });

        let pane = self.clone();
        reload_button.connect_clicked(move |_| {
            pane.reload();
        });

        let pane = self.clone();
        new_folder_button.connect_clicked(move |_| {
            pane.prompt_new_folder();
        });

        let pane = self.clone();
        let view_toggle_button_clone = view_toggle_button.clone();
        view_toggle_button.connect_clicked(move |_| {
            let showing_grid = pane.view_stack.visible_child_name().as_deref() == Some("grid");
            if showing_grid {
                pane.view_stack.set_visible_child_name("list");
                view_toggle_button_clone.set_icon_name("view-grid-symbolic");
                view_toggle_button_clone.set_tooltip_text(Some(&tr("toolbar.view_grid")));
            } else {
                pane.view_stack.set_visible_child_name("grid");
                view_toggle_button_clone.set_icon_name("view-list-symbolic");
                view_toggle_button_clone.set_tooltip_text(Some(&tr("toolbar.view_list")));
            }
        });

        let pane = self.clone();
        search_entry.connect_search_changed(move |entry| {
            let query = entry.text().to_string();
            pane.run_search(query);
        });

        let pane = self.clone();
        search_entry.connect_stop_search(move |entry| {
            entry.set_text("");
            pane.reload();
        });
    }

    fn run_search(self: &Rc<Self>, query: String) {
        if query.trim().is_empty() {
            self.reload();
            return;
        }

        let Some(root_path) = self.current_target.borrow().writable_path() else {
            return;
        };

        self.status_label.set_text(&tr("status.loading"));
        let pane = self.clone();

        glib::spawn_future_local(async move {
            let show_hidden = *pane.app_state.show_hidden_files.borrow();
            let results = crate::fs::reader::search_async(root_path, query, 500, show_hidden).await;

            pane.list_store.remove_all();
            let count = results.len();
            for entry in results {
                pane.list_store.append(&FileEntryObject::new(entry));
            }

            if count == 0 {
                pane.status_label.set_text(&tr("status.no_results"));
            } else {
                pane.status_label
                    .set_text(&crate::i18n::tr_with("status.search_results", &count.to_string()));
            }
        });
    }

    fn navigate_back(self: &Rc<Self>) {
        let Some(previous) = self.back_history.borrow_mut().pop() else {
            return;
        };
        let current = self.current_target.borrow().clone();
        self.forward_history.borrow_mut().push(current);
        self.navigate_to_internal(previous);
    }

    fn navigate_forward(self: &Rc<Self>) {
        let Some(next) = self.forward_history.borrow_mut().pop() else {
            return;
        };
        let current = self.current_target.borrow().clone();
        self.back_history.borrow_mut().push(current);
        self.navigate_to_internal(next);
    }

    fn navigate_up(self: &Rc<Self>) {
        let current = self.current_target.borrow().clone();
        if let Some(path) = current.writable_path() {
            if let Some(parent) = path.parent() {
                self.navigate_to(SidebarTarget::Path(parent.to_path_buf()));
            }
        }
    }

    fn prompt_empty_trash(self: &Rc<Self>) {
        let dialog = adw::AlertDialog::builder()
            .heading(tr("menu.empty_trash"))
            .body(tr("dialog.delete_confirm_body"))
            .build();
        dialog.add_response("cancel", &tr("dialog.cancel"));
        dialog.add_response("empty", &tr("dialog.delete"));
        dialog.set_response_appearance("empty", adw::ResponseAppearance::Destructive);
        dialog.set_default_response(Some("cancel"));

        let pane = self.clone();
        dialog.connect_response(None, move |dialog, response| {
            if response == "empty" {
                let pane = pane.clone();
                glib::spawn_future_local(async move {
                    if let Err(err) = trash_ops::empty_trash_async().await {
                        pane.show_error_toast(crate::i18n::tr_with(
                            "error.delete_failed",
                            &err.to_string(),
                        ));
                    }
                    pane.reload();
                });
            }
            dialog.close();
        });

        dialog.present(Some(&self.window));
    }

    fn update_nav_button_sensitivity(self: &Rc<Self>) {
        self.nav_back_button
            .set_sensitive(!self.back_history.borrow().is_empty());
        self.nav_forward_button
            .set_sensitive(!self.forward_history.borrow().is_empty());
    }

    fn show_context_menu(self: &Rc<Self>, x: f64, y: f64) {
        let menu = gio::Menu::new();
        let has_selection = self.selection_model.selection().size() > 0;
        let is_trash_view = matches!(*self.current_source.borrow(), ContentSource::Trash);

        if has_selection {
            menu.append(Some(&tr("menu.copy")), Some("pane.copy"));
            if !is_trash_view {
                menu.append(Some(&tr("menu.cut")), Some("pane.cut"));
                menu.append(Some(&tr("menu.rename")), Some("pane.rename"));
            }
            if is_trash_view {
                menu.append(Some(&tr("menu.restore_from_trash")), Some("pane.restore"));
            } else {
                menu.append(Some(&tr("menu.move_to_trash")), Some("pane.trash"));
            }
            menu.append(
                Some(&tr("menu.delete_permanently")),
                Some("pane.delete-permanently"),
            );
            menu.append(Some(&tr("menu.properties")), Some("pane.properties"));
        } else {
            menu.append(Some(&tr("menu.new_folder")), Some("pane.new-folder"));
            menu.append(Some(&tr("menu.new_file")), Some("pane.new-file"));
            menu.append(Some(&tr("toolbar.reload")), Some("pane.reload"));
            menu.append(Some(&tr("menu.paste")), Some("pane.paste"));
        }

        let action_group = gio::SimpleActionGroup::new();

        let pane_for_copy = self.clone();
        let copy_action = gio::SimpleAction::new("copy", None);
        copy_action.connect_activate(move |_, _| {
            pane_for_copy.copy_selection_to_clipboard(false);
        });
        action_group.add_action(&copy_action);

        let pane_for_cut = self.clone();
        let cut_action = gio::SimpleAction::new("cut", None);
        cut_action.connect_activate(move |_, _| {
            pane_for_cut.copy_selection_to_clipboard(true);
        });
        action_group.add_action(&cut_action);

        let pane_for_rename = self.clone();
        let rename_action = gio::SimpleAction::new("rename", None);
        rename_action.connect_activate(move |_, _| {
            pane_for_rename.prompt_rename_selected();
        });
        action_group.add_action(&rename_action);

        let pane_for_trash = self.clone();
        let trash_action = gio::SimpleAction::new("trash", None);
        trash_action.connect_activate(move |_, _| {
            pane_for_trash.move_selection_to_trash();
        });
        action_group.add_action(&trash_action);

        let pane_for_delete_permanently = self.clone();
        let delete_permanently_action = gio::SimpleAction::new("delete-permanently", None);
        delete_permanently_action.connect_activate(move |_, _| {
            pane_for_delete_permanently.prompt_delete_permanently();
        });
        action_group.add_action(&delete_permanently_action);

        let pane_for_restore = self.clone();
        let restore_action = gio::SimpleAction::new("restore", None);
        restore_action.connect_activate(move |_, _| {
            pane_for_restore.restore_selected_from_trash();
        });
        action_group.add_action(&restore_action);

        let pane_for_properties = self.clone();
        let properties_action = gio::SimpleAction::new("properties", None);
        properties_action.connect_activate(move |_, _| {
            pane_for_properties.show_properties_dialog();
        });
        action_group.add_action(&properties_action);

        let pane_for_new_folder = self.clone();
        let new_folder_action = gio::SimpleAction::new("new-folder", None);
        new_folder_action.connect_activate(move |_, _| {
            pane_for_new_folder.prompt_new_folder();
        });
        action_group.add_action(&new_folder_action);

        let pane_for_new_file = self.clone();
        let new_file_action = gio::SimpleAction::new("new-file", None);
        new_file_action.connect_activate(move |_, _| {
            pane_for_new_file.prompt_new_file();
        });
        action_group.add_action(&new_file_action);

        let pane_for_reload = self.clone();
        let reload_action = gio::SimpleAction::new("reload", None);
        reload_action.connect_activate(move |_, _| {
            pane_for_reload.reload();
        });
        action_group.add_action(&reload_action);

        let pane_for_paste = self.clone();
        let paste_action = gio::SimpleAction::new("paste", None);
        paste_action.connect_activate(move |_, _| {
            pane_for_paste.paste_clipboard();
        });
        action_group.add_action(&paste_action);

        self.root_widget
            .insert_action_group("pane", Some(&action_group));

        let popover = gtk::PopoverMenu::from_model(Some(&menu));
        popover.set_parent(&self.root_widget);
        popover.set_pointing_to(Some(&gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
        popover.popup();
    }

    pub fn navigate_to(self: &Rc<Self>, target: SidebarTarget) {
        let current = self.current_target.borrow().clone();
        if current != target {
            self.back_history.borrow_mut().push(current);
            self.forward_history.borrow_mut().clear();
        }
        self.navigate_to_internal(target);
    }

    fn navigate_to_internal(self: &Rc<Self>, target: SidebarTarget) {
        *self.current_target.borrow_mut() = target.clone();
        *self.current_source.borrow_mut() = target_to_content_source(&target);
        self.update_nav_button_sensitivity();
        self.reload();
    }

    pub fn reload(self: &Rc<Self>) {
        let target = self.current_target.borrow().clone();

        match &target {
            SidebarTarget::Path(path) => {
                self.path_bar_label.set_text(&path.to_string_lossy());
            }
            SidebarTarget::Device(path) => {
                self.path_bar_label.set_text(&path.to_string_lossy());
            }
            SidebarTarget::Applications => {
                self.path_bar_label.set_text(&tr("apps.title"));
            }
            SidebarTarget::Trash => {
                self.path_bar_label.set_text(&tr("sidebar.trash"));
            }
        }

        self.empty_trash_button
            .set_visible(matches!(target, SidebarTarget::Trash));

        self.status_label.set_text(&tr("status.loading"));
        self.list_store.remove_all();

        let pane = self.clone();
        glib::spawn_future_local(async move {
            let entries: Vec<FileEntry> = match &target {
                SidebarTarget::Path(path) => {
                    read_dir_async(path.clone()).await.unwrap_or_default()
                }
                SidebarTarget::Device(path) => {
                    read_dir_async(path.clone()).await.unwrap_or_default()
                }
                SidebarTarget::Applications => {
                    let lang_code = crate::i18n::current_lang().code().to_string();
                    load_applications_listing(lang_code).await
                }
                SidebarTarget::Trash => {
                    let items = trash_ops::list_trash_items_async()
                        .await
                        .unwrap_or_default();
                    items
                        .into_iter()
                        .map(|item| FileEntry {
                            path: item.original_path.clone(),
                            display_name: item.display_name,
                            kind: EntryKind::File,
                            size: 0,
                            modified: None,
                            is_hidden: false,
                            icon_name: "user-trash-full".to_string(),
                            content_type: None,
                            exec_command: None,
                            app_id: None,
                        })
                        .collect()
                }
            };

            let show_hidden = *pane.app_state.show_hidden_files.borrow();
            let mut filtered: Vec<FileEntry> = entries
                .into_iter()
                .filter(|e| show_hidden || !e.is_hidden)
                .collect();

            let sort_key = *pane.app_state.sort_key.borrow();
            let sort_direction = *pane.app_state.sort_direction.borrow();
            sort_entries(&mut filtered, sort_key, sort_direction);

            for entry in filtered {
                pane.list_store.append(&FileEntryObject::new(entry));
            }

            pane.refresh_status_label_idle();
        });
    }

    fn prompt_new_folder(self: &Rc<Self>) {
        let Some(base_path) = self.current_target.borrow().writable_path() else {
            return;
        };

        let entry = gtk::Entry::new();
        entry.set_text(&tr("dialog.new_folder_placeholder"));

        let dialog = adw::AlertDialog::builder()
            .heading(tr("dialog.new_folder_title"))
            .extra_child(&entry)
            .build();
        dialog.add_response("cancel", &tr("dialog.cancel"));
        dialog.add_response("create", &tr("dialog.confirm"));
        dialog.set_response_appearance("create", adw::ResponseAppearance::Suggested);
        dialog.set_default_response(Some("create"));

        let pane = self.clone();
        dialog.connect_response(None, move |dialog, response| {
            if response == "create" {
                let name = entry.text().to_string();
                if !name.is_empty() {
                    let target_path = base_path.join(&name);
                    let pane = pane.clone();
                    glib::spawn_future_local(async move {
                        if let Err(err) = ops::create_directory(target_path).await {
                            pane.show_error_toast(crate::i18n::tr_with(
                                "error.create_failed",
                                &err.to_string(),
                            ));
                        }
                        pane.reload();
                    });
                }
            }
            dialog.close();
        });

        dialog.present(Some(&self.window));
    }

    fn prompt_new_file(self: &Rc<Self>) {
        let Some(base_path) = self.current_target.borrow().writable_path() else {
            return;
        };

        let entry = gtk::Entry::new();
        entry.set_text(&tr("dialog.new_file_placeholder"));

        let dialog = adw::AlertDialog::builder()
            .heading(tr("dialog.new_file_title"))
            .extra_child(&entry)
            .build();
        dialog.add_response("cancel", &tr("dialog.cancel"));
        dialog.add_response("create", &tr("dialog.confirm"));
        dialog.set_response_appearance("create", adw::ResponseAppearance::Suggested);
        dialog.set_default_response(Some("create"));

        let pane = self.clone();
        dialog.connect_response(None, move |dialog, response| {
            if response == "create" {
                let name = entry.text().to_string();
                if !name.is_empty() {
                    let target_path = base_path.join(&name);
                    let pane = pane.clone();
                    glib::spawn_future_local(async move {
                        if let Err(err) = ops::create_empty_file(target_path).await {
                            pane.show_error_toast(crate::i18n::tr_with(
                                "error.create_failed",
                                &err.to_string(),
                            ));
                        }
                        pane.reload();
                    });
                }
            }
            dialog.close();
        });

        dialog.present(Some(&self.window));
    }

    fn prompt_rename_selected(self: &Rc<Self>) {
        let selected = self.selected_entries();
        let Some(target_entry) = selected.into_iter().next() else {
            return;
        };
        let Some(parent_dir) = target_entry.path.parent().map(|p| p.to_path_buf()) else {
            return;
        };

        let entry = gtk::Entry::new();
        entry.set_text(&target_entry.display_name);

        let dialog = adw::AlertDialog::builder()
            .heading(tr("dialog.rename_title"))
            .extra_child(&entry)
            .build();
        dialog.add_response("cancel", &tr("dialog.cancel"));
        dialog.add_response("rename", &tr("dialog.confirm"));
        dialog.set_response_appearance("rename", adw::ResponseAppearance::Suggested);
        dialog.set_default_response(Some("rename"));

        let pane = self.clone();
        let old_path = target_entry.path.clone();
        dialog.connect_response(None, move |dialog, response| {
            if response == "rename" {
                let new_name = entry.text().to_string();
                if !new_name.is_empty() && new_name != target_entry.display_name {
                    let new_path = parent_dir.join(&new_name);
                    let pane = pane.clone();
                    let old_path = old_path.clone();
                    glib::spawn_future_local(async move {
                        if let Err(err) = ops::rename_entry(old_path, new_path).await {
                            pane.show_error_toast(crate::i18n::tr_with(
                                "error.rename_failed",
                                &err.to_string(),
                            ));
                        }
                        pane.reload();
                    });
                }
            }
            dialog.close();
        });

        dialog.present(Some(&self.window));
    }

    fn restore_selected_from_trash(self: &Rc<Self>) {
        let selected = self.selected_entries();
        if selected.is_empty() {
            return;
        }
        let pane = self.clone();
        glib::spawn_future_local(async move {
            let mut failed_count = 0usize;
            for entry in selected {
                if trash_ops::restore_by_display_name_async(entry.display_name.clone())
                    .await
                    .is_err()
                {
                    failed_count += 1;
                }
            }
            if failed_count > 0 {
                pane.show_error_toast(crate::i18n::tr_with(
                    "error.generic_title",
                    &failed_count.to_string(),
                ));
            }
            pane.reload();
        });
    }

    fn show_properties_dialog(self: &Rc<Self>) {
        let selected = self.selected_entries();
        let Some(target_entry) = selected.into_iter().next() else {
            return;
        };

        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
        content_box.set_margin_top(16);
        content_box.set_margin_bottom(16);
        content_box.set_margin_start(16);
        content_box.set_margin_end(16);

        let add_row = |label_key: &str, value: String| {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            let label = gtk::Label::new(Some(&tr(label_key)));
            label.set_halign(gtk::Align::Start);
            label.set_width_chars(12);
            label.add_css_class("dim-label");
            row.append(&label);

            let value_label = gtk::Label::new(Some(&value));
            value_label.set_halign(gtk::Align::Start);
            value_label.set_hexpand(true);
            value_label.set_wrap(true);
            value_label.set_selectable(true);
            row.append(&value_label);

            content_box.append(&row);
        };

        add_row("properties.name", target_entry.display_name.clone());

        let type_str = match target_entry.kind {
            EntryKind::Directory => tr("properties.folder"),
            EntryKind::Symlink => tr("properties.symlink"),
            EntryKind::AppLauncher => tr("apps.title"),
            EntryKind::File => target_entry
                .content_type
                .clone()
                .unwrap_or_else(|| tr("properties.file")),
        };
        add_row("properties.type", type_str);

        if target_entry.kind == EntryKind::AppLauncher {
            if let Some(app_id) = target_entry.app_id.clone() {
                add_row("properties.app_id", app_id);
            }
        }

        if target_entry.kind != EntryKind::Directory {
            add_row(
                "properties.size",
                crate::util::format::format_size(target_entry.size),
            );
        }

        add_row(
            "properties.location",
            target_entry
                .path
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
        );

        add_row(
            "properties.modified",
            crate::util::format::format_timestamp(target_entry.modified),
        );

        let dialog = adw::AlertDialog::builder()
            .heading(tr("properties.title"))
            .extra_child(&content_box)
            .build();
        dialog.add_response("close", &tr("dialog.close"));
        dialog.present(Some(&self.window));
    }

    pub fn selected_entries(self: &Rc<Self>) -> Vec<FileEntry> {
        let mut result = Vec::new();
        let bitset = self.selection_model.selection();

        if let Some((mut iter, first_value)) = gtk::BitsetIter::init_first(&bitset) {
            let mut current = Some(first_value);
            while let Some(pos) = current {
                if let Some(obj) = self.list_store.item(pos) {
                    if let Ok(file_object) = obj.downcast::<FileEntryObject>() {
                        result.push(file_object.to_file_entry());
                    }
                }
                current = iter.next();
            }
        }

        result
    }

    pub fn copy_selection_to_clipboard(self: &Rc<Self>, is_cut: bool) {
        let paths: Vec<PathBuf> = self
            .selected_entries()
            .into_iter()
            .map(|e| e.path)
            .collect();
        *self.app_state.clipboard_paths.borrow_mut() = paths;
        *self.app_state.clipboard_is_cut.borrow_mut() = is_cut;
    }

    pub fn paste_clipboard(self: &Rc<Self>) {
        let Some(dst_dir) = self.current_target.borrow().writable_path() else {
            return;
        };
        let paths = self.app_state.clipboard_paths.borrow().clone();
        let is_cut = *self.app_state.clipboard_is_cut.borrow();

        if paths.is_empty() {
            return;
        }

        if is_cut {
            let pane = self.clone();
            glib::spawn_future_local(async move {
                let mut failed_count = 0usize;
                for path in paths {
                    if ops::move_entry(path, dst_dir.clone()).await.is_err() {
                        failed_count += 1;
                    }
                }
                if failed_count > 0 {
                    pane.show_error_toast(crate::i18n::tr_with(
                        "error.move_failed",
                        &failed_count.to_string(),
                    ));
                }
                pane.app_state.clipboard_paths.borrow_mut().clear();
                pane.reload();
            });
        } else {
            for path in paths {
                let window = self.window.clone();
                let pane = self.clone();
                let dst_dir = dst_dir.clone();
                spawn_copy_with_dialog(&window, path, dst_dir, move |_dest| {
                    pane.reload();
                });
            }
        }
    }

    pub fn move_selection_to_trash(self: &Rc<Self>) {
        let selected = self.selected_entries();
        if selected.is_empty() {
            return;
        }

        let (app_entries, regular_entries): (Vec<FileEntry>, Vec<FileEntry>) = selected
            .into_iter()
            .partition(|e| e.kind == EntryKind::AppLauncher);

        if !app_entries.is_empty() {
            self.prompt_uninstall_applications(app_entries);
        }

        if regular_entries.is_empty() {
            return;
        }

        let paths: Vec<PathBuf> = regular_entries.into_iter().map(|e| e.path).collect();
        let pane = self.clone();
        glib::spawn_future_local(async move {
            if let Err(err) = trash_ops::move_to_trash_async(paths).await {
                pane.show_error_toast(crate::i18n::tr_with(
                    "error.delete_failed",
                    &err.to_string(),
                ));
            }
            pane.reload();
        });
    }

    fn prompt_delete_permanently(self: &Rc<Self>) {
        let paths: Vec<PathBuf> = self
            .selected_entries()
            .into_iter()
            .map(|e| e.path)
            .collect();
        if paths.is_empty() {
            return;
        }

        let dialog = adw::AlertDialog::builder()
            .heading(tr("dialog.delete_confirm_title"))
            .body(tr("dialog.delete_confirm_body"))
            .build();
        dialog.add_response("cancel", &tr("dialog.cancel"));
        dialog.add_response("delete", &tr("dialog.delete"));
        dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
        dialog.set_default_response(Some("cancel"));

        let pane = self.clone();
        dialog.connect_response(None, move |dialog, response| {
            if response == "delete" {
                let pane = pane.clone();
                let paths = paths.clone();
                glib::spawn_future_local(async move {
                    let mut failed_count = 0usize;
                    for path in paths {
                        if ops::delete_permanently(path).await.is_err() {
                            failed_count += 1;
                        }
                    }
                    if failed_count > 0 {
                        pane.show_error_toast(crate::i18n::tr_with(
                            "error.delete_failed",
                            &failed_count.to_string(),
                        ));
                    }
                    pane.reload();
                });
            }
            dialog.close();
        });

        dialog.present(Some(&self.window));
    }

    fn prompt_uninstall_applications(self: &Rc<Self>, app_entries: Vec<FileEntry>) {
        for entry in app_entries {
            self.uninstall_one_application(entry);
        }
    }

    fn uninstall_one_application(self: &Rc<Self>, entry: FileEntry) {
        let display_name = entry.display_name.clone();

        let body_label = gtk::Label::new(Some(&tr("apps.uninstall_detecting")));
        body_label.set_wrap(true);
        body_label.set_xalign(0.0);

        let spinner = gtk::Spinner::new();
        spinner.set_spinning(true);
        spinner.set_halign(gtk::Align::Start);

        let detecting_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        detecting_box.append(&spinner);
        detecting_box.append(&body_label);

        let dialog = adw::AlertDialog::builder()
            .heading(format!("{}: {}", tr("apps.uninstall_title"), display_name))
            .extra_child(&detecting_box)
            .build();
        dialog.add_response("cancel", &tr("dialog.cancel"));
        dialog.set_default_response(Some("cancel"));
        dialog.present(Some(&self.window));

        let pane = self.clone();
        let entry_path = entry.path.clone();
        glib::spawn_future_local(async move {
            let real_binary_path = resolve_real_binary_path(&entry_path);

            let package_manager = detect_package_manager();

            let Some(package_manager) = package_manager else {
                body_label.set_text(&crate::i18n::tr_with(
                    "apps.uninstall_body_unknown_distro",
                    &display_name,
                ));
                dialog.remove_response("cancel");
                dialog.add_response("cancel", &tr("dialog.cancel"));
                dialog.add_response("unsafe-remove", &tr("apps.uninstall_confirm_unsafe"));
                dialog.set_response_appearance(
                    "unsafe-remove",
                    adw::ResponseAppearance::Destructive,
                );
                dialog.set_default_response(Some("cancel"));
                spinner.set_visible(false);

                let pane = pane.clone();
                let path_for_removal = entry_path.clone();
                dialog.connect_response(None, move |dialog, response| {
                    if response == "unsafe-remove" {
                        let pane = pane.clone();
                        let path_for_removal = path_for_removal.clone();
                        glib::spawn_future_local(async move {
                            if let Err(err) = ops::delete_permanently(path_for_removal).await {
                                pane.show_error_toast(err.to_string());
                            }
                            pane.reload();
                        });
                    }
                    dialog.close();
                });
                return;
            };

            let package_name =
                find_owning_package_async(package_manager, real_binary_path).await;

            spinner.set_visible(false);

            let Some(package_name) = package_name else {
                body_label.set_text(&crate::i18n::tr_with_many(
                    "apps.uninstall_body_unknown_package",
                    &[&display_name, package_manager.display_name()],
                ));
                dialog.remove_response("cancel");
                dialog.add_response("cancel", &tr("dialog.cancel"));
                dialog.add_response("unsafe-remove", &tr("apps.uninstall_confirm_unsafe"));
                dialog.set_response_appearance(
                    "unsafe-remove",
                    adw::ResponseAppearance::Destructive,
                );
                dialog.set_default_response(Some("cancel"));

                let pane = pane.clone();
                let path_for_removal = entry_path.clone();
                dialog.connect_response(None, move |dialog, response| {
                    if response == "unsafe-remove" {
                        let pane = pane.clone();
                        let path_for_removal = path_for_removal.clone();
                        glib::spawn_future_local(async move {
                            if let Err(err) = ops::delete_permanently(path_for_removal).await {
                                pane.show_error_toast(err.to_string());
                            }
                            pane.reload();
                        });
                    }
                    dialog.close();
                });
                return;
            };

            let uninstall_command =
                build_uninstall_command(package_manager, &package_name);

            let needs_password = !is_running_as_root();
            let mut body_text = crate::i18n::tr_with_many(
                "apps.uninstall_body_with_package",
                &[&display_name, &package_name, package_manager.display_name()],
            );
            if needs_password {
                body_text.push_str("\n\n");
                body_text.push_str(&tr("apps.uninstall_needs_password"));
            }
            body_label.set_text(&body_text);

            dialog.remove_response("cancel");
            dialog.add_response("cancel", &tr("dialog.cancel"));
            dialog.add_response("remove", &tr("apps.uninstall_confirm"));
            dialog.set_response_appearance("remove", adw::ResponseAppearance::Destructive);
            dialog.set_default_response(Some("cancel"));

            let pane = pane.clone();
            let display_name_for_response = display_name.clone();
            dialog.connect_response(None, move |dialog, response| {
                if response == "remove" {
                    let pane = pane.clone();
                    let uninstall_command = uninstall_command.clone();
                    let display_name = display_name_for_response.clone();
                    glib::spawn_future_local(async move {
                        pane.show_error_toast(tr("apps.uninstall_in_progress"));
                        match run_uninstall_async(uninstall_command.clone()).await {
                            Ok(()) => {
                                pane.show_error_toast(crate::i18n::tr_with(
                                    "apps.uninstall_success",
                                    &display_name,
                                ));
                                pane.reload();
                            }
                            Err(_) => {
                                pane.show_error_toast(tr("apps.uninstall_pkexec_failed"));
                                pane.offer_terminal_fallback(&uninstall_command.display_command);
                            }
                        }
                    });
                }
                dialog.close();
            });
        });
    }

    fn offer_terminal_fallback(self: &Rc<Self>, display_command: &str) {
        self.terminal_panel.revealer.set_reveal_child(true);

        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        self.terminal_panel.ensure_shell_spawned(cwd);

        let sudo_command = format!("sudo {display_command}");
        self.terminal_panel.feed_command_text(&sudo_command);
    }

    pub fn activate_entry(self: &Rc<Self>, entry: &FileEntry) {
        match entry.kind {
            EntryKind::Directory => {
                self.navigate_to(SidebarTarget::Path(entry.path.clone()));
            }
            EntryKind::AppLauncher => {
                let entry_clone = entry.clone();
                std::thread::spawn(move || {
                    let _ = launch_application(&entry_clone);
                });
            }
            EntryKind::File | EntryKind::Symlink => {
                let path = entry.path.clone();
                std::thread::spawn(move || {
                    let _ = open::that_detached(path);
                });
            }
        }
    }
}
