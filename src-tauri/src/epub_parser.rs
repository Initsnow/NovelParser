use epub::doc::{EpubDoc, NavPoint};
use html2text::from_read;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::path::PathBuf;

/// Keywords that indicate a page is metadata/boilerplate, not actual chapter content.
const SKIP_KEYWORDS: &[&str] = &[
    "版权",
    "copyright",
    "ISBN",
    "出版社",
    "制作信息",
    "作者简介",
    "内容简介",
    "编辑推荐",
    "封面",
    "Cover",
    "扉页",
    "前折页",
    "后折页",
    "All Rights Reserved",
    "all rights reserved",
    "本书由",
    "授权",
    "版次",
    "印次",
    "译序",
    "译者",
    "校对",
];

/// A preview chapter returned to the frontend for user selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubPreviewChapter {
    pub index: usize,
    pub title: String,
    pub char_count: usize,
    /// Whether this chapter is suggested for import (true = real content, false = metadata/boilerplate)
    pub suggested: bool,
}

/// Parse an EPUB file and return a preview of all chapters for user selection.
/// Returns (book_title, preview_chapters).
pub fn preview_epub(path: &str) -> Result<(String, Vec<EpubPreviewChapter>), String> {
    let (title, raw_chapters) = extract_all_spine_items(path)?;

    let previews: Vec<EpubPreviewChapter> = raw_chapters
        .iter()
        .enumerate()
        .map(|(i, (ch_title, content))| {
            let char_count = content.chars().count();
            let suggested = char_count >= 100 && !is_metadata_page(content);
            EpubPreviewChapter {
                index: i,
                title: ch_title.clone(),
                char_count,
                suggested,
            }
        })
        .collect();

    Ok((title, previews))
}

/// Parse an EPUB and return only the selected chapters (by index).
pub fn parse_epub_selected(
    path: &str,
    selected_indices: &[usize],
) -> Result<(String, Vec<(String, String)>), String> {
    let (title, raw_chapters) = extract_all_spine_items(path)?;

    let chapters: Vec<(String, String)> = raw_chapters
        .into_iter()
        .enumerate()
        .filter(|(i, _)| selected_indices.contains(i))
        .map(|(_, ch)| ch)
        .collect();

    if chapters.is_empty() {
        return Err("没有选择任何章节".to_string());
    }

    Ok((title, chapters))
}

/// Internal: extract all spine items from an EPUB as (title, content) pairs.
fn extract_all_spine_items(path: &str) -> Result<(String, Vec<(String, String)>), String> {
    let mut doc = EpubDoc::new(path).map_err(|e| format!("无法打开 EPUB 文件: {}", e))?;

    let title = doc
        .mdata("title")
        .map(|m| m.value.clone())
        .unwrap_or_else(|| {
            std::path::Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("未知书名")
                .to_string()
        });

    // Flatten TOC into a list of (play_order, label, resource_path)
    // We only care about top-level NavPoints to merge their sub-chapters,
    // or recursively flatten them? The user said "chapters might be volumes with sub-chapters".
    // If we only take top level, a "Volume" might contain 50 chapters, which would merge into 1 giant string.
    // That's bad. We should keep the leaves (deepest children) or just flatten the TOC with combined labels.
    let mut flat_toc: Vec<(String, PathBuf)> = Vec::new();
    fn flatten_nav(navs: &[NavPoint], prefix: &str, out: &mut Vec<(String, PathBuf)>) {
        for nav in navs {
            let label = if prefix.is_empty() {
                nav.label.clone()
            } else {
                format!("{} - {}", prefix, nav.label)
            };

            // Extract just the file path, ignoring #anchors
            let path_str = nav.content.to_string_lossy().to_string();
            let base_path = path_str.split('#').next().unwrap_or(&path_str);
            out.push((label.clone(), PathBuf::from(base_path)));

            flatten_nav(&nav.children, &label, out);
        }
    }
    let toc_clone = doc.toc.clone();
    flatten_nav(&toc_clone, "", &mut flat_toc);

    let spine_ids: Vec<String> = doc.spine.iter().map(|s| s.idref.clone()).collect();

    // Create a mapping from resource path to its TOC label
    let mut path_to_label = std::collections::HashMap::new();
    // Keep order of first appearance to roughly know the TOC sequence
    let mut toc_sequence = Vec::new();
    for (label, p) in &flat_toc {
        if !path_to_label.contains_key(p) {
            path_to_label.insert(p.clone(), label.clone());
            toc_sequence.push(p.clone());
        }
    }

    let mut chapters: Vec<(String, String)> = Vec::new();
    let mut current_merged_content = String::new();
    let mut current_merged_title = String::new();

    for spine_id in &spine_ids {
        if let Some(resource) = doc.resources.get(spine_id) {
            let resource_path = resource.path.clone();

            // Read content
            let mut text = String::new();
            if let Some(content_bytes) = doc.get_resource_by_path(&resource_path) {
                let html = String::from_utf8_lossy(&content_bytes).to_string();
                text = from_read(Cursor::new(html.as_bytes()), 80).unwrap_or_else(|_| html.clone());
            }
            let trimmed = text.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Check if this path corresponds to a TOC entry
            if let Some(toc_label) = path_to_label.get(&resource_path) {
                // If we already have accumulated content, push it as a chapter
                if !current_merged_content.is_empty() {
                    let final_title = if current_merged_title.is_empty() {
                        format!("第 {} 节", chapters.len() + 1)
                    } else {
                        current_merged_title.clone()
                    };
                    chapters.push((final_title, current_merged_content.clone()));
                    current_merged_content.clear();
                }
                current_merged_title = toc_label.clone();
            }

            // Append content
            if !current_merged_content.is_empty() {
                current_merged_content.push_str("\n\n");
            }
            current_merged_content.push_str(trimmed);

            // If TOC was completely empty, fallback to naive title extraction per spine item
            if flat_toc.is_empty() && current_merged_title.is_empty() {
                current_merged_title = extract_chapter_title(trimmed)
                    .unwrap_or_else(|| format!("第 {} 节", chapters.len() + 1));
            }

            // If flat_toc is empty, push immediately (no merging)
            if flat_toc.is_empty() {
                chapters.push((current_merged_title.clone(), current_merged_content.clone()));
                current_merged_content.clear();
                current_merged_title.clear();
            }
        }
    }

    // Push the last merged chapter
    if !current_merged_content.is_empty() {
        let final_title = if current_merged_title.is_empty() {
            format!("第 {} 节", chapters.len() + 1)
        } else {
            current_merged_title
        };
        chapters.push((final_title, current_merged_content));
    }

    if chapters.is_empty() {
        return Err("未能从 EPUB 中提取到任何内容".to_string());
    }

    Ok((title, chapters))
}

