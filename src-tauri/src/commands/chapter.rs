use crate::models::*;
use crate::token_utils;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn list_chapters(state: State<AppState>, novel_id: String) -> Result<Vec<ChapterMeta>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.list_chapter_metas(&novel_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_chapter(state: State<AppState>, chapter_id: i64) -> Result<Chapter, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.load_chapter(chapter_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_chapter_content(state: State<AppState>, chapter_id: i64) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.load_chapter_content(chapter_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn hydrate_chapter_token_estimates(
    state: State<AppState>,
    chapter_ids: Vec<i64>,
) -> Result<Vec<ChapterTokenCount>, String> {
    let mut updates = Vec::new();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let config = db.load_llm_config().unwrap_or_default();

    for chapter_id in chapter_ids {
        if let Some(existing) = db
            .load_chapter_token_count(chapter_id)
            .map_err(|e| e.to_string())?
        {
            updates.push(ChapterTokenCount {
                chapter_id,
                token_count: existing,
            });
            continue;
        }

        let content = db.load_chapter_content(chapter_id).map_err(|e| e.to_string())?;
        let token_count = token_utils::estimate_tokens_for_model(&content, &config.model);
        db.save_chapter_token_count(chapter_id, token_count)
            .map_err(|e| e.to_string())?;
        updates.push(ChapterTokenCount {
            chapter_id,
            token_count,
        });
    }

    Ok(updates)
}

#[tauri::command]
pub fn delete_chapter(state: State<AppState>, chapter_id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let chapter = db.load_chapter(chapter_id).map_err(|e| e.to_string())?;
    db.delete_chapter(chapter_id).map_err(|e| e.to_string())?;
    db.clear_book_outline(&chapter.novel_id)
        .map_err(|e| e.to_string())?;
    db.clear_outline_cache(&chapter.novel_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_chapters(state: State<AppState>, chapter_ids: Vec<i64>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let novel_ids = chapter_ids
        .iter()
        .filter_map(|id| db.load_chapter(*id).ok().map(|chapter| chapter.novel_id))
        .collect::<std::collections::HashSet<_>>();
    db.delete_chapters(&chapter_ids).map_err(|e| e.to_string())?;
    for novel_id in novel_ids {
        db.clear_book_outline(&novel_id)
            .map_err(|e| e.to_string())?;
        db.clear_outline_cache(&novel_id)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn clear_chapter_analysis(state: State<AppState>, chapter_id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.clear_chapter_analysis(chapter_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_chapter_outline(state: State<AppState>, chapter_id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let chapter = db.load_chapter(chapter_id).map_err(|e| e.to_string())?;
    db.clear_chapter_outline(chapter_id)
        .map_err(|e| e.to_string())?;
    db.clear_book_outline(&chapter.novel_id)
        .map_err(|e| e.to_string())?;
    db.clear_outline_cache(&chapter.novel_id)
        .map_err(|e| e.to_string())
}
