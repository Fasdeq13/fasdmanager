use crate::i18n::tr;
use crate::ui::app_state::{new_shared_state, SharedAppState};
use crate::ui::content_view::ContentPane;
use crate::ui::sidebar::build_sidebar_content;
use crate::ui::tabs::{default_home_target, TabManager};
use crate::ui::terminal_panel::TerminalPanel;
use adw::prelude::*;
use gtk::prelude::*;
use gtk::{gdk, gio, glib};
use std::cell::RefCell;
use std::rc::Rc;

pub struct FasdManagerWindow {
    pub adw_window: adw::ApplicationWindow,
    tab_manager: Rc<TabManager>,
    terminal_panel: Rc<TerminalPanel>,
    split_view_button: gtk::ToggleButton,
    outer_split: adw::OverlaySplitView,
    _volume_monitor: RefCell<gio::VolumeMonitor>,
}

impl FasdManagerWindow {
    pub fn new(app: &adw::Application) -> adw::ApplicationWindow {
        let app_state = new_shared_state();

        let adw_window = adw::ApplicationWindow::builder()
            .application(app)
            .title(tr("app.title"))
            .default_width(1200)
            .default_height(760)
            .build();

        let terminal_panel = TerminalPanel::new();
        let tab_manager = TabManager::new(
            app_state.clone(),
            adw_window.clone().upcast(),
            terminal_panel.clone(),
        );

        let outer_split = adw::OverlaySplitView::builder()
            .max_sidebar_width(260.0)
            .min_sidebar_width(200.0)
            .sidebar_width_fraction(0.22)
            .build();

        let tab_manager_for_sidebar = tab_manager.clone();
        let (sidebar_widget, volume_monitor) = build_sidebar_content(move |target| {
            tab_manager_for_sidebar.navigate_current_tab(target);
        });
        outer_split.set_sidebar(Some(&sidebar_widget));

        let content_split = adw::OverlaySplitView::builder()
            .collapsed(true)
            .show_sidebar(false)
            .sidebar_width_fraction(0.5)
            .build();

        let primary_tab_stack = gtk::Box::new(gtk::Orientation::Vertical, 0);
        primary_tab_stack.append(&tab_manager.tab_bar);
        primary_tab_stack.append(&tab_manager.tab_view);
        primary_tab_stack.set_vexpand(true);

        content_split.set_content(Some(&primary_tab_stack));

        let content_and_terminal = gtk::Box::new(gtk::Orientation::Vertical, 0);
        content_and_terminal.set_vexpand(true);
        content_and_terminal.append(&content_split);
        content_and_terminal.append(&terminal_panel.root_widget);

        outer_split.set_content(Some(&content_and_terminal));

        let (header_bar, split_view_button) = build_header_bar(
            &tab_manager,
            &terminal_panel,
            &outer_split,
            &content_split,
            app_state.clone(),
            adw_window.clone().upcast(),
        );

        let toolbar_view = adw::ToolbarView::new();
        toolbar_view.add_top_bar(&header_bar);
        toolbar_view.set_content(Some(&outer_split));

        adw_window.set_content(Some(&toolbar_view));

        let window = Rc::new(FasdManagerWindow {
            adw_window: adw_window.clone(),
            tab_manager: tab_manager.clone(),
            terminal_panel: terminal_panel.clone(),
            split_view_button,
            outer_split,
            _volume_monitor: RefCell::new(volume_monitor),
        });

        tab_manager.open_new_tab(default_home_target());

        setup_keyboard_shortcuts(&window);
        register_app_actions(app, &window, app_state);

        adw_window
    }

    fn rebuild_sidebar(self: &Rc<Self>) {
        let window_for_select = self.tab_manager.clone();
        let (new_sidebar_widget, new_volume_monitor) = build_sidebar_content(move |target| {
            window_for_select.navigate_current_tab(target);
        });
        self.outer_split.set_sidebar(Some(&new_sidebar_widget));
        *self._volume_monitor.borrow_mut() = new_volume_monitor;
    }

    fn reload_all_open_panes(self: &Rc<Self>) {
        for pane in self.tab_manager.all_content_panes() {
            pane.reload();
        }
    }
}

fn register_app_actions(app: &adw::Application, window: &Rc<FasdManagerWindow>, app_state: SharedAppState) {
    let settings_action = gio::SimpleAction::new("settings", None);
    let window_for_settings = window.clone();
    let app_state_for_settings = app_state;
    settings_action.connect_activate(move |_, _| {
        let window_for_callback = window_for_settings.clone();
        crate::ui::settings_dialog::show_settings_dialog(
            &window_for_settings.adw_window,
            app_state_for_settings.clone(),
            move || {
                window_for_callback.rebuild_sidebar();
                window_for_callback.reload_all_open_panes();
            },
        );
    });
    app.add_action(&settings_action);

    let about_action = gio::SimpleAction::new("about", None);
    let window_for_about = window.clone();
    about_action.connect_activate(move |_, _| {
        let about_dialog = adw::AboutDialog::builder()
            .application_name(tr("app.title"))
            .application_icon("org.fasd.manager")
            .version(env!("CARGO_PKG_VERSION"))
            .developer_name("Fasdeq13")
            .comments(tr("about.description"))
            .build();
        about_dialog.present(Some(&window_for_about.adw_window));
    });
    app.add_action(&about_action);
}

