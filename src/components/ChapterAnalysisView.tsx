import type { ChapterAnalysis, AnalysisDimension } from '../types';
import {
    Users, BookOpen, Sparkles, PenTool, Palette, Heart, Building, Globe,
} from 'lucide-react';
import { motion, Variants } from 'framer-motion';

interface Props {
    analysis: ChapterAnalysis;
    dimensions: AnalysisDimension[];
    chapterTitle: string;
}

const DIMENSION_CONFIG: Record<AnalysisDimension, { icon: React.ReactNode; label: string; color: string }> = {
    characters: { icon: <Users size={16} />, label: '人物图谱', color: 'badge-primary' },
    plot: { icon: <BookOpen size={16} />, label: '剧情脉络', color: 'badge-secondary' },
    foreshadowing: { icon: <Sparkles size={16} />, label: '伏笔与转折', color: 'badge-accent' },
    writing_technique: { icon: <PenTool size={16} />, label: '写作技法', color: 'badge-info' },
    rhetoric: { icon: <Palette size={16} />, label: '修辞与语言', color: 'badge-warning' },
    emotion: { icon: <Heart size={16} />, label: '情感与氛围', color: 'badge-error' },
    themes: { icon: <Building size={16} />, label: '主题与思想', color: 'badge-success' },
    worldbuilding: { icon: <Globe size={16} />, label: '世界观设定', color: 'badge-neutral' },
};

const containerVariants: Variants = {
    hidden: { opacity: 0 },
    show: {
        opacity: 1,
        transition: {
            staggerChildren: 0.1
        }
    }
};

const itemVariants: Variants = {
    hidden: { opacity: 0, y: 20 },
    show: { opacity: 1, y: 0, transition: { type: "spring", bounce: 0, duration: 0.5 } }
};

