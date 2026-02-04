import { useState } from 'react';
import { X } from 'lucide-react';
import { api } from '../lib/api';

interface PricingModalProps {
    channelId: number;
    onClose: () => void;
    onSuccess: () => void;
}

export function PricingModal({ channelId, onClose, onSuccess }: PricingModalProps) {
    const [format, setFormat] = useState('Post');
    const [duration, setDuration] = useState('48h');
    const [price, setPrice] = useState('');
    const [submitting, setSubmitting] = useState(false);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setSubmitting(true);
        try {
            await api.post(`/channels/${channelId}/ad-formats`, {
                format_name: `${format} (${duration})`,
                price_ton: parseFloat(price),
                description: `Standard ${format.toLowerCase()} kept for ${duration}`,
            });
            onSuccess();
            onClose();
        } catch (_error) {
            alert('Failed to add pricing format');
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <div className="fixed inset-0 z-[110] flex items-end sm:items-center justify-center sm:p-4 bg-black/60 backdrop-blur-sm animate-in fade-in duration-200">
            <div className="w-full sm:max-w-sm glass rounded-t-2xl sm:rounded-2xl p-6 relative max-h-[90vh] overflow-y-auto">
                <div className="pb-20">
                    <button
                        onClick={onClose}
                        className="absolute top-4 right-4 text-white/40 hover:text-white"
                    >
                        <X size={20} />
                    </button>

                    <h3 className="text-lg font-bold mb-4">💰 Set Ad Pricing</h3>

                    <form onSubmit={handleSubmit} className="space-y-4">
                        <div className="space-y-1.5">
                            <label className="text-xs text-white/60 ml-1">Format Type</label>
                            <select
                                value={format}
                                onChange={e => setFormat(e.target.value)}
                                className="w-full h-10 px-3 bg-black/20 border border-white/10 rounded-xl text-sm appearance-none focus:outline-none"
                            >
                                <option value="Post">Standard Post</option>
                                <option value="Repost">Repost / Forward</option>
                                <option value="Story">Story</option>
                            </select>
                        </div>

                        <div className="space-y-1.5">
                            <label className="text-xs text-white/60 ml-1">Duration</label>
                            <select
                                value={duration}
                                onChange={e => setDuration(e.target.value)}
                                className="w-full h-10 px-3 bg-black/20 border border-white/10 rounded-xl text-sm appearance-none focus:outline-none"
                            >
                                <option value="24h">24 Hours</option>
                                <option value="48h">48 Hours</option>
                                <option value="72h">3 Days</option>
                                <option value="Permanent">Permanent</option>
                            </select>
                        </div>

                        <div className="space-y-1.5">
                            <label className="text-xs text-white/60 ml-1">Price (TON)</label>
                            <input
                                type="number"
                                step="0.1"
                                value={price}
                                onChange={e => setPrice(e.target.value)}
                                placeholder="50.0"
                                className="w-full h-10 px-3 bg-black/20 border border-white/10 rounded-xl text-sm focus:outline-none focus:border-blue-500/50"
                                required
                            />
                        </div>

                        <button
                            type="submit"
                            disabled={submitting}
                            className="w-full h-11 bg-green-500 hover:bg-green-600 rounded-xl font-medium active:scale-95 transition-all disabled:opacity-50 text-black font-bold"
                        >
                            {submitting ? 'Saving...' : 'Add Pricing'}
                        </button>
                    </form>
                </div>
            </div>
        </div>
    );
}
