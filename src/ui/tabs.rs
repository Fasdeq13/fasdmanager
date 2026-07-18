use crate::i18n::tr;
use crate::ui::app_state::SharedAppState;
use crate::ui::content_view::ContentPane;
use crate::ui::sidebar::SidebarTarget;
use crate::ui::terminal_panel::TerminalPanel;
use adw::prelude::*;
use gtk::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

pub struct TabManager {
    pub tab_view: adw::TabView,
    pub tab_bar: adw::TabBar,
    app_state: SharedAppState,
    window: gtk::Window,
    terminal_panel: Rc<TerminalPanel>,
    panes: RefCell<Vec<(adw::TabPage, Rc<ContentPane>)>>,
}

impl TabManager {
    pub fn new(
        app_state: SharedAppState,
        window: gtk::Window,
        terminal_panel: Rc<TerminalPanel>,
    ) -> Rc<Self> {
        let tab_view = adw::TabView::new();
        let tab_bar = adw::TabBar::new();
        tab_bar.set_view(Some(&tab_view));
        tab_bar.set_autohide(false);

        let manager = Rc::new(TabManager {
            tab_view,
            tab_bar,
            app_state,
            window,
            terminal_panel,
            panes: RefCell::new(Vec::new()),
        });

        manager
    }

    pub fn open_new_tab(self: &Rc<Self>, target: SidebarTarget) -> Rc<ContentPane> {
        let content_pane = ContentPane::new(
            self.app_state.clone(),
            self.window.clone(),
            self.terminal_panel.clone(),
        );

        let page = self.tab_view.append(&content_pane.root_widget);
        page.set_title(&tab_title_for_target(&target));
        page.set_icon(Some(&gtk::gio::ThemedIcon::new("folder-symbolic")));

        self.tab_view.set_selected_page(&page);

        self.panes
            .borrow_mut()
            .push((page, content_pane.clone()));

        content_pane.navigate_to(target);
        content_pane
    }

    pub fn close_current_tab(self: &Rc<Self>) {
        if let Some(page) = self.tab_view.selected_page() {
            if self.tab_view.n_pages() > 1 {
                self.panes.borrow_mut().retain(|(p, _)| *p != page);
                self.tab_view.close_page(&page);
            }
        }
    }

    pub fn current_content_pane(self: &Rc<Self>) -> Option<Rc<ContentPane>> {
        let selected = self.tab_view.selected_page()?;
        self.panes
            .borrow()
            .iter()
            .find(|(page, _)| *page == selected)
            .map(|(_, pane)| pane.clone())
    }

    pub fn navigate_current_tab(self: &Rc<Self>, target: SidebarTarget) {
        if let Some(pane) = self.current_content_pane() {
            pane.navigate_to(target.clone());
            if let Some(page) = self.tab_view.selected_page() {
                page.set_title(&tab_title_for_target(&target));
            }
        } else {
            self.open_new_tab(target);
        }
    }
}

fn tab_title_for_target(target: &SidebarTarget) -> String {
    match target {
        SidebarTarget::Path(path) => path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string()),
        SidebarTarget::Device(path) => path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string()),
        SidebarTarget::Applications => tr("apps.title"),
        SidebarTarget::Trash => tr("sidebar.trash"),
    }
}

pub fn default_home_target() -> SidebarTarget {
    SidebarTarget::Path(dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")))
}
