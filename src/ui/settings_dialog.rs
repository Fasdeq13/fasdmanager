use glib::object::Cast;
use crate::fs::settings::{FavoriteItem, Settings, SidebarSection};
use crate::i18n::{self, tr, Lang};
use crate::ui::app_state::SharedAppState;
use crate::util::icons::{active_icon_theme, set_active_icon_theme, IconTheme};
use adw::prelude::*;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

fn favorite_item_label_key(item: FavoriteItem) -> &'static str {
    match item {
        FavoriteItem::Home => "sidebar.home",
        FavoriteItem::Desktop => "sidebar.desktop",
        FavoriteItem::Documents => "sidebar.documents",
        FavoriteItem::Downloads => "sidebar.downloads",
        FavoriteItem::Pictures => "sidebar.pictures",
        FavoriteItem::Music => "sidebar.music",
        FavoriteItem::Videos => "sidebar.videos",
        FavoriteItem::Applications => "sidebar.applications",
    }
}

fn build_hidden_items_group(
    settings: Rc<RefCell<Settings>>,
    on_changed: Rc<dyn Fn()>,
) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title(tr("sidebar.restore_hidden_title"))
        .build();

    let mut any_hidden = false;

    for item in FavoriteItem::ALL {
        if settings.borrow().is_favorite_visible(item) {
            continue;
        }
        any_hidden = true;

        let row = adw::ActionRow::builder()
            .title(tr(favorite_item_label_key(item)))
            .build();

        let restore_button = gtk::Button::from_icon_name("edit-undo-symbolic");
        restore_button.add_css_class("flat");
        restore_button.set_valign(gtk::Align::Center);

        let settings_for_restore = settings.clone();
        let on_changed_for_restore = on_changed.clone();
        let row_for_restore = row.clone();
        let group_for_restore = group.clone();
        restore_button.connect_clicked(move |_| {
            settings_for_restore
                .borrow_mut()
                .set_favorite_visible(item, true);
            let _ = settings_for_restore.borrow().save();
            group_for_restore.remove(&row_for_restore);
            on_changed_for_restore();
        });

        row.add_suffix(&restore_button);
        group.add(&row);
    }

    let devices_hidden =
        !settings.borrow().is_sidebar_section_visible(SidebarSection::Devices);
    if devices_hidden {
        any_hidden = true;

        let row = adw::ActionRow::builder()
            .title(tr("sidebar.devices"))
            .build();

        let restore_button = gtk::Button::from_icon_name("edit-undo-symbolic");
        restore_button.add_css_class("flat");
        restore_button.set_valign(gtk::Align::Center);

        let settings_for_restore = settings.clone();
        let on_changed_for_restore = on_changed.clone();
        let row_for_restore = row.clone();
        let group_for_restore = group.clone();
        restore_button.connect_clicked(move |_| {
            settings_for_restore
                .borrow_mut()
                .set_sidebar_section_visible(SidebarSection::Devices, true);
            let _ = settings_for_restore.borrow().save();
            group_for_restore.remove(&row_for_restore);
            on_changed_for_restore();
        });

        row.add_suffix(&restore_button);
        group.add(&row);
    }

    if !any_hidden {
        let row = adw::ActionRow::builder()
            .title(tr("sidebar.restore_hidden_empty"))
            .build();
        row.add_css_class("dim-label");
        group.add(&row);
    }

    group
}

