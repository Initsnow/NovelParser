import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useNovelStore } from '../store/novelStore';
import { open } from '@tauri-apps/plugin-dialog';
import { Plus, BookOpen, Trash2, FileText } from 'lucide-react';
import EpubPreviewModal from '../components/EpubPreviewModal';
import ConfirmDialog from '../components/ConfirmDialog';
import type { EpubPreview } from '../types';

export default function HomePage() {
    const { novels, fetchNovels, previewEpub, importEpubSelected, importSingleTxt, importTxtFiles, deleteNovel, selectNovel } = useNovelStore();
    const navigate = useNavigate();
    const [epubPreview, setEpubPreview] = useState<EpubPreview | null>(null);
    const [novelToDelete, setNovelToDelete] = useState<{ id: string; title: string } | null>(null);

    useEffect(() => { fetchNovels(); }, [fetchNovels]);

    const handleImport = async () => {
        const result = await open({
            multiple: true,
            filters: [
                { name: '小说文件', extensions: ['epub', 'txt'] },
            ],
        });

        if (!result || result.length === 0) return;

        try {
            const paths = result as string[];

            if (paths.length === 1 && paths[0].endsWith('.epub')) {
                // EPUB: show preview modal for user selection
                const preview = await previewEpub(paths[0]);
                setEpubPreview(preview);
                return;
            }

            let novelId: string;
            if (paths.length === 1) {
                novelId = await importSingleTxt(paths[0]);
            } else {
                const txtPaths = paths.filter(p => p.endsWith('.txt'));
                if (txtPaths.length === 0) {
                    throw new Error('请选择 TXT 文件');
                }
                novelId = await importTxtFiles(txtPaths);
            }

            await selectNovel(novelId);
            navigate(`/novel/${novelId}`);
        } catch (e) {
            console.error('Import failed:', e);
        }
    };

    const handleEpubConfirm = async (selectedIndices: number[]) => {
        if (!epubPreview) return;
        try {
            const novelId = await importEpubSelected(epubPreview.path, selectedIndices);
            setEpubPreview(null);
            await selectNovel(novelId);
            navigate(`/novel/${novelId}`);
        } catch (e) {
            console.error('EPUB import failed:', e);
        }
    };

    const handleOpen = async (id: string) => {
        await selectNovel(id);
        navigate(`/novel/${id}`);
    };

    const handleDelete = (e: React.MouseEvent, id: string, title: string) => {
        e.stopPropagation();
        setNovelToDelete({ id, title });
    };

    return (
        <>
            <div className="flex-1 p-8 overflow-y-auto">
                <div className="max-w-4xl mx-auto">
                    {/* Header */}
                    <div className="flex items-center justify-between mb-8">
                        <div>
                            <h1 className="text-3xl font-bold bg-gradient-to-r from-primary to-secondary bg-clip-text text-transparent">
                                NovelParser
                            </h1>
                            <p className="text-base-content/60 mt-1">AI 驱动的小说分析工具</p>
                        </div>
                        <button className="btn btn-primary gap-2" onClick={handleImport}>
                            <Plus size={18} />
                            导入小说
                        </button>
                    </div>

                    {/* Novel Grid */}
                    {novels.length === 0 ? (
                        <div className="card bg-base-200 border border-base-300">
                            <div className="card-body items-center text-center py-16">
                                <FileText size={48} className="text-base-content/30 mb-4" />
                                <h2 className="card-title text-base-content/50">还没有导入任何小说</h2>
                                <p className="text-base-content/40">支持 EPUB 和 TXT 格式</p>
                                <button className="btn btn-primary btn-sm mt-4 gap-2" onClick={handleImport}>
                                    <Plus size={16} />
                                    导入第一本
                                </button>
                            </div>
                        </div>
                    ) : (
                        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                            {novels.map((novel) => (
                                <div
                                    key={novel.id}
                                    className="card bg-base-200 border border-base-300 hover:border-primary/50 cursor-pointer transition-all duration-200 hover:shadow-lg"
                                    onClick={() => handleOpen(novel.id)}
                                >
                                    <div className="card-body p-5">
                                        <div className="flex items-start justify-between">
                                            <div className="flex items-center gap-2">
                                                <BookOpen size={18} className="text-primary" />
                                                <h3 className="card-title text-base line-clamp-1">{novel.title}</h3>
                                            </div>
                                            <button
                                                className="btn btn-ghost btn-xs text-error"
                                                onClick={(e) => handleDelete(e, novel.id, novel.title)}
                                                title="删除"
                                            >
                                                <Trash2 size={14} />
                                            </button>
                                        </div>

                                        <div className="flex gap-3 mt-3">
                                            <div className="badge badge-outline badge-sm">{novel.chapter_count} 章</div>
                                            <div className={`badge badge-sm ${novel.analyzed_count === novel.chapter_count ? 'badge-success' : 'badge-warning'} badge-outline`}>
                                                {novel.analyzed_count}/{novel.chapter_count} 已分析
                                            </div>
                                        </div>

                                        {novel.chapter_count > 0 && (
                                            <progress
                                                className="progress progress-primary w-full mt-2"
                                                value={novel.analyzed_count}
                                                max={novel.chapter_count}
                                            />
                                        )}

                                        <p className="text-xs text-base-content/40 mt-2">
                                            {new Date(novel.created_at).toLocaleDateString('zh-CN')}
                                        </p>
                                    </div>
                                </div>
                            ))}
                        </div>
                    )}
                </div>
            </div>

            {epubPreview && (
                <EpubPreviewModal
                    preview={epubPreview}
                    onConfirm={handleEpubConfirm}
                    onCancel={() => setEpubPreview(null)}
                />
            )}

            {novelToDelete && (
                <ConfirmDialog
                    title="删除小说"
                    message={`确定要删除《${novelToDelete.title}》及其所有分析数据吗？删除后无法恢复。`}
                    confirmText="删除"
                    kind="error"
                    onConfirm={async () => {
                        await deleteNovel(novelToDelete.id);
                        setNovelToDelete(null);
                    }}
                    onCancel={() => setNovelToDelete(null)}
                />
            )}
        </>
    );
}
