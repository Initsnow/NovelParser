import { useEffect, useMemo, useRef, useState } from 'react';

interface VirtualListProps<T> {
    items: T[];
    itemHeight: number;
    overscan?: number;
    className?: string;
    renderItem: (item: T, index: number) => React.ReactNode;
    onVisibleItemsChange?: (items: T[], startIndex: number) => void;
}

export default function VirtualList<T>({
    items,
    itemHeight,
    overscan = 6,
    className,
    renderItem,
    onVisibleItemsChange,
}: VirtualListProps<T>) {
    const containerRef = useRef<HTMLDivElement | null>(null);
    const [scrollTop, setScrollTop] = useState(0);
    const [viewportHeight, setViewportHeight] = useState(0);

    useEffect(() => {
        const node = containerRef.current;
        if (!node) return;

        const updateSize = () => setViewportHeight(node.clientHeight);
        updateSize();

        const observer = new ResizeObserver(updateSize);
        observer.observe(node);
        return () => observer.disconnect();
    }, []);

    const { startIndex, offsetTop, visibleItems } = useMemo(() => {
        const visibleCount = Math.ceil(viewportHeight / itemHeight);
        const startIndex = Math.max(0, Math.floor(scrollTop / itemHeight) - overscan);
        const endIndex = Math.min(
            items.length,
            startIndex + visibleCount + overscan * 2,
        );

        return {
            startIndex,
            endIndex,
            offsetTop: startIndex * itemHeight,
            visibleItems: items.slice(startIndex, endIndex),
        };
    }, [itemHeight, items, overscan, scrollTop, viewportHeight]);

    useEffect(() => {
        onVisibleItemsChange?.(visibleItems, startIndex);
    }, [onVisibleItemsChange, visibleItems, startIndex]);

    return (
        <div
            ref={containerRef}
            className={className}
            onScroll={(event) => setScrollTop(event.currentTarget.scrollTop)}
        >
            <div style={{ height: items.length * itemHeight, position: 'relative' }}>
                <div
                    style={{
                        position: 'absolute',
                        top: offsetTop,
                        left: 0,
                        right: 0,
                    }}
                >
                    {visibleItems.map((item, index) => renderItem(item, startIndex + index))}
                </div>
            </div>
        </div>
    );
}
