import { useEffect, useState, useRef } from 'react';

export default function StreamingJsonViewer({ content }: { content: string }) {
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
