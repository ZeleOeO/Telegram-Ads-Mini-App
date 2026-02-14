import { useState, useEffect } from 'react';
import { X, Users } from 'lucide-react';
import { api } from '../lib/api';
import { DealDetailsModal } from './DealDetailsModal';
import { useTelegram } from '../hooks/useTelegram';
import type { CampaignApplication } from '../types';

interface CampaignApplicationsModalProps {
    campaignId: number;
    campaignTitle: string;
    onClose: () => void;
}

export function CampaignApplicationsModal({ campaignId, campaignTitle, onClose }: CampaignApplicationsModalProps) {
    useTelegram();
    const [applications, setApplications] = useState<CampaignApplication[]>([]);
    const [loading, setLoading] = useState(true);
    const [submitting, setSubmitting] = useState<number | null>(null);
    const [selectedDealId, setSelectedDealId] = useState<number | null>(null);

    const fetchApplications = async () => {
        try {
            const res = await api.get(`/campaigns/${campaignId}/applications`);
            setApplications(res.data);
        } catch (err) {
            console.error('Failed to fetch applications:', err);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchApplications();
    }, [campaignId]);

    const handleStatusUpdate = async (appId: number, status: 'accepted' | 'rejected') => {
        setSubmitting(appId);
        try {
            await api.put(`/campaigns/applications/${appId}/status`, { status });
            // Refresh applications to get the new deal_id
            await fetchApplications();
        } catch (e) {
            console.error('Failed to update status', e);
            alert('Failed to update application status');
        } finally {
            setSubmitting(null);
        }
    };

    if (selectedDealId) {
        return (
            <DealDetailsModal
                dealId={selectedDealId}
                onClose={() => setSelectedDealId(null)}
            />
        );
    }

    return (
        <div className="fixed inset-0 z-[110] flex items-end sm:items-center justify-center sm:p-4 bg-black/60 backdrop-blur-sm animate-in fade-in duration-200">
            <div className="w-full sm:max-w-md glass rounded-t-2xl sm:rounded-2xl p-6 relative max-h-[90vh] overflow-y-auto font-sans">
                <div className="pb-8">
                    <button
                        onClick={onClose}
                        className="absolute top-4 right-4 text-muted hover:text-white"
                    >
                        <X size={20} />
                    </button>

                    <h3 className="text-lg font-bold mb-1">Applications</h3>
                    <p className="text-[10px] text-muted uppercase tracking-widest mb-6">{campaignTitle}</p>

                    {loading ? (
                        <div className="text-center py-20 opacity-50">Loading applications...</div>
                    ) : applications.length === 0 ? (
                        <div className="text-center py-20 opacity-30 flex flex-col items-center gap-3">
                            <Users size={48} />
                            <p className="text-sm font-medium">No applications yet</p>
                        </div>
                    ) : (
                        <div className="space-y-4">
                            {applications.map(app => (
                                <div key={app.id} className="p-4 bg-white/5 rounded-2xl border border-white/5 space-y-3">
                                    <div className="flex justify-between items-start">
                                        <div>
                                            <div className="font-bold text-sm">{app.channel_title}</div>
                                            <div className="text-[10px] text-blue-400">@{app.channel_username}</div>
                                        </div>
                                        <div className="text-right">
                                            <div className="text-sm font-bold text-green-400">{Number(app.price_ton || 0).toFixed(3)} TON</div>
                                            <div className="text-[9px] text-muted">{app.subscribers?.toLocaleString()} subs</div>
                                        </div>
                                    </div>

                                    {app.message && (
                                        <div className="p-3 glass rounded-xl border border-white/5 text-[11px] text-foreground italic">
                                            "{app.message}"
                                        </div>
                                    )}

                                    {app.status === 'pending' ? (
                                        <div className="flex gap-2">
                                            <button
                                                onClick={() => handleStatusUpdate(app.id, 'accepted')}
                                                disabled={submitting === app.id}
                                                className="flex-1 py-2 bg-green-500/10 text-green-400 rounded-xl font-bold text-xs hover:bg-green-500/20 disabled:opacity-50"
                                            >
                                                {submitting === app.id ? 'Creating Deal...' : 'Accept & Create Deal'}
                                            </button>
                                            <button
                                                onClick={() => handleStatusUpdate(app.id, 'rejected')}
                                                disabled={submitting === app.id}
                                                className="px-3 py-2 bg-red-500/10 text-red-400 rounded-xl font-bold text-xs hover:bg-red-500/20 disabled:opacity-50"
                                            >
                                                Reject
                                            </button>
                                        </div>
                                    ) : app.deal_id ? (
                                        <button
                                            onClick={() => setSelectedDealId(app.deal_id!)}
                                            className="w-full py-2 bg-blue-500/10 text-blue-400 rounded-xl font-bold text-xs hover:bg-blue-500/20 flex items-center justify-center gap-2"
                                        >
                                            <Users size={14} /> View Deal #{app.deal_id}
                                        </button>
                                    ) : (
                                        <div className={`w-full py-2 rounded-xl text-[10px] font-bold text-center border ${app.status === 'accepted' ? 'text-green-400 bg-green-400/10 border-green-400/20' : app.status === 'rejected' ? 'text-red-400 bg-red-400/10 border-red-500/20' : 'text-sky-400 bg-sky-400/10 border-sky-400/20'
                                            }`}>
                                            {app.status.toUpperCase()}
                                        </div>
                                    )}
                                </div>
                            ))}
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
