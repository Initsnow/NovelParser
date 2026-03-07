use crate::models::*;
use crate::{analysis as analysis_mod, llm, prompt, AppState};
use tauri::{Emitter, State};

#[tauri::command]
pub fn get_full_summary_manual_prompt(
    state: State<'_, AppState>,
    novel_id: String,
) -> Result<String, String> {
    let (novel, chapters) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let novel = db.load_novel(&novel_id).map_err(|e| e.to_string())?;
        let chapters: Vec<Chapter> = db
            .list_chapter_metas(&novel_id)
            .map_err(|e| e.to_string())?
            .into_iter()
            .filter(|m| m.has_analysis)
            .filter_map(|m| db.load_chapter(m.id).ok())
            .collect();
        (novel, chapters)
    };

    if chapters.is_empty() {
        return Err("当前没有已分析的章节可以用来生成汇总".to_string());
    }

    let dims = &novel.enabled_dimensions;
    let chapter_summaries: Vec<(usize, String)> = chapters
        .into_iter()
        .filter_map(|ch| {
            ch.analysis
                .map(|a| (ch.index, serde_json::to_string(&a).unwrap_or_default()))
        })
        .collect();

    if chapter_summaries.is_empty() {
        return Err("章节分析数据为空".to_string());
    }

    Ok(prompt::generate_manual_full_summary_prompt(
        &chapter_summaries,
        dims,
    ))
}

#[tauri::command]
pub fn parse_manual_summary_result(json_str: String) -> Result<NovelSummary, String> {
    analysis_mod::parse_summary_json(&json_str)
}

#[tauri::command]
pub fn get_novel_summary(
    state: State<AppState>,
    novel_id: String,
) -> Result<Option<NovelSummary>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.load_novel_summary(&novel_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_novel_summary(
    state: State<AppState>,
    novel_id: String,
    summary: NovelSummary,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.save_novel_summary(&novel_id, &summary)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_novel_summary(state: State<AppState>, novel_id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.clear_novel_summary(&novel_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_full_summary(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    novel_id: String,
) -> Result<NovelSummary, String> {
    let (novel, chapters, config) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let novel = db.load_novel(&novel_id).map_err(|e| e.to_string())?;
        let chapters: Vec<Chapter> = db
            .list_chapter_metas(&novel_id)
            .map_err(|e| e.to_string())?
            .into_iter()
            .filter(|m| m.has_analysis)
            .filter_map(|m| db.load_chapter(m.id).ok())
            .collect();
        let config = db.load_llm_config().map_err(|e| e.to_string())?;
        (novel, chapters, config)
    };

    if chapters.is_empty() {
        return Err("当前没有已分析的章节可以用来生成汇总".to_string());
    }

    let _ = app.emit(
        "analysis_progress",
        ProgressEvent {
            novel_id: novel_id.clone(),
            chapter_id: None,
            status: "summarizing".to_string(),
            current: 0,
            total: 100,
            message: "准备生成全书汇总...".to_string(),
        },
    );

    let dims = &novel.enabled_dimensions;
    let max_group_size = 10;

    let chapter_summaries: Vec<(usize, String)> = chapters
        .into_iter()
        .filter_map(|ch| {
            ch.analysis
                .map(|a| (ch.index, serde_json::to_string(&a).unwrap_or_default()))
        })
        .collect();

    if chapter_summaries.is_empty() {
        return Err("章节分析数据为空".to_string());
    }

    let mut group_summaries = Vec::new();
    let chunks: Vec<_> = chapter_summaries.chunks(max_group_size).collect();
    let total_chunks = chunks.len();

    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.clear_summary_cache(&novel_id)
            .map_err(|e| e.to_string())?;
    }

    for (i, chunk) in chunks.into_iter().enumerate() {
        let _ = app.emit(
            "analysis_progress",
            ProgressEvent {
                novel_id: novel_id.clone(),
                chapter_id: None,
                status: "summarizing".to_string(),
                current: i + 1,
                total: total_chunks + 1,
                message: format!("正在合并阶段汇总 ({}/{})", i + 1, total_chunks),
            },
        );

        let prompt_text = prompt::generate_group_summary_prompt(chunk, dims);
        let response = llm::call_api(&config, &prompt_text, config.summary_max_tokens).await?;
        let summary_content = analysis_mod::clean_json_response(&response);
        group_summaries.push(summary_content.clone());

        {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            db.save_summary_cache(&novel_id, 1, i as i32, &summary_content)
                .ok();
        }
    }

    let _ = app.emit(
        "analysis_progress",
        ProgressEvent {
            novel_id: novel_id.clone(),
            chapter_id: None,
            status: "summarizing".to_string(),
            current: total_chunks + 1,
            total: total_chunks + 1,
            message: "正在生成终极全书汇总...".to_string(),
        },
    );

    let mut final_summary = if group_summaries.len() == 1 {
        analysis_mod::parse_summary_json(&group_summaries[0])?
    } else {
        let final_prompt = prompt::generate_final_summary_prompt(&group_summaries, dims);
        let response = llm::call_api(&config, &final_prompt, config.summary_max_tokens).await?;
        analysis_mod::parse_summary_json(&response)?
    };

    final_summary.created_at = chrono::Utc::now().to_rfc3339();

    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.save_novel_summary(&novel_id, &final_summary)
            .map_err(|e| e.to_string())?;
    }

    let _ = app.emit(
        "analysis_progress",
        ProgressEvent {
            novel_id: novel_id.clone(),
            chapter_id: None,
            status: "done".to_string(),
            current: 100,
            total: 100,
            message: "全书汇总完成".to_string(),
        },
    );

    Ok(final_summary)
}
