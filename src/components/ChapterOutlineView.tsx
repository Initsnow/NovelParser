import type { ChapterOutline } from '../types';

interface ChapterOutlineViewProps {
    outline: ChapterOutline;
    chapterTitle: string;
}

export default function ChapterOutlineView({ outline, chapterTitle }: ChapterOutlineViewProps) {
    const detail = outline.detail?.trim() || outline.brief;

    return (
        <div className="space-y-5">
            <div>
                <h2 className="text-xl font-bold">{chapterTitle}</h2>
                <p className="mt-2 text-sm leading-relaxed whitespace-pre-wrap">{outline.brief}</p>
            </div>

            <div className="card bg-base-200 border border-base-300">
                <div className="card-body p-5">
                    <h3 className="font-semibold mb-3">章节提纲</h3>
                    <div className="text-sm leading-7 whitespace-pre-wrap text-base-content/85">
                        {detail}
                    </div>
                </div>
            </div>
        </div>
    );
}
