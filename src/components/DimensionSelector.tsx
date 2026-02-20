import { useEffect } from 'react';
import { useNovelStore } from '../store/novelStore';
import type { AnalysisDimension } from '../types';

export default function DimensionSelector() {
    const { dimensions, currentNovel, fetchDimensions, updateDimensions } = useNovelStore();

    useEffect(() => {
        if (dimensions.length === 0) fetchDimensions();
    }, []);

    if (!currentNovel) return null;

    const enabled = currentNovel.enabled_dimensions;

    const toggle = (dim: AnalysisDimension) => {
        const newDims = enabled.includes(dim)
            ? enabled.filter(d => d !== dim)
            : [...enabled, dim];
        if (newDims.length > 0) {
            updateDimensions(newDims);
        }
    };

    return (
        <div className="space-y-1">
            <p className="text-xs font-semibold text-base-content/50 mb-2">分析维度</p>
            {dimensions.map((dim) => (
                <label
                    key={dim.id}
                    className="flex items-center gap-2 py-1 px-1 rounded hover:bg-base-300/50 cursor-pointer text-sm"
                >
                    <input
                        type="checkbox"
                        className="checkbox checkbox-xs checkbox-primary"
                        checked={enabled.includes(dim.id)}
                        onChange={() => toggle(dim.id)}
                    />
                    <span>{dim.icon}</span>
                    <span className="flex-1">{dim.name}</span>
                </label>
            ))}
        </div>
    );
}
