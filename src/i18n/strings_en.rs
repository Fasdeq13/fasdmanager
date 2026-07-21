use std::collections::HashMap;

pub fn build_table() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();

    m.insert("app.title", "FasdManager");

    m.insert("sidebar.favorites", "Favorites");
    m.insert("sidebar.home", "Home");
    m.insert("sidebar.desktop", "Desktop");
    m.insert("sidebar.documents", "Documents");
    m.insert("sidebar.downloads", "Downloads");
    m.insert("sidebar.pictures", "Pictures");
    m.insert("sidebar.music", "Music");
    m.insert("sidebar.videos", "Videos");
    m.insert("sidebar.applications", "Applications");
    m.insert("sidebar.trash", "Trash");
    m.insert("sidebar.locations", "Locations");
    m.insert("sidebar.filesystem", "File System");
    m.insert("sidebar.network", "Network");
    m.insert("sidebar.devices", "Devices");
    m.insert("sidebar.bookmarks", "Bookmarks");
    m.insert("devices.unmount", "Unmount");
    m.insert("devices.unmounting", "Unmounting…");
    m.insert("devices.unmount_flushing", "Flushing data to device…");
    m.insert("devices.unmount_flushing_bytes", "Remaining to flush: {}");
    m.insert("devices.unmount_success", "\"{}\" safely unmounted");
    m.insert("devices.unmount_failed", "Failed to unmount \"{}\": {}");
    m.insert("sidebar.hide_item", "Hide from Sidebar");
    m.insert("sidebar.hide_section", "Hide Section");
    m.insert("sidebar.manage_items", "Show Hidden Items…");
    m.insert("sidebar.restore_hidden_title", "Hidden Sidebar Items");
    m.insert("sidebar.restore_hidden_empty", "Nothing is hidden");

    m.insert("toolbar.back", "Back");
    m.insert("toolbar.forward", "Forward");
    m.insert("toolbar.up", "Up");
    m.insert("toolbar.reload", "Reload");
    m.insert("toolbar.new_folder", "New Folder");
    m.insert("toolbar.new_tab", "New Tab");
    m.insert("toolbar.split_view", "Split View");
    m.insert("toolbar.terminal", "Terminal");
    m.insert("toolbar.search", "Search");
    m.insert("toolbar.view_grid", "Grid");
    m.insert("toolbar.view_list", "List");
    m.insert("toolbar.sort", "Sort");
    m.insert("toolbar.menu", "Menu");
    m.insert("toolbar.properties", "Properties");

    m.insert("menu.open", "Open");
    m.insert("menu.open_with", "Open With…");
    m.insert("menu.open_in_terminal", "Open in Terminal");
    m.insert("menu.open_new_tab", "Open in New Tab");
    m.insert("menu.rename", "Rename");
    m.insert("menu.cut", "Cut");
    m.insert("menu.copy", "Copy");
    m.insert("menu.paste", "Paste");
    m.insert("menu.duplicate", "Duplicate");
    m.insert("menu.move_to_trash", "Move to Trash");
    m.insert("menu.delete_permanently", "Delete Permanently");
    m.insert("menu.restore_from_trash", "Restore from Trash");
    m.insert("menu.empty_trash", "Empty Trash");
    m.insert("menu.compress", "Compress");
    m.insert("menu.extract_here", "Extract Here");
    m.insert("menu.properties", "Properties");
    m.insert("menu.copy_path", "Copy Path");
    m.insert("menu.new_folder", "New Folder");
    m.insert("menu.new_file", "New File");
    m.insert("menu.select_all", "Select All");
    m.insert("menu.invert_selection", "Invert Selection");
    m.insert("menu.show_hidden", "Show Hidden Files");

    m.insert("dialog.rename_title", "Rename Item");
    m.insert("dialog.rename_placeholder", "New name");
    m.insert("dialog.new_folder_title", "New Folder");
    m.insert("dialog.new_folder_placeholder", "Folder name");
    m.insert("dialog.new_file_title", "New File");
    m.insert("dialog.new_file_placeholder", "File name");
    m.insert("dialog.delete_confirm_title", "Delete Permanently?");
    m.insert(
        "dialog.delete_confirm_body",
        "This action cannot be undone. The selected items will be permanently deleted.",
    );
    m.insert("dialog.cancel", "Cancel");
    m.insert("dialog.confirm", "Confirm");
    m.insert("dialog.delete", "Delete");
    m.insert("dialog.ok", "OK");
    m.insert("dialog.close", "Close");
    m.insert("dialog.overwrite_title", "File Already Exists");
    m.insert(
        "dialog.overwrite_body",
        "An item with this name already exists in this folder. Replace it?",
    );
    m.insert("dialog.overwrite_replace", "Replace");
    m.insert("dialog.overwrite_skip", "Skip");

    m.insert("properties.title", "Properties");
    m.insert("properties.name", "Name");
    m.insert("properties.type", "Type");
    m.insert("properties.size", "Size");
    m.insert("properties.location", "Location");
    m.insert("properties.modified", "Modified");
    m.insert("properties.created", "Created");
    m.insert("properties.accessed", "Accessed");
    m.insert("properties.permissions", "Permissions");
    m.insert("properties.items_count", "Items");
    m.insert("properties.folder", "Folder");
    m.insert("properties.file", "File");
    m.insert("properties.symlink", "Symbolic Link");
    m.insert("properties.app_id", "Application ID");

    m.insert("status.items_selected", "{} items selected");
    m.insert("status.items_total", "{} items");
    m.insert("status.loading", "Loading…");
    m.insert("status.empty_folder", "This folder is empty");
    m.insert("status.search_placeholder", "Search files and folders…");
    m.insert("status.search_results", "{} results");
    m.insert("status.no_results", "No results found");

    m.insert("apps.title", "Applications");
    m.insert("apps.loading", "Loading applications…");
    m.insert("apps.launch_failed", "Failed to launch application: {}");
    m.insert("apps.no_apps_found", "No applications found");
    m.insert("apps.uninstall_title", "Uninstall Application");
    m.insert("apps.uninstall_detecting", "Detecting distribution and package…");
    m.insert(
        "apps.uninstall_body_with_package",
        "\"{}\" belongs to the package \"{}\", installed via {}. The application and any dependencies not used by other programs will be permanently removed.",
    );
    m.insert(
        "apps.uninstall_body_unknown_package",
        "Could not determine which package \"{}\" belongs to in {}. The application may not have been installed through the system package manager.",
    );
    m.insert(
        "apps.uninstall_body_unknown_distro",
        "Could not detect the distribution or its package manager. Only the file \"{}\" itself will be removed, without dependencies — this is NOT recommended, as it may leave unused clutter or break other programs relying on the same libraries.",
    );
    m.insert("apps.uninstall_confirm", "Remove");
    m.insert("apps.uninstall_confirm_unsafe", "Remove file only anyway");
    m.insert("apps.uninstall_needs_password", "Administrator password will be required");
    m.insert("apps.uninstall_in_progress", "Removing…");
    m.insert("apps.uninstall_success", "\"{}\" removed");
    m.insert("apps.uninstall_failed", "Failed to remove \"{}\": {}");
    m.insert(
        "apps.uninstall_pkexec_failed",
        "Could not automatically request administrator privileges. The command has been pasted into the terminal — enter your password there.",
    );
    m.insert("apps.search_placeholder", "Search applications…");

    m.insert("terminal.title", "Embedded Terminal");
    m.insert("terminal.close", "Close Terminal");
    m.insert("terminal.toggle_hint", "Ctrl+` — toggle terminal");

    m.insert("error.generic_title", "An Error Occurred");
    m.insert("error.permission_denied", "Permission Denied");
    m.insert("error.not_found", "Path Not Found");
    m.insert("error.io_error", "I/O error: {}");
    m.insert("error.rename_failed", "Failed to rename item: {}");
    m.insert("error.delete_failed", "Failed to delete item: {}");
    m.insert("error.create_failed", "Failed to create item: {}");
    m.insert("error.copy_failed", "Failed to copy item: {}");
    m.insert("error.move_failed", "Failed to move item: {}");

    m.insert("settings.title", "Settings");
    m.insert("settings.language", "Interface Language");
    m.insert(
        "settings.language_restart_notice",
        "Changes will take effect after restarting FasdManager.",
    );
    m.insert("settings.appearance", "Appearance");
    m.insert("settings.icon_theme", "Icon Theme");
    m.insert("settings.icon_theme_fasd_finder", "FasdManager (Default)");
    m.insert("settings.icon_theme_adwaita", "Adwaita");
    m.insert("settings.icon_theme_breeze", "Breeze");
    m.insert("settings.icon_theme_papirus", "Papirus");
    m.insert("settings.icon_theme_system", "System Theme");
    m.insert(
        "settings.icon_theme_not_found",
        "Theme \"{}\" was not found on this system, showing available icons",
    );
    m.insert("settings.theme_system", "Follow System");
    m.insert("settings.theme_light", "Light");
    m.insert("settings.theme_dark", "Dark");
    m.insert("settings.show_hidden_files", "Show Hidden Files");
    m.insert("about.title", "About FasdManager");
    m.insert(
        "about.description",
        "A file manager for Linux. Created by Fasdeq13",
    );

    m
}
