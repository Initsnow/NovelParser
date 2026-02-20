import { useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import { useNovelStore } from '../store/novelStore';
import { ArrowLeft, BookOpen, Download } from 'lucide-react';
import { save } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';

export default function SummaryPage() {
    const { novelId } = useParams<{ novelId: string }>();
    const {
        currentNovel, novelSummary, selectNovel, fetchSummary,
        generateFullSummary, loading, progress, chapters, fetchChapters
    } = useNovelStore();

    useEffect(() => {
        if (novelId) {
            if (!currentNovel || currentNovel.id !== novelId) {
                selectNovel(novelId);
            }
            fetchSummary();
            fetchChapters(novelId);
        }
    }, [novelId]);

    const analyzedCount = chapters.filter(c => c.has_analysis).length;

    if (!currentNovel) {
        return <div className="flex-1 flex items-center justify-center"><span className="loading loading-spinner loading-lg" /></div>;
    }

    const handleExport = async () => {
        try {
            const path = await save({
                filters: [{ name: 'Markdown', extensions: ['md'] }],
                defaultPath: `${currentNovel.title}分析报告.md`
            });
            if (path) {
                await invoke('export_novel_report', { novelId: currentNovel.id, path });
            }
        } catch (e) {
            console.error('Export failed:', e);
        }
    };

    return (
        <div className="flex-1 p-8 overflow-y-auto">
            <div className="max-w-3xl mx-auto">
                <div className="flex items-center gap-3 mb-6">
                    <Link to={`/novel/${currentNovel.id}`} className="btn btn-ghost btn-sm btn-square">
                        <ArrowLeft size={16} />
                    </Link>
                    <div>
                        <h1 className="text-2xl font-bold">{currentNovel.title}</h1>
                        <p className="text-sm text-base-content/50">全书分析报告</p>
                    </div>
                    <div className="flex-1"></div>
                    <button
                        className={`btn btn-primary ${loading ? 'btn-disabled' : ''}`}
                        onClick={() => generateFullSummary(currentNovel.id)}
                        disabled={loading || analyzedCount === 0}
                    >
                        {loading && progress?.status.startsWith('summarizing') ? (
                            <span className="loading loading-spinner"></span>
                        ) : null}
                        生成全书汇总
                    </button>
                    {novelSummary && (
                        <button className="btn btn-secondary" onClick={handleExport}>
                            <Download size={16} /> 导出报告
                        </button>
                    )}
                </div>

                {progress && progress.status.startsWith('summarizing') && (
                    <div className="card bg-base-200 border border-base-300 mb-6">
                        <div className="card-body p-4 flex flex-col gap-2">
                            <div className="flex justify-between text-sm">
                                <span>{progress.message}</span>
                                <span>{progress.current}/{progress.total}</span>
                            </div>
                            <progress className="progress progress-primary w-full" value={progress.current} max={progress.total}></progress>
                        </div>
                    </div>
                )}

                {!novelSummary ? (
                    <div className="card bg-base-200 border border-base-300">
                        <div className="card-body items-center py-16">
                            <BookOpen size={48} className="text-base-content/20 mb-4" />
                            <p className="text-base-content/40">尚未生成全书汇总</p>
                            <p className="text-xs text-base-content/30 mt-1">目前有 {analyzedCount} 个章节已分析</p>
                            {analyzedCount > 0 && (
                                <button
                                    className="btn btn-outline btn-sm mt-4"
                                    onClick={() => generateFullSummary(currentNovel.id)}
                                >
                                    使用已有数据生成
                                </button>
                            )}
                        </div>
                    </div>
                ) : (
                    <div className="space-y-6">
                        {novelSummary.overall_plot && (
                            <div className="card bg-base-200 border border-base-300">
                                <div className="card-body p-5">
                                    <h3 className="font-bold text-lg mb-2">📖 整体剧情</h3>
                                    <p className="text-sm leading-relaxed">{novelSummary.overall_plot}</p>
                                </div>
                            </div>
                        )}

                        {novelSummary.character_arcs && novelSummary.character_arcs.length > 0 && (
                            <div className="card bg-base-200 border border-base-300">
                                <div className="card-body p-5">
                                    <h3 className="font-bold text-lg mb-3">👤 人物弧线</h3>
                                    <div className="space-y-3">
                                        {novelSummary.character_arcs.map((arc, i) => (
                                            <div key={i} className="bg-base-300/50 rounded-lg p-3">
                                                <span className="font-bold text-primary">{arc.name}</span>
                                                <p className="text-sm mt-1">{arc.arc}</p>
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            </div>
                        )}

                        {novelSummary.themes && novelSummary.themes.length > 0 && (
                            <div className="card bg-base-200 border border-base-300">
                                <div className="card-body p-5">
                                    <h3 className="font-bold text-lg mb-2">🏛️ 主题</h3>
                                    <div className="flex flex-wrap gap-2">
                                        {novelSummary.themes.map((t, i) => (
                                            <span key={i} className="badge badge-lg badge-outline">{t}</span>
                                        ))}
                                    </div>
                                </div>
                            </div>
                        )}

                        {novelSummary.writing_style && (
                            <div className="card bg-base-200 border border-base-300">
                                <div className="card-body p-5">
                                    <h3 className="font-bold text-lg mb-2">✍️ 写作风格</h3>
                                    <p className="text-sm leading-relaxed">{novelSummary.writing_style}</p>
                                </div>
                            </div>
                        )}

                        {novelSummary.worldbuilding && (
                            <div className="card bg-base-200 border border-base-300">
                                <div className="card-body p-5">
                                    <h3 className="font-bold text-lg mb-2">🌍 世界观</h3>
                                    <p className="text-sm leading-relaxed">{novelSummary.worldbuilding}</p>
                                </div>
                            </div>
                        )}
                    </div>
                )}
            </div>
        </div>
    );
}
