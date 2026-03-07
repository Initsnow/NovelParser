import { useEffect, useState } from 'react';

export function formatTime(ms: number) {
    const s = Math.floor(ms / 1000);
    if (s < 60) return `${s}s`;
    const m = Math.floor(s / 60);
    return `${m}m ${s % 60}s`;
}

export function BatchTimeStats({ startTime, current, total }: { startTime: number, current: number, total: number }) {
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
