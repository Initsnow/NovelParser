import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { useNovelStore } from '../store/index';
import ChapterAnalysisView from '../components/ChapterAnalysisView';
import ManualPromptPanel from '../components/ManualPromptPanel';
import DimensionSelector from '../components/DimensionSelector';
import { BatchTimeStats } from '../components/BatchTimeStats';
import StreamingJsonViewer from '../components/StreamingJsonViewer';
import FullBookSummaryView from '../components/FullBookSummaryView';
import BookOutlineView from '../components/BookOutlineView';
import ChapterOutlineView from '../components/ChapterOutlineView';
import VirtualList from '../components/VirtualList';
import { Play, CheckCircle, Circle, ChevronRight, Zap, ClipboardCopy, Settings2, Trash2, ListChecks, X, Download } from 'lucide-react';
import ConfirmDialog from '../components/ConfirmDialog';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';



export default function NovelPage() {
    const { novelId } = useParams<{ novelId: string }>();
    const [exportAlert, setExportAlert] = useState<{ title: string, msg: string, kind: 'info' | 'error' } | null>(null);
    const [selectedChapterInputTokens, setSelectedChapterInputTokens] = useState<number | null>(null);
    const {
        currentNovel, chapters, selectedChapter,
        selectNovel, selectChapter, analysisMode, setAnalysisMode,
        analyzeChapterApi, batchAnalyzeNovel, batchAnalyzeChapters, cancelBatch,
        generateChapterOutlineApi, batchGenerateOutlines, batchGenerateOutlineChapters, generateBookOutline, bookOutline,
        fetchBookOutline, clearBookOutline, outliningChapterIds, outlineProgress, outlineBatchProgress, outlineBatchStartTime,
        deleteChapter, clearChapterAnalysis, clearChapterOutline, analyzingChapterIds, loading, fetchDimensions, hydrateChapterTokenEstimates,
        progress, batchProgress, streamContent, outlineStreamContent, batchStartTime
    } = useNovelStore();

    const hasAnyAnalysis = chapters.some(c => c.has_analysis);
    const hasAnyOutline = chapters.some(c => c.has_outline);

    const [showDimSelector, setShowDimSelector] = useState(false);
    const [viewMode, setViewMode] = useState<'chapter' | 'full-book'>('chapter');
    const [chapterToDelete, setChapterToDelete] = useState<{ id: number; title: string } | null>(null);
    const [clearTarget, setClearTarget] = useState<{ id: number; title: string; type: 'analysis' | 'outline' } | null>(null);
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
        fetchBookOutline();
    }, [novelId]);

    const activeBatchProgress = analysisMode === 'outline' ? outlineBatchProgress : batchProgress;
    const activeBatchStartTime = analysisMode === 'outline' ? outlineBatchStartTime : batchStartTime;
    const activeSingleProgress = analysisMode === 'outline' ? outlineProgress : progress;
    const completedCount = analysisMode === 'outline'
        ? chapters.filter(c => c.has_outline).length
        : chapters.filter(c => c.has_analysis).length;
    const selectedChapterTokenEstimate = selectedChapter?.id != null
        ? chapters.find(c => c.id === selectedChapter.id)?.token_estimate ?? 0
        : 0;
    const pendingBatchTokenEstimate = chapters
        .filter(c => analysisMode === 'outline' ? !c.has_outline : !c.has_analysis)
        .reduce((sum, c) => sum + c.token_estimate, 0);
    const selectedBatchTokenEstimate = chapters
        .filter(c => multiSelectIds.has(c.id))
        .reduce((sum, c) => sum + c.token_estimate, 0);

    useEffect(() => {
        if (!activeBatchProgress || activeBatchProgress.status === 'batch_done' || activeBatchProgress.status === 'batch_cancelled') {
            setIsCancelling(false);
        }
    }, [activeBatchProgress?.status]);

    useEffect(() => {
        let cancelled = false;

        const loadExactTokens = async () => {
            if (!selectedChapter?.id || analysisMode === 'manual') {
                setSelectedChapterInputTokens(null);
                return;
            }

            try {
                const count = analysisMode === 'outline'
                    ? await invoke<number>('estimate_outline_prompt_tokens', { chapterId: selectedChapter.id })
                    : await invoke<number>('estimate_prompt_tokens', {
                        chapterId: selectedChapter.id,
                        dimensions: currentNovel?.enabled_dimensions || [],
                    });
                if (!cancelled) {
                    setSelectedChapterInputTokens(count);
                }
            } catch {
                if (!cancelled) {
                    setSelectedChapterInputTokens(null);
                }
            }
        };

        loadExactTokens();
        return () => {
            cancelled = true;
        };
    }, [selectedChapter?.id, analysisMode, currentNovel?.enabled_dimensions]);

    if (!currentNovel) {
        return <div className="flex-1 flex items-center justify-center"><span className="loading loading-spinner loading-lg" /></div>;
    }

    const handleAnalyze = async (chapterId: number) => {
        try {
            if (analysisMode === 'outline') {
                await generateChapterOutlineApi(chapterId);
            } else {
                await analyzeChapterApi(chapterId);
            }
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
            if (analysisMode === 'outline') {
                await batchGenerateOutlineChapters(currentNovel.id, Array.from(multiSelectIds));
            } else {
                await batchAnalyzeChapters(currentNovel.id, Array.from(multiSelectIds));
            }
            exitMultiSelect();
        } catch (e) {
            console.error('Batch analysis failed:', e);
        }
    };

    const handleExport = async () => {
        try {
            const dirPath = await open({
                directory: true,
                multiple: false,
                title: analysisMode === 'outline' ? '选择大纲导出目录' : '选择导出目录'
            });
            if (dirPath) {
                if (analysisMode === 'outline') {
                    await invoke('export_book_outline', { novelId: currentNovel.id, dirPath: dirPath as string });
                    setExportAlert({ title: '导出成功', msg: `已成功将大纲文件导出至:\n${dirPath}`, kind: 'info' });
                } else {
                    await invoke('export_novel_report', { novelId: currentNovel.id, dirPath: dirPath as string });
                    setExportAlert({ title: '导出成功', msg: `已成功将分析报告导出至:\n${dirPath}`, kind: 'info' });
                }
            }
        } catch (e) {
            console.error('Export failed:', e);
            setExportAlert({ title: '错误', msg: `导出失败: ${e}`, kind: 'error' });
        }
    };

    const isChapterBusy = (id: number) =>
        analysisMode === 'outline'
            ? outliningChapterIds.has(id) || (outlineBatchProgress?.status === 'batch_outlining' && outlineBatchProgress?.chapter_id === id)
            : analyzingChapterIds.has(id) || (batchProgress?.status === 'batch_analyzing' && batchProgress?.chapter_id === id);

    return (
        <div className="flex-1 flex overflow-hidden">
            {/* Chapter sidebar */}
            <aside className="w-72 bg-base-200 border-r border-base-300 flex flex-col overflow-hidden">
                <div className="p-4 border-b border-base-300">
                    <h2 className="font-bold text-lg line-clamp-1">{currentNovel.title}</h2>
                    <div className="flex items-center gap-2 mt-2">
                        <div className="badge badge-sm badge-outline">
                            {completedCount}/{chapters.length} {analysisMode === 'outline' ? '已提纲' : '已分析'}
                        </div>
                    </div>

                    <div className="flex gap-2 mt-4 tabs tabs-boxed bg-base-300/50 p-1 rounded-full">
                        <button
                            className={`tab tab-sm flex-1 rounded-full ${viewMode === 'chapter' ? 'tab-active' : ''}`}
                            onClick={() => setViewMode('chapter')}
                        >
                            章节列表
                        </button>
                        <button
                            className={`tab tab-sm flex-1 rounded-full ${viewMode === 'full-book' ? 'tab-active' : ''}`}
                            onClick={() => setViewMode('full-book')}
                        >
                            全书概览
                        </button>
                    </div>

                    {/* Mode toggle + Dimension selector + Multi-select */}
                    <div className="mt-3 space-y-2">
                        <div className="join w-full">
                            <button
                                className={`join-item btn btn-xs flex-1 whitespace-nowrap ${analysisMode === 'api' ? 'btn-primary' : 'btn-ghost'}`}
                                onClick={() => setAnalysisMode('api')}
                            >
                                <Zap size={12} /> API
                            </button>
                            <button
                                className={`join-item btn btn-xs flex-1 whitespace-nowrap ${analysisMode === 'manual' ? 'btn-primary' : 'btn-ghost'}`}
                                onClick={() => setAnalysisMode('manual')}
                            >
                                <ClipboardCopy size={12} /> 手动
                            </button>
                            <button
                                className={`join-item btn btn-xs flex-1 whitespace-nowrap ${analysisMode === 'outline' ? 'btn-primary' : 'btn-ghost'}`}
                                onClick={() => setAnalysisMode('outline')}
                            >
                                提纲
                            </button>
                        </div>
                        <div className="flex gap-2">
                            <button
                                className={`btn btn-xs btn-square ${multiSelectMode ? 'btn-accent' : 'btn-ghost'}`}
                                onClick={() => multiSelectMode ? exitMultiSelect() : setMultiSelectMode(true)}
                                title={multiSelectMode ? '退出多选' : '多选模式'}
                            >
                                {multiSelectMode ? <X size={14} /> : <ListChecks size={14} />}
                            </button>
                            <button
                                className="btn btn-ghost btn-xs btn-square"
                                onClick={handleExport}
                                title={analysisMode === 'outline' ? '导出大纲文件' : '导出分析报告'}
                                disabled={analysisMode === 'outline' && !bookOutline}
                            >
                                <Download size={14} />
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
                </div>

                {/* Dimension selector dropdown */}
                {showDimSelector && (
                    <div className="border-b border-base-300 p-3 bg-base-300/30">
                        {hasAnyAnalysis && analysisMode !== 'outline' && (
                            <p className="text-xs text-warning mb-2">⚠ 已有分析结果，修改维度不会影响已完成的分析</p>
                        )}
                        {analysisMode === 'outline' ? (
                            <p className="text-xs text-base-content/60">快速提纲模式不依赖分析维度，只提取可归并的章节摘要与正文提纲。</p>
                        ) : (
                            <DimensionSelector />
                        )}
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
                                    onClick={() => setMultiSelectIds(new Set(
                                        chapters
                                            .filter(c => analysisMode === 'outline' ? !c.has_outline : !c.has_analysis)
                                            .map(c => c.id)
                                    ))}
                                >
                                    {analysisMode === 'outline' ? '选未提纲' : '选未分析'}
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
                            {analysisMode !== 'manual' && (
                                <div className="flex-1 space-y-1">
                                    <p className="text-[11px] text-base-content/50">
                                        粗略预估输入：{selectedBatchTokenEstimate.toLocaleString()} tokens
                                    </p>
                                    <button
                                        className="btn btn-primary btn-sm w-full gap-1"
                                        onClick={handleBatchSelected}
                                        disabled={multiSelectIds.size === 0 || loading || !!activeBatchProgress}
                                    >
                                        <Play size={14} /> {analysisMode === 'outline' ? '批量提纲' : '分析'} ({multiSelectIds.size})
                                    </button>
                                </div>
                            )}
                            <button
                                className="btn btn-error btn-outline btn-sm flex-none gap-1"
                                onClick={() => setConfirmBatchDelete(true)}
                                disabled={multiSelectIds.size === 0 || loading || !!activeBatchProgress}
                            >
                                <Trash2 size={14} /> 删除 ({multiSelectIds.size})
                            </button>
                        </div>
                    </div>
                )}

                {/* Batch Analysis Progress */}
                {viewMode === 'chapter' && activeBatchProgress && (
                    <div className="p-3 border-b border-base-300 bg-base-200/50">
                        <div className="flex flex-col gap-2">
                            <div className="flex justify-between text-xs font-medium">
                                <span className="line-clamp-1">{activeBatchProgress.message}</span>
                                <span className="flex-shrink-0 ml-2">{activeBatchProgress.current}/{activeBatchProgress.total}</span>
                            </div>
                            <progress
                                className="progress progress-primary w-full"
                                value={activeBatchProgress.current}
                                max={activeBatchProgress.total}
                            ></progress>

                            {activeBatchStartTime && (activeBatchProgress.status === 'batch_analyzing' || activeBatchProgress.status === 'batch_outlining') && activeBatchProgress.current >= 0 && (
                                <BatchTimeStats
                                    startTime={activeBatchStartTime}
                                    current={activeBatchProgress.current}
                                    total={activeBatchProgress.total}
                                />
                            )}

                            {(activeBatchProgress.status === 'batch_analyzing' || activeBatchProgress.status === 'batch_outlining') && (
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
                {viewMode === 'chapter' && !multiSelectMode && !activeBatchProgress && (
                    <div className="p-3 border-b border-base-300 bg-base-200/50">
                        <div className="space-y-1">
                            <p className="text-[11px] text-base-content/50">
                                粗略预估输入：{pendingBatchTokenEstimate.toLocaleString()} tokens
                            </p>
                            <button
                                className={`btn btn-primary btn-sm w-full ${loading ? 'btn-disabled' : ''}`}
                                onClick={() => analysisMode === 'outline' ? batchGenerateOutlines(currentNovel.id) : batchAnalyzeNovel(currentNovel.id)}
                                disabled={loading || chapters.every(c => analysisMode === 'outline' ? c.has_outline : c.has_analysis)}
                            >
                                <Play size={14} /> {analysisMode === 'outline' ? '批量提取未提纲章节' : '批量分析未分析章节'}
                            </button>
                        </div>
                    </div>
                )}

                {/* Chapter list */}
                <VirtualList
                    className="flex-1 overflow-y-auto"
                    items={chapters}
                    itemHeight={84}
                    onVisibleItemsChange={(visibleItems) => {
                        void hydrateChapterTokenEstimates(visibleItems.map((item) => item.id));
                    }}
                    renderItem={(ch) => (
                        <button
                            key={ch.id}
                            className={`w-full h-[84px] text-left px-4 py-3 border-b border-base-300/50 flex items-center gap-3 hover:bg-base-300/50 transition-colors group ${!multiSelectMode && selectedChapter?.id === ch.id && viewMode === 'chapter' ? 'bg-base-300' : ''
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
                                ) : (analysisMode === 'outline' ? ch.has_outline : ch.has_analysis) ? (
                                    <CheckCircle size={16} className="text-success" />
                                ) : (
                                    <Circle size={16} className="text-base-content/30" />
                                )}
                            </div>
                            <div className="flex-1 min-w-0">
                                <div
                                    className="truncate font-medium text-sm"
                                    title={ch.title || `第 ${ch.index + 1} 章`}
                                >
                                    {ch.title || `第 ${ch.index + 1} 章`}
                                </div>
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
                    )}
                />
            </aside>

            {/* Main content */}
            <main className="flex-1 bg-base-100 flex flex-col relative overflow-hidden shrink min-w-0">
                {viewMode === 'full-book' ? (
                    analysisMode === 'outline' ? (
                        <div className="flex-1 overflow-y-auto p-6 space-y-4">
                            <div className="flex flex-wrap items-center gap-3">
                                <button
                                    className="btn btn-primary btn-sm"
                                    onClick={() => generateBookOutline(currentNovel.id)}
                                    disabled={loading || !hasAnyOutline}
                                >
                                    <Play size={14} /> 生成全书提纲
                                </button>
                                {bookOutline && (
                                    <button
                                        className="btn btn-error btn-outline btn-sm"
                                        onClick={() => clearBookOutline(currentNovel.id)}
                                    >
                                        清除提纲
                                    </button>
                                )}
                                <div className="text-xs text-base-content/60">
                                    当前有 {chapters.filter(c => c.has_outline).length} 个章节已提纲
                                </div>
                            </div>

                            {outlineProgress && outlineProgress.status === 'summarizing' && (
                                <div className="card bg-base-200 border border-base-300">
                                    <div className="card-body p-4">
                                        <div className="flex justify-between text-sm">
                                            <span>{outlineProgress.message}</span>
                                            <span>{outlineProgress.current}/{outlineProgress.total}</span>
                                        </div>
                                        <progress
                                            className="progress progress-primary w-full"
                                            value={outlineProgress.current}
                                            max={outlineProgress.total}
                                        ></progress>
                                    </div>
                                </div>
                            )}

                            {bookOutline ? (
                                <BookOutlineView outline={bookOutline} />
                            ) : (
                                <div className="flex items-center justify-center h-full text-base-content/40">
                                    <p>先生成章节提纲，再归并为全书提纲</p>
                                </div>
                            )}
                        </div>
                    ) : (
                        <FullBookSummaryView novelId={currentNovel.id} />
                    )
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
                                        onSuccess={() => toggleMultiSelect(ch.id)}
                                    />
                                </div>
                            ))
                        }
                    </div>
                ) : !selectedChapter ? (
                    <div className="flex items-center justify-center h-full text-base-content/40">
                        <p>← 选择一个章节查看{analysisMode === 'outline' ? '提纲' : '分析'}</p>
                    </div>
                ) : analysisMode === 'outline' && isChapterBusy(selectedChapter.id!) ? (
                    <div className="flex flex-col w-full h-full p-4 gap-4">
                        <div className="flex justify-between items-center bg-base-200 rounded-lg p-3 border border-base-300">
                            <div className="flex items-center gap-3">
                                <span className="loading loading-spinner loading-sm text-primary" />
                                <span className="text-sm font-medium">
                                    {activeSingleProgress && activeSingleProgress.chapter_id === selectedChapter.id
                                        ? activeSingleProgress.message
                                        : "提纲生成中..."}
                                </span>
                            </div>
                            <span className="text-xs text-base-content/50">正在提取文本提纲</span>
                        </div>
                        <StreamingJsonViewer content={outlineStreamContent[selectedChapter.id!] || ''} />
                    </div>
                ) : analysisMode === 'outline' && selectedChapter.outline ? (
                    <div className="flex-1 overflow-y-auto p-6">
                        <div className="flex justify-end mb-4">
                            <button
                                className="btn btn-ghost btn-xs text-error gap-1"
                                onClick={() => setClearTarget({ id: selectedChapter.id!, title: selectedChapter.title, type: 'outline' })}
                            >
                                <Trash2 size={12} />
                                清除提纲
                            </button>
                        </div>
                        <ChapterOutlineView outline={selectedChapter.outline} chapterTitle={selectedChapter.title} />
                    </div>
                ) : analysisMode === 'outline' ? (
                    <div className="flex flex-col items-center justify-center h-full gap-4">
                        <p className="text-base-content/60">「{selectedChapter.title}」尚未生成提纲</p>
                        <p className="text-xs text-base-content/50">
                            预估输入：{(selectedChapterInputTokens ?? selectedChapterTokenEstimate).toLocaleString()} tokens
                        </p>

                        <button
                            className="btn btn-primary gap-2"
                            onClick={() => handleAnalyze(selectedChapter.id!)}
                            disabled={!!outlineBatchProgress}
                        >
                            <Play size={16} />
                            生成快速提纲
                        </button>
                    </div>
                ) : selectedChapter.analysis ? (
                    <div className="flex-1 overflow-y-auto p-6 space-y-4">
                        <div className="flex justify-end">
                            <button
                                className="btn btn-ghost btn-xs text-error gap-1"
                                onClick={() => {
                                    setChapterToDelete(null);
                                    setClearTarget({ id: selectedChapter.id!, title: selectedChapter.title, type: 'analysis' });
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
                                    {activeSingleProgress && activeSingleProgress.chapter_id === selectedChapter.id
                                        ? activeSingleProgress.message
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
                        <p className="text-xs text-base-content/50">
                            预估输入：{(selectedChapterInputTokens ?? selectedChapterTokenEstimate).toLocaleString()} tokens
                        </p>

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
                    title={clearTarget.type === 'outline' ? '清除提纲' : '清除分析'}
                    message={
                        clearTarget.type === 'outline'
                            ? `确定要清除「${clearTarget.title}」的提纲结果吗？可以重新生成。`
                            : `确定要清除「${clearTarget.title}」的分析结果吗？可以重新分析。`
                    }
                    confirmText="清除"
                    kind="warning"
                    onConfirm={() => {
                        if (clearTarget.type === 'outline') {
                            clearChapterOutline(clearTarget.id, currentNovel.id);
                        } else {
                            clearChapterAnalysis(clearTarget.id, currentNovel.id);
                        }
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

            {exportAlert && (
                <ConfirmDialog
                    title={exportAlert.title}
                    message={exportAlert.msg}
                    kind={exportAlert.kind}
                    hideCancel={true}
                    onConfirm={() => setExportAlert(null)}
                    onCancel={() => setExportAlert(null)}
                />
            )}
        </div>
    );
}
