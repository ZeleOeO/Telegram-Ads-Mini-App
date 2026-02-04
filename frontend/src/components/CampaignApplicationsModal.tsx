import { useState, useEffect } from 'react';
import { X, CheckCircle, Ban, Users } from 'lucide-react';
import { api } from '../lib/api';
import { useTelegram } from '../hooks/useTelegram';

import type { CampaignApplication } from '../types';

interface CampaignApplicationsModalProps {
    campaignId: number;
    campaignTitle: string;
    onClose: () => void;
}

export function CampaignApplicationsModal({ campaignId, campaignTitle, onClose }: CampaignApplicationsModalProps) {
    const [applications, setApplications] = useState<CampaignApplication[]>([]);
    const [loading, setLoading] = useState(true);
    const [actioning, setActioning] = useState<number | null>(null);
    const { tg } = useTelegram();

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

    const handleAction = async (appId: number, status: 'accepted' | 'rejected') => {
        setActioning(appId);
        try {
            await api.post(`/campaigns/applications/${appId}/status`, { status });
            tg?.HapticFeedback.notificationOccurred('success');
            fetchApplications();
        } catch (err: unknown) {
            console.error('Action failed:', err);
            tg?.HapticFeedback.notificationOccurred('error');
            const error = err as any;
            alert(error.response?.data?.error || 'Failed to update status');
        } finally {
            setActioning(null);
        }
    };

    return (
        <div className="fixed inset-0 z-[110] flex items-end sm:items-center justify-center sm:p-4 bg-black/60 backdrop-blur-sm animate-in fade-in duration-200">
            <div className="w-full sm:max-w-md glass rounded-t-2xl sm:rounded-2xl p-6 relative max-h-[90vh] overflow-y-auto font-sans">
                <div className="pb-8">
                    <button
                        onClick={onClose}
                        className="absolute top-4 right-4 text-white/40 hover:text-white"
                    >
                        <X size={20} />
                    </button>

                    <h3 className="text-lg font-bold mb-1">📩 Applications</h3>
                    <p className="text-[10px] text-white/40 uppercase tracking-widest mb-6">{campaignTitle}</p>

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
                                            <div className="text-sm font-bold text-green-400">{Number(app.price_ton).toFixed(3)} TON</div>
                                            <div className="text-[9px] text-white/40">{app.subscribers?.toLocaleString()} subs</div>
                                        </div>
                                    </div>

                                    {app.message && (
                                        <div className="p-3 bg-black/40 rounded-xl border border-white/5 text-[11px] text-white/70 italic">
                                            "{app.message}"
                                        </div>
                                    )}

                                    <div className="flex gap-2 pt-1">
                                        {app.status === 'pending' ? (
                                            <>
                                                <button
                                                    onClick={() => handleAction(app.id, 'rejected')}
                                                    disabled={actioning !== null}
                                                    className="flex-1 h-10 bg-white/5 hover:bg-red-500/10 hover:text-red-400 border border-white/10 rounded-xl text-[10px] font-bold transition-all flex items-center justify-center gap-2"
                                                >
                                                    <Ban size={14} /> Reject
                                                </button>
                                                <button
                                                    onClick={() => handleAction(app.id, 'accepted')}
                                                    disabled={actioning !== null}
                                                    className="flex-[2] h-10 bg-green-500 text-black rounded-xl text-[10px] font-bold hover:bg-green-400 transition-all flex items-center justify-center gap-2"
                                                >
                                                    <CheckCircle size={14} /> {actioning === app.id ? 'Accepting...' : 'Accept & Start Deal'}
                                                </button>
                                            </>
                                        ) : (
                                            <div className={`w-full py-2 rounded-xl text-[10px] font-bold text-center border ${app.status === 'accepted' ? 'text-green-400 bg-green-400/10 border-green-400/20' : 'text-red-400 bg-red-400/10 border-red-500/20'
                                                }`}>
                                                {app.status.toUpperCase()}
                                            </div>
                                        )}
                                    </div>
                                </div>
                            ))}
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