pub fn show_settings_dialog(
    parent_window: &impl IsA<gtk::Window>,
    app_state: SharedAppState,
    on_sidebar_changed: impl Fn() + 'static,
) {
    let on_sidebar_changed: Rc<dyn Fn()> = Rc::new(on_sidebar_changed);
    let settings: Rc<RefCell<Settings>> = Rc::new(RefCell::new(Settings::load()));

    let page = adw::PreferencesPage::new();

    let general_group = adw::PreferencesGroup::builder()
        .title(tr("settings.appearance"))
        .build();

    let hidden_files_row = adw::SwitchRow::builder()
        .title(tr("settings.show_hidden_files"))
        .active(*app_state.show_hidden_files.borrow())
        .build();

    let app_state_for_switch = app_state.clone();
    hidden_files_row.connect_active_notify(move |row| {
        *app_state_for_switch.show_hidden_files.borrow_mut() = row.is_active();
    });
    general_group.add(&hidden_files_row);

    let language_row = adw::ComboRow::builder().title(tr("settings.language")).build();
    let language_model = gtk::StringList::new(&[
        Lang::Ru.native_name(),
        Lang::En.native_name(),
        Lang::It.native_name(),
        Lang::Zh.native_name(),
    ]);
    language_row.set_model(Some(&language_model));
    let current_lang_index = match i18n::current_lang() {
        Lang::Ru => 0,
        Lang::En => 1,
        Lang::It => 2,
        Lang::Zh => 3,
    };
    language_row.set_selected(current_lang_index);

    let parent_window_for_language = parent_window.clone().upcast::<gtk::Window>();
    language_row.connect_selected_notify(move |row| {
        let selected_lang = match row.selected() {
            0 => Lang::Ru,
            2 => Lang::It,
            3 => Lang::Zh,
            _ => Lang::En,
        };
        if selected_lang != i18n::current_lang() {
            i18n::set_lang(selected_lang);

            let notice = adw::AlertDialog::builder()
                .heading(selected_lang.native_name())
                .body(tr("settings.language_restart_notice"))
                .build();
            notice.add_response("ok", &tr("dialog.ok"));
            notice.set_default_response(Some("ok"));
            notice.present(Some(&parent_window_for_language));
        }
    });
    general_group.add(&language_row);

    let icon_theme_row = adw::ComboRow::builder().title(tr("settings.icon_theme")).build();
    let icon_theme_labels: Vec<String> = IconTheme::SELECTABLE
        .iter()
        .map(|theme| tr(theme.display_name_key()))
        .collect();
    let icon_theme_label_refs: Vec<&str> = icon_theme_labels.iter().map(|s| s.as_str()).collect();
    let icon_theme_model = gtk::StringList::new(&icon_theme_label_refs);
    icon_theme_row.set_model(Some(&icon_theme_model));

    let current_theme = active_icon_theme();
    let current_theme_index = IconTheme::SELECTABLE
        .iter()
        .position(|t| *t == current_theme)
        .unwrap_or(0) as u32;
    icon_theme_row.set_selected(current_theme_index);

    let settings_for_icon_theme = settings.clone();
    let on_sidebar_changed_for_icon_theme = on_sidebar_changed.clone();
    icon_theme_row.connect_selected_notify(move |row| {
        let index = row.selected() as usize;
        let Some(&selected_theme) = IconTheme::SELECTABLE.get(index) else {
            return;
        };

        if selected_theme == active_icon_theme() {
            return;
        }

        set_active_icon_theme(selected_theme);
        settings_for_icon_theme
            .borrow_mut()
            .set_icon_theme_value(selected_theme.settings_value());
        let _ = settings_for_icon_theme.borrow().save();

        on_sidebar_changed_for_icon_theme();
    });
    general_group.add(&icon_theme_row);

    page.add(&general_group);

    let hidden_items_group = build_hidden_items_group(settings.clone(), on_sidebar_changed.clone());
    page.add(&hidden_items_group);

    let toolbar_view = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&adw::WindowTitle::new(&tr("settings.title"), "")));
    toolbar_view.add_top_bar(&header);
    toolbar_view.set_content(Some(&page));

    let dialog = adw::Dialog::builder()
        .content_width(480)
        .content_height(560)
        .child(&toolbar_view)
        .build();

    if let Some(parent_widget) = parent_window.dynamic_cast_ref::<gtk::Widget>() {
        dialog.present(Some(parent_widget));
    } else {
        dialog.present(None::<&gtk::Widget>);
    }

}
