use std::collections::HashMap;

pub fn build_table() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();

    m.insert("app.title", "FasdManager");

    m.insert("sidebar.favorites", "Избранное");
    m.insert("sidebar.home", "Домашняя папка");
    m.insert("sidebar.desktop", "Рабочий стол");
    m.insert("sidebar.documents", "Документы");
    m.insert("sidebar.downloads", "Загрузки");
    m.insert("sidebar.pictures", "Изображения");
    m.insert("sidebar.music", "Музыка");
    m.insert("sidebar.videos", "Видео");
    m.insert("sidebar.applications", "Приложения");
    m.insert("sidebar.trash", "Корзина");
    m.insert("sidebar.locations", "Расположения");
    m.insert("sidebar.filesystem", "Файловая система");
    m.insert("sidebar.network", "Сеть");
    m.insert("sidebar.devices", "Устройства");
    m.insert("sidebar.bookmarks", "Закладки");

    m.insert("toolbar.back", "Назад");
    m.insert("toolbar.forward", "Вперёд");
    m.insert("toolbar.up", "Наверх");
    m.insert("toolbar.reload", "Обновить");
    m.insert("toolbar.new_folder", "Новая папка");
    m.insert("toolbar.new_tab", "Новая вкладка");
    m.insert("toolbar.split_view", "Разделить экран");
    m.insert("toolbar.terminal", "Терминал");
    m.insert("toolbar.search", "Поиск");
    m.insert("toolbar.view_grid", "Сетка");
    m.insert("toolbar.view_list", "Список");
    m.insert("toolbar.sort", "Сортировка");
    m.insert("toolbar.menu", "Меню");
    m.insert("toolbar.properties", "Свойства");

    m.insert("menu.open", "Открыть");
    m.insert("menu.open_with", "Открыть с помощью…");
    m.insert("menu.open_in_terminal", "Открыть в терминале");
    m.insert("menu.open_new_tab", "Открыть в новой вкладке");
    m.insert("menu.rename", "Переименовать");
    m.insert("menu.cut", "Вырезать");
    m.insert("menu.copy", "Копировать");
    m.insert("menu.paste", "Вставить");
    m.insert("menu.duplicate", "Дублировать");
    m.insert("menu.move_to_trash", "Переместить в корзину");
    m.insert("menu.delete_permanently", "Удалить безвозвратно");
    m.insert("menu.restore_from_trash", "Восстановить из корзины");
    m.insert("menu.empty_trash", "Очистить корзину");
    m.insert("menu.compress", "Сжать");
    m.insert("menu.extract_here", "Извлечь сюда");
    m.insert("menu.properties", "Свойства");
    m.insert("menu.copy_path", "Скопировать путь");
    m.insert("menu.new_folder", "Создать папку");
    m.insert("menu.new_file", "Создать файл");
    m.insert("menu.select_all", "Выделить всё");
    m.insert("menu.invert_selection", "Инвертировать выделение");
    m.insert("menu.show_hidden", "Показать скрытые файлы");

    m.insert("dialog.rename_title", "Переименовать элемент");
    m.insert("dialog.rename_placeholder", "Новое имя");
    m.insert("dialog.new_folder_title", "Новая папка");
    m.insert("dialog.new_folder_placeholder", "Имя папки");
    m.insert("dialog.new_file_title", "Новый файл");
    m.insert("dialog.new_file_placeholder", "Имя файла");
    m.insert("dialog.delete_confirm_title", "Удалить безвозвратно?");
    m.insert(
        "dialog.delete_confirm_body",
        "Это действие нельзя отменить. Выбранные элементы будут удалены навсегда.",
    );
    m.insert("dialog.cancel", "Отмена");
    m.insert("dialog.confirm", "Подтвердить");
    m.insert("dialog.delete", "Удалить");
    m.insert("dialog.ok", "ОК");
    m.insert("dialog.close", "Закрыть");
    m.insert("dialog.overwrite_title", "Файл уже существует");
    m.insert(
        "dialog.overwrite_body",
        "Элемент с таким именем уже существует в этой папке. Заменить его?",
    );
    m.insert("dialog.overwrite_replace", "Заменить");
    m.insert("dialog.overwrite_skip", "Пропустить");

    m.insert("properties.title", "Свойства");
    m.insert("properties.name", "Имя");
    m.insert("properties.type", "Тип");
    m.insert("properties.size", "Размер");
    m.insert("properties.location", "Расположение");
    m.insert("properties.modified", "Изменён");
    m.insert("properties.created", "Создан");
    m.insert("properties.accessed", "Открыт");
    m.insert("properties.permissions", "Права доступа");
    m.insert("properties.items_count", "Элементов");
    m.insert("properties.folder", "Папка");
    m.insert("properties.file", "Файл");
    m.insert("properties.symlink", "Символическая ссылка");
    m.insert("properties.app_id", "Идентификатор приложения");

    m.insert("status.items_selected", "Выбрано элементов: {}");
    m.insert("status.items_total", "Элементов: {}");
    m.insert("status.loading", "Загрузка…");
    m.insert("status.empty_folder", "Папка пуста");
    m.insert("status.search_placeholder", "Поиск файлов и папок…");
    m.insert("status.search_results", "Результаты поиска: {}");
    m.insert("status.no_results", "Ничего не найдено");

    m.insert("apps.title", "Приложения");
    m.insert("apps.loading", "Загрузка списка приложений…");
    m.insert("apps.launch_failed", "Не удалось запустить приложение: {}");
    m.insert("apps.no_apps_found", "Приложения не найдены");
    m.insert("apps.uninstall_title", "Удалить программу");
    m.insert("apps.uninstall_detecting", "Определение дистрибутива и пакета…");
    m.insert(
        "apps.uninstall_body_with_package",
        "«{}» относится к пакету «{}», установленному через {}. Приложение и все его зависимости, не используемые другими программами, будут удалены безвозвратно.",
    );
    m.insert(
        "apps.uninstall_body_unknown_package",
        "Не удалось определить пакет, которому принадлежит «{}» в менеджере {}. Возможно, приложение установлено не через системный менеджер пакетов.",
    );
    m.insert(
        "apps.uninstall_body_unknown_distro",
        "Не удалось определить дистрибутив и его пакетный менеджер. Будет удалён только сам файл «{}», без зависимостей — это НЕ рекомендуется, так как может остаться неиспользуемый мусор или сломать другие программы, зависящие от тех же библиотек.",
    );
    m.insert("apps.uninstall_confirm", "Удалить");
    m.insert("apps.uninstall_confirm_unsafe", "Всё равно удалить только файл");
    m.insert("apps.uninstall_needs_password", "Потребуется пароль администратора");
    m.insert("apps.uninstall_in_progress", "Удаление…");
    m.insert("apps.uninstall_success", "«{}» удалено");
    m.insert("apps.uninstall_failed", "Не удалось удалить «{}»: {}");
    m.insert(
        "apps.uninstall_pkexec_failed",
        "Не удалось запросить права администратора автоматически. Команда для ручного запуска вставлена в терминал — введите пароль там.",
    );
    m.insert("apps.search_placeholder", "Поиск приложений…");

    m.insert("terminal.title", "Встроенный терминал");
    m.insert("terminal.close", "Закрыть терминал");
    m.insert("terminal.toggle_hint", "Ctrl+` — показать/скрыть терминал");

    m.insert("error.generic_title", "Произошла ошибка");
    m.insert("error.permission_denied", "Доступ запрещён");
    m.insert("error.not_found", "Путь не найден");
    m.insert("error.io_error", "Ошибка ввода-вывода: {}");
    m.insert("error.rename_failed", "Не удалось переименовать элемент: {}");
    m.insert("error.delete_failed", "Не удалось удалить элемент: {}");
    m.insert("error.create_failed", "Не удалось создать элемент: {}");
    m.insert("error.copy_failed", "Не удалось скопировать элемент: {}");
    m.insert("error.move_failed", "Не удалось переместить элемент: {}");

    m.insert("settings.title", "Настройки");
    m.insert("settings.language", "Язык интерфейса");
    m.insert("settings.appearance", "Оформление");
    m.insert("settings.theme_system", "Как в системе");
    m.insert("settings.theme_light", "Светлая");
    m.insert("settings.theme_dark", "Тёмная");
    m.insert("settings.show_hidden_files", "Показывать скрытые файлы");
    m.insert("about.title", "О программе FasdManager");
    m.insert(
        "about.description",
        "Гибридный файловый менеджер с интерфейсом macOS Finder и GNOME Nautilus",
    );

    m
}
