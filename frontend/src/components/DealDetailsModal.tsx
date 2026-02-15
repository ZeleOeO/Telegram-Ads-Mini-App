import { useState, useEffect } from 'react';
import { X, CheckCircle2, Clock, MessageSquare, ExternalLink, ShieldCheck, DollarSign, Send, Ban, CheckCircle, Copy, Eye, Megaphone, Hash, Wallet, XCircle, Calendar } from 'lucide-react';
import { api } from '../lib/api';
import { useTelegram } from '../hooks/useTelegram';

import type { Deal } from '../types';

interface DealDetailsModalProps {
    deal?: Deal | null;
    dealId?: number;
    onClose: () => void;
    onRefresh?: () => void;
}

export function DealDetailsModal({ deal, dealId, onClose, onRefresh }: DealDetailsModalProps) {
    const { user, tg } = useTelegram();
    const [fetchedDeal, setFetchedDeal] = useState<Deal | null>(null);
    const [loading, setLoading] = useState(false);

    // Derived state for fields that can be edited
    const [submitting, setSubmitting] = useState(false);
    const [content, setContent] = useState('');
    const [feedback, setFeedback] = useState('');
    const [showRevisionInput, setShowRevisionInput] = useState(false);
    const [rejectionReason, setRejectionReason] = useState('');
    const [showRejectInput, setShowRejectInput] = useState(false);
    const [scheduledTime, setScheduledTime] = useState('');

    useEffect(() => {
        if (!deal && dealId) {
            setLoading(true);
            api.get<Deal>(`/deals/${dealId}`)
                .then(res => setFetchedDeal(res.data))
                .catch(err => {
                    console.error(err);
                    alert('Failed to load deal details');
                    onClose();
                })
                .finally(() => setLoading(false));
        }
    }, [deal, dealId]);

    const activeDeal = deal || fetchedDeal;

    useEffect(() => {
        if (activeDeal) {
            setContent(activeDeal.post_content || '');
            setScheduledTime(activeDeal.scheduled_post_time || '');
        }
    }, [activeDeal]);

    if (loading) return (
        <div className="fixed inset-0 z-[110] flex items-center justify-center bg-black/60 backdrop-blur-sm">
            <div className="text-white/50">Loading deal...</div>
        </div>
    );

    if (!activeDeal) return null;

    const isAdvertiser = user ? activeDeal.advertiser_telegram_id === user.id : false;
    const isChannelOwner = user ? activeDeal.channel_owner_telegram_id === user.id : false;
    const isOwner = user ? activeDeal.owner_telegram_id === user.id : false;
    const isApplicant = user ? activeDeal.applicant_telegram_id === user.id : false;

    const isAccepted = activeDeal.state !== 'pending' && activeDeal.state !== 'rejected';

    const formatDate = (dateStr: string) => {
        if (!dateStr) return 'Pending...';
        const date = new Date(dateStr);
        return date.toLocaleDateString() + ' ' + date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    };

    const handleMarkPaid = async () => {
        setSubmitting(true);
        try {
            await api.post(`/deals/${activeDeal.id}/mark-paid`);
            tg?.HapticFeedback.notificationOccurred('success');
            tg?.showAlert('Payment confirmed! The channel owner can now draft the post.');
            onRefresh?.();
            onClose();
        } catch (error: any) {
            alert(error.response?.data?.error || 'Failed to mark as paid');
        } finally {
            setSubmitting(false);
        }
    };

    const handleVerifyPost = async () => {
        setSubmitting(true);
        try {
            await api.post(`/deals/${activeDeal.id}/verify-post`);
            tg?.HapticFeedback.notificationOccurred('success');
            tg?.showAlert('Post verified! Deal completed successfully.');
            onRefresh?.();
            onClose();
        } catch (error: any) {
            alert(error.response?.data?.error || 'Failed to verify post');
        } finally {
            setSubmitting(false);
        }
    };

    const handleAcceptDeal = async () => {
        setSubmitting(true);
        try {
            await api.post(`/deals/${activeDeal.id}/accept`);
            tg?.HapticFeedback.notificationOccurred('success');
            tg?.showAlert('Deal accepted! Waiting for advertiser to pay.');
            onRefresh?.();
            onClose();
        } catch (error: any) {
            alert(error.response?.data?.error || 'Failed to accept deal');
        } finally {
            setSubmitting(false);
        }
    };

    const handleRejectDeal = async () => {
        setSubmitting(true);
        try {
            await api.post(`/deals/${activeDeal.id}/reject`, { reason: rejectionReason || undefined });
            tg?.HapticFeedback.notificationOccurred('success');
            tg?.showAlert('Deal rejected and archived.');
            onRefresh?.();
            onClose();
        } catch (error: any) {
            alert(error.response?.data?.error || 'Failed to reject deal');
        } finally {
            setSubmitting(false);
        }
    };

    const handleSubmitDraft = async () => {
        if (!content || !scheduledTime) return;
        setSubmitting(true);
        try {
            await api.post(`/deals/${activeDeal.id}/draft`, {
                content,
                scheduled_post_time: scheduledTime
            });
            tg?.HapticFeedback.notificationOccurred('success');
            tg?.showAlert('Draft submitted! Waiting for advertiser review.');
            onRefresh?.();
            onClose();
        } catch (error: any) {
            alert(error.response?.data?.error || 'Failed to submit draft');
        } finally {
            setSubmitting(false);
        }
    };

    const handleReviewDraft = async (approved: boolean) => {
        setSubmitting(true);
        try {
            await api.post(`/deals/${activeDeal.id}/review-draft`, {
                approved,
                edit_reason: approved ? undefined : feedback
            });
            tg?.HapticFeedback.notificationOccurred('success');
            tg?.showAlert(approved ? 'Draft approved! Ad will be posted as scheduled.' : 'Edit requested. Channel owner will revise.');
            onRefresh?.();
            onClose();
        } catch (error: any) {
            alert(error.response?.data?.error || 'Failed to review draft');
        } finally {
            setSubmitting(false);
        }
    };

    const handleCopyTelegramId = () => {
        const username = isAdvertiser ? activeDeal.channel_owner_username : activeDeal.advertiser_username;
        if (username) {
            navigator.clipboard.writeText(`@${username}`);
            tg?.HapticFeedback.notificationOccurred('success');
            tg?.showAlert(`Username @${username} copied!`);
        } else {
            const idToCopy = isAdvertiser ? activeDeal.channel_owner_id : activeDeal.advertiser_id;
            navigator.clipboard.writeText(idToCopy.toString());
            tg?.HapticFeedback.notificationOccurred('warning');
            tg?.showAlert('Username not available. Telegram ID copied instead.');
        }
    };

    return (
        <div className="fixed inset-0 z-[110] flex items-end sm:items-center justify-center sm:p-4 bg-black/60 backdrop-blur-sm animate-in fade-in duration-200">
            <div className="w-full sm:max-w-md glass rounded-t-2xl sm:rounded-2xl p-6 relative max-h-[90vh] overflow-y-auto font-sans">
                <div className="pb-20">
                    <button
                        onClick={onClose}
                        className="absolute top-4 right-4 text-muted hover:text-white"
                    >
                        <X size={20} />
                    </button>

                    <div className="flex items-center gap-3 mb-6">
                        <div className="w-12 h-12 bg-white/5 rounded-2xl flex items-center justify-center border border-white/10">
                            <MessageSquare size={20} className="text-blue-400" />
                        </div>
                        <div className="flex-1">
                            <div className="flex items-center gap-2">
                                <h3 className="text-lg font-bold">Deal Details</h3>
                                {isOwner && (
                                    <span className="px-2 py-0.5 rounded-full text-[9px] font-bold border text-emerald-400 bg-emerald-400/10 border-emerald-400/20">
                                        OWNER
                                    </span>
                                )}
                                {!isOwner && (
                                    <span className="px-2 py-0.5 rounded-full text-[9px] font-bold border text-sky-400 bg-sky-400/10 border-sky-400/20">
                                        APPLICANT
                                    </span>
                                )}
                            </div>
                            <div className="text-[10px] text-muted">
                                {activeDeal.channel_title ? `@${activeDeal.channel_username || activeDeal.channel_title}` : 'Channel'}
                            </div>
                        </div>
                    </div>

                    <div className="space-y-6">
                        <div className={`p-4 rounded-2xl border flex items-center justify-between ${getStatusColor(activeDeal.state)}`}>
                            <div className="flex items-center gap-3">
                                <div className="w-8 h-8 rounded-full bg-current opacity-20 flex items-center justify-center">
                                    <ShieldCheck size={16} />
                                </div>
                                <div>
                                    <div className="text-[10px] uppercase font-bold tracking-wider mb-0.5">Current Status</div>
                                    <div className="text-sm font-bold uppercase">{activeDeal.state.replace('_', ' ')}</div>
                                </div>
                            </div>
                            <div className={`px-3 py-1.5 rounded-xl text-[10px] font-bold flex items-center gap-1.5 ${activeDeal.deal_type === 'campaign_request'
                                ? 'bg-purple-500/20 text-purple-300 border border-purple-500/30'
                                : 'bg-blue-500/20 text-blue-300 border border-blue-500/30'
                                }`}>
                                {activeDeal.deal_type === 'campaign_request' ? (
                                    <div className="flex flex-col items-end">
                                        <div className="flex items-center gap-1"><Megaphone size={12} /> Campaign</div>
                                        {activeDeal.campaign_title && (
                                            <div className="text-[9px] opacity-70 max-w-[120px] truncate text-right">{activeDeal.campaign_title}</div>
                                        )}
                                    </div>
                                ) : (
                                    <><Hash size={12} /> Channel</>
                                )}
                            </div>
                        </div>

                        <div className="grid grid-cols-2 gap-3">
                            <div className="p-4 bg-white/5 rounded-2xl border border-white/10 space-y-1">
                                <div className="text-[10px] text-muted uppercase">Price</div>
                                <div className="text-lg font-bold flex items-center gap-1">
                                    <DollarSign size={16} className="text-blue-400" />
                                    {activeDeal.price_ton ? Number(activeDeal.price_ton).toFixed(3) : '-'} TON
                                </div>
                            </div>
                            <div className="p-4 bg-white/5 rounded-2xl border border-white/10 space-y-1">
                                <div className="text-[10px] text-muted uppercase">Channel</div>
                                <div
                                    className="text-lg font-bold flex items-center gap-1 group/channel cursor-pointer hover:text-blue-400 transition-colors min-w-0"
                                    onClick={() => {
                                        if (activeDeal.channel_username) {
                                            tg?.openTelegramLink(`https://t.me/${activeDeal.channel_username}`);
                                        }
                                    }}
                                >
                                    <ExternalLink size={16} className="text-purple-400 shrink-0" />
                                    <span className="truncate">
                                        {activeDeal.channel_username ? `@${activeDeal.channel_username}` : (activeDeal.channel_title || 'Channel')}
                                    </span>
                                    {activeDeal.channel_username && (
                                        <button
                                            onClick={(e) => {
                                                e.stopPropagation();
                                                navigator.clipboard.writeText(`@${activeDeal.channel_username}`);
                                                tg?.HapticFeedback.notificationOccurred('success');
                                            }}
                                            className="opacity-0 group-hover/channel:opacity-100 p-1.5 hover:bg-white/10 rounded-lg transition-all ml-1"
                                        >
                                            <Copy size={12} className="text-muted" />
                                        </button>
                                    )}
                                </div>
                            </div>
                        </div>

                        {activeDeal.escrow_address && (
                            <div className="p-4 bg-white/5 rounded-2xl border border-white/10 space-y-2">
                                <div className="text-[10px] text-muted uppercase flex items-center gap-1">
                                    <Wallet size={12} /> Escrow Wallet
                                </div>
                                <div className="flex items-center gap-2">
                                    <code className="text-xs text-blue-400 bg-black/30 px-2 py-1 rounded-lg flex-1 truncate">
                                        {activeDeal.escrow_address}
                                    </code>
                                    <button
                                        onClick={() => {
                                            navigator.clipboard.writeText(activeDeal.escrow_address || '');
                                            tg?.HapticFeedback.notificationOccurred('success');
                                        }}
                                        className="p-2 bg-white/5 rounded-lg hover:bg-white/10 transition-colors"
                                    >
                                        <Copy size={14} />
                                    </button>
                                </div>
                            </div>
                        )}

                        <div className="space-y-4">
                            <h4 className="text-xs font-bold text-muted uppercase tracking-wider ml-1">Workflow Stage</h4>
                            <div className="space-y-3 relative before:absolute before:left-3 before:top-2 before:bottom-2 before:w-px before:bg-white/10">
                                <div className="flex gap-4 relative items-center">
                                    <div className={`w-6 h-6 rounded-full flex items-center justify-center shrink-0 z-10 ${activeDeal.created_at ? 'bg-blue-500 text-black' : 'bg-white/10'}`}>
                                        <Clock size={12} />
                                    </div>
                                    <div className="space-y-0.5">
                                        <div className="text-xs font-bold">Deal Created</div>
                                        <div className="text-[10px] text-muted">{activeDeal.created_at ? formatDate(activeDeal.created_at) : 'Pending...'}</div>
                                    </div>
                                </div>
                                <div className="flex gap-4 relative items-center">
                                    <div className={`w-6 h-6 rounded-full flex items-center justify-center shrink-0 z-10 ${activeDeal.creative_submitted_at ? 'bg-blue-500 text-black' : 'bg-white/10'}`}>
                                        <MessageSquare size={12} />
                                    </div>
                                    <div className="space-y-0.5">
                                        <div className="text-xs font-bold">Creative Submitted</div>
                                        <div className="text-[10px] text-muted">{activeDeal.creative_submitted_at ? formatDate(activeDeal.creative_submitted_at) : 'Waiting for advertiser...'}</div>
                                    </div>
                                </div>
                                <div className="flex gap-4 relative items-center">
                                    <div className={`w-6 h-6 rounded-full flex items-center justify-center shrink-0 z-10 ${activeDeal.state === 'Completed' ? 'bg-green-500 text-black' : 'bg-white/10'}`}>
                                        <CheckCircle2 size={12} />
                                    </div>
                                    <div className="space-y-0.5">
                                        <div className="text-xs font-bold">Proof of Work / Completion</div>
                                        <div className="text-[10px] text-muted">{activeDeal.state === 'Completed' ? 'Verified on chain' : 'Outcome pending...'}</div>
                                    </div>
                                </div>
                            </div>
                        </div>

                        {isChannelOwner && activeDeal.state === 'awaiting_payment' && (
                            <div className="space-y-3 p-4 bg-amber-500/5 rounded-2xl border border-amber-500/20">
                                <div className="flex items-center gap-2">
                                    <Wallet size={14} className="text-amber-400" />
                                    <h4 className="text-[10px] font-bold text-amber-400 uppercase tracking-widest">Awaiting Payment</h4>
                                </div>
                                <p className="text-xs text-muted">
                                    The advertiser must complete payment before you can submit your ad draft.
                                    Once payment is confirmed, this section will unlock.
                                </p>
                            </div>
                        )}

                        {isChannelOwner && (activeDeal.state === 'drafting' || (activeDeal.state === 'reviewing' && activeDeal.creative_status === 'edit_requested')) && (
                            <div className="space-y-3 p-4 bg-white/5 rounded-2xl border border-white/10">
                                <div className="flex items-center justify-between">
                                    <h4 className="text-[10px] font-bold text-green-400 uppercase tracking-widest">
                                        {activeDeal.creative_status === 'edit_requested' ? 'Revision Requested' : 'Payment Confirmed - Draft Your Post'}
                                    </h4>
                                </div>
                                {activeDeal.edit_request_reason && (
                                    <div className="p-3 bg-amber-500/10 rounded-xl border border-amber-500/20 text-xs text-amber-300">
                                        <span className="font-bold">Advertiser Feedback:</span> {activeDeal.edit_request_reason}
                                    </div>
                                )}
                                <textarea
                                    value={content}
                                    onChange={e => setContent(e.target.value)}
                                    placeholder="Write the ad post text here..."
                                    className="w-full h-32 glass rounded-xl p-3 text-sm focus:outline-none focus:border-green-500/50 resize-none transition-all"
                                />
                                <div className="space-y-2">
                                    <label className="text-[10px] text-muted uppercase flex items-center gap-1">
                                        <Calendar size={12} /> Schedule Post Time (UTC)
                                    </label>
                                    <input
                                        type="datetime-local"
                                        value={scheduledTime}
                                        onChange={e => setScheduledTime(e.target.value)}
                                        className="w-full h-12 glass rounded-xl px-3 text-sm focus:outline-none focus:border-green-500/50 flex items-center"
                                    />
                                </div>
                                <button
                                    onClick={handleSubmitDraft}
                                    disabled={submitting || !content || !scheduledTime}
                                    className="w-full h-12 bg-green-500 hover:bg-green-400 disabled:opacity-30 text-black rounded-xl text-xs font-bold transition-all flex items-center justify-center gap-2"
                                >
                                    <Send size={14} /> {submitting ? 'Submitting...' : 'Submit Draft for Review'}
                                </button>
                            </div>
                        )}

                        {isAdvertiser && activeDeal.state === 'reviewing' && (
                            <div className="space-y-4">
                                <div className="p-4 bg-white/5 rounded-2xl border border-white/10 space-y-3">
                                    <h4 className="text-[10px] font-bold text-green-400 uppercase tracking-widest">Review Channel Owner's Draft</h4>
                                    <div className="p-3 glass rounded-xl border border-white/5 text-sm italic text-foreground whitespace-pre-wrap">
                                        {activeDeal.post_content}
                                    </div>

                                    {activeDeal.scheduled_post_time && (
                                        <div className="flex items-center gap-2 text-xs text-muted p-2 bg-white/5 rounded-lg">
                                            <Calendar size={14} className="text-blue-400" />
                                            <span>Scheduled for: <span className="text-white font-medium">{new Date(activeDeal.scheduled_post_time).toLocaleString()}</span></span>
                                        </div>
                                    )}

                                    {!showRevisionInput ? (
                                        <div className="grid grid-cols-2 gap-3">
                                            <button
                                                onClick={() => setShowRevisionInput(true)}
                                                className="h-11 bg-white/5 border border-white/10 rounded-xl text-[10px] font-bold hover:bg-red-500/10 hover:text-red-400 hover:border-red-500/20 transition-all flex items-center justify-center gap-2"
                                            >
                                                <Ban size={14} /> Request Edits
                                            </button>
                                            <button
                                                onClick={() => handleReviewDraft(true)}
                                                disabled={submitting}
                                                className="h-11 bg-green-500 text-black rounded-xl text-[10px] font-bold hover:bg-green-400 transition-all flex items-center justify-center gap-2"
                                            >
                                                <CheckCircle size={14} /> {submitting ? 'Approving...' : 'Approve & Schedule'}
                                            </button>
                                        </div>
                                    ) : (
                                        <div className="space-y-3 animate-in slide-in-from-top-2 duration-300">
                                            <textarea
                                                value={feedback}
                                                onChange={e => setFeedback(e.target.value)}
                                                placeholder="What needs to be changed?"
                                                className="w-full h-24 glass border border-white/10 rounded-xl p-3 text-sm text-white focus:outline-none focus:border-red-500/50 resize-none"
                                            />
                                            <div className="flex gap-2">
                                                <button onClick={() => setShowRevisionInput(false)} className="flex-1 h-10 bg-white/5 rounded-xl text-[10px] font-bold">Cancel</button>
                                                <button
                                                    onClick={() => handleReviewDraft(false)}
                                                    disabled={submitting || !feedback}
                                                    className="flex-[2] h-10 bg-red-500 text-white rounded-xl text-[10px] font-bold"
                                                >
                                                    Request Revision
                                                </button>
                                            </div>
                                        </div>
                                    )}
                                </div>
                            </div>
                        )}

                        <div className="pt-2">
                            {isAccepted && (isAdvertiser ? activeDeal.channel_owner_username : activeDeal.advertiser_username) ? (
                                <div className="flex gap-2 pt-2">
                                    <button
                                        onClick={() => {
                                            tg?.HapticFeedback.impactOccurred('medium');
                                            const username = isAdvertiser ? activeDeal.channel_owner_username : activeDeal.advertiser_username;
                                            if (username) tg?.openTelegramLink(`https://t.me/${username}`);
                                        }}
                                        className="flex-1 h-12 bg-blue-500/10 border border-blue-500/20 rounded-xl text-sm font-bold active:scale-95 transition-all flex items-center justify-center gap-2 hover:bg-blue-500/20 text-blue-400"
                                    >
                                        <MessageSquare size={18} />
                                        Message
                                    </button>
                                    <button
                                        onClick={() => {
                                            const username = isAdvertiser ? activeDeal.channel_owner_username : activeDeal.advertiser_username;
                                            if (username) {
                                                navigator.clipboard.writeText(`@${username}`);
                                                tg?.HapticFeedback.notificationOccurred('success');
                                                tg?.showAlert('Username copied to clipboard');
                                            }
                                        }}
                                        className="w-12 h-12 bg-white/5 border border-white/10 rounded-xl flex items-center justify-center active:scale-95 transition-all hover:bg-white/10 text-muted"
                                    >
                                        <Copy size={18} />
                                    </button>
                                </div>
                            ) : isAccepted ? (
                                <button
                                    onClick={handleCopyTelegramId}
                                    className="w-full h-10 bg-white/5 border border-white/10 rounded-xl text-xs font-medium active:scale-95 transition-all flex items-center justify-center gap-2 hover:bg-white/10 text-muted mt-2"
                                >
                                    <Copy size={14} />
                                    Copy Telegram ID (Username hidden)
                                </button>
                            ) : null}
                        </div>

                        <div className="space-y-3 pt-4 border-t border-white/10">
                            {isOwner && activeDeal.state === 'pending' && (
                                <div className="space-y-3">
                                    {!showRejectInput ? (
                                        <>
                                            <button
                                                onClick={handleAcceptDeal}
                                                disabled={submitting}
                                                className="w-full h-12 bg-emerald-500 text-black rounded-xl text-sm font-bold active:scale-95 transition-all flex items-center justify-center gap-2 disabled:opacity-50 shadow-lg shadow-emerald-500/20"
                                            >
                                                <CheckCircle2 size={18} />
                                                {submitting ? 'Accepting...' : 'Accept Deal'}
                                            </button>
                                            <button
                                                onClick={() => setShowRejectInput(true)}
                                                className="w-full h-10 bg-white/5 border border-white/10 rounded-xl text-xs font-medium active:scale-95 transition-all flex items-center justify-center gap-2 hover:bg-red-500/10 hover:border-red-500/20 hover:text-red-400"
                                            >
                                                <XCircle size={14} />
                                                Reject Deal
                                            </button>
                                        </>
                                    ) : (
                                        <div className="space-y-3 animate-in slide-in-from-top-2 duration-300">
                                            <textarea
                                                value={rejectionReason}
                                                onChange={e => setRejectionReason(e.target.value)}
                                                placeholder="Reason for rejection (optional)"
                                                className="w-full h-20 glass border border-white/10 rounded-xl p-3 text-sm focus:outline-none focus:border-red-500/50 resize-none"
                                            />
                                            <div className="flex gap-2">
                                                <button
                                                    onClick={() => setShowRejectInput(false)}
                                                    className="flex-1 h-10 bg-white/5 rounded-xl text-xs font-bold"
                                                >
                                                    Cancel
                                                </button>
                                                <button
                                                    onClick={handleRejectDeal}
                                                    disabled={submitting}
                                                    className="flex-[2] h-10 bg-red-500 text-white rounded-xl text-xs font-bold disabled:opacity-50"
                                                >
                                                    {submitting ? 'Rejecting...' : 'Confirm Rejection'}
                                                </button>
                                            </div>
                                        </div>
                                    )}
                                </div>
                            )}

                            {isApplicant && activeDeal.state === 'pending' && (
                                <div className="p-4 bg-sky-500/10 border border-sky-500/20 rounded-xl text-center">
                                    <Clock size={24} className="mx-auto mb-2 text-sky-400" />
                                    <div className="text-sm font-bold text-sky-400">Waiting for Owner Response</div>
                                    <div className="text-[10px] text-muted mt-1">The owner will review your application</div>
                                </div>
                            )}

                            {isAdvertiser && activeDeal.state === 'awaiting_payment' && (
                                <button
                                    onClick={handleMarkPaid}
                                    disabled={submitting}
                                    className="w-full h-12 bg-green-500 text-black rounded-xl text-sm font-bold active:scale-95 transition-all flex items-center justify-center gap-2 disabled:opacity-50"
                                >
                                    <CheckCircle size={18} />
                                    {submitting ? 'Verifying on Chain...' : 'Verify Payment'}
                                </button>
                            )}



                            {activeDeal.state === 'scheduled' && (
                                <div className="p-4 bg-blue-500/10 border border-blue-500/20 rounded-xl text-center animate-pulse">
                                    <Clock size={24} className="mx-auto mb-2 text-blue-400" />
                                    <div className="text-sm font-bold text-blue-400">Scheduled for Auto-Post</div>
                                    <div className="text-[10px] text-muted mt-1">
                                        {activeDeal.scheduled_post_time ? new Date(activeDeal.scheduled_post_time).toLocaleString() : 'Pending...'}
                                    </div>
                                </div>
                            )}

                            {isAdvertiser && activeDeal.state === 'published' && (
                                <button
                                    onClick={handleVerifyPost}
                                    disabled={submitting}
                                    className="w-full h-12 bg-green-500 text-black rounded-xl text-sm font-bold active:scale-95 transition-all flex items-center justify-center gap-2 disabled:opacity-50"
                                >
                                    <Eye size={18} />
                                    {submitting ? 'Processing...' : 'Verify Post & Complete Deal'}
                                </button>
                            )}

                            {(activeDeal.state === 'released' || activeDeal.state === 'completed') && (
                                <div className="p-4 bg-green-500/10 border border-green-500/20 rounded-xl text-center">
                                    <CheckCircle2 size={24} className="mx-auto mb-2 text-green-400" />
                                    <div className="text-sm font-bold text-green-400">Deal Completed!</div>
                                    <div className="text-[10px] text-muted mt-1">Payment released to channel owner</div>
                                </div>
                            )}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}

const getStatusColor = (state: string) => {
    switch (state.toLowerCase()) {
        case 'completed': return 'text-green-400 bg-green-400/10 border-green-400/20';
        case 'awaiting_payment': return 'text-orange-400 bg-orange-400/10 border-orange-400/20';
        case 'payment_received': return 'text-purple-400 bg-purple-400/10 border-purple-400/20';
        case 'creative_submitted': return 'text-blue-400 bg-blue-400/10 border-blue-400/20';
        case 'creative_approved': return 'text-green-400 bg-green-400/10 border-green-400/20';
        case 'scheduled': return 'text-blue-400 bg-blue-400/10 border-blue-400/20';
        case 'published': return 'text-purple-400 bg-purple-400/10 border-purple-400/20';
        case 'rejected': return 'text-red-400 bg-red-400/10 border-red-400/20';
        case 'revision_requested': return 'text-yellow-400 bg-yellow-400/10 border-yellow-400/20';
        default: return 'text-muted bg-white/5 border-white/10';
    }
};
