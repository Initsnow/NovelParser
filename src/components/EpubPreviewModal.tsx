import { useState, useMemo } from 'react';
import { createPortal } from 'react-dom';
import type { EpubPreview } from '../types';
import { X, CheckSquare, Square } from 'lucide-react';
import { motion } from 'framer-motion';

interface Props {
    preview: EpubPreview;
    onConfirm: (selectedIndices: number[]) => void;
    onCancel: () => void;
}

export default function EpubPreviewModal({ preview, onConfirm, onCancel }: Props) {
    const [selected, setSelected] = useState<Set<number>>(() => {
        const initial = new Set<number>();
        preview.chapters.forEach((ch) => {
            if (ch.suggested) initial.add(ch.index);
        });
        return initial;
    });

    const toggle = (idx: number) => {
        setSelected((prev) => {
            const next = new Set(prev);
            if (next.has(idx)) next.delete(idx);
            else next.add(idx);
            return next;
        });
    };

    const selectAll = () => {
        setSelected(new Set(preview.chapters.map((ch) => ch.index)));
    };

    const deselectAll = () => {
        setSelected(new Set());
    };

    const selectedCount = selected.size;
    const totalChars = useMemo(
        () => preview.chapters.filter((ch) => selected.has(ch.index)).reduce((sum, ch) => sum + ch.char_count, 0),
        [selected, preview.chapters]
    );

    const handleConfirm = () => {
        const indices = Array.from(selected).sort((a, b) => a - b);
        onConfirm(indices);
    };

    return createPortal(
        <div className="fixed inset-0 z-[9999] flex items-center justify-center pointer-events-auto">
            <motion.div
                className="absolute inset-0 bg-base-300/40 backdrop-blur-sm"
                onClick={onCancel}
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                transition={{ duration: 0.2 }}
            />

            <motion.div
                className="relative z-10 w-full max-w-2xl bg-base-100/95 backdrop-blur shadow-2xl overflow-hidden flex flex-col max-h-[90vh] rounded-2xl border border-base-content/10 m-4"
                initial={{ opacity: 0, scale: 0.95, y: 20 }}
                animate={{ opacity: 1, scale: 1, y: 0 }}
                exit={{ opacity: 0, scale: 0.95, y: 20 }}
                transition={{ type: "spring", bounce: 0, duration: 0.4 }}
            >
                {/* Header */}
                <div className="flex items-center justify-between p-4 border-b border-base-300 shrink-0">
                    <div>
                        <h3 className="font-bold text-lg">选择要导入的章节</h3>
                        <p className="text-sm text-base-content/50">《{preview.title}》 — 共 {preview.chapters.length} 个条目</p>
                    </div>
                    <button className="btn btn-ghost btn-sm btn-square" onClick={onCancel}>
                        <X size={16} />
                    </button>
                </div>

                {/* Toolbar */}
                <div className="flex items-center gap-2 px-4 py-2 border-b border-base-300 shrink-0 bg-base-100/50">
                    <button className="btn btn-xs btn-outline" onClick={selectAll}>全选</button>
                    <button className="btn btn-xs btn-outline" onClick={deselectAll}>全不选</button>
                    <div className="flex-1" />
                    <span className="text-xs text-base-content/50">
                        已选 {selectedCount} 章 · 约 {totalChars.toLocaleString()} 字
                    </span>
                </div>

                {/* Chapter List */}
                <div className="overflow-y-auto flex-1 p-2">
                    {preview.chapters.map((ch) => (
                        <label
                            key={ch.index}
                            className={`flex items-center gap-3 px-3 py-2 rounded-lg cursor-pointer transition-colors hover:bg-base-300/50 ${selected.has(ch.index) ? '' : 'opacity-50'
                                }`}
                        >
                            <button
                                className="btn btn-ghost btn-xs btn-square p-0"
                                onClick={(e) => { e.preventDefault(); toggle(ch.index); }}
                            >
                                {selected.has(ch.index)
                                    ? <CheckSquare size={18} className="text-primary" />
                                    : <Square size={18} className="text-base-content/30" />
                                }
                            </button>
                            <div className="flex-1 min-w-0">
                                <span className="text-sm font-medium truncate block">{ch.title}</span>
                            </div>
                            <span className="text-xs text-base-content/40 shrink-0">
                                {ch.char_count.toLocaleString()} 字
                            </span>
                            {!ch.suggested && (
                                <span className="badge badge-xs badge-warning shrink-0">疑似非正文</span>
                            )}
                        </label>
                    ))}
                </div>

                {/* Footer */}
                <div className="flex justify-end gap-2 p-4 border-t border-base-300 shrink-0 bg-base-200 rounded-b-2xl">
                    <button className="btn btn-ghost btn-sm" onClick={onCancel}>取消</button>
                    <button
                        className="btn btn-primary btn-sm"
                        onClick={handleConfirm}
                        disabled={selectedCount === 0}
                    >
                        导入 {selectedCount} 个章节
                    </button>
                </div>
            </motion.div>
        </div>,
        document.body
    );
}
