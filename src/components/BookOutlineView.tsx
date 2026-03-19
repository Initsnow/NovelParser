import type { BookOutline } from '../types';

interface BookOutlineViewProps {
    outline: BookOutline;
}

export default function BookOutlineView({ outline }: BookOutlineViewProps) {
    return (
        <div className="space-y-6">
            <div className="card bg-base-200 border border-base-300">
                <div className="card-body p-5">
                    <h3 className="font-bold text-lg mb-2">一句话梗概</h3>
                    <p className="text-sm leading-relaxed whitespace-pre-wrap">{outline.logline}</p>
                </div>
            </div>

            <div className="card bg-base-200 border border-base-300">
                <div className="card-body p-5">
                    <h3 className="font-bold text-lg mb-2">故事大纲</h3>
                    <p className="text-sm leading-relaxed whitespace-pre-wrap">{outline.story_outline}</p>
                </div>
            </div>

            <div className="card bg-base-200 border border-base-300">
                <div className="card-body p-5">
                    <h3 className="font-bold text-lg mb-2">世界观设定</h3>
                    <p className="text-sm leading-relaxed whitespace-pre-wrap">{outline.world_setting}</p>
                </div>
            </div>

            {outline.volumes.length > 0 && (
                <div className="card bg-base-200 border border-base-300">
                    <div className="card-body p-5">
                        <h3 className="font-bold text-lg mb-3">分卷</h3>
                        <div className="space-y-3">
                            {outline.volumes.map((segment, index) => (
                                <div key={`${segment.title}-${index}`} className="rounded-xl bg-base-300/40 p-4">
                                    <div className="flex items-center justify-between gap-3">
                                        <div className="font-semibold">{segment.title}</div>
                                        <div className="badge badge-outline">
                                            第 {segment.volume_number} 卷 · 第 {segment.chapter_start + 1}-{segment.chapter_end + 1} 章
                                        </div>
                                    </div>
                                    <p className="mt-2 text-sm leading-relaxed whitespace-pre-wrap">{segment.summary}</p>
                                </div>
                            ))}
                        </div>
                    </div>
                </div>
            )}

            {outline.character_cards.length > 0 && (
                <div className="card bg-base-200 border border-base-300">
                    <div className="card-body p-5">
                        <h3 className="font-bold text-lg mb-3">角色卡</h3>
                        <div className="space-y-4">
                            {outline.character_cards.map((card, index) => (
                                <div key={`${card.name}-${index}`} className="rounded-xl bg-base-300/40 p-4 space-y-2">
                                    <div className="flex flex-wrap items-center gap-2">
                                        <div className="font-semibold text-base">{card.name}</div>
                                        <span className="badge badge-outline">{card.character_type}</span>
                                        <span className="badge badge-outline">{card.lifecycle}</span>
                                        {card.first_volume != null && card.last_volume != null && (
                                            <span className="badge badge-outline">第 {card.first_volume} 卷 - 第 {card.last_volume} 卷</span>
                                        )}
                                    </div>
                                    <p className="text-sm whitespace-pre-wrap"><span className="font-medium">描述：</span>{card.description}</p>
                                    <p className="text-sm whitespace-pre-wrap"><span className="font-medium">性格：</span>{card.personality}</p>
                                    <p className="text-sm whitespace-pre-wrap"><span className="font-medium">核心驱动力：</span>{card.core_drive}</p>
                                    <p className="text-sm whitespace-pre-wrap"><span className="font-medium">角色弧光：</span>{card.arc}</p>
                                    {card.key_scenes.length > 0 && (
                                        <p className="text-sm whitespace-pre-wrap"><span className="font-medium">出场/常驻场景：</span>{card.key_scenes.join('、')}</p>
                                    )}
                                </div>
                            ))}
                        </div>
                    </div>
                </div>
            )}

            {outline.scene_cards.length > 0 && (
                <div className="card bg-base-200 border border-base-300">
                    <div className="card-body p-5">
                        <h3 className="font-bold text-lg mb-3">场景卡</h3>
                        <div className="space-y-4">
                            {outline.scene_cards.map((card, index) => (
                                <div key={`${card.name}-${index}`} className="rounded-xl bg-base-300/40 p-4 space-y-2">
                                    <div className="flex flex-wrap items-center gap-2">
                                        <div className="font-semibold text-base">{card.name}</div>
                                        <span className="badge badge-outline">{card.lifecycle}</span>
                                        {card.first_volume != null && card.last_volume != null && (
                                            <span className="badge badge-outline">第 {card.first_volume} 卷 - 第 {card.last_volume} 卷</span>
                                        )}
                                    </div>
                                    <p className="text-sm whitespace-pre-wrap"><span className="font-medium">描述：</span>{card.description}</p>
                                    <p className="text-sm whitespace-pre-wrap"><span className="font-medium">剧情作用：</span>{card.story_function}</p>
                                </div>
                            ))}
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}
