use std::collections::HashMap;

pub fn build_table() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();

    m.insert("app.title", "FasdManager");

    m.insert("sidebar.favorites", "收藏夹");
    m.insert("sidebar.home", "主文件夹");
    m.insert("sidebar.desktop", "桌面");
    m.insert("sidebar.documents", "文档");
    m.insert("sidebar.downloads", "下载");
    m.insert("sidebar.pictures", "图片");
    m.insert("sidebar.music", "音乐");
    m.insert("sidebar.videos", "视频");
    m.insert("sidebar.applications", "应用程序");
    m.insert("sidebar.trash", "回收站");
    m.insert("sidebar.locations", "位置");
    m.insert("sidebar.filesystem", "文件系统");
    m.insert("sidebar.network", "网络");
    m.insert("sidebar.devices", "设备");
    m.insert("sidebar.bookmarks", "书签");

    m.insert("toolbar.back", "后退");
    m.insert("toolbar.forward", "前进");
    m.insert("toolbar.up", "上一级");
    m.insert("toolbar.reload", "刷新");
    m.insert("toolbar.new_folder", "新建文件夹");
    m.insert("toolbar.new_tab", "新建标签页");
    m.insert("toolbar.split_view", "分屏视图");
    m.insert("toolbar.terminal", "终端");
    m.insert("toolbar.search", "搜索");
    m.insert("toolbar.view_grid", "网格视图");
    m.insert("toolbar.view_list", "列表视图");
    m.insert("toolbar.sort", "排序");
    m.insert("toolbar.menu", "菜单");
    m.insert("toolbar.properties", "属性");

    m.insert("menu.open", "打开");
    m.insert("menu.open_with", "打开方式…");
    m.insert("menu.open_in_terminal", "在终端中打开");
    m.insert("menu.open_new_tab", "在新标签页中打开");
    m.insert("menu.rename", "重命名");
    m.insert("menu.cut", "剪切");
    m.insert("menu.copy", "复制");
    m.insert("menu.paste", "粘贴");
    m.insert("menu.duplicate", "创建副本");
    m.insert("menu.move_to_trash", "移到回收站");
    m.insert("menu.delete_permanently", "永久删除");
    m.insert("menu.restore_from_trash", "从回收站还原");
    m.insert("menu.empty_trash", "清空回收站");
    m.insert("menu.compress", "压缩");
    m.insert("menu.extract_here", "解压到当前文件夹");
    m.insert("menu.properties", "属性");
    m.insert("menu.copy_path", "复制路径");
    m.insert("menu.new_folder", "新建文件夹");
    m.insert("menu.new_file", "新建文件");
    m.insert("menu.select_all", "全选");
    m.insert("menu.invert_selection", "反向选择");
    m.insert("menu.show_hidden", "显示隐藏文件");

    m.insert("dialog.rename_title", "重命名项目");
    m.insert("dialog.rename_placeholder", "新名称");
    m.insert("dialog.new_folder_title", "新建文件夹");
    m.insert("dialog.new_folder_placeholder", "文件夹名称");
    m.insert("dialog.new_file_title", "新建文件");
    m.insert("dialog.new_file_placeholder", "文件名称");
    m.insert("dialog.delete_confirm_title", "确定要永久删除吗？");
    m.insert(
        "dialog.delete_confirm_body",
        "此操作无法撤销。所选项目将被永久删除。",
    );
    m.insert("dialog.cancel", "取消");
    m.insert("dialog.confirm", "确认");
    m.insert("dialog.delete", "删除");
    m.insert("dialog.ok", "确定");
    m.insert("dialog.close", "关闭");
    m.insert("dialog.overwrite_title", "文件已存在");
    m.insert(
        "dialog.overwrite_body",
        "此文件夹中已存在同名项目。是否替换它？",
    );
    m.insert("dialog.overwrite_replace", "替换");
    m.insert("dialog.overwrite_skip", "跳过");

    m.insert("properties.title", "属性");
    m.insert("properties.name", "名称");
    m.insert("properties.type", "类型");
    m.insert("properties.size", "大小");
    m.insert("properties.location", "位置");
    m.insert("properties.modified", "修改时间");
    m.insert("properties.created", "创建时间");
    m.insert("properties.accessed", "访问时间");
    m.insert("properties.permissions", "权限");
    m.insert("properties.items_count", "项目数");
    m.insert("properties.folder", "文件夹");
    m.insert("properties.file", "文件");
    m.insert("properties.symlink", "符号链接");
    m.insert("properties.app_id", "应用程序标识符");

    m.insert("status.items_selected", "已选择 {} 个项目");
    m.insert("status.items_total", "共 {} 个项目");
    m.insert("status.loading", "正在加载…");
    m.insert("status.empty_folder", "此文件夹为空");
    m.insert("status.search_placeholder", "搜索文件和文件夹…");
    m.insert("status.search_results", "找到 {} 个结果");
    m.insert("status.no_results", "未找到任何结果");

    m.insert("apps.title", "应用程序");
    m.insert("apps.loading", "正在加载应用程序列表…");
    m.insert("apps.launch_failed", "无法启动应用程序：{}");
    m.insert("apps.no_apps_found", "未找到应用程序");
    m.insert("apps.uninstall_title", "卸载应用程序");
    m.insert("apps.uninstall_detecting", "正在检测发行版和软件包…");
    m.insert(
        "apps.uninstall_body_with_package",
        "「{}」属于软件包「{}」，通过 {} 安装。该应用程序及未被其他程序使用的依赖项将被永久删除。",
    );
    m.insert(
        "apps.uninstall_body_unknown_package",
        "无法确定「{}」在 {} 中属于哪个软件包。该应用程序可能不是通过系统软件包管理器安装的。",
    );
    m.insert(
        "apps.uninstall_body_unknown_distro",
        "无法检测发行版及其软件包管理器。将仅删除文件「{}」本身，不含依赖项——不建议这样做，因为可能会留下未使用的残留文件，或破坏依赖相同库的其他程序。",
    );
    m.insert("apps.uninstall_confirm", "删除");
    m.insert("apps.uninstall_confirm_unsafe", "仍然仅删除文件");
    m.insert("apps.uninstall_needs_password", "将需要管理员密码");
    m.insert("apps.uninstall_in_progress", "正在删除…");
    m.insert("apps.uninstall_success", "「{}」已删除");
    m.insert("apps.uninstall_failed", "删除「{}」失败：{}");
    m.insert(
        "apps.uninstall_pkexec_failed",
        "无法自动请求管理员权限。命令已粘贴到终端中——请在那里输入密码。",
    );
    m.insert("apps.search_placeholder", "搜索应用程序…");

    m.insert("terminal.title", "内置终端");
    m.insert("terminal.close", "关闭终端");
    m.insert("terminal.toggle_hint", "Ctrl+` — 显示/隐藏终端");

    m.insert("error.generic_title", "发生错误");
    m.insert("error.permission_denied", "权限被拒绝");
    m.insert("error.not_found", "未找到路径");
    m.insert("error.io_error", "输入输出错误：{}");
    m.insert("error.rename_failed", "重命名项目失败：{}");
    m.insert("error.delete_failed", "删除项目失败：{}");
    m.insert("error.create_failed", "创建项目失败：{}");
    m.insert("error.copy_failed", "复制项目失败：{}");
    m.insert("error.move_failed", "移动项目失败：{}");

    m.insert("settings.title", "设置");
    m.insert("settings.language", "界面语言");
    m.insert("settings.appearance", "外观");
    m.insert("settings.theme_system", "跟随系统");
    m.insert("settings.theme_light", "浅色");
    m.insert("settings.theme_dark", "深色");
    m.insert("settings.show_hidden_files", "显示隐藏文件");
    m.insert("about.title", "关于 FasdManager");
    m.insert(
        "about.description",
        "融合 macOS Finder 与 GNOME Nautilus 界面风格的混合式文件管理器",
    );

    m
}