/// Check if a page is metadata/boilerplate rather than actual story content.
/// Uses keyword density: if multiple skip keywords appear in a short page, it's likely metadata.
fn is_metadata_page(text: &str) -> bool {
    let lower = text.to_lowercase();
    let text_len = text.chars().count();

    // Count how many skip keywords appear
    let keyword_hits: usize = SKIP_KEYWORDS
        .iter()
        .filter(|kw| lower.contains(&kw.to_lowercase()))
        .count();

    // For short pages (< 500 chars), even 2 keyword hits is suspicious
    if text_len < 500 && keyword_hits >= 2 {
        return true;
    }

    // For medium pages (< 1500 chars), need 3+ hits
    if text_len < 1500 && keyword_hits >= 3 {
        return true;
    }

    // Check if the page is structured like a metadata block:
    // many short lines (key-value pairs) rather than prose paragraphs
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.len() >= 3 {
        let short_lines = lines
            .iter()
            .filter(|l| l.trim().chars().count() < 30)
            .count();
        let short_ratio = short_lines as f64 / lines.len() as f64;
        // If >70% of lines are very short AND we have at least 1 keyword hit, it's metadata
        if short_ratio > 0.7 && keyword_hits >= 1 && text_len < 2000 {
            return true;
        }
    }

    false
}

/// Try to extract a chapter title from the beginning of text content.
fn extract_chapter_title(text: &str) -> Option<String> {
    let first_line = text.lines().next()?.trim();

    // Skip if first line is too long or too short
    let char_count = first_line.chars().count();
    if char_count < 2 || char_count > 60 {
        return None;
    }

    // Common chapter title patterns (Chinese + English)
    let title_patterns = [
        "第", "Chapter", "CHAPTER", "卷", "序章", "序", "楔子", "引子", "尾声", "番外", "后记",
        "前言", "Prologue", "Epilogue", "Part", "PART",
    ];

    for p in &title_patterns {
        if first_line.starts_with(p) {
            return Some(first_line.to_string());
        }
    }

    // If the first line is short and the second line is substantially longer,
    // treat the first line as a title (common in CJK ebooks)
    if char_count < 30 {
        if let Some(second_line) = text.lines().nth(1) {
            let second_chars = second_line.trim().chars().count();
            if second_chars > char_count * 2 && second_chars > 20 {
                return Some(first_line.to_string());
            }
        }
    }

    None
}
