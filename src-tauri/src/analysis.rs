use crate::models::*;
use std::sync::OnceLock;

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

fn append_string(existing: &mut String, new: &str, separator: &str) {
    if !new.is_empty() {
        if !existing.is_empty() {
            existing.push_str(separator);
        }
        existing.push_str(new);
    }
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
                append_string(e, &ins, " ");
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
            append_string(&mut existing.summary, &plot.summary, " ");
            existing.key_events.extend(plot.key_events);
            existing.conflicts.extend(plot.conflicts);
            existing.suspense.extend(plot.suspense);
            if let Some(ins) = plot.insights {
                let e = existing.insights.get_or_insert_with(String::new);
                append_string(e, &ins, " ");
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

        // Merge writing_technique (combine string fields)
        if let Some(wt) = seg.writing_technique {
            let existing =
                merged
                    .writing_technique
                    .get_or_insert_with(|| WritingTechniqueAnalysis {
                        narrative_perspective: String::new(),
                        time_sequence: String::new(),
                        pacing: String::new(),
                        structural_notes: String::new(),
                        insights: None,
                    });
            append_string(
                &mut existing.narrative_perspective,
                &wt.narrative_perspective,
                "; ",
            );
            append_string(&mut existing.time_sequence, &wt.time_sequence, "; ");
            append_string(&mut existing.pacing, &wt.pacing, "; ");
            append_string(&mut existing.structural_notes, &wt.structural_notes, "; ");

            if let Some(ins) = wt.insights {
                let e = existing.insights.get_or_insert_with(String::new);
                append_string(e, &ins, " ");
            }
        }

        if let Some(rhet) = seg.rhetoric {
            let existing = merged.rhetoric.get_or_insert_with(|| RhetoricAnalysis {
                devices: Vec::new(),
                language_style: String::new(),
                notable_quotes: Vec::new(),
                insights: None,
            });
            existing.devices.extend(rhet.devices);
            if !rhet.language_style.is_empty() {
                existing.language_style = rhet.language_style;
            }
            existing.notable_quotes.extend(rhet.notable_quotes);
        }

        if let Some(emo) = seg.emotion {
            let existing = merged.emotion.get_or_insert_with(|| EmotionAnalysis {
                overall_tone: String::new(),
                emotion_arc: Vec::new(),
                atmosphere_techniques: Vec::new(),
                insights: None,
            });
            if !emo.overall_tone.is_empty() {
                existing.overall_tone = emo.overall_tone;
            }
            existing.emotion_arc.extend(emo.emotion_arc);
            existing
                .atmosphere_techniques
                .extend(emo.atmosphere_techniques);
        }

        // Merge themes (combine lists)
        if let Some(th) = seg.themes {
            let existing = merged.themes.get_or_insert_with(|| ThemesAnalysis {
                motifs: Vec::new(),
                values: Vec::new(),
                social_commentary: None,
                insights: None,
            });
            existing.motifs.extend(th.motifs);
            existing.values.extend(th.values);
            if let Some(sc) = th.social_commentary {
                let e = existing.social_commentary.get_or_insert_with(String::new);
                append_string(e, &sc, " ");
            }
            if let Some(ins) = th.insights {
                let e = existing.insights.get_or_insert_with(String::new);
                append_string(e, &ins, " ");
            }
        }

        if let Some(wb) = seg.worldbuilding {
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
            existing.locations.extend(wb.locations);
            existing.organizations.extend(wb.organizations);
            existing.power_systems.extend(wb.power_systems);
            existing.items.extend(wb.items);
            existing.rules.extend(wb.rules);
        }
    }

    if let Some(th) = &mut merged.themes {
        th.motifs.sort_unstable();
        th.motifs.dedup();
        th.values.sort_unstable();
        th.values.dedup();
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
    static TRAILING_COMMA_RE: OnceLock<regex::Regex> = OnceLock::new();
    let re = TRAILING_COMMA_RE.get_or_init(|| regex::Regex::new(r",(\s*[}\]])").unwrap());
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
