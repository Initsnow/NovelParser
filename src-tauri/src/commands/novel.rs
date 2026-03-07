use crate::models::*;
use crate::{epub_parser, txt_parser, AppState};
use tauri::State;

/// Shared import helper: creates novel + saves chapters in a transaction.
fn do_import_novel(
    state: &State<AppState>,
    title: String,
    source_type: SourceType,
    chapters: Vec<(String, String)>,
) -> Result<String, String> {
    let novel_id = uuid::Uuid::new_v4().to_string();
    let novel = Novel {
        id: novel_id.clone(),
        title,
        source_type,
        enabled_dimensions: AnalysisDimension::default_set(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.save_novel_with_chapters(&novel, chapters)
        .map_err(|e| e.to_string())?;
    Ok(novel_id)
}

#[tauri::command]
pub fn list_novels(state: State<AppState>) -> Result<Vec<NovelMeta>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.list_novels().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn preview_epub(path: String) -> Result<EpubPreview, String> {
    let (title, chapters) = epub_parser::preview_epub(&path)?;
    Ok(EpubPreview {
        title,
        path,
        chapters,
    })
}

#[tauri::command]
pub fn import_epub_selected(
    state: State<AppState>,
    path: String,
    selected_indices: Vec<usize>,
) -> Result<String, String> {
    let (title, chapters) = epub_parser::parse_epub_selected(&path, &selected_indices)?;
    do_import_novel(&state, title, SourceType::Epub(path), chapters)
}

#[tauri::command]
pub fn import_txt_files(state: State<AppState>, paths: Vec<String>) -> Result<String, String> {
    let (title, chapters) = txt_parser::parse_txt_files(paths.clone())?;
    do_import_novel(&state, title, SourceType::TxtFiles(paths), chapters)
}

#[tauri::command]
pub fn import_single_txt(state: State<AppState>, path: String) -> Result<String, String> {
    let (title, chapters) = txt_parser::parse_single_txt(&path)?;
    do_import_novel(&state, title, SourceType::SingleTxt(path), chapters)
}

#[tauri::command]
pub fn delete_novel(state: State<AppState>, novel_id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_novel(&novel_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_novel(state: State<AppState>, novel_id: String) -> Result<Novel, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.load_novel(&novel_id).map_err(|e| e.to_string())
}