export default function ChapterAnalysisView({ analysis, dimensions, chapterTitle }: Props) {
    return (
        <motion.div
            className="space-y-6 max-w-3xl mx-auto"
            variants={containerVariants}
            initial="hidden"
            animate="show"
        >
            <motion.h2 variants={itemVariants} className="text-xl font-bold">{chapterTitle}</motion.h2>

            {/* Characters */}
            {analysis.characters && dimensions.includes('characters') && (
                <AnalysisSection dim="characters">
                    <div className="space-y-3">
                        <h4 className="font-semibold text-sm">出场人物</h4>
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
                            {analysis.characters.characters.map((ch, i) => (
                                <div key={i} className="bg-base-300/50 rounded-lg p-3">
                                    <div className="flex items-center gap-2">
                                        <span className="font-bold">{ch.name}</span>
                                        <span className="badge badge-outline badge-xs">{ch.role}</span>
                                    </div>
                                    {ch.traits.length > 0 && (
                                        <div className="flex flex-wrap gap-1 mt-1">
                                            {ch.traits.map((t, j) => <span key={j} className="badge badge-ghost badge-xs">{t}</span>)}
                                        </div>
                                    )}
                                    {ch.actions && <p className="text-xs text-base-content/60 mt-1">{ch.actions}</p>}
                                </div>
                            ))}
                        </div>
                        {analysis.characters.relationships.length > 0 && (
                            <>
                                <h4 className="font-semibold text-sm mt-4">人物关系</h4>
                                <div className="overflow-x-auto">
                                    <table className="table table-xs w-full">
                                        <thead><tr><th>人物A</th><th>关系</th><th>人物B</th><th>描述</th></tr></thead>
                                        <tbody>
                                            {analysis.characters.relationships.map((r, i) => (
                                                <tr key={i}>
                                                    <td className="whitespace-nowrap">{r.from}</td>
                                                    <td><span className="badge badge-outline badge-xs whitespace-nowrap">{r.relation_type}</span></td>
                                                    <td className="whitespace-nowrap">{r.to}</td>
                                                    <td className="text-xs text-base-content/60 max-w-[200px] break-words">{r.description}</td>
                                                </tr>
                                            ))}
                                        </tbody>
                                    </table>
                                </div>
                            </>
                        )}
                        {analysis.characters.insights && <InsightBlock text={analysis.characters.insights} />}
                    </div>
                </AnalysisSection>
            )}

            {/* Plot */}
            {analysis.plot && dimensions.includes('plot') && (
                <AnalysisSection dim="plot">
                    <p className="text-sm">{analysis.plot.summary}</p>
                    {analysis.plot.key_events.length > 0 && (
                        <div className="mt-3">
                            <h4 className="font-semibold text-sm mb-2">关键事件</h4>
                            <ul className="timeline timeline-vertical timeline-compact">
                                {analysis.plot.key_events.map((ev, i) => (
                                    <li key={i}>
                                        {i > 0 && <hr />}
                                        <div className="timeline-start text-xs text-base-content/40">{ev.cause || ''}</div>
                                        <div className="timeline-middle"><div className="w-2 h-2 rounded-full bg-secondary" /></div>
                                        <div className="timeline-end text-sm">{ev.event}</div>
                                        <hr />
                                    </li>
                                ))}
                            </ul>
                        </div>
                    )}
                    {analysis.plot.conflicts.length > 0 && (
                        <TagList title="核心冲突" items={analysis.plot.conflicts} />
                    )}
                    {analysis.plot.suspense.length > 0 && (
                        <TagList title="悬念" items={analysis.plot.suspense} />
                    )}
                    {analysis.plot.insights && <InsightBlock text={analysis.plot.insights} />}
                </AnalysisSection>
            )}

            {/* Foreshadowing */}
            {analysis.foreshadowing && dimensions.includes('foreshadowing') && (
                <AnalysisSection dim="foreshadowing">
                    {analysis.foreshadowing.setups.length > 0 && (
                        <div>
                            <h4 className="font-semibold text-sm mb-1">伏笔铺设</h4>
                            {analysis.foreshadowing.setups.map((s, i) => (
                                <div key={i} className="text-sm py-1 border-l-2 border-accent pl-3 my-1">{s.content}</div>
                            ))}
                        </div>
                    )}
                    {analysis.foreshadowing.callbacks.length > 0 && (
                        <div className="mt-3">
                            <h4 className="font-semibold text-sm mb-1">呼应前文</h4>
                            {analysis.foreshadowing.callbacks.map((c, i) => (
                                <div key={i} className="text-sm py-1 border-l-2 border-success pl-3 my-1">
                                    {c.content} {c.chapter_ref && <span className="badge badge-ghost badge-xs ml-1">{c.chapter_ref}</span>}
                                </div>
                            ))}
                        </div>
                    )}
                    {analysis.foreshadowing.turning_points.length > 0 && <TagList title="转折点" items={analysis.foreshadowing.turning_points} />}
                    {analysis.foreshadowing.cliffhangers.length > 0 && <TagList title="悬念钩子" items={analysis.foreshadowing.cliffhangers} />}
                    {analysis.foreshadowing.insights && <InsightBlock text={analysis.foreshadowing.insights} />}
                </AnalysisSection>
            )}

            {/* Writing Technique */}
            {analysis.writing_technique && dimensions.includes('writing_technique') && (
                <AnalysisSection dim="writing_technique">
                    <div className="grid grid-cols-2 gap-3">
                        <InfoCard label="叙事视角" value={analysis.writing_technique.narrative_perspective} />
                        <InfoCard label="时序处理" value={analysis.writing_technique.time_sequence} />
                        <InfoCard label="节奏控制" value={analysis.writing_technique.pacing} />
                        <InfoCard label="结构特点" value={analysis.writing_technique.structural_notes} />
                    </div>
                    {analysis.writing_technique.insights && <InsightBlock text={analysis.writing_technique.insights} />}
                </AnalysisSection>
            )}

            {/* Rhetoric */}
            {analysis.rhetoric && dimensions.includes('rhetoric') && (
                <AnalysisSection dim="rhetoric">
                    {analysis.rhetoric.devices.length > 0 && (
                        <div>
                            <h4 className="font-semibold text-sm mb-2">修辞手法</h4>
                            {analysis.rhetoric.devices.map((d, i) => (
                                <div key={i} className="mb-2">
                                    <span className="badge badge-warning badge-sm">{d.name}</span>
                                    <p className="text-xs text-base-content/60 mt-1 italic">「{d.example}」</p>
                                </div>
                            ))}
                        </div>
                    )}
                    {analysis.rhetoric.language_style && <InfoCard label="语言风格" value={analysis.rhetoric.language_style} />}
                    {analysis.rhetoric.notable_quotes.length > 0 && (
                        <div className="mt-3">
                            <h4 className="font-semibold text-sm mb-1">佳句摘录</h4>
                            {analysis.rhetoric.notable_quotes.map((q, i) => (
                                <blockquote key={i} className="border-l-2 border-warning pl-3 py-1 text-sm italic my-1">
                                    {q}
                                </blockquote>
                            ))}
                        </div>
                    )}
                    {analysis.rhetoric.insights && <InsightBlock text={analysis.rhetoric.insights} />}
                </AnalysisSection>
            )}

            {/* Emotion */}
            {analysis.emotion && dimensions.includes('emotion') && (
                <AnalysisSection dim="emotion">
                    <InfoCard label="整体基调" value={analysis.emotion.overall_tone} />
                    {analysis.emotion.emotion_arc.length > 0 && (
                        <div className="mt-3">
                            <h4 className="font-semibold text-sm mb-2">情感变化</h4>
                            <div className="flex flex-wrap gap-2">
                                {analysis.emotion.emotion_arc.map((pt, i) => (
                                    <div key={i} className="bg-base-300/50 rounded-lg px-3 py-2 text-xs">
                                        <div className="font-medium">{pt.segment}</div>
                                        <div>{pt.emotion} <span className="badge badge-ghost badge-xs">{pt.intensity}</span></div>
                                    </div>
                                ))}
                            </div>
                        </div>
                    )}
                    {analysis.emotion.atmosphere_techniques.length > 0 && (
                        <TagList title="氛围营造手法" items={analysis.emotion.atmosphere_techniques} />
                    )}
                    {analysis.emotion.insights && <InsightBlock text={analysis.emotion.insights} />}
                </AnalysisSection>
            )}

            {/* Themes */}
            {analysis.themes && dimensions.includes('themes') && (
                <AnalysisSection dim="themes">
                    {analysis.themes.motifs.length > 0 && <TagList title="母题" items={analysis.themes.motifs} />}
                    {analysis.themes.values.length > 0 && <TagList title="价值观" items={analysis.themes.values} />}
                    {analysis.themes.social_commentary && (
                        <InfoCard label="社会议题" value={analysis.themes.social_commentary} />
                    )}
                    {analysis.themes.insights && <InsightBlock text={analysis.themes.insights} />}
                </AnalysisSection>
            )}

            {/* Worldbuilding */}
            {analysis.worldbuilding && dimensions.includes('worldbuilding') && (
                <AnalysisSection dim="worldbuilding">
                    {analysis.worldbuilding.locations.length > 0 && (
                        <div>
                            <h4 className="font-semibold text-sm mb-1">地点</h4>
                            <div className="flex flex-wrap gap-2">
                                {analysis.worldbuilding.locations.map((loc, i) => (
                                    <div key={i} className="tooltip" data-tip={loc.description}>
                                        <span className="badge badge-outline">{loc.name}</span>
                                    </div>
                                ))}
                            </div>
                        </div>
                    )}
                    {analysis.worldbuilding.organizations.length > 0 && (
                        <div className="mt-3">
                            <h4 className="font-semibold text-sm mb-1">组织/势力</h4>
                            <div className="flex flex-wrap gap-2">
                                {analysis.worldbuilding.organizations.map((org, i) => (
                                    <div key={i} className="tooltip" data-tip={org.description}>
                                        <span className="badge badge-outline">{org.name}</span>
                                    </div>
                                ))}
                            </div>
                        </div>
                    )}
                    {analysis.worldbuilding.power_systems.length > 0 && <TagList title="力量体系" items={analysis.worldbuilding.power_systems} />}
                    {analysis.worldbuilding.items.length > 0 && (
                        <div className="mt-3">
                            <h4 className="font-semibold text-sm mb-1">重要物品</h4>
                            <div className="flex flex-wrap gap-2">
                                {analysis.worldbuilding.items.map((item, i) => (
                                    <div key={i} className="tooltip" data-tip={item.description}>
                                        <span className="badge badge-outline">{item.name}</span>
                                    </div>
                                ))}
                            </div>
                        </div>
                    )}
                    {analysis.worldbuilding.rules.length > 0 && <TagList title="规则" items={analysis.worldbuilding.rules} />}
                    {analysis.worldbuilding.insights && <InsightBlock text={analysis.worldbuilding.insights} />}
                </AnalysisSection>
            )}
        </motion.div>
    );
}

