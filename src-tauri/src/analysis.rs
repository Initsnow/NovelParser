use crate::models::*;

/// Parse LLM JSON response into a ChapterAnalysis struct.
/// Includes tolerance for common LLM output issues (markdown fences, trailing commas).
pub fn parse_analysis_json(json_str: &str) -> Result<ChapterAnalysis, String> {
    let cleaned = clean_json_response(json_str);
    serde_json::from_str::<ChapterAnalysis>(&cleaned).map_err(|e| {
        format!(
            "JSON 解析失败: {}。原始文本前200字: {}",
            e,
            &cleaned[..cleaned.len().min(200)]
        )
    })
}

/// Parse LLM JSON response into a NovelSummary struct.
pub fn parse_summary_json(json_str: &str) -> Result<NovelSummary, String> {
    let cleaned = clean_json_response(json_str);
    serde_json::from_str::<NovelSummary>(&cleaned).map_err(|e| {
        format!(
            "汇总 JSON 解析失败: {}。原始文本前200字: {}",
            e,
            &cleaned[..cleaned.len().min(200)]
        )
    })
}

/// Merge multiple segment analyses into one combined analysis.
pub fn merge_segment_analyses(segments: Vec<ChapterAnalysis>) -> ChapterAnalysis {
    let mut merged = ChapterAnalysis::default();

    for seg in segments {
        // Merge characters
        if let Some(chars) = seg.characters {
            let existing = merged.characters.get_or_insert_with(|| CharactersAnalysis {
                characters: Vec::new(),
                relationships: Vec::new(),
                insights: None,
            });
            for ch in chars.characters {
                if !existing.characters.iter().any(|c| c.name == ch.name) {
                    existing.characters.push(ch);
                }
            }
            for rel in chars.relationships {
                if !existing
                    .relationships
                    .iter()
                    .any(|r| r.from == rel.from && r.to == rel.to)
                {
                    existing.relationships.push(rel);
                }
            }
            if let Some(ins) = chars.insights {
                let e = existing.insights.get_or_insert_with(String::new);
                if !e.is_empty() {
                    e.push(' ');
                }
                e.push_str(&ins);
            }
        }

        // Merge plot
        if let Some(plot) = seg.plot {
            let existing = merged.plot.get_or_insert_with(|| PlotAnalysis {
                summary: String::new(),
                key_events: Vec::new(),
                conflicts: Vec::new(),
                suspense: Vec::new(),
                insights: None,
            });
            if !plot.summary.is_empty() {
                if !existing.summary.is_empty() {
                    existing.summary.push_str(" ");
                }
                existing.summary.push_str(&plot.summary);
            }
            existing.key_events.extend(plot.key_events);
            existing.conflicts.extend(plot.conflicts);
            existing.suspense.extend(plot.suspense);
            if let Some(ins) = plot.insights {
                let e = existing.insights.get_or_insert_with(String::new);
                if !e.is_empty() {
                    e.push(' ');
                }
                e.push_str(&ins);
            }
        }

        // Merge foreshadowing
        if let Some(fore) = seg.foreshadowing {
            let existing = merged
                .foreshadowing
                .get_or_insert_with(|| ForeshadowingAnalysis {
                    setups: Vec::new(),
                    callbacks: Vec::new(),
                    turning_points: Vec::new(),
                    cliffhangers: Vec::new(),
                    insights: None,
                });
            existing.setups.extend(fore.setups);
            existing.callbacks.extend(fore.callbacks);
            existing.turning_points.extend(fore.turning_points);
            existing.cliffhangers.extend(fore.cliffhangers);
        }

        // For other dimensions, take the first non-None value or the last one
        if seg.writing_technique.is_some() {
            merged.writing_technique = seg.writing_technique;
        }
        if seg.rhetoric.is_some() {
            let existing = merged.rhetoric.get_or_insert_with(|| RhetoricAnalysis {
                devices: Vec::new(),
                language_style: String::new(),
                notable_quotes: Vec::new(),
                insights: None,
            });
            if let Some(rhet) = seg.rhetoric.as_ref() {
                existing.devices.extend(rhet.devices.clone());
                if !rhet.language_style.is_empty() {
                    existing.language_style = rhet.language_style.clone();
                }
                existing.notable_quotes.extend(rhet.notable_quotes.clone());
            }
        }
        if seg.emotion.is_some() {
            let existing = merged.emotion.get_or_insert_with(|| EmotionAnalysis {
                overall_tone: String::new(),
                emotion_arc: Vec::new(),
                atmosphere_techniques: Vec::new(),
                insights: None,
            });
            if let Some(emo) = seg.emotion.as_ref() {
                if !emo.overall_tone.is_empty() {
                    existing.overall_tone = emo.overall_tone.clone();
                }
                existing.emotion_arc.extend(emo.emotion_arc.clone());
                existing
                    .atmosphere_techniques
                    .extend(emo.atmosphere_techniques.clone());
            }
        }
        if seg.themes.is_some() {
            merged.themes = seg.themes;
        }
        if seg.worldbuilding.is_some() {
            let existing = merged
                .worldbuilding
                .get_or_insert_with(|| WorldbuildingAnalysis {
                    locations: Vec::new(),
                    organizations: Vec::new(),
                    power_systems: Vec::new(),
                    items: Vec::new(),
                    rules: Vec::new(),
                    insights: None,
                });
            if let Some(wb) = seg.worldbuilding.as_ref() {
                existing.locations.extend(wb.locations.clone());
                existing.organizations.extend(wb.organizations.clone());
                existing.power_systems.extend(wb.power_systems.clone());
                existing.items.extend(wb.items.clone());
                existing.rules.extend(wb.rules.clone());
            }
        }
    }

    merged
}

/// Clean common LLM output issues from JSON response.
pub fn clean_json_response(raw: &str) -> String {
    let mut s = raw.trim().to_string();

    // Remove markdown code fences
    if s.starts_with("```json") {
        s = s.strip_prefix("```json").unwrap_or(&s).to_string();
    } else if s.starts_with("```") {
        s = s.strip_prefix("```").unwrap_or(&s).to_string();
    }
    if s.ends_with("```") {
        s = s.strip_suffix("```").unwrap_or(&s).to_string();
    }

    s = s.trim().to_string();

    // Remove trailing commas before } or ]
    let re = regex::Regex::new(r",(\s*[}\]])").unwrap();
    s = re.replace_all(&s, "$1").to_string();

    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_json_response_with_fences() {
        let raw = "```json\n{\"plot\": {\"summary\": \"test\"}}\n```";
        let cleaned = clean_json_response(raw);
        assert_eq!(cleaned, "{\"plot\": {\"summary\": \"test\"}}");
    }

    #[test]
    fn test_clean_json_response_trailing_comma() {
        let raw = r#"{"plot": {"summary": "test",}}"#;
        let cleaned = clean_json_response(raw);
        assert_eq!(cleaned, r#"{"plot": {"summary": "test"}}"#);
    }

    #[test]
    fn test_parse_analysis_basic() {
        let json = r#"{
            "plot": {
                "summary": "测试摘要",
                "key_events": [{"event": "事件1"}],
                "conflicts": [],
                "suspense": []
            }
        }"#;
        let result = parse_analysis_json(json);
        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(analysis.plot.is_some());
        assert_eq!(analysis.plot.unwrap().summary, "测试摘要");
    }
}
