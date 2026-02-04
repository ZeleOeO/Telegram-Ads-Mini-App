import { useState } from 'react';
import { X, CheckCircle2, Clock, MessageSquare, ExternalLink, ShieldCheck, DollarSign, Send, Ban, CheckCircle } from 'lucide-react';
import { api } from '../lib/api';
import { useTelegram } from '../hooks/useTelegram';

interface Deal {
    id: number;
    advertiser_id: number;
    channel_owner_id: number;
    channel_id: number;
    state: string;
    price_ton: string | number;
    post_content?: string;
    creative_status?: string;
    created_at?: string;
    creative_submitted_at?: string;
}

interface DealDetailsModalProps {
    deal: Deal | null;
    onClose: () => void;
}

export function DealDetailsModal({ deal, onClose }: DealDetailsModalProps) {
    const { user, tg } = useTelegram();
    const [submitting, setSubmitting] = useState(false);
    const [content, setContent] = useState(deal?.post_content || '');
    const [feedback, setFeedback] = useState('');
    const [showRevisionInput, setShowRevisionInput] = useState(false);

    if (!deal) return null;

    // Identify role
    const isAdvertiser = user && deal.advertiser_id === user.id;
    const isPublisher = user && deal.channel_owner_id === user.id; // Corrected to use enriched field

    const formatDate = (dateStr: string) => {
        if (!dateStr) return 'Pending...';
        const date = new Date(dateStr);
        return date.toLocaleDateString() + ' ' + date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    };

    const handleSubmitCreative = async () => {
        if (!content) return;
        setSubmitting(true);
        try {
            await api.post(`/deals/${deal.id}/creative`, { content });
            tg?.HapticFeedback.notificationOccurred('success');
            onClose(); // Close and let parent refresh
        } catch (_error) {
            alert('Failed to submit creative');
        } finally {
            setSubmitting(false);
        }
    };

    const handleReviewCreative = async (approved: boolean) => {
        setSubmitting(true);
        try {
            await api.post(`/deals/${deal.id}/review`, { approved, feedback: approved ? undefined : feedback });
            tg?.HapticFeedback.notificationOccurred('success');
            onClose();
        } catch (_error) {
            alert('Failed to review creative');
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

                    <div className="flex items-center gap-3 mb-6">
                        <div className="w-12 h-12 bg-white/5 rounded-2xl flex items-center justify-center text-xl border border-white/10">
                            🤝
                        </div>
                        <div>
                            <h3 className="text-lg font-bold">Deal Details</h3>
                            <div className="text-[10px] text-white/40 font-mono">ID: {deal.id}</div>
                        </div>
                    </div>

                    <div className="space-y-6">
                        {/* Status Banner */}
                        <div className={`p-4 rounded-2xl border flex items-center justify-between ${getStatusColor(deal.state)}`}>
                            <div className="flex items-center gap-3">
                                <div className="w-8 h-8 rounded-full bg-current opacity-20 flex items-center justify-center">
                                    <ShieldCheck size={16} />
                                </div>
                                <div>
                                    <div className="text-[10px] uppercase font-bold tracking-wider mb-0.5">Current Status</div>
                                    <div className="text-sm font-bold uppercase">{deal.state.replace('_', ' ')}</div>
                                </div>
                            </div>
                        </div>

                        {/* Details Grid */}
                        <div className="grid grid-cols-2 gap-3">
                            <div className="p-4 bg-white/5 rounded-2xl border border-white/10 space-y-1">
                                <div className="text-[10px] text-white/40 uppercase">Price</div>
                                <div className="text-lg font-bold flex items-center gap-1">
                                    <DollarSign size={16} className="text-blue-400" />
                                    {Number(deal.price_ton).toFixed(3)}
                                </div>
                            </div>
                            <div className="p-4 bg-white/5 rounded-2xl border border-white/10 space-y-1">
                                <div className="text-[10px] text-white/40 uppercase">Channel ID</div>
                                <div className="text-lg font-bold flex items-center gap-1">
                                    <ExternalLink size={16} className="text-purple-400" />
                                    {deal.channel_id}
                                </div>
                            </div>
                        </div>

                        {/* Steps / Timeline */}
                        <div className="space-y-4">
                            <h4 className="text-xs font-bold text-white/60 uppercase tracking-wider ml-1">Workflow Stage</h4>
                            <div className="space-y-3 relative before:absolute before:left-3 before:top-2 before:bottom-2 before:w-px before:bg-white/10">
                                <div className="flex gap-4 relative">
                                    <div className={`w-6 h-6 rounded-full flex items-center justify-center shrink-0 z-10 ${deal.created_at ? 'bg-blue-500 text-black' : 'bg-white/10'}`}>
                                        <Clock size={12} />
                                    </div>
                                    <div className="space-y-0.5">
                                        <div className="text-xs font-bold">Deal Created</div>
                                        <div className="text-[10px] text-white/40">{deal.created_at ? formatDate(deal.created_at) : 'Pending...'}</div>
                                    </div>
                                </div>
                                <div className="flex gap-4 relative">
                                    <div className={`w-6 h-6 rounded-full flex items-center justify-center shrink-0 z-10 ${deal.creative_submitted_at ? 'bg-blue-500 text-black' : 'bg-white/10'}`}>
                                        <MessageSquare size={12} />
                                    </div>
                                    <div className="space-y-0.5">
                                        <div className="text-xs font-bold">Creative Submitted</div>
                                        <div className="text-[10px] text-white/40">{deal.creative_submitted_at ? formatDate(deal.creative_submitted_at) : 'Waiting for advertiser...'}</div>
                                    </div>
                                </div>
                                <div className="flex gap-4 relative">
                                    <div className={`w-6 h-6 rounded-full flex items-center justify-center shrink-0 z-10 ${deal.state === 'Completed' ? 'bg-green-500 text-black' : 'bg-white/10'}`}>
                                        <CheckCircle2 size={12} />
                                    </div>
                                    <div className="space-y-0.5">
                                        <div className="text-xs font-bold">Proof of Work / Completion</div>
                                        <div className="text-[10px] text-white/40">{deal.state === 'Completed' ? 'Verified on chain' : 'Outcome pending...'}</div>
                                    </div>
                                </div>
                            </div>
                        </div>

                        {/* Advertiser: Submit Creative */}
                        {isAdvertiser && (deal.state === 'negotiating' || deal.state === 'awaiting_payment' || deal.state === 'payment_received' || deal.creative_status === 'revision_requested') && (
                            <div className="space-y-3 p-4 bg-white/5 rounded-2xl border border-white/10">
                                <h4 className="text-[10px] font-bold text-blue-400 uppercase tracking-widest">Submit Ad Creative</h4>
                                <textarea
                                    value={content}
                                    onChange={e => setContent(e.target.value)}
                                    placeholder="Write your ad text here..."
                                    className="w-full h-32 bg-black/40 border border-white/10 rounded-xl p-3 text-sm text-white focus:outline-none focus:border-blue-500/50 resize-none transition-all"
                                />
                                <button
                                    onClick={handleSubmitCreative}
                                    disabled={submitting || !content}
                                    className="w-full h-12 bg-blue-500 hover:bg-blue-400 disabled:opacity-30 text-black rounded-xl text-xs font-bold transition-all flex items-center justify-center gap-2"
                                >
                                    <Send size={14} /> {submitting ? 'Submitting...' : 'Submit to Publisher'}
                                </button>
                            </div>
                        )}

                        {/* Publisher: Review Creative */}
                        {isPublisher && deal.state === 'creative_submitted' && (
                            <div className="space-y-4">
                                <div className="p-4 bg-white/5 rounded-2xl border border-white/10 space-y-3">
                                    <h4 className="text-[10px] font-bold text-blue-400 uppercase tracking-widest">Review Submission</h4>
                                    <div className="p-3 bg-black/40 rounded-xl border border-white/5 text-sm italic text-white/80">
                                        "{deal.post_content}"
                                    </div>

                                    {!showRevisionInput ? (
                                        <div className="grid grid-cols-2 gap-3">
                                            <button
                                                onClick={() => setShowRevisionInput(true)}
                                                className="h-11 bg-white/5 border border-white/10 rounded-xl text-[10px] font-bold hover:bg-red-500/10 hover:text-red-400 hover:border-red-500/20 transition-all flex items-center justify-center gap-2"
                                            >
                                                <Ban size={14} /> Request Revision
                                            </button>
                                            <button
                                                onClick={() => handleReviewCreative(true)}
                                                disabled={submitting}
                                                className="h-11 bg-green-500 text-black rounded-xl text-[10px] font-bold hover:bg-green-400 transition-all flex items-center justify-center gap-2"
                                            >
                                                <CheckCircle size={14} /> {submitting ? 'Approve...' : 'Approve Media'}
                                            </button>
                                        </div>
                                    ) : (
                                        <div className="space-y-3 animate-in slide-in-from-top-2 duration-300">
                                            <textarea
                                                value={feedback}
                                                onChange={e => setFeedback(e.target.value)}
                                                placeholder="What needs to be changed?"
                                                className="w-full h-24 bg-black/40 border border-white/10 rounded-xl p-3 text-sm text-white focus:outline-none focus:border-red-500/50 resize-none"
                                            />
                                            <div className="flex gap-2">
                                                <button onClick={() => setShowRevisionInput(false)} className="flex-1 h-10 bg-white/5 rounded-xl text-[10px] font-bold">Cancel</button>
                                                <button
                                                    onClick={() => handleReviewCreative(false)}
                                                    disabled={submitting || !feedback}
                                                    className="flex-[2] h-10 bg-red-500 text-black rounded-xl text-[10px] font-bold"
                                                >
                                                    Send Feedback
                                                </button>
                                            </div>
                                        </div>
                                    )}
                                </div>
                            </div>
                        )}

                        <div className="pt-2">
                            <button
                                onClick={() => {
                                    tg?.HapticFeedback.impactOccurred('medium');
                                    // Deep link to bot with deal ID as start param
                                    // Replace 'YOUR_BOT_USERNAME' with the actual bot username if known, 
                                    // or use a generic link if the bot username is in env
                                    // For now, I'll use a placeholder or assume the bot name
                                    const botUsername = 'AntigravityAdsBot'; // Placeholder - should ideally come from public config
                                    tg?.openTelegramLink(`https://t.me/${botUsername}?start=deal_${deal.id}`);
                                }}
                                className="w-full h-12 bg-blue-500/10 border border-blue-500/20 rounded-xl text-sm font-bold active:scale-95 transition-all flex items-center justify-center gap-2 hover:bg-blue-500/20 text-blue-400"
                            >
                                <MessageSquare size={18} /> Negotiate via Bot
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}

// Add state color helper outside or inside
const getStatusColor = (state: string) => {
    switch (state.toLowerCase()) {
        case 'completed': return 'text-green-400 bg-green-400/10 border-green-400/20';
        case 'awaiting_payment': return 'text-orange-400 bg-orange-400/10 border-orange-400/20';
        case 'payment_received': return 'text-purple-400 bg-purple-400/10 border-purple-400/20';
        case 'creative_submitted': return 'text-blue-400 bg-blue-400/10 border-blue-400/20';
        case 'creative_approved': return 'text-green-400 bg-green-400/10 border-green-400/20';
        case 'rejected': return 'text-red-400 bg-red-400/10 border-red-400/20';
        case 'revision_requested': return 'text-yellow-400 bg-yellow-400/10 border-yellow-400/20';
        default: return 'text-white/40 bg-white/5 border-white/10';
    }
};
