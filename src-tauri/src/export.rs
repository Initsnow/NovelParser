use crate::models::*;

pub fn generate_markdown_report(
    novel: &Novel,
    summary: Option<&NovelSummary>,
    chapters: &[Chapter],
) -> String {
    let mut md = String::new();
    md.push_str(&format!("# 《{}》分析报告\n\n", novel.title));

    if let Some(s) = summary {
        md.push_str("## 全书汇总\n\n");
        if let Some(plot) = &s.overall_plot {
            md.push_str("### 整体剧情\n");
            md.push_str(plot);
            md.push_str("\n\n");
        }
        if let Some(arcs) = &s.character_arcs {
            if !arcs.is_empty() {
                md.push_str("### 人物弧线\n");
                for arc in arcs {
                    md.push_str(&format!("- **{}**: {}\n", arc.name, arc.arc));
                }
                md.push_str("\n");
            }
        }
        if let Some(themes) = &s.themes {
            if !themes.is_empty() {
                md.push_str("### 主题\n");
                for t in themes {
                    md.push_str(&format!("- {}\n", t));
                }
                md.push_str("\n");
            }
        }
        if let Some(style) = &s.writing_style {
            md.push_str("### 写作风格\n");
            md.push_str(style);
            md.push_str("\n\n");
        }
        if let Some(wb) = &s.worldbuilding {
            md.push_str("### 世界观\n");
            md.push_str(wb);
            md.push_str("\n\n");
        }
    }

    md.push_str("## 章节详情\n\n");
    for ch in chapters {
        md.push_str(&format!("### {}\n\n", ch.title));
        if let Some(a) = &ch.analysis {
            if let Some(plot) = &a.plot {
                md.push_str("**剧情摘要**：\n");
                md.push_str(&plot.summary);
                md.push_str("\n\n");
            }
            if let Some(chars) = &a.characters {
                if !chars.characters.is_empty() {
                    md.push_str("**出场人物**：\n");
                    for c in &chars.characters {
                        md.push_str(&format!("- **{}** ({}): {}\n", c.name, c.role, c.actions));
                    }
                    md.push_str("\n");
                }
            }
            if let Some(wt) = &a.writing_technique {
                md.push_str("**写作技法**：\n");
                md.push_str(&format!(
                    "视角: {} | 时序: {} | 节奏: {}\n\n",
                    wt.narrative_perspective, wt.time_sequence, wt.pacing
                ));
            }
        } else {
            md.push_str("*本章尚未分析*\n\n");
        }
    }

    md
}
