import { useEffect, useState } from 'react';
import { useNovelStore } from '../store/index';

import FullBookManualPromptPanel from '../components/FullBookManualPromptPanel';
import ConfirmDialog from '../components/ConfirmDialog';
import { Zap } from 'lucide-react';

export default function FullBookSummaryView({ novelId }: { novelId: string }) {
    const { novelSummary, generateFullSummary, loading, chapters, analysisMode } = useNovelStore();
    const analyzedCount = chapters.filter(c => c.has_analysis).length;
    const [confirmClearSummary, setConfirmClearSummary] = useState(false);

    useEffect(() => {
        useNovelStore.getState().fetchSummary();
    }, [novelId]);

    const handleGenerate = async () => {
        try {
            await generateFullSummary(novelId);
        } catch (e) {
            console.error(e);
        }
    };

    if (loading) {
        return (
            <div className="flex-1 flex flex-col items-center justify-center p-8 gap-4">
                <span className="loading loading-spinner text-primary loading-lg"></span>
                <p className="text-base-content/60">正在生成全书总结，预计耗时较长，请耐心等待...</p>
            </div>
        );
    }

    if (!novelSummary) {
        return (
            <div className="flex-1 flex flex-col items-center justify-center p-8 text-center max-w-md mx-auto">
                <div className="bg-base-200 rounded-box p-8 border border-base-300 shadow-sm w-full">
                    <h2 className="text-xl font-bold mb-4">全书分析总结</h2>
                    <p className="text-base-content/70 mb-6 text-sm">
                        根据当前已分析的 {analyzedCount} 个章节（共 {chapters.length} 章），生成整本书籍的剧情脉络、人物关系及主题分析。<br />
                        建议在大部分章节分析完成后再生成。
                    </p>
                    {analysisMode === 'api' ? (
                        <>
                            <button
                                className="btn btn-primary w-full"
                                onClick={handleGenerate}
                                disabled={analyzedCount === 0}
                            >
                                <Zap size={18} />
                                {analyzedCount > 0 ? '生成全书总结' : '需先分析部分章节'}
                            </button>
                            {analyzedCount === 0 && (
                                <p className="text-error text-xs mt-3">至少需要分析一个章节才能生成汇总</p>
                            )}
                        </>
                    ) : (
                        <div className="text-left mt-4 border-t border-base-300 pt-4">
                            {analyzedCount > 0 ? (
                                <FullBookManualPromptPanel novelId={novelId} />
                            ) : (
                                <p className="text-error text-center text-sm">至少需要分析一个章节才能生成汇总</p>
                            )}
                        </div>
                    )}
                </div>
            </div>
        );
    }

    return (
        <div className="flex-1 overflow-y-auto bg-base-100 p-8">
            <div className="max-w-4xl mx-auto space-y-8">
                <div className="flex justify-between items-end border-b border-base-300 pb-4">
                    <div>
                        <h1 className="text-3xl font-bold font-serif mb-2">全书脉络分析</h1>
                        <p className="text-sm text-base-content/50">
                            生成时间: {new Date(novelSummary.created_at).toLocaleString()}
                        </p>
                    </div>
                    <div className="flex gap-2">
                        <button
                            className="btn btn-outline btn-sm btn-error"
                            onClick={() => setConfirmClearSummary(true)}
                        >
                            清除分析
                        </button>
                        <button
                            className="btn btn-outline btn-sm"
                            onClick={handleGenerate}
                        >
                            <Zap size={14} /> 重新生成
                        </button>
                    </div>
                </div>

                <div className="prose prose-sm md:prose-base max-w-none">
                    <h3>主要剧情与结局走势</h3>
                    <p className="whitespace-pre-line bg-base-200/50 p-4 rounded-xl border border-base-300">
                        {novelSummary.overall_plot || "暂无"}
                    </p>

                    <h3 className="mt-8">核心主题</h3>
                    {novelSummary.themes && novelSummary.themes.length > 0 ? (
                        <ul className="bg-base-200/50 p-4 rounded-xl border border-base-300">
                            {novelSummary.themes.map((t, i) => <li key={i}>{t}</li>)}
                        </ul>
                    ) : (
                        <p className="whitespace-pre-line bg-base-200/50 p-4 rounded-xl border border-base-300">暂无</p>
                    )}

                    <h3 className="mt-8">人物成长线</h3>
                    {novelSummary.character_arcs && novelSummary.character_arcs.length > 0 ? (
                        <div className="space-y-4">
                            {novelSummary.character_arcs.map((arc, i) => (
                                <div key={i} className="bg-base-200/50 p-4 rounded-xl border border-base-300">
                                    <strong>{arc.name}</strong>: {arc.arc}
                                </div>
                            ))}
                        </div>
                    ) : (
                        <p className="whitespace-pre-line bg-base-200/50 p-4 rounded-xl border border-base-300">暂无</p>
                    )}

                    <h3 className="mt-8">写作风格</h3>
                    <p className="whitespace-pre-line bg-base-200/50 p-4 rounded-xl border border-base-300">
                        {novelSummary.writing_style || "暂无"}
                    </p>

                    <h3 className="mt-8">世界观设定</h3>
                    <p className="whitespace-pre-line bg-base-200/50 p-4 rounded-xl border border-base-300">
                        {novelSummary.worldbuilding || "暂无"}
                    </p>
                </div>
            </div>
            {confirmClearSummary && (
                <ConfirmDialog
                    title="清除全书分析"
                    message="确定要清除全书脉络分析吗？清除后可以重新生成。"
                    confirmText="清除"
                    kind="warning"
                    onConfirm={async () => {
                        await useNovelStore.getState().clearNovelSummary(novelId);
                        setConfirmClearSummary(false);
                    }}
                    onCancel={() => setConfirmClearSummary(false)}
                />
            )}
        </div>
    );
}
