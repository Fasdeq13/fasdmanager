use std::collections::HashMap;

pub fn build_table() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();

    m.insert("app.title", "FasdManager");

    m.insert("sidebar.favorites", "Preferiti");
    m.insert("sidebar.home", "Home");
    m.insert("sidebar.desktop", "Scrivania");
    m.insert("sidebar.documents", "Documenti");
    m.insert("sidebar.downloads", "Scaricati");
    m.insert("sidebar.pictures", "Immagini");
    m.insert("sidebar.music", "Musica");
    m.insert("sidebar.videos", "Video");
    m.insert("sidebar.applications", "Applicazioni");
    m.insert("sidebar.trash", "Cestino");
    m.insert("sidebar.locations", "Percorsi");
    m.insert("sidebar.filesystem", "File System");
    m.insert("sidebar.network", "Rete");
    m.insert("sidebar.devices", "Dispositivi");
    m.insert("sidebar.bookmarks", "Segnalibri");
    m.insert("devices.unmount", "Smonta");
    m.insert("devices.unmounting", "Smontaggio…");
    m.insert("devices.unmount_flushing", "Scrittura dati sul dispositivo…");
    m.insert("devices.unmount_flushing_bytes", "Rimanente da scrivere: {}");
    m.insert("devices.unmount_success", "\"{}\" smontato in sicurezza");
    m.insert("devices.unmount_failed", "Impossibile smontare \"{}\": {}");
    m.insert("sidebar.hide_item", "Nascondi dalla Barra Laterale");
    m.insert("sidebar.hide_section", "Nascondi Sezione");
    m.insert("sidebar.manage_items", "Mostra Elementi Nascosti…");
    m.insert("sidebar.restore_hidden_title", "Elementi Nascosti della Barra Laterale");
    m.insert("sidebar.restore_hidden_empty", "Niente è nascosto");

    m.insert("toolbar.back", "Indietro");
    m.insert("toolbar.forward", "Avanti");
    m.insert("toolbar.up", "Su");
    m.insert("toolbar.reload", "Aggiorna");
    m.insert("toolbar.new_folder", "Nuova Cartella");
    m.insert("toolbar.new_tab", "Nuova Scheda");
    m.insert("toolbar.split_view", "Vista Divisa");
    m.insert("toolbar.terminal", "Terminale");
    m.insert("toolbar.search", "Cerca");
    m.insert("toolbar.view_grid", "Griglia");
    m.insert("toolbar.view_list", "Elenco");
    m.insert("toolbar.sort", "Ordina");
    m.insert("toolbar.menu", "Menu");
    m.insert("toolbar.properties", "Proprietà");

    m.insert("menu.open", "Apri");
    m.insert("menu.open_with", "Apri Con…");
    m.insert("menu.open_in_terminal", "Apri nel Terminale");
    m.insert("menu.open_new_tab", "Apri in Nuova Scheda");
    m.insert("menu.rename", "Rinomina");
    m.insert("menu.cut", "Taglia");
    m.insert("menu.copy", "Copia");
    m.insert("menu.paste", "Incolla");
    m.insert("menu.duplicate", "Duplica");
    m.insert("menu.move_to_trash", "Sposta nel Cestino");
    m.insert("menu.delete_permanently", "Elimina Definitivamente");
    m.insert("menu.restore_from_trash", "Ripristina dal Cestino");
    m.insert("menu.empty_trash", "Svuota Cestino");
    m.insert("menu.compress", "Comprimi");
    m.insert("menu.extract_here", "Estrai Qui");
    m.insert("menu.properties", "Proprietà");
    m.insert("menu.copy_path", "Copia Percorso");
    m.insert("menu.new_folder", "Crea Cartella");
    m.insert("menu.new_file", "Crea File");
    m.insert("menu.select_all", "Seleziona Tutto");
    m.insert("menu.invert_selection", "Inverti Selezione");
    m.insert("menu.show_hidden", "Mostra File Nascosti");

    m.insert("dialog.rename_title", "Rinomina Elemento");
    m.insert("dialog.rename_placeholder", "Nuovo nome");
    m.insert("dialog.new_folder_title", "Nuova Cartella");
    m.insert("dialog.new_folder_placeholder", "Nome cartella");
    m.insert("dialog.new_file_title", "Nuovo File");
    m.insert("dialog.new_file_placeholder", "Nome file");
    m.insert("dialog.delete_confirm_title", "Eliminare Definitivamente?");
    m.insert(
        "dialog.delete_confirm_body",
        "Questa azione non può essere annullata. Gli elementi selezionati saranno eliminati per sempre.",
    );
    m.insert("dialog.cancel", "Annulla");
    m.insert("dialog.confirm", "Conferma");
    m.insert("dialog.delete", "Elimina");
    m.insert("dialog.ok", "OK");
    m.insert("dialog.close", "Chiudi");
    m.insert("dialog.overwrite_title", "Il File Esiste Già");
    m.insert(
        "dialog.overwrite_body",
        "Un elemento con questo nome esiste già in questa cartella. Sostituirlo?",
    );
    m.insert("dialog.overwrite_replace", "Sostituisci");
    m.insert("dialog.overwrite_skip", "Salta");

    m.insert("properties.title", "Proprietà");
    m.insert("properties.name", "Nome");
    m.insert("properties.type", "Tipo");
    m.insert("properties.size", "Dimensione");
    m.insert("properties.location", "Percorso");
    m.insert("properties.modified", "Modificato");
    m.insert("properties.created", "Creato");
    m.insert("properties.accessed", "Accesso");
    m.insert("properties.permissions", "Permessi");
    m.insert("properties.items_count", "Elementi");
    m.insert("properties.folder", "Cartella");
    m.insert("properties.file", "File");
    m.insert("properties.symlink", "Collegamento Simbolico");
    m.insert("properties.app_id", "ID Applicazione");

    m.insert("status.items_selected", "{} elementi selezionati");
    m.insert("status.items_total", "{} elementi");
    m.insert("status.loading", "Caricamento…");
    m.insert("status.empty_folder", "Questa cartella è vuota");
    m.insert("status.search_placeholder", "Cerca file e cartelle…");
    m.insert("status.search_results", "{} risultati");
    m.insert("status.no_results", "Nessun risultato trovato");

    m.insert("apps.title", "Applicazioni");
    m.insert("apps.loading", "Caricamento applicazioni…");
    m.insert("apps.launch_failed", "Impossibile avviare l'applicazione: {}");
    m.insert("apps.no_apps_found", "Nessuna applicazione trovata");
    m.insert("apps.uninstall_title", "Disinstalla Applicazione");
    m.insert("apps.uninstall_detecting", "Rilevamento distribuzione e pacchetto…");
    m.insert(
        "apps.uninstall_body_with_package",
        "\"{}\" appartiene al pacchetto \"{}\", installato tramite {}. L'applicazione e le dipendenze non utilizzate da altri programmi saranno rimosse definitivamente.",
    );
    m.insert(
        "apps.uninstall_body_unknown_package",
        "Impossibile determinare a quale pacchetto appartiene \"{}\" in {}. L'applicazione potrebbe non essere stata installata tramite il gestore pacchetti di sistema.",
    );
    m.insert(
        "apps.uninstall_body_unknown_distro",
        "Impossibile rilevare la distribuzione o il suo gestore pacchetti. Verrà rimosso solo il file \"{}\", senza dipendenze — questo NON è consigliato, poiché potrebbe lasciare file inutilizzati o danneggiare altri programmi che dipendono dalle stesse librerie.",
    );
    m.insert("apps.uninstall_confirm", "Rimuovi");
    m.insert("apps.uninstall_confirm_unsafe", "Rimuovi comunque solo il file");
    m.insert("apps.uninstall_needs_password", "Sarà richiesta la password di amministratore");
    m.insert("apps.uninstall_in_progress", "Rimozione…");
    m.insert("apps.uninstall_success", "\"{}\" rimosso");
    m.insert("apps.uninstall_failed", "Impossibile rimuovere \"{}\": {}");
    m.insert(
        "apps.uninstall_pkexec_failed",
        "Impossibile richiedere automaticamente i privilegi di amministratore. Il comando è stato incollato nel terminale — inserisci la password lì.",
    );
    m.insert("apps.search_placeholder", "Cerca applicazioni…");

    m.insert("terminal.title", "Terminale Integrato");
    m.insert("terminal.close", "Chiudi Terminale");
    m.insert("terminal.toggle_hint", "Ctrl+` — mostra/nascondi terminale");

    m.insert("error.generic_title", "Si è Verificato un Errore");
    m.insert("error.permission_denied", "Permesso Negato");
    m.insert("error.not_found", "Percorso Non Trovato");
    m.insert("error.io_error", "Errore di I/O: {}");
    m.insert("error.rename_failed", "Impossibile rinominare l'elemento: {}");
    m.insert("error.delete_failed", "Impossibile eliminare l'elemento: {}");
    m.insert("error.create_failed", "Impossibile creare l'elemento: {}");
    m.insert("error.copy_failed", "Impossibile copiare l'elemento: {}");
    m.insert("error.move_failed", "Impossibile spostare l'elemento: {}");

    m.insert("settings.title", "Impostazioni");
    m.insert("settings.language", "Lingua dell'Interfaccia");
    m.insert(
        "settings.language_restart_notice",
        "Le modifiche avranno effetto dopo aver riavviato FasdManager.",
    );
    m.insert("settings.appearance", "Aspetto");
    m.insert("settings.icon_theme", "Tema delle Icone");
    m.insert("settings.icon_theme_fasd_finder", "FasdManager (Predefinito)");
    m.insert("settings.icon_theme_adwaita", "Adwaita");
    m.insert("settings.icon_theme_breeze", "Breeze");
    m.insert("settings.icon_theme_papirus", "Papirus");
    m.insert("settings.icon_theme_system", "Tema di Sistema");
    m.insert(
        "settings.icon_theme_not_found",
        "Il tema \"{}\" non è stato trovato su questo sistema, vengono mostrate le icone disponibili",
    );
    m.insert("settings.theme_system", "Segui il Sistema");
    m.insert("settings.theme_light", "Chiaro");
    m.insert("settings.theme_dark", "Scuro");
    m.insert("settings.show_hidden_files", "Mostra File Nascosti");
    m.insert("about.title", "Informazioni su FasdManager");
    m.insert(
        "about.description",
        "Un file manager per Linux. Creato da Fasdeq13",
    );

    m
}
