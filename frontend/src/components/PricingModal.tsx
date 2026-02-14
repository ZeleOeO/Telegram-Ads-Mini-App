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
    const [price, setPrice] = useState('');
    const [submitting, setSubmitting] = useState(false);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setSubmitting(true);
        try {
            await api.post(`/channels/${channelId}/ad-formats`, {
                format_name: format,
                price_ton: parseFloat(price),
                description: `Standard ${format.toLowerCase()} ad placement`,
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
                        className="absolute top-4 right-4 text-muted hover:text-white"
                    >
                        <X size={20} />
                    </button>

                    <h3 className="text-lg font-bold mb-4">Set Ad Pricing</h3>

                    <form onSubmit={handleSubmit} className="space-y-4">
                        <div className="space-y-1.5">
                            <label className="text-xs text-muted ml-1">Format Type</label>
                            <select
                                value={format}
                                onChange={e => setFormat(e.target.value)}
                                className="w-full h-10 px-3 glass rounded-xl text-sm appearance-none focus:outline-none"
                            >
                                <option value="Post">Standard Post</option>
                                <option value="Repost">Repost / Forward</option>
                                <option value="Story">Story</option>
                            </select>
                        </div>

                        <div className="space-y-1.5">
                            <label className="text-xs text-muted ml-1">Price (TON)</label>
                            <input
                                type="number"
                                step="0.1"
                                value={price}
                                onChange={e => setPrice(e.target.value)}
                                placeholder="50.0"
                                className="w-full h-10 px-3 glass rounded-xl text-sm focus:outline-none focus:border-blue-500/50"
                                required
                            />
                        </div>

                        <p className="text-[10px] text-muted text-center">
                            All ads are permanent placements
                        </p>

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

