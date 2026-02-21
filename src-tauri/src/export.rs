use crate::models::*;

pub fn generate_global_summary_md(novel: &Novel, summary: Option<&NovelSummary>) -> String {
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

    md
}

pub fn generate_chapter_md(ch: &Chapter) -> String {
    let mut md = String::new();
    md.push_str(&format!("# {}\n\n", ch.title));

    if let Some(a) = &ch.analysis {
        if let Some(plot) = &a.plot {
            md.push_str("## 剧情走向\n");
            md.push_str(&format!("**摘要**：{}\n\n", plot.summary));
            if !plot.key_events.is_empty() {
                md.push_str("**关键事件**：\n");
                for e in &plot.key_events {
                    md.push_str(&format!("- **{}**", e.event));
                    if let Some(cause) = &e.cause {
                        md.push_str(&format!(" (起因：{})", cause));
                    }
                    if let Some(effect) = &e.effect {
                        md.push_str(&format!(" -> (影响：{})", effect));
                    }
                    md.push_str("\n");
                }
                md.push_str("\n");
            }
            if !plot.conflicts.is_empty() {
                md.push_str("**冲突点**：\n");
                for c in &plot.conflicts {
                    md.push_str(&format!("- {}\n", c));
                }
                md.push_str("\n");
            }
            if !plot.suspense.is_empty() {
                md.push_str("**悬念设置**：\n");
                for s in &plot.suspense {
                    md.push_str(&format!("- {}\n", s));
                }
                md.push_str("\n");
            }
            if let Some(insight) = &plot.insights {
                md.push_str(&format!("*深度见解*：{}\n\n", insight));
            }
        }

        if let Some(chars) = &a.characters {
            md.push_str("## 人物刻画\n");
            if !chars.characters.is_empty() {
                md.push_str("**出场人物**：\n");
                for c in &chars.characters {
                    let traits = if c.traits.is_empty() {
                        String::new()
                    } else {
                        format!(" [{}]", c.traits.join("，"))
                    };
                    md.push_str(&format!(
                        "- **{}** ({}): {}{}\n",
                        c.name, c.role, c.actions, traits
                    ));
                }
                md.push_str("\n");
            }
            if !chars.relationships.is_empty() {
                md.push_str("**人物关系网络**：\n");
                for r in &chars.relationships {
                    md.push_str(&format!(
                        "- **{}** -> **{}** ({}): {}",
                        r.from, r.to, r.relation_type, r.description
                    ));
                    if let Some(change) = &r.change {
                        md.push_str(&format!(" (变化：{})", change));
                    }
                    md.push_str("\n");
                }
                md.push_str("\n");
            }
            if let Some(insight) = &chars.insights {
                md.push_str(&format!("*深度见解*：{}\n\n", insight));
            }
        }

        if let Some(fw) = &a.foreshadowing {
            md.push_str("## 伏笔分析\n");
            if !fw.setups.is_empty() {
                md.push_str("**铺垫**：\n");
                for s in &fw.setups {
                    md.push_str(&format!("- {}\n", s.content));
                }
                md.push_str("\n");
            }
            if !fw.callbacks.is_empty() {
                md.push_str("**回收**：\n");
                for c in &fw.callbacks {
                    md.push_str(&format!("- {}", c.content));
                    if let Some(ref_ch) = &c.chapter_ref {
                        md.push_str(&format!(" (呼应章节：{})", ref_ch));
                    }
                    md.push_str("\n");
                }
                md.push_str("\n");
            }
            if !fw.turning_points.is_empty() {
                md.push_str("**转折点**：\n");
                for tp in &fw.turning_points {
                    md.push_str(&format!("- {}\n", tp));
                }
                md.push_str("\n");
            }
            if !fw.cliffhangers.is_empty() {
                md.push_str("**悬念留白**：\n");
                for ch in &fw.cliffhangers {
                    md.push_str(&format!("- {}\n", ch));
                }
                md.push_str("\n");
            }
            if let Some(insight) = &fw.insights {
                md.push_str(&format!("*深度见解*：{}\n\n", insight));
            }
        }

        if let Some(wt) = &a.writing_technique {
            md.push_str("## 写作技法\n");
            md.push_str(&format!(
                "- **视角**: {}\n- **时序**: {}\n- **节奏**: {}\n",
                wt.narrative_perspective, wt.time_sequence, wt.pacing
            ));
            if !wt.structural_notes.is_empty() {
                md.push_str(&format!("**结构分析**: {}\n", wt.structural_notes));
            }
            if let Some(insight) = &wt.insights {
                md.push_str(&format!("*深度见解*：{}\n", insight));
            }
            md.push_str("\n");
        }

        if let Some(rhe) = &a.rhetoric {
            md.push_str("## 修辞特色\n");
            if !rhe.devices.is_empty() {
                md.push_str("**手法**：\n");
                for d in &rhe.devices {
                    md.push_str(&format!("- **{}**: {}\n", d.name, d.example));
                }
                md.push_str("\n");
            }
            if !rhe.language_style.is_empty() {
                md.push_str(&format!("**语言风格**：{}\n", rhe.language_style));
            }
            if !rhe.notable_quotes.is_empty() {
                md.push_str("**金句摘录**：\n");
                for q in &rhe.notable_quotes {
                    md.push_str(&format!("> {}\n", q));
                }
                md.push_str("\n");
            }
            if let Some(insight) = &rhe.insights {
                md.push_str(&format!("*深度见解*：{}\n\n", insight));
            }
        }

        if let Some(emo) = &a.emotion {
            md.push_str("## 情绪流与氛围\n");
            md.push_str(&format!("**整体基调**：{}\n\n", emo.overall_tone));
            if !emo.emotion_arc.is_empty() {
                md.push_str("**情绪起伏**：\n");
                for arc in &emo.emotion_arc {
                    md.push_str(&format!(
                        "- {} | {} (强度: {})\n",
                        arc.segment, arc.emotion, arc.intensity
                    ));
                }
                md.push_str("\n");
            }
            if !emo.atmosphere_techniques.is_empty() {
                md.push_str("**氛围营造**：\n");
                for t in &emo.atmosphere_techniques {
                    md.push_str(&format!("- {}\n", t));
                }
                md.push_str("\n");
            }
            if let Some(insight) = &emo.insights {
                md.push_str(&format!("*深度见解*：{}\n\n", insight));
            }
        }

        if let Some(thm) = &a.themes {
            md.push_str("## 思想主题\n");
            if !thm.motifs.is_empty() {
                md.push_str("**母题**：\n");
                for m in &thm.motifs {
                    md.push_str(&format!("- {}\n", m));
                }
                md.push_str("\n");
            }
            if !thm.values.is_empty() {
                md.push_str("**价值观**：\n");
                for v in &thm.values {
                    md.push_str(&format!("- {}\n", v));
                }
                md.push_str("\n");
            }
            if let Some(sc) = &thm.social_commentary {
                md.push_str(&format!("**社会隐喻**：{}\n\n", sc));
            }
            if let Some(insight) = &thm.insights {
                md.push_str(&format!("*深度见解*：{}\n\n", insight));
            }
        }

        if let Some(wb) = &a.worldbuilding {
            md.push_str("## 世界观构建\n");
            if !wb.locations.is_empty() {
                md.push_str("**场景/地点**：\n");
                for l in &wb.locations {
                    md.push_str(&format!("- **{}**: {}\n", l.name, l.description));
                }
                md.push_str("\n");
            }
            if !wb.organizations.is_empty() {
                md.push_str("**势力/组织**：\n");
                for o in &wb.organizations {
                    md.push_str(&format!("- **{}**: {}\n", o.name, o.description));
                }
                md.push_str("\n");
            }
            if !wb.power_systems.is_empty() {
                md.push_str("**战力体系**：\n");
                for p in &wb.power_systems {
                    md.push_str(&format!("- {}\n", p));
                }
                md.push_str("\n");
            }
            if !wb.items.is_empty() {
                md.push_str("**特有道具**：\n");
                for i in &wb.items {
                    md.push_str(&format!("- **{}**: {}\n", i.name, i.description));
                }
                md.push_str("\n");
            }
            if !wb.rules.is_empty() {
                md.push_str("**隐藏规则**：\n");
                for r in &wb.rules {
                    md.push_str(&format!("- {}\n", r));
                }
                md.push_str("\n");
            }
            if let Some(insight) = &wb.insights {
                md.push_str(&format!("*深度见解*：{}\n\n", insight));
            }
        }
    } else {
        md.push_str("*本章尚未分析*\n\n");
    }

    md
}