fn build_header_bar(
    tab_manager: &Rc<TabManager>,
    terminal_panel: &Rc<TerminalPanel>,
    outer_split: &adw::OverlaySplitView,
    content_split: &adw::OverlaySplitView,
    app_state: crate::ui::app_state::SharedAppState,
    window_handle: gtk::Window,
) -> (adw::HeaderBar, gtk::ToggleButton) {
    let header_bar = adw::HeaderBar::new();

    let sidebar_toggle = gtk::ToggleButton::new();
    sidebar_toggle.set_icon_name("sidebar-show-symbolic");
    sidebar_toggle.set_active(true);
    let outer_split_for_toggle = outer_split.clone();
    sidebar_toggle.connect_toggled(move |btn| {
        outer_split_for_toggle.set_show_sidebar(btn.is_active());
    });
    header_bar.pack_start(&sidebar_toggle);

    let new_tab_button = gtk::Button::from_icon_name("tab-new-symbolic");
    new_tab_button.set_tooltip_text(Some(&tr("toolbar.new_tab")));
    let tab_manager_for_new_tab = tab_manager.clone();
    new_tab_button.connect_clicked(move |_| {
        tab_manager_for_new_tab.open_new_tab(default_home_target());
    });
    header_bar.pack_start(&new_tab_button);

    let split_view_button = gtk::ToggleButton::new();
    split_view_button.set_icon_name("view-dual-symbolic");
    split_view_button.set_tooltip_text(Some(&tr("toolbar.split_view")));
    let content_split_for_button = content_split.clone();
    let secondary_pane_slot: Rc<RefCell<Option<Rc<ContentPane>>>> = Rc::new(RefCell::new(None));
    let split_view_button_for_return = split_view_button.clone();
    let terminal_panel_for_secondary = terminal_panel.clone();
    split_view_button.connect_toggled(move |btn| {
        content_split_for_button.set_show_sidebar(btn.is_active());
        content_split_for_button.set_collapsed(!btn.is_active());

        if btn.is_active() && secondary_pane_slot.borrow().is_none() {
            let secondary_pane = ContentPane::new(
                app_state.clone(),
                window_handle.clone(),
                terminal_panel_for_secondary.clone(),
            );
            secondary_pane.navigate_to(default_home_target());
            content_split_for_button.set_sidebar(Some(&secondary_pane.root_widget));
            *secondary_pane_slot.borrow_mut() = Some(secondary_pane);
        }
    });
    header_bar.pack_start(&split_view_button);

    let terminal_button = gtk::ToggleButton::new();
    terminal_button.set_icon_name("utilities-terminal-symbolic");
    terminal_button.set_tooltip_text(Some(&tr("toolbar.terminal")));
    let terminal_panel_for_button = terminal_panel.clone();
    terminal_button.connect_toggled(move |btn| {
        terminal_panel_for_button.revealer.set_reveal_child(btn.is_active());
        if btn.is_active() {
            let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/"));
            terminal_panel_for_button.ensure_shell_spawned(cwd);
        }
    });
    header_bar.pack_end(&terminal_button);

    let menu_button = gtk::MenuButton::new();
    menu_button.set_icon_name("open-menu-symbolic");
    let menu_model = gio::Menu::new();
    menu_model.append(Some(&tr("settings.title")), Some("app.settings"));
    menu_model.append(Some(&tr("about.title")), Some("app.about"));
    menu_button.set_menu_model(Some(&menu_model));
    header_bar.pack_end(&menu_button);

    let title_widget = adw::WindowTitle::new(&tr("app.title"), "");
    header_bar.set_title_widget(Some(&title_widget));

    (header_bar, split_view_button_for_return)
}

fn setup_keyboard_shortcuts(window: &Rc<FasdManagerWindow>) {
    let controller = gtk::EventControllerKey::new();
    let window_for_shortcuts = window.clone();

    controller.connect_key_pressed(move |_ctrl, keyval, _keycode, modifier_state| {
        let ctrl_pressed = modifier_state.contains(gdk::ModifierType::CONTROL_MASK);

        if ctrl_pressed && keyval == gdk::Key::t {
            window_for_shortcuts
                .tab_manager
                .open_new_tab(default_home_target());
            return glib::Propagation::Stop;
        }

        if ctrl_pressed && keyval == gdk::Key::w {
            window_for_shortcuts.tab_manager.close_current_tab();
            return glib::Propagation::Stop;
        }

        if ctrl_pressed && keyval == gdk::Key::grave {
            window_for_shortcuts.terminal_panel.toggle();
            if window_for_shortcuts.terminal_panel.is_visible() {
                let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/"));
                window_for_shortcuts.terminal_panel.ensure_shell_spawned(cwd);
            }
            return glib::Propagation::Stop;
        }

        if ctrl_pressed && keyval == gdk::Key::backslash {
            let currently_active = window_for_shortcuts.split_view_button.is_active();
            window_for_shortcuts
                .split_view_button
                .set_active(!currently_active);
            return glib::Propagation::Stop;
        }

        glib::Propagation::Proceed
    });

    window.adw_window.add_controller(controller);
}
