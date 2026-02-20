mod analysis;
mod epub_parser;
mod export;
mod llm;
mod models;
mod prompt;
mod storage;
mod token_utils;
mod txt_parser;

use models::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use storage::Database;
use tauri::{Emitter, Manager, State};

struct AppState {
    db: Mutex<Database>,
    batch_cancel: AtomicBool,
}

// ---- Novel Management Commands ----

#[tauri::command]
fn list_novels(state: State<AppState>) -> Result<Vec<NovelMeta>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.list_novels().map_err(|e| e.to_string())
}

#[tauri::command]
fn preview_epub(path: String) -> Result<EpubPreview, String> {
    let (title, chapters) = epub_parser::preview_epub(&path)?;
    Ok(EpubPreview {
        title,
        path,
        chapters,
    })
}

#[tauri::command]
fn import_epub_selected(
    state: State<AppState>,
    path: String,
    selected_indices: Vec<usize>,
) -> Result<String, String> {
    let (title, chapters) = epub_parser::parse_epub_selected(&path, &selected_indices)?;

    let novel_id = uuid::Uuid::new_v4().to_string();
    let novel = Novel {
        id: novel_id.clone(),
        title,
        source_type: SourceType::Epub(path),
        enabled_dimensions: AnalysisDimension::default_set(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.save_novel(&novel).map_err(|e| e.to_string())?;

    for (i, (chapter_title, content)) in chapters.into_iter().enumerate() {
        let chapter = Chapter {
            id: None,
            novel_id: novel_id.clone(),
            index: i,
            title: chapter_title,
            content,
            analysis: None,
        };
        db.save_chapter(&chapter).map_err(|e| e.to_string())?;
    }

    Ok(novel_id)
}

#[tauri::command]
fn import_txt_files(state: State<AppState>, paths: Vec<String>) -> Result<String, String> {
    let (title, chapters) = txt_parser::parse_txt_files(paths.clone())?;

    let novel_id = uuid::Uuid::new_v4().to_string();
    let novel = Novel {
        id: novel_id.clone(),
        title,
        source_type: SourceType::TxtFiles(paths),
        enabled_dimensions: AnalysisDimension::default_set(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.save_novel(&novel).map_err(|e| e.to_string())?;

    for (i, (chapter_title, content)) in chapters.into_iter().enumerate() {
        let chapter = Chapter {
            id: None,
            novel_id: novel_id.clone(),
            index: i,
            title: chapter_title,
            content,
            analysis: None,
        };
        db.save_chapter(&chapter).map_err(|e| e.to_string())?;
    }

    Ok(novel_id)
}

#[tauri::command]
fn import_single_txt(state: State<AppState>, path: String) -> Result<String, String> {
    let (title, chapters) = txt_parser::parse_single_txt(&path)?;

    let novel_id = uuid::Uuid::new_v4().to_string();
    let novel = Novel {
        id: novel_id.clone(),
        title,
        source_type: SourceType::SingleTxt(path),
        enabled_dimensions: AnalysisDimension::default_set(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.save_novel(&novel).map_err(|e| e.to_string())?;

    for (i, (chapter_title, content)) in chapters.into_iter().enumerate() {
        let chapter = Chapter {
            id: None,
            novel_id: novel_id.clone(),
            index: i,
            title: chapter_title,
            content,
            analysis: None,
        };
        db.save_chapter(&chapter).map_err(|e| e.to_string())?;
    }

    Ok(novel_id)
}

#[tauri::command]
fn delete_novel(state: State<AppState>, novel_id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_novel(&novel_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_chapter(state: State<AppState>, chapter_id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_chapter(chapter_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_chapters(state: State<AppState>, chapter_ids: Vec<i64>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_chapters(&chapter_ids).map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_chapter_analysis(state: State<AppState>, chapter_id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.clear_chapter_analysis(chapter_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_novel(state: State<AppState>, novel_id: String) -> Result<Novel, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.load_novel(&novel_id).map_err(|e| e.to_string())
}

// ---- Chapter Commands ----

#[tauri::command]
fn list_chapters(state: State<AppState>, novel_id: String) -> Result<Vec<ChapterMeta>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.list_chapter_metas(&novel_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_chapter(state: State<AppState>, chapter_id: i64) -> Result<Chapter, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.load_chapter(chapter_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_chapter_content(state: State<AppState>, chapter_id: i64) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.load_chapter_content(chapter_id)
        .map_err(|e| e.to_string())
}

// ---- Analysis Commands ----

#[tauri::command]
fn generate_prompt(
    state: State<AppState>,
    chapter_id: i64,
    dimensions: Vec<AnalysisDimension>,
) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let chapter = db.load_chapter(chapter_id).map_err(|e| e.to_string())?;
    let prompt_text =
        prompt::generate_chapter_prompt(&chapter.title, &chapter.content, &dimensions);
    Ok(prompt_text)
}

#[tauri::command]
fn estimate_prompt_tokens(
    state: State<AppState>,
    chapter_id: i64,
    dimensions: Vec<AnalysisDimension>,
) -> Result<usize, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let chapter = db.load_chapter(chapter_id).map_err(|e| e.to_string())?;
    let prompt_text =
        prompt::generate_chapter_prompt(&chapter.title, &chapter.content, &dimensions);
    Ok(token_utils::estimate_tokens(&prompt_text))
}

#[tauri::command]
fn parse_manual_result(json_str: String) -> Result<ChapterAnalysis, String> {
    analysis::parse_analysis_json(&json_str)
}

#[tauri::command]
fn save_analysis(
    state: State<AppState>,
    chapter_id: i64,
    analysis_data: ChapterAnalysis,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.save_chapter_analysis(chapter_id, &analysis_data)
        .map_err(|e| e.to_string())
}

async fn do_analyze_chapter(
    app: &tauri::AppHandle,
    db_mutex: &Mutex<Database>,
    chapter_id: i64,
    dimensions: &[AnalysisDimension],
) -> Result<ChapterAnalysis, String> {
    let (chapter, config) = {
        let db = db_mutex.lock().map_err(|e| e.to_string())?;
        let chapter = db.load_chapter(chapter_id).map_err(|e| e.to_string())?;
        let config = db.load_llm_config().map_err(|e| e.to_string())?;
        (chapter, config)
    };

    let prompt_text = prompt::generate_chapter_prompt(&chapter.title, &chapter.content, dimensions);
    let prompt_tokens = token_utils::estimate_tokens(&prompt_text);
    let available = token_utils::calculate_available_tokens(&config, 0);

    if prompt_tokens > available {
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

            let seg_prompt =
                prompt::generate_segment_prompt(&chapter.title, seg, i, segments.len(), dimensions);
            let response = llm::call_api_stream(&config, &seg_prompt, app, chapter_id).await?;
            let seg_analysis = analysis::parse_analysis_json(&response)?;
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

        let merged = analysis::merge_segment_analyses(segment_analyses);

        let db = db_mutex.lock().map_err(|e| e.to_string())?;
        db.save_chapter_analysis(chapter_id, &merged)
            .map_err(|e| e.to_string())?;

        Ok(merged)
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

        let response = llm::call_api_stream(&config, &prompt_text, app, chapter_id).await?;
        let analysis_result = analysis::parse_analysis_json(&response)?;

        let db = db_mutex.lock().map_err(|e| e.to_string())?;
        db.save_chapter_analysis(chapter_id, &analysis_result)
            .map_err(|e| e.to_string())?;

        Ok(analysis_result)
    }
}

#[tauri::command]
async fn analyze_chapter_api(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    chapter_id: i64,
    dimensions: Vec<AnalysisDimension>,
) -> Result<ChapterAnalysis, String> {
    do_analyze_chapter(&app, &state.db, chapter_id, &dimensions).await
}

#[tauri::command]
async fn batch_analyze_novel(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    novel_id: String,
) -> Result<(), String> {
    let (novel, metas) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let novel = db.load_novel(&novel_id).map_err(|e| e.to_string())?;
        let metas = db
            .list_chapter_metas(&novel_id)
            .map_err(|e| e.to_string())?;
        (novel, metas)
    };

    let unanalyzed: Vec<_> = metas.into_iter().filter(|m| !m.has_analysis).collect();
    let total = unanalyzed.len();

    if total == 0 {
        return Ok(());
    }

    use futures::StreamExt;
    let completed = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let mut futures = futures::stream::iter(unanalyzed.into_iter().map(|meta| {
        let app = app.clone();
        let db = &state.db;
        let dimensions = &novel.enabled_dimensions;
        let novel_id = novel_id.clone();
        let completed = completed.clone();
        let cancel_flag = &state.batch_cancel;

        async move {
            if cancel_flag.load(Ordering::Relaxed) {
                return Ok(true);
            }

            let completed_count = completed.load(Ordering::Relaxed);
            let _ = app.emit(
                "batch_progress",
                ProgressEvent {
                    novel_id: novel_id.clone(),
                    chapter_id: Some(meta.id),
                    status: "batch_analyzing".to_string(),
                    current: completed_count,
                    total,
                    message: format!("派发任务: {} (已完成 {}/{})", meta.title, completed_count, total),
                },
            );

            match do_analyze_chapter(&app, db, meta.id, dimensions).await {
                Ok(_) => {
                    let completed_count = completed.fetch_add(1, Ordering::Relaxed) + 1;
                    let _ = app.emit(
                        "batch_progress",
                        ProgressEvent {
                            novel_id,
                            chapter_id: Some(meta.id),
                            status: "chapter_done".to_string(),
                            current: completed_count,
                            total,
                            message: format!("已完成: {} (总计 {}/{})", meta.title, completed_count, total),
                        },
                    );
                    Ok(false)
                }
                Err(e) => {
                    let completed_count = completed.load(Ordering::Relaxed);
                    let _ = app.emit(
                        "batch_progress",
                        ProgressEvent {
                            novel_id,
                            chapter_id: Some(meta.id),
                            status: "error".to_string(),
                            current: completed_count,
                            total,
                            message: format!("分析 {} 失败: {}", meta.title, e),
                        },
                    );
                    Err(e)
                }
            }
        }
    }))
    .buffer_unordered(3);

    while let Some(res) = futures.next().await {
        match res {
            Ok(cancelled) => {
                if cancelled {
                    state.batch_cancel.store(false, Ordering::Relaxed);
                    let current = completed.load(Ordering::Relaxed);
                    let _ = app.emit(
                        "batch_progress",
                        ProgressEvent {
                            novel_id: novel_id.clone(),
                            chapter_id: None,
                            status: "batch_cancelled".to_string(),
                            current,
                            total,
                            message: format!("批量分析已取消 ({}/{})", current, total),
                        },
                    );
                    return Ok(());
                }
            }
            Err(e) => return Err(e),
        }
    }

    let _ = app.emit(
        "batch_progress",
        ProgressEvent {
            novel_id: novel_id.clone(),
            chapter_id: None,
            status: "batch_done".to_string(),
            current: total,
            total,
            message: "批量分析完成".to_string(),
        },
    );

    Ok(())
}

#[tauri::command]
async fn batch_analyze_chapters(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    novel_id: String,
    chapter_ids: Vec<i64>,
) -> Result<(), String> {
    let (novel, metas) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let novel = db.load_novel(&novel_id).map_err(|e| e.to_string())?;
        let all_metas = db
            .list_chapter_metas(&novel_id)
            .map_err(|e| e.to_string())?;
        let selected: Vec<_> = all_metas
            .into_iter()
            .filter(|m| chapter_ids.contains(&m.id))
            .collect();
        (novel, selected)
    };

    let total = metas.len();
    if total == 0 {
        return Ok(());
    }

    state.batch_cancel.store(false, Ordering::Relaxed);

    use futures::StreamExt;
    let completed = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let mut futures = futures::stream::iter(metas.into_iter().map(|meta| {
        let app = app.clone();
        let db = &state.db;
        let dimensions = &novel.enabled_dimensions;
        let novel_id = novel_id.clone();
        let completed = completed.clone();
        let cancel_flag = &state.batch_cancel;

        async move {
            if cancel_flag.load(Ordering::Relaxed) {
                return Ok(true);
            }

            let completed_count = completed.load(Ordering::Relaxed);
            let _ = app.emit(
                "batch_progress",
                ProgressEvent {
                    novel_id: novel_id.clone(),
                    chapter_id: Some(meta.id),
                    status: "batch_analyzing".to_string(),
                    current: completed_count,
                    total,
                    message: format!("派发任务: {} (已完成 {}/{})", meta.title, completed_count, total),
                },
            );

            match do_analyze_chapter(&app, db, meta.id, dimensions).await {
                Ok(_) => {
                    let completed_count = completed.fetch_add(1, Ordering::Relaxed) + 1;
                    let _ = app.emit(
                        "batch_progress",
                        ProgressEvent {
                            novel_id,
                            chapter_id: Some(meta.id),
                            status: "chapter_done".to_string(),
                            current: completed_count,
                            total,
                            message: format!("已完成: {} (总计 {}/{})", meta.title, completed_count, total),
                        },
                    );
                    Ok(false)
                }
                Err(e) => {
                    let completed_count = completed.load(Ordering::Relaxed);
                    let _ = app.emit(
                        "batch_progress",
                        ProgressEvent {
                            novel_id,
                            chapter_id: Some(meta.id),
                            status: "error".to_string(),
                            current: completed_count,
                            total,
                            message: format!("分析 {} 失败: {}", meta.title, e),
                        },
                    );
                    Err(e)
                }
            }
        }
    }))
    .buffer_unordered(3);

    while let Some(res) = futures.next().await {
        match res {
            Ok(cancelled) => {
                if cancelled {
                    state.batch_cancel.store(false, Ordering::Relaxed);
                    let current = completed.load(Ordering::Relaxed);
                    let _ = app.emit(
                        "batch_progress",
                        ProgressEvent {
                            novel_id: novel_id.clone(),
                            chapter_id: None,
                            status: "batch_cancelled".to_string(),
                            current,
                            total,
                            message: format!("批量分析已取消 ({}/{})", current, total),
                        },
                    );
                    return Ok(());
                }
            }
            Err(e) => return Err(e),
        }
    }

    let _ = app.emit(
        "batch_progress",
        ProgressEvent {
            novel_id: novel_id.clone(),
            chapter_id: None,
            status: "batch_done".to_string(),
            current: total,
            total,
            message: "批量分析完成".to_string(),
        },
    );

    Ok(())
}

// ---- Settings Commands ----

#[tauri::command]
fn get_llm_config(state: State<AppState>) -> Result<LlmConfig, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.load_llm_config().map_err(|e| e.to_string())
}

#[tauri::command]
fn save_llm_config(state: State<AppState>, config: LlmConfig) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.save_llm_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_models(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let config = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.load_llm_config().map_err(|e| e.to_string())?
    };
    llm::list_models(&config).await
}

#[tauri::command]
fn update_novel_dimensions(
    state: State<AppState>,
    novel_id: String,
    dimensions: Vec<AnalysisDimension>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut novel = db.load_novel(&novel_id).map_err(|e| e.to_string())?;
    novel.enabled_dimensions = dimensions;
    db.save_novel(&novel).map_err(|e| e.to_string())
}

// ---- Summary Commands ----

#[tauri::command]
fn get_full_summary_manual_prompt(
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
fn get_novel_summary(
    state: State<AppState>,
    novel_id: String,
) -> Result<Option<NovelSummary>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.load_novel_summary(&novel_id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn generate_full_summary(
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
        let response = llm::call_api(&config, &prompt_text).await?;
        let summary_content = analysis::clean_json_response(&response);
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

    let final_summary = if group_summaries.len() == 1 {
        analysis::parse_summary_json(&group_summaries[0])?
    } else {
        let final_prompt = prompt::generate_final_summary_prompt(&group_summaries, dims);
        let response = llm::call_api(&config, &final_prompt).await?;
        analysis::parse_summary_json(&response)?
    };

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

#[tauri::command]
async fn export_novel_report(
    state: State<'_, AppState>,
    novel_id: String,
    path: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let novel = db.load_novel(&novel_id).map_err(|e| e.to_string())?;
    let summary = db
        .load_novel_summary(&novel_id)
        .map_err(|e| e.to_string())?;

    let mut chapters = Vec::new();
    let metas = db
        .list_chapter_metas(&novel_id)
        .map_err(|e| e.to_string())?;
    for meta in metas {
        if let Ok(ch) = db.load_chapter(meta.id) {
            chapters.push(ch);
        }
    }

    let md = export::generate_markdown_report(&novel, summary.as_ref(), &chapters);
    std::fs::write(&path, md).map_err(|e| e.to_string())?;
    Ok(())
}

// ---- Dimension Info ----

#[tauri::command]
fn get_all_dimensions() -> Vec<serde_json::Value> {
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

#[tauri::command]
fn cancel_batch(state: State<AppState>) {
    state.batch_cancel.store(true, Ordering::Relaxed);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("."));
            let db = Database::new(&app_data_dir)
                .map_err(|e| format!("数据库初始化失败: {}", e))
                .expect("Failed to initialize database");
            app.manage(AppState {
                db: Mutex::new(db),
                batch_cancel: AtomicBool::new(false),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_novels,
            preview_epub,
            import_epub_selected,
            import_txt_files,
            import_single_txt,
            delete_novel,
            delete_chapter,
            delete_chapters,
            clear_chapter_analysis,
            get_novel,
            list_chapters,
            get_chapter,
            get_chapter_content,
            generate_prompt,
            estimate_prompt_tokens,
            parse_manual_result,
            save_analysis,
            analyze_chapter_api,
            batch_analyze_novel,
            cancel_batch,
            batch_analyze_chapters,
            get_llm_config,
            save_llm_config,
            update_novel_dimensions,
            get_novel_summary,
            get_full_summary_manual_prompt,
            generate_full_summary,
            export_novel_report,
            get_all_dimensions,
            list_models,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
