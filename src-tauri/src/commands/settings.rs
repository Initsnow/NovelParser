use crate::models::*;
use crate::{llm, AppState};
use tauri::State;

#[tauri::command]
pub fn get_llm_config(state: State<AppState>) -> Result<LlmConfig, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.load_llm_config().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_llm_config(state: State<AppState>, config: LlmConfig) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.save_llm_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_models(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let config = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.load_llm_config().map_err(|e| e.to_string())?
    };
    llm::list_models(&state.http_client, &config).await
}

#[tauri::command]
pub fn update_novel_dimensions(
    state: State<AppState>,
    novel_id: String,
    dimensions: Vec<AnalysisDimension>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut novel = db.load_novel(&novel_id).map_err(|e| e.to_string())?;
    novel.enabled_dimensions = dimensions;
    db.save_novel(&novel).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_all_dimensions() -> Vec<serde_json::Value> {
    AnalysisDimension::all()
        .into_iter()
        .map(|d| {
            serde_json::json!({
                "id": d,
                "name": d.display_name(),
                "icon": d.icon(),
                "description": d.description(),
                "default": AnalysisDimension::default_set().contains(&d),
            })
        })
        .collect()
}
