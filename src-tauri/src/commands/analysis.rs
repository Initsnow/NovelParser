use crate::models::*;
use crate::storage::Database;
use crate::{analysis as analysis_mod, llm, prompt, token_utils, AppState};
use std::sync::Mutex;
use tauri::{Emitter, State};

pub(crate) fn build_context_string(
    db: &Database,
    novel_id: &str,
    chapter_index: usize,
    mode: &ContextInjectionMode,
) -> Result<Option<String>, String> {
    match mode {
        ContextInjectionMode::None => Ok(None),
        ContextInjectionMode::PreviousChapter => {
            if let Some(prev) = db
                .load_previous_chapter_analysis(novel_id, chapter_index)
                .map_err(|e| e.to_string())?
            {
                Ok(Some(prev.to_context_string()))
            } else {
                Ok(None)
            }
        }
        ContextInjectionMode::AllPrevious => {
            let mut context = String::new();
            let all_prev = db
                .load_all_previous_analyses(novel_id, chapter_index)
                .map_err(|e| e.to_string())?;

            if all_prev.is_empty() {
                return Ok(None);
            }

            for (_, title, prev) in &all_prev {
                if let Some(plot) = &prev.plot {
                    context.push_str(&format!("{} 摘要：{}\n", title, plot.summary));
                }
            }

            if let Some((_, _, last)) = all_prev.last() {
                context.push_str("\n【最近一章详细状态】\n");
                context.push_str(&last.to_context_string());
            }

            Ok(Some(context))
        }
    }
}

#[tauri::command]
pub fn generate_prompt(
    state: State<AppState>,
    chapter_id: i64,
    dimensions: Vec<AnalysisDimension>,
) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let chapter = db.load_chapter(chapter_id).map_err(|e| e.to_string())?;
    let config = db.load_llm_config().unwrap_or_default();

    let context_str = build_context_string(
        &db,
        &chapter.novel_id,
        chapter.index,
        &config.context_injection_mode,
    )?;

    let prompt_text = prompt::generate_chapter_prompt(
        &chapter.title,
        &chapter.content,
        &dimensions,
        context_str.as_deref(),
        false, // Manual mode, assume user has memory in chat session
    );
    Ok(prompt_text)
}

#[tauri::command]
pub fn estimate_prompt_tokens(
    state: State<AppState>,
    chapter_id: i64,
    dimensions: Vec<AnalysisDimension>,
) -> Result<usize, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let chapter = db.load_chapter(chapter_id).map_err(|e| e.to_string())?;
    let config = db.load_llm_config().unwrap_or_default();

    let context_str = build_context_string(
        &db,
        &chapter.novel_id,
        chapter.index,
        &config.context_injection_mode,
    )?;

    let prompt_text = prompt::generate_chapter_prompt(
        &chapter.title,
        &chapter.content,
        &dimensions,
        context_str.as_deref(),
        false,
    );
    Ok(token_utils::estimate_tokens(&prompt_text))
}

#[tauri::command]
pub fn parse_manual_result(json_str: String) -> Result<ChapterAnalysis, String> {
    analysis_mod::parse_analysis_json(&json_str)
}

#[tauri::command]
pub fn save_analysis(
    state: State<AppState>,
    chapter_id: i64,
    analysis_data: ChapterAnalysis,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.save_chapter_analysis(chapter_id, &analysis_data)
        .map_err(|e| e.to_string())
}

pub(crate) async fn do_analyze_chapter(
    app: &tauri::AppHandle,
    db_mutex: &Mutex<Database>,
    chapter_id: i64,
    dimensions: &[AnalysisDimension],
) -> Result<ChapterAnalysis, String> {
    let (chapter, config, context_str) = {
        let db = db_mutex.lock().map_err(|e| e.to_string())?;
        let chapter = db.load_chapter(chapter_id).map_err(|e| e.to_string())?;
        let config = db.load_llm_config().map_err(|e| e.to_string())?;
        let ctx = build_context_string(
            &db,
            &chapter.novel_id,
            chapter.index,
            &config.context_injection_mode,
        )?;
        (chapter, config, ctx)
    };

    let forbid_callbacks =
        config.context_injection_mode == ContextInjectionMode::None || context_str.is_none();

    let prompt_text = prompt::generate_chapter_prompt(
        &chapter.title,
        &chapter.content,
        dimensions,
        context_str.as_deref(),
        forbid_callbacks,
    );
    let prompt_tokens = token_utils::estimate_tokens(&prompt_text);
    let available = token_utils::calculate_available_tokens(&config, 0);

    let analysis_result = if prompt_tokens > available {
        let content_budget = token_utils::calculate_available_tokens(&config, 500);
        let segments = token_utils::split_content_by_tokens(&chapter.content, content_budget);
        let mut segment_analyses = Vec::new();

        for (i, seg) in segments.iter().enumerate() {
            let _ = app.emit(
                "analysis_progress",
                ProgressEvent {
                    novel_id: chapter.novel_id.clone(),
                    chapter_id: Some(chapter_id),
                    status: "analyzing_segment".to_string(),
                    current: i + 1,
                    total: segments.len(),
                    message: format!("正在分析分段 {}/{}...", i + 1, segments.len()),
                },
            );

            let seg_prompt = prompt::generate_segment_prompt(
                &chapter.title,
                seg,
                i,
                segments.len(),
                dimensions,
                context_str.as_deref(),
                forbid_callbacks,
            );
            let response = llm::call_api_stream(
                &config,
                &seg_prompt,
                app,
                chapter_id,
                config.chapter_max_tokens,
            )
            .await?;
            let seg_analysis = analysis_mod::parse_analysis_json(&response)?;
            segment_analyses.push(seg_analysis);
        }

        let _ = app.emit(
            "analysis_progress",
            ProgressEvent {
                novel_id: chapter.novel_id.clone(),
                chapter_id: Some(chapter_id),
                status: "merging_segments".to_string(),
                current: segments.len(),
                total: segments.len(),
                message: "正在汇总分段分析...".to_string(),
            },
        );

        analysis_mod::merge_segment_analyses(segment_analyses)
    } else {
        let _ = app.emit(
            "analysis_progress",
            ProgressEvent {
                novel_id: chapter.novel_id.clone(),
                chapter_id: Some(chapter_id),
                status: "analyzing".to_string(),
                current: 0,
                total: 1,
                message: "正在生成分析...".to_string(),
            },
        );

        let response = llm::call_api_stream(
            &config,
            &prompt_text,
            app,
            chapter_id,
            config.chapter_max_tokens,
        )
        .await?;
        analysis_mod::parse_analysis_json(&response)?
    };

    let db = db_mutex.lock().map_err(|e| e.to_string())?;
    db.save_chapter_analysis(chapter_id, &analysis_result)
        .map_err(|e| e.to_string())?;

    Ok(analysis_result)
}

#[tauri::command]
pub async fn analyze_chapter_api(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    chapter_id: i64,
    dimensions: Vec<AnalysisDimension>,
) -> Result<ChapterAnalysis, String> {
    do_analyze_chapter(&app, &state.db, chapter_id, &dimensions).await
}
