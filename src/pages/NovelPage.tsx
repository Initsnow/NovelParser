import { useEffect, useState, useRef } from 'react';
import { useParams } from 'react-router-dom';
import { useNovelStore } from '../store/novelStore';
import ChapterAnalysisView from '../components/ChapterAnalysisView';
import ManualPromptPanel from '../components/ManualPromptPanel';
import FullBookManualPromptPanel from '../components/FullBookManualPromptPanel';
import DimensionSelector from '../components/DimensionSelector';
import { Play, CheckCircle, Circle, ChevronRight, Zap, ClipboardCopy, Settings2, Trash2, ListChecks, X } from 'lucide-react';
import ConfirmDialog from '../components/ConfirmDialog';

const formatTime = (ms: number) => {
    const s = Math.floor(ms / 1000);
    if (s < 60) return `${s}s`;
    const m = Math.floor(s / 60);
    return `${m}m ${s % 60}s`;
};

function BatchTimeStats({ startTime, current, total }: { startTime: number, current: number, total: number }) {
    const [now, setNow] = useState(Date.now());

    useEffect(() => {
        const interval = setInterval(() => setNow(Date.now()), 1000);
        return () => clearInterval(interval);
    }, []);

    const elapsedMs = now - startTime;
    const estimatedTotalMs = current > 0 ? (elapsedMs / current) * total : 0;
    const remainingMs = Math.max(0, estimatedTotalMs - elapsedMs);

    return (
        <div className="text-[10px] text-base-content/60 flex justify-between">
            <span>已用: {formatTime(elapsedMs)}</span>
            <span>预计剩余: {current > 0 ? formatTime(remainingMs) : '计算中...'}</span>
        </div>
    );
}

function StreamingJsonViewer({ content }: { content: string }) {
    const scrollRef = useRef<HTMLDivElement>(null);
    const [autoScroll, setAutoScroll] = useState(true);

    const handleScroll = () => {
        if (!scrollRef.current) return;
        const { scrollTop, scrollHeight, clientHeight } = scrollRef.current;
        const isAtBottom = scrollHeight - scrollTop - clientHeight < 10;
        setAutoScroll(isAtBottom);
    };

    useEffect(() => {
        if (autoScroll && scrollRef.current) {
            scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
        }
    }, [content, autoScroll]);

    return (
        <div className="relative flex-1 min-h-0 min-w-0 flex flex-col">
            <div
                ref={scrollRef}
                onScroll={handleScroll}
                className="flex-1 bg-base-300/30 rounded-lg p-4 font-mono text-sm overflow-y-auto whitespace-pre-wrap border border-base-300 shadow-inner"
            >
                {content || <span className="text-base-content/30 italic">等待模型响应...</span>}
            </div>
            {!autoScroll && (
                <button
                    className="absolute bottom-4 right-4 btn btn-xs btn-primary shadow-lg opacity-90"
                    onClick={() => {
                        setAutoScroll(true);
                        if (scrollRef.current) {
                            scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
                        }
                    }}
                >
                    ↓ 回到最新
                </button>
            )}
        </div>
    );
}

function FullBookSummaryView({ novelId }: { novelId: string }) {
    const { novelSummary, generateFullSummary, loading, chapters, analysisMode } = useNovelStore();
    const analyzedCount = chapters.filter(c => c.has_analysis).length;

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
                    <button
                        className="btn btn-outline btn-sm"
                        onClick={handleGenerate}
                    >
                        <Zap size={14} /> 重新生成
                    </button>
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
        </div>
    );
}

