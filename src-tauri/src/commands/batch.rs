use crate::models::*;
use crate::storage::Database;
use crate::AppState;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{Emitter, State};

use super::analysis::do_analyze_chapter;

/// Shared batch analysis helper with error tolerance.
/// Failed chapters are skipped and errors collected instead of aborting.
async fn do_batch_analyze(
    app: &tauri::AppHandle,
    db_mutex: &Mutex<Database>,
    cancel_flag: &AtomicBool,
    novel_id: &str,
    metas: Vec<ChapterMeta>,
    dimensions: &[AnalysisDimension],
    concurrency: usize,
) -> Result<(), String> {
    let total = metas.len();
    if total == 0 {
        return Ok(());
    }

    cancel_flag.store(false, Ordering::Relaxed);

    use futures::StreamExt;
    let completed = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let failed = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let mut futures = futures::stream::iter(metas.into_iter().map(|meta| {
        let app = app.clone();
        let novel_id = novel_id.to_string();
        let completed = completed.clone();
        let failed = failed.clone();

        async move {
            if cancel_flag.load(Ordering::Relaxed) {
                return Ok::<bool, String>(true);
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
                    message: format!(
                        "派发任务: {} (已完成 {}/{})",
                        meta.title, completed_count, total
                    ),
                },
            );

            match do_analyze_chapter(&app, db_mutex, meta.id, dimensions).await {
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
                            message: format!(
                                "已完成: {} (总计 {}/{})",
                                meta.title, completed_count, total
                            ),
                        },
                    );
                }
                Err(e) => {
                    failed.fetch_add(1, Ordering::Relaxed);
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
                }
            }
            Ok(false)
        }
    }))
    .buffer_unordered(concurrency);

    while let Some(res) = futures.next().await {
        if let Ok(true) = res {
            cancel_flag.store(false, Ordering::Relaxed);
            let current = completed.load(Ordering::Relaxed);
            let _ = app.emit(
                "batch_progress",
                ProgressEvent {
                    novel_id: novel_id.to_string(),
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

    let fail_count = failed.load(Ordering::Relaxed);
    let success_count = completed.load(Ordering::Relaxed);
    let done_message = if fail_count > 0 {
        format!("批量分析完成：成功 {}，失败 {}", success_count, fail_count)
    } else {
        "批量分析完成".to_string()
    };

    let _ = app.emit(
        "batch_progress",
        ProgressEvent {
            novel_id: novel_id.to_string(),
            chapter_id: None,
            status: "batch_done".to_string(),
            current: total,
            total,
            message: done_message,
        },
    );

    Ok(())
}

#[tauri::command]
pub async fn batch_analyze_novel(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    novel_id: String,
) -> Result<(), String> {
    let (novel, unanalyzed, config) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let novel = db.load_novel(&novel_id).map_err(|e| e.to_string())?;
        let metas = db
            .list_chapter_metas(&novel_id)
            .map_err(|e| e.to_string())?;
        let unanalyzed: Vec<_> = metas.into_iter().filter(|m| !m.has_analysis).collect();
        let config = db.load_llm_config().unwrap_or_default();
        (novel, unanalyzed, config)
    };

    let concurrency = if config.context_injection_mode != ContextInjectionMode::None {
        1
    } else {
        config.max_concurrent_tasks as usize
    };

    do_batch_analyze(
        &app,
        &state.db,
        &state.batch_cancel,
        &novel_id,
        unanalyzed,
        &novel.enabled_dimensions,
        concurrency,
    )
    .await
}

#[tauri::command]
pub async fn batch_analyze_chapters(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    novel_id: String,
    chapter_ids: Vec<i64>,
) -> Result<(), String> {
    let (novel, metas, config) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let novel = db.load_novel(&novel_id).map_err(|e| e.to_string())?;
        let all_metas = db
            .list_chapter_metas(&novel_id)
            .map_err(|e| e.to_string())?;
        let selected: Vec<_> = all_metas
            .into_iter()
            .filter(|m| chapter_ids.contains(&m.id))
            .collect();
        let config = db.load_llm_config().unwrap_or_default();
        (novel, selected, config)
    };

    let concurrency = if config.context_injection_mode != ContextInjectionMode::None {
        1
    } else {
        config.max_concurrent_tasks as usize
    };

    do_batch_analyze(
        &app,
        &state.db,
        &state.batch_cancel,
        &novel_id,
        metas,
        &novel.enabled_dimensions,
        concurrency,
    )
    .await
}

#[tauri::command]
pub fn cancel_batch(state: State<AppState>) {
    state.batch_cancel.store(true, Ordering::Relaxed);
}
