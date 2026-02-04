import { useCallback, useEffect, useState } from 'react';
import { api } from '../lib/api';
import { useTelegram } from '../hooks/useTelegram';
import { Clock, MessageSquare, AlertCircle, RefreshCw } from 'lucide-react';
import { DealDetailsModal } from '../components/DealDetailsModal';
import type { Deal, BackendError } from '../types';

export function Deals() {
    const [deals, setDeals] = useState<Deal[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [selectedDeal, setSelectedDeal] = useState<Deal | null>(null);
    const { user, tg } = useTelegram();

    const fetchDeals = useCallback(async () => {
        if (!user) return;
        setLoading(true);
        setError(null);
        try {
            const res = await api.get('/deals/my');
            setDeals(res.data);
        } catch (err: unknown) {
            console.error('Failed to fetch deals:', err);
            const error = err as BackendError;
            setError(error.response?.data?.error || error.message || 'Failed to load deals');
        } finally {
            setLoading(false);
        }
    }, [user]);

    useEffect(() => {
        fetchDeals();
    }, [fetchDeals]);

    const handleViewDeal = (deal: Deal) => {
        tg?.HapticFeedback.impactOccurred('light');
        setSelectedDeal(deal);
    };

    // Helper for status colors
    const getStatusStyle = (state: string) => {
        switch (state.toLowerCase()) {
            case 'completed': return 'text-green-400 bg-green-400/10 border-green-400/20';
            case 'awaitingpayment': return 'text-orange-400 bg-orange-400/10 border-orange-400/20';
            case 'creative_submitted': return 'text-blue-400 bg-blue-400/10 border-blue-400/20';
            default: return 'text-white/40 bg-white/5 border-white/10';
        }
    };

    return (
        <div className="space-y-6 pb-24">
            <div className="flex justify-between items-center py-4">
                <h2 className="text-xl font-bold">My Deals</h2>
                <button
                    onClick={fetchDeals}
                    className="w-10 h-10 flex items-center justify-center bg-white/5 rounded-xl border border-white/10 active:rotate-180 transition-all duration-500"
                >
                    <RefreshCw size={18} className={loading ? 'animate-spin' : ''} />
                </button>
            </div>

            {error && (
                <div className="p-4 bg-red-500/10 border border-red-500/20 rounded-2xl flex items-center gap-3 text-red-400 text-sm">
                    <AlertCircle size={20} />
                    {error}
                </div>
            )}

            {loading ? (
                <div className="space-y-4">
                    {[1, 2, 3].map(i => (
                        <div key={i} className="h-24 bg-white/5 rounded-2xl border border-white/5 animate-pulse" />
                    ))}
                </div>
            ) : deals.length === 0 ? (
                <div className="text-center py-20 opacity-30">
                    <MessageSquare size={48} className="mx-auto mb-4" />
                    <p>No active deals yet</p>
                    <p className="text-[10px] mt-2">Check the Explorer to start collaborating</p>
                </div>
            ) : (
                <div className="space-y-3">
                    {deals.map((deal) => (
                        <div
                            key={deal.id}
                            onClick={() => handleViewDeal(deal)}
                            className="p-4 bg-white/5 rounded-2xl border border-white/5 flex items-center justify-between group active:scale-[0.98] transition-all cursor-pointer hover:border-white/20"
                        >
                            <div className="flex items-center gap-4">
                                <div className="w-10 h-10 bg-blue-500/10 rounded-xl flex items-center justify-center text-blue-500 border border-blue-500/20">
                                    <Clock size={20} />
                                </div>
                                <div className="space-y-1">
                                    <div className="text-xs font-bold text-white/90">Deal #{deal.id}</div>
                                    <div className="flex items-center gap-2">
                                        <span className={`px-2 py-0.5 rounded-full text-[9px] font-bold border uppercase tracking-wider ${getStatusStyle(deal.state)}`}>
                                            {deal.state.replace('_', ' ')}
                                        </span>
                                    </div>
                                </div>
                            </div>
                            <div className="text-right space-y-1">
                                <div className="text-sm font-bold text-green-400">{Number(deal.price_ton).toFixed(3)} TON</div>
                                <div className="text-[10px] text-white/20">View Details</div>
                            </div>
                        </div>
                    ))}
                </div>
            )}

            {selectedDeal && (
                <DealDetailsModal
                    deal={selectedDeal}
                    onClose={() => setSelectedDeal(null)}
                />
            )}
        </div>
    );
}