export default function NovelPage() {
    const { novelId } = useParams<{ novelId: string }>();
    const {
        currentNovel, chapters, selectedChapter,
        selectNovel, selectChapter, analysisMode, setAnalysisMode,
        analyzeChapterApi, batchAnalyzeNovel, batchAnalyzeChapters, cancelBatch,
        deleteChapter, clearChapterAnalysis, analyzingChapterIds, loading, fetchDimensions,
        progress, batchProgress, streamContent, batchStartTime
    } = useNovelStore();

    const hasAnyAnalysis = chapters.some(c => c.has_analysis);

    const [showDimSelector, setShowDimSelector] = useState(false);
    const [viewMode, setViewMode] = useState<'chapter' | 'full-book'>('chapter');
    const [chapterToDelete, setChapterToDelete] = useState<{ id: number; title: string } | null>(null);
    const [clearTarget, setClearTarget] = useState<{ id: number; title: string } | null>(null);
    const [isCancelling, setIsCancelling] = useState(false);
    const [confirmBatchDelete, setConfirmBatchDelete] = useState(false);

    // Multi-select state
    const [multiSelectMode, setMultiSelectMode] = useState(false);
    const [multiSelectIds, setMultiSelectIds] = useState<Set<number>>(new Set());

    useEffect(() => {
        if (novelId && (!currentNovel || currentNovel.id !== novelId)) {
            selectNovel(novelId);
        }
        fetchDimensions();
    }, [novelId]);

    if (!currentNovel) {
        return <div className="flex-1 flex items-center justify-center"><span className="loading loading-spinner loading-lg" /></div>;
    }

    useEffect(() => {
        if (!batchProgress || batchProgress.status === 'batch_done' || batchProgress.status === 'batch_cancelled') {
            setIsCancelling(false);
        }
    }, [batchProgress?.status]);

    const handleAnalyze = async (chapterId: number) => {
        try {
            await analyzeChapterApi(chapterId);
        } catch (e) {
            console.error('Analysis failed:', e);
        }
    };

    const toggleMultiSelect = (id: number) => {
        setMultiSelectIds(prev => {
            const next = new Set(prev);
            if (next.has(id)) {
                next.delete(id);
            } else {
                next.add(id);
            }
            return next;
        });
    };

    const exitMultiSelect = () => {
        setMultiSelectMode(false);
        setMultiSelectIds(new Set());
    };

    const handleBatchSelected = async () => {
        if (multiSelectIds.size === 0) return;
        try {
            await batchAnalyzeChapters(currentNovel.id, Array.from(multiSelectIds));
            exitMultiSelect();
        } catch (e) {
            console.error('Batch analysis failed:', e);
        }
    };

    const isChapterBusy = (id: number) =>
        analyzingChapterIds.has(id) || (batchProgress?.status === 'batch_analyzing' && batchProgress?.chapter_id === id);

    return (
        <div className="flex-1 flex overflow-hidden">
            {/* Chapter sidebar */}
            <aside className="w-72 bg-base-200 border-r border-base-300 flex flex-col overflow-hidden">
                <div className="p-4 border-b border-base-300">
                    <h2 className="font-bold text-lg line-clamp-1">{currentNovel.title}</h2>
                    <div className="flex items-center gap-2 mt-2">
                        <div className="badge badge-sm badge-outline">
                            {chapters.filter(c => c.has_analysis).length}/{chapters.length} 已分析
                        </div>
                    </div>

                    <div className="flex gap-2 mt-4 tabs tabs-boxed bg-base-300/50 p-1">
                        <button
                            className={`tab tab-sm flex-1 ${viewMode === 'chapter' ? 'tab-active' : ''}`}
                            onClick={() => setViewMode('chapter')}
                        >
                            章节列表
                        </button>
                        <button
                            className={`tab tab-sm flex-1 ${viewMode === 'full-book' ? 'tab-active' : ''}`}
                            onClick={() => setViewMode('full-book')}
                        >
                            全书概览
                        </button>
                    </div>

                    {/* Mode toggle + Dimension selector + Multi-select */}
                    <div className="flex gap-2 mt-3">
                        <div className="join flex-1">
                            <button
                                className={`join-item btn btn-xs flex-1 ${analysisMode === 'api' ? 'btn-primary' : 'btn-ghost'}`}
                                onClick={() => setAnalysisMode('api')}
                            >
                                <Zap size={12} /> API
                            </button>
                            <button
                                className={`join-item btn btn-xs flex-1 ${analysisMode === 'manual' ? 'btn-primary' : 'btn-ghost'}`}
                                onClick={() => setAnalysisMode('manual')}
                            >
                                <ClipboardCopy size={12} /> 手动
                            </button>
                        </div>
                        <button
                            className={`btn btn-xs btn-square ${multiSelectMode ? 'btn-accent' : 'btn-ghost'}`}
                            onClick={() => multiSelectMode ? exitMultiSelect() : setMultiSelectMode(true)}
                            title={multiSelectMode ? '退出多选' : '多选模式'}
                        >
                            {multiSelectMode ? <X size={14} /> : <ListChecks size={14} />}
                        </button>
                        <button
                            className="btn btn-ghost btn-xs btn-square"
                            onClick={() => setShowDimSelector(!showDimSelector)}
                            title="维度设置"
                        >
                            <Settings2 size={14} />
                        </button>
                    </div>
                </div>

                {/* Dimension selector dropdown */}
                {showDimSelector && (
                    <div className="border-b border-base-300 p-3 bg-base-300/30">
                        {hasAnyAnalysis && (
                            <p className="text-xs text-warning mb-2">⚠ 已有分析结果，修改维度不会影响已完成的分析</p>
                        )}
                        <DimensionSelector />
                    </div>
                )}

                {/* Multi-select action bar */}
                {multiSelectMode && (
                    <div className="p-3 border-b border-base-300 bg-accent/10 space-y-2">
                        <div className="flex justify-between items-center text-xs">
                            <span className="font-medium">已选 {multiSelectIds.size} 章</span>
                            <div className="flex gap-1">
                                <button
                                    className="btn btn-ghost btn-xs"
                                    onClick={() => setMultiSelectIds(new Set(chapters.map(c => c.id)))}
                                >
                                    全选
                                </button>
                                <button
                                    className="btn btn-ghost btn-xs"
                                    onClick={() => setMultiSelectIds(new Set(chapters.filter(c => !c.has_analysis).map(c => c.id)))}
                                >
                                    选未分析
                                </button>
                                <button
                                    className="btn btn-ghost btn-xs"
                                    onClick={() => setMultiSelectIds(new Set())}
                                >
                                    清空
                                </button>
                            </div>
                        </div>
                        <div className="flex gap-2">
                            {analysisMode === 'api' && (
                                <button
                                    className="btn btn-primary btn-sm flex-1 gap-1"
                                    onClick={handleBatchSelected}
                                    disabled={multiSelectIds.size === 0 || loading || !!batchProgress}
                                >
                                    <Play size={14} /> 分析 ({multiSelectIds.size})
                                </button>
                            )}
                            <button
                                className="btn btn-error btn-outline btn-sm flex-none gap-1"
                                onClick={() => setConfirmBatchDelete(true)}
                                disabled={multiSelectIds.size === 0 || loading || !!batchProgress}
                            >
                                <Trash2 size={14} /> 删除 ({multiSelectIds.size})
                            </button>
                        </div>
                    </div>
                )}

                {/* Batch Analysis Progress */}
                {viewMode === 'chapter' && batchProgress && (
                    <div className="p-3 border-b border-base-300 bg-base-200/50">
                        <div className="flex flex-col gap-2">
                            <div className="flex justify-between text-xs font-medium">
                                <span className="line-clamp-1">{batchProgress.message}</span>
                                <span className="flex-shrink-0 ml-2">{batchProgress.current}/{batchProgress.total}</span>
                            </div>
                            <progress
                                className="progress progress-primary w-full"
                                value={batchProgress.current}
                                max={batchProgress.total}
                            ></progress>

                            {batchStartTime && batchProgress.status === 'batch_analyzing' && batchProgress.current >= 0 && (
                                <BatchTimeStats
                                    startTime={batchStartTime}
                                    current={batchProgress.current}
                                    total={batchProgress.total}
                                />
                            )}

                            {batchProgress.status === 'batch_analyzing' && (
                                <button
                                    className={`btn btn-xs w-full ${isCancelling ? 'btn-disabled' : 'btn-ghost text-error'}`}
                                    onClick={() => {
                                        setIsCancelling(true);
                                        cancelBatch();
                                    }}
                                    disabled={isCancelling}
                                >
                                    {isCancelling ? '正在停止(等待当前完成)...' : '⬛ 停止'}
                                </button>
                            )}
                        </div>
                    </div>
                )}

                {/* Batch all unanalyzed (only when not in multi-select) */}
                {viewMode === 'chapter' && !multiSelectMode && !batchProgress && (
                    <div className="p-3 border-b border-base-300 bg-base-200/50">
                        <button
                            className={`btn btn-primary btn-sm w-full ${loading ? 'btn-disabled' : ''}`}
                            onClick={() => batchAnalyzeNovel(currentNovel.id)}
                            disabled={loading || chapters.every(c => c.has_analysis)}
                        >
                            <Play size={14} /> 批量分析未分析章节
                        </button>
                    </div>
                )}

                {/* Chapter list */}
                <div className="flex-1 overflow-y-auto">
                    {chapters.map((ch) => (
                        <button
                            key={ch.id}
                            className={`w-full text-left px-4 py-3 border-b border-base-300/50 flex items-center gap-3 hover:bg-base-300/50 transition-colors group ${!multiSelectMode && selectedChapter?.id === ch.id && viewMode === 'chapter' ? 'bg-base-300' : ''
                                } ${multiSelectMode && multiSelectIds.has(ch.id) ? 'bg-accent/10' : ''}`}
                            onClick={() => {
                                setViewMode('chapter');
                                if (multiSelectMode) {
                                    toggleMultiSelect(ch.id);
                                } else {
                                    selectChapter(ch.id);
                                }
                            }}
                        >
                            <div className="flex-shrink-0">
                                {multiSelectMode ? (
                                    <input
                                        type="checkbox"
                                        className="checkbox checkbox-xs checkbox-accent"
                                        checked={multiSelectIds.has(ch.id)}
                                        onChange={() => toggleMultiSelect(ch.id)}
                                        onClick={(e) => e.stopPropagation()}
                                    />
                                ) : isChapterBusy(ch.id) ? (
                                    <span className="loading loading-spinner loading-xs text-primary" />
                                ) : ch.has_analysis ? (
                                    <CheckCircle size={16} className="text-success" />
                                ) : (
                                    <Circle size={16} className="text-base-content/30" />
                                )}
                            </div>
                            <div className="flex-1 min-w-0">
                                <p className="text-sm font-medium line-clamp-1">{ch.title || `第 ${ch.index + 1} 章`}</p>
                                <p className="text-xs text-base-content/40">~{ch.token_estimate.toLocaleString()} tokens</p>
                            </div>
                            {!multiSelectMode && (
                                <>
                                    <button
                                        className="btn btn-ghost btn-xs text-error opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0"
                                        onClick={(e) => {
                                            e.stopPropagation();
                                            setChapterToDelete({ id: ch.id, title: ch.title });
                                        }}
                                        title="删除章节"
                                    >
                                        <Trash2 size={13} />
                                    </button>
                                    <ChevronRight size={14} className="text-base-content/30 flex-shrink-0" />
                                </>
                            )}
                        </button>
                    ))}
                </div>
            </aside>

            {/* Main content */}
            <main className="flex-1 bg-base-100 flex flex-col relative overflow-hidden shrink min-w-0">
                {viewMode === 'full-book' ? (
                    <FullBookSummaryView novelId={currentNovel.id} />
                ) : multiSelectMode && multiSelectIds.size > 0 && analysisMode === 'manual' ? (
                    <div className="flex-1 overflow-y-auto p-6 space-y-6">
                        <h3 className="text-lg font-bold">手动分析 - 已选 {multiSelectIds.size} 章</h3>
                        {chapters
                            .filter(ch => multiSelectIds.has(ch.id))
                            .map(ch => (
                                <div key={ch.id} className="border border-base-300 rounded-xl overflow-hidden">
                                    <ManualPromptPanel
                                        chapterId={ch.id}
                                        chapterTitle={ch.title}
                                    />
                                </div>
                            ))
                        }
                    </div>
                ) : !selectedChapter ? (
                    <div className="flex items-center justify-center h-full text-base-content/40">
                        <p>← 选择一个章节查看分析</p>
                    </div>
                ) : selectedChapter.analysis ? (
                    <div className="flex-1 overflow-y-auto p-6 space-y-4">
                        <div className="flex justify-end">
                            <button
                                className="btn btn-ghost btn-xs text-error gap-1"
                                onClick={() => {
                                    setChapterToDelete(null);
                                    setClearTarget({ id: selectedChapter.id!, title: selectedChapter.title });
                                }}
                            >
                                <Trash2 size={12} />
                                清除分析
                            </button>
                        </div>
                        <ChapterAnalysisView
                            analysis={selectedChapter.analysis}
                            dimensions={currentNovel.enabled_dimensions}
                            chapterTitle={selectedChapter.title}
                        />
                    </div>
                ) : isChapterBusy(selectedChapter.id!) ? (
                    <div className="flex flex-col w-full h-full p-4 gap-4">
                        <div className="flex justify-between items-center bg-base-200 rounded-lg p-3 border border-base-300">
                            <div className="flex items-center gap-3">
                                <span className="loading loading-spinner loading-sm text-primary" />
                                <span className="text-sm font-medium">
                                    {progress && progress.chapter_id === selectedChapter.id
                                        ? progress.message
                                        : "分析中..."}
                                </span>
                            </div>
                            <span className="text-xs text-base-content/50">正在流式接收</span>
                        </div>
                        <StreamingJsonViewer content={streamContent[selectedChapter.id!] || ''} />
                    </div>
                ) : analysisMode === 'api' ? (
                    <div className="flex flex-col items-center justify-center h-full gap-4">
                        <p className="text-base-content/60">「{selectedChapter.title}」尚未分析</p>

                        <button
                            className={`btn btn-primary gap-2`}
                            onClick={() => handleAnalyze(selectedChapter.id!)}
                            disabled={!!batchProgress}
                        >
                            <Play size={16} />
                            开始 API 分析
                        </button>
                    </div>
                ) : (
                    <div className="flex-1 overflow-y-auto p-6">
                        <ManualPromptPanel
                            chapterId={selectedChapter.id!}
                            chapterTitle={selectedChapter.title}
                        />
                    </div>
                )}
            </main>

            {chapterToDelete && (
                <ConfirmDialog
                    title="删除章节"
                    message={`确定要删除「${chapterToDelete.title}」吗？删除后无法恢复。`}
                    confirmText="删除"
                    kind="error"
                    onConfirm={() => {
                        deleteChapter(chapterToDelete.id, currentNovel.id);
                        setChapterToDelete(null);
                    }}
                    onCancel={() => setChapterToDelete(null)}
                />
            )}

            {clearTarget && (
                <ConfirmDialog
                    title="清除分析"
                    message={`确定要清除「${clearTarget.title}」的分析结果吗？可以重新分析。`}
                    confirmText="清除"
                    kind="warning"
                    onConfirm={() => {
                        clearChapterAnalysis(clearTarget.id, currentNovel.id);
                        setClearTarget(null);
                    }}
                    onCancel={() => setClearTarget(null)}
                />
            )}

            {confirmBatchDelete && (
                <ConfirmDialog
                    title="批量删除章节"
                    message={`确定要删除选定的 ${multiSelectIds.size} 个章节吗？删除后不可恢复。`}
                    confirmText="删除"
                    kind="error"
                    onConfirm={() => {
                        useNovelStore.getState().deleteChapters(Array.from(multiSelectIds), currentNovel.id);
                        setConfirmBatchDelete(false);
                        exitMultiSelect();
                    }}
                    onCancel={() => setConfirmBatchDelete(false)}
                />
            )}
        </div>
    );
}