// ---- Helper Components ----

function AnalysisSection({ dim, children }: { dim: AnalysisDimension; children: React.ReactNode }) {
    const cfg = DIMENSION_CONFIG[dim];
    return (
        <motion.div variants={itemVariants} className="card bg-base-200 border border-base-300">
            <div className="card-body p-4">
                <div className="flex items-center gap-2 mb-3">
                    {cfg.icon}
                    <span className={`badge ${cfg.color} badge-sm`}>{cfg.label}</span>
                </div>
                {children}
            </div>
        </motion.div>
    );
}

function TagList({ title, items }: { title: string; items: string[] }) {
    return (
        <div className="mt-3">
            <h4 className="font-semibold text-sm mb-1">{title}</h4>
            <div className="flex flex-col gap-1.5">
                {items.map((item, i) => (
                    <div key={i} className="text-sm bg-base-300/50 rounded-lg px-3 py-1.5 leading-relaxed">
                        {item}
                    </div>
                ))}
            </div>
        </div>
    );
}

function InfoCard({ label, value }: { label: string; value: string }) {
    return (
        <div className="bg-base-300/50 rounded-lg p-3">
            <p className="text-xs text-base-content/50 mb-1">{label}</p>
            <p className="text-sm">{value}</p>
        </div>
    );
}

function InsightBlock({ text }: { text: string }) {
    return (
        <div className="mt-4 border-l-2 border-primary/40 pl-3 py-1">
            <p className="text-xs font-semibold text-primary/60 mb-1">💡 深度解读</p>
            <p className="text-sm text-base-content/80 leading-relaxed">{text}</p>
        </div>
    );
}
