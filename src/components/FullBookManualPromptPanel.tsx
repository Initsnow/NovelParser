import { useState, useEffect } from 'react';
import { useNovelStore } from '../store/novelStore';
import { Copy, Check, Send, AlertTriangle } from 'lucide-react';

interface Props {
    novelId: string;
}

export default function FullBookManualPromptPanel({ novelId }: Props) {
    const { getFullSummaryManualPrompt, parseManualFullSummaryResult, llmConfig } = useNovelStore();
    const [prompt, setPrompt] = useState('');
    const [tokenCount, setTokenCount] = useState(0);
    const [responseJson, setResponseJson] = useState('');
    const [copied, setCopied] = useState(false);
    const [parsing, setParsing] = useState(false);
    const [parseError, setParseError] = useState<string | null>(null);

    useEffect(() => {
        loadPrompt();
    }, [novelId]);

    const loadPrompt = async () => {
        try {
            const [p] = await Promise.all([
                getFullSummaryManualPrompt(novelId)
            ]);
            setPrompt(p);
            // Rough estimation for token count: length / 1.5 roughly for CJK models
            setTokenCount(Math.ceil(p.length / 1.5));
            setResponseJson('');
            setParseError(null);
        } catch (e) {
            console.error('Failed to generate full summary prompt:', e);
            setParseError(String(e));
        }
    };

    const handleCopy = async () => {
        await navigator.clipboard.writeText(prompt);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };

    const handleParseAndSave = async () => {
        setParsing(true);
        setParseError(null);
        try {
            await parseManualFullSummaryResult(responseJson);
        } catch (e) {
            setParseError(String(e));
        }
        setParsing(false);
    };

    const isOverLimit = tokenCount > llmConfig.max_context_tokens;

    return (
        <div className="space-y-4 max-w-3xl mx-auto">
            <h3 className="text-lg font-bold">手动分析全书概览</h3>

            {/* Token info */}
            <div className={`flex items-center gap-2 text-sm ${isOverLimit ? 'text-error' : 'text-base-content/60'}`}>
                {isOverLimit && <AlertTriangle size={14} />}
                <span>
                    预估 Prompt: {tokenCount.toLocaleString()} tokens
                    / 模型上限: {llmConfig.max_context_tokens.toLocaleString()}
                </span>
            </div>

            {/* Step 1: Copy prompt */}
            <div className="card bg-base-200 border border-base-300">
                <div className="card-body p-4">
                    <div className="flex items-center justify-between mb-2">
                        <h4 className="font-semibold text-sm">① 复制全书分析 Prompt</h4>
                        <button className="btn btn-sm btn-outline gap-2" onClick={handleCopy} disabled={!prompt}>
                            {copied ? <Check size={14} className="text-success" /> : <Copy size={14} />}
                            {copied ? '已复制' : '复制'}
                        </button>
                    </div>
                    {parseError && !prompt ? (
                        <div className="alert alert-error text-xs mb-2">
                            {parseError}
                        </div>
                    ) : (
                        <textarea
                            className="textarea textarea-bordered text-xs font-mono w-full h-40 resize-y"
                            value={prompt}
                            readOnly
                        />
                    )}
                </div>
            </div>

            {/* Step 2: Paste response */}
            <div className="card bg-base-200 border border-base-300">
                <div className="card-body p-4">
                    <h4 className="font-semibold text-sm mb-2">② 粘贴 AI 返回的全书 JSON</h4>
                    <textarea
                        className="textarea textarea-bordered text-xs font-mono w-full h-40 resize-y"
                        value={responseJson}
                        onChange={(e) => setResponseJson(e.target.value)}
                        placeholder='将 AI 返回的 JSON 结果粘贴到这里...'
                    />

                    {parseError && (
                        <div className="alert alert-error alert-sm mt-2">
                            <AlertTriangle size={14} />
                            <span className="text-xs">{parseError}</span>
                        </div>
                    )}

                    <button
                        className="btn btn-primary btn-sm gap-2 mt-3"
                        onClick={handleParseAndSave}
                        disabled={!responseJson.trim() || parsing}
                    >
                        {parsing ? <span className="loading loading-spinner loading-xs" /> : <Send size={14} />}
                        解析并展示
                    </button>
                </div>
            </div>
        </div>
    );
}
