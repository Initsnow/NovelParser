import { createPortal } from 'react-dom';
import { motion } from 'framer-motion';

interface Props {
    title: string;
    message: string;
    confirmText?: string;
    cancelText?: string;
    kind?: 'warning' | 'error' | 'info';
    hideCancel?: boolean;
    onConfirm: () => void;
    onCancel: () => void;
}

export default function ConfirmDialog({
    title, message, confirmText = '确定', cancelText = '取消',
    kind = 'warning', hideCancel = false, onConfirm, onCancel,
}: Props) {
    const btnClass = kind === 'error' ? 'btn-error' : kind === 'warning' ? 'btn-warning' : 'btn-primary';

    return createPortal(
        <div className="fixed inset-0 z-[100] flex items-center justify-center p-4">
            <motion.div
                className="absolute inset-0 bg-black/40 backdrop-blur-sm"
                onClick={onCancel}
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                transition={{ duration: 0.2 }}
            />
            <motion.div
                className="relative z-10 bg-base-200 rounded-2xl shadow-2xl border border-base-content/10 w-full max-w-sm p-6 flex flex-col gap-4"
                initial={{ opacity: 0, scale: 0.95, y: 10 }}
                animate={{ opacity: 1, scale: 1, y: 0 }}
                exit={{ opacity: 0, scale: 0.95, y: 10 }}
                transition={{ type: "spring", bounce: 0, duration: 0.3 }}
            >
                <h3 className="font-bold text-lg">{title}</h3>
                <p className="text-sm text-base-content/70">{message}</p>
                <div className="flex justify-end gap-2 mt-2">
                    {!hideCancel && (
                        <button className="btn btn-ghost btn-sm" onClick={onCancel}>{cancelText}</button>
                    )}
                    <button className={`btn btn-sm ${btnClass}`} onClick={onConfirm}>{confirmText}</button>
                </div>
            </motion.div>
        </div>,
        document.body
    );
}
