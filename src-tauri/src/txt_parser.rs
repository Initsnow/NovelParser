use encoding_rs::{GB18030, GBK};
use regex::Regex;
use std::fs;
use std::path::Path;

/// Parse multiple TXT files as individual chapters.
pub fn parse_txt_files(paths: Vec<String>) -> Result<(String, Vec<(String, String)>), String> {
    let mut chapters: Vec<(String, String)> = Vec::new();

    for path in &paths {
        let content = read_txt_auto_encoding(path)?;
        let title = Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("未知章节")
            .to_string();
        chapters.push((title, content));
    }

    let title = if paths.len() == 1 {
        Path::new(&paths[0])
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("未知书名")
            .to_string()
    } else {
        // Try to find common prefix
        let names: Vec<&str> = paths
            .iter()
            .filter_map(|p| {
                Path::new(p)
                    .parent()
                    .and_then(|d| d.file_name())
                    .and_then(|n| n.to_str())
            })
            .collect();
        names.first().unwrap_or(&"未知书名").to_string()
    };

    Ok((title, chapters))
}

/// Parse a single large TXT file, splitting by chapter headings.
pub fn parse_single_txt(path: &str) -> Result<(String, Vec<(String, String)>), String> {
    let content = read_txt_auto_encoding(path)?;
    let title = Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("未知书名")
        .to_string();

    let chapters = split_by_chapters(&content);

    if chapters.is_empty() {
        // If no chapter markers found, treat the entire file as one chapter
        return Ok((title, vec![("全文".to_string(), content)]));
    }

    Ok((title, chapters))
}

/// Split text content by Chinese chapter heading patterns.
fn split_by_chapters(content: &str) -> Vec<(String, String)> {
    // Match patterns like: 第一章, 第1章, 第二十三章, 第一回, 第三节 etc.
    let pattern = r"(?m)^[　\s]*(第[零〇一二三四五六七八九十百千万亿\d]+[章回节卷集篇部][^\n]*)";
    let re = Regex::new(pattern).unwrap();

    let matches: Vec<(usize, &str)> = re
        .find_iter(content)
        .map(|m| (m.start(), m.as_str().trim()))
        .collect();

    if matches.is_empty() {
        return Vec::new();
    }

    let mut chapters: Vec<(String, String)> = Vec::new();

    for i in 0..matches.len() {
        let (start, title) = matches[i];
        let end = if i + 1 < matches.len() {
            matches[i + 1].0
        } else {
            content.len()
        };

        let chapter_content = &content[start..end];
        // Remove the title line from content
        let body = chapter_content
            .strip_prefix(title)
            .unwrap_or(chapter_content)
            .trim()
            .to_string();

        if body.len() > 10 {
            chapters.push((title.to_string(), body));
        }
    }

    chapters
}

/// Read a TXT file with automatic encoding detection (UTF-8 / GBK / GB18030).
fn read_txt_auto_encoding(path: &str) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("无法读取文件 {}: {}", path, e))?;

    // Try UTF-8 first
    if let Ok(text) = std::str::from_utf8(&bytes) {
        return Ok(text.to_string());
    }

    // Try UTF-8 with BOM
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        if let Ok(text) = std::str::from_utf8(&bytes[3..]) {
            return Ok(text.to_string());
        }
    }

    // Try GBK
    let (cow, _, had_errors) = GBK.decode(&bytes);
    if !had_errors {
        return Ok(cow.into_owned());
    }

    // Try GB18030
    let (cow, _, had_errors) = GB18030.decode(&bytes);
    if !had_errors {
        return Ok(cow.into_owned());
    }

    // Fallback: lossy UTF-8
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_by_chapters() {
        let content = "前言内容\n\n第一章 开端\n这是第一章的内容。\n第二章 发展\n这是第二章的内容。\n第三章 结局\n这是第三章的内容。";
        let chapters = split_by_chapters(content);
        assert_eq!(chapters.len(), 3);
        assert!(chapters[0].0.contains("第一章"));
        assert!(chapters[1].0.contains("第二章"));
        assert!(chapters[2].0.contains("第三章"));
    }

    #[test]
    fn test_no_chapters() {
        let content = "这是一段没有章节标记的普通文本。";
        let chapters = split_by_chapters(content);
        assert!(chapters.is_empty());
    }
}
