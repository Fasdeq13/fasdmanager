use std::path::PathBuf;

#[derive(Debug)]
pub enum TrashOpError {
    Backend(String),
}

impl std::fmt::Display for TrashOpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrashOpError::Backend(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for TrashOpError {}

pub fn move_to_trash(paths: &[PathBuf]) -> Result<(), TrashOpError> {
    trash::delete_all(paths).map_err(|e| TrashOpError::Backend(e.to_string()))
}

#[derive(Debug, Clone)]
pub struct TrashedItem {
    pub id_name: String,
    pub original_path: PathBuf,
    pub display_name: String,
}

#[cfg(target_os = "linux")]
pub fn list_trash_items() -> Result<Vec<TrashedItem>, TrashOpError> {
    let items = trash::os_limited::list().map_err(|e| TrashOpError::Backend(e.to_string()))?;
    Ok(items
        .into_iter()
        .map(|item| TrashedItem {
            id_name: item.id.clone().to_string_lossy().to_string(),
            original_path: item.original_path(),
            display_name: item.name.clone(),
        })
        .collect())
}

#[cfg(not(target_os = "linux"))]
pub fn list_trash_items() -> Result<Vec<TrashedItem>, TrashOpError> {
    Ok(Vec::new())
}

#[cfg(target_os = "linux")]
pub fn empty_trash() -> Result<(), TrashOpError> {
    let items = trash::os_limited::list().map_err(|e| TrashOpError::Backend(e.to_string()))?;
    trash::os_limited::purge_all(items).map_err(|e| TrashOpError::Backend(e.to_string()))
}

#[cfg(not(target_os = "linux"))]
pub fn empty_trash() -> Result<(), TrashOpError> {
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn restore_by_display_name(display_name: &str) -> Result<(), TrashOpError> {
    let items = trash::os_limited::list().map_err(|e| TrashOpError::Backend(e.to_string()))?;
    let matching: Vec<_> = items
        .into_iter()
        .filter(|item| item.name == display_name)
        .collect();
    if matching.is_empty() {
        return Err(TrashOpError::Backend(format!(
            "trashed item not found: {display_name}"
        )));
    }
    trash::os_limited::restore_all(matching).map_err(|e| TrashOpError::Backend(e.to_string()))
}

#[cfg(not(target_os = "linux"))]
pub fn restore_by_display_name(_display_name: &str) -> Result<(), TrashOpError> {
    Err(TrashOpError::Backend(
        "restore is only supported on linux in this build".to_string(),
    ))
}

pub async fn move_to_trash_async(paths: Vec<PathBuf>) -> Result<(), TrashOpError> {
    let handle = tokio::task::spawn_blocking(move || move_to_trash(&paths));
    handle
        .await
        .map_err(|e| TrashOpError::Backend(e.to_string()))?
}

pub async fn list_trash_items_async() -> Result<Vec<TrashedItem>, TrashOpError> {
    let handle = tokio::task::spawn_blocking(list_trash_items);
    handle
        .await
        .map_err(|e| TrashOpError::Backend(e.to_string()))?
}

pub async fn empty_trash_async() -> Result<(), TrashOpError> {
    let handle = tokio::task::spawn_blocking(empty_trash);
    handle
        .await
        .map_err(|e| TrashOpError::Backend(e.to_string()))?
}

pub async fn restore_by_display_name_async(display_name: String) -> Result<(), TrashOpError> {
    let handle = tokio::task::spawn_blocking(move || restore_by_display_name(&display_name));
    handle
        .await
        .map_err(|e| TrashOpError::Backend(e.to_string()))?
}
