import { useEffect, useState } from 'react';
import { X, Send, LayoutList, DollarSign, AlertCircle } from 'lucide-react';
import { api } from '../lib/api';
import { useTelegram } from '../hooks/useTelegram';
import type { Channel, BackendError } from '../types';

interface ApplyCampaignModalProps {
    campaignId: number;
    campaignTitle: string;
    onClose: () => void;
}

export function ApplyCampaignModal({ campaignId, campaignTitle, onClose }: ApplyCampaignModalProps) {
    const [channels, setChannels] = useState<Channel[]>([]);
    const [selectedChannelId, setSelectedChannelId] = useState<string>('');
    const [price, setPrice] = useState('');
    const [message, setMessage] = useState('');
    const [loading, setLoading] = useState(true);
    const [submitting, setSubmitting] = useState(false);
    const { tg } = useTelegram();

    useEffect(() => {
        const fetchMyChannels = async () => {
            try {
                const res = await api.get('/channels/my');
                setChannels(res.data);
            } catch (err) {
                console.error('Failed to fetch my channels:', err);
            } finally {
                setLoading(false);
            }
        };
        fetchMyChannels();
    }, []);

    const handleApply = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!selectedChannelId) return;

        setSubmitting(true);
        try {
            await api.post(`/campaigns/${campaignId}/channels/${selectedChannelId}/apply`, {
                proposed_price_ton: parseFloat(price),
                message
            });
            tg?.HapticFeedback.notificationOccurred('success');
            tg?.showAlert('Application sent successfully! The advertiser will review it soon.');
            onClose();
        } catch (err: unknown) {
            console.error('Application failed:', err);
            tg?.HapticFeedback.notificationOccurred('error');
            const error = err as BackendError;
            tg?.showAlert(`Error: ${error.response?.data?.error || error.message}`);
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <div className="fixed inset-0 z-[110] flex items-end sm:items-center justify-center sm:p-4 bg-black/60 backdrop-blur-sm animate-in fade-in duration-200">
            <div className="w-full sm:max-w-md glass rounded-t-2xl sm:rounded-2xl p-6 relative max-h-[90vh] overflow-y-auto font-sans">
                <div className="pb-20">
                    <button
                        onClick={onClose}
                        className="absolute top-4 right-4 text-white/40 hover:text-white"
                    >
                        <X size={20} />
                    </button>

                    <h3 className="text-lg font-bold mb-2">🚀 Apply to Campaign</h3>
                    <p className="text-xs text-white/40 mb-6">{campaignTitle}</p>

                    {loading ? (
                        <div className="text-center py-10 opacity-50">Loading your channels...</div>
                    ) : channels.length === 0 ? (
                        <div className="p-6 bg-red-500/10 rounded-2xl border border-red-500/20 text-center space-y-3">
                            <AlertCircle size={32} className="mx-auto text-red-400 opacity-50" />
                            <div className="text-sm font-bold text-red-400">No Channels Found</div>
                            <p className="text-[10px] text-white/60">You need to add a channel to your profile before you can apply to campaigns.</p>
                            <button
                                onClick={onClose}
                                className="px-4 h-9 bg-white/10 rounded-xl text-xs font-bold"
                            >
                                Got it
                            </button>
                        </div>
                    ) : (
                        <form onSubmit={handleApply} className="space-y-4">
                            <div className="space-y-1.5">
                                <label className="text-xs text-white/60 ml-1 flex items-center gap-2">
                                    <LayoutList size={14} /> Select Channel
                                </label>
                                <select
                                    value={selectedChannelId}
                                    onChange={e => setSelectedChannelId(e.target.value)}
                                    className="w-full h-12 px-4 bg-black/20 border border-white/10 rounded-xl text-sm appearance-none focus:outline-none focus:border-blue-500/50"
                                    required
                                >
                                    <option value="" disabled>Choose a channel...</option>
                                    {channels.map(c => (
                                        <option key={c.id} value={c.id}>
                                            {c.title} ({c.subscribers.toLocaleString()} subs)
                                        </option>
                                    ))}
                                </select>
                            </div>

                            <div className="space-y-1.5">
                                <label className="text-xs text-white/60 ml-1 flex items-center gap-2">
                                    <DollarSign size={14} /> Proposed Price (TON)
                                </label>
                                <input
                                    type="number"
                                    step="0.1"
                                    value={price}
                                    onChange={e => setPrice(e.target.value)}
                                    placeholder="e.g. 50.0"
                                    className="w-full h-12 px-4 bg-black/20 border border-white/10 rounded-xl text-sm focus:outline-none focus:border-blue-500/50"
                                    required
                                />
                            </div>

                            <div className="space-y-1.5">
                                <label className="text-xs text-white/60 ml-1">Message to advertiser (Optional)</label>
                                <textarea
                                    value={message}
                                    onChange={e => setMessage(e.target.value)}
                                    placeholder="Tell them why your channel is a good fit..."
                                    className="w-full h-24 px-4 py-3 bg-black/20 border border-white/10 rounded-xl text-sm focus:outline-none focus:border-blue-500/50 resize-none"
                                />
                            </div>

                            <button
                                type="submit"
                                disabled={submitting}
                                className="w-full h-14 bg-gradient-to-r from-blue-500 to-blue-600 rounded-2xl font-bold flex items-center justify-center gap-2 active:scale-95 transition-all disabled:opacity-50 mt-4"
                            >
                                {submitting ? 'Sending...' : (
                                    <>Send Application <Send size={18} /></>
                                )}
                            </button>
                        </form>
                    )}
                </div>
            </div>
        </div>
    );
}
