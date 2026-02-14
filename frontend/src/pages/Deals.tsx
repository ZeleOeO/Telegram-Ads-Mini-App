import { useCallback, useEffect, useState } from 'react';
import { api } from '../lib/api';
import { useTelegram } from '../hooks/useTelegram';
import { Clock, MessageSquare, AlertCircle, RefreshCw, Megaphone, Hash, User, Briefcase } from 'lucide-react';
import { DealDetailsModal } from '../components/DealDetailsModal';
import type { Deal, BackendError } from '../types';

export function Deals() {
    const [deals, setDeals] = useState<Deal[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [selectedDeal, setSelectedDeal] = useState<Deal | null>(null);
    const [roleFilter, setRoleFilter] = useState<'all' | 'owner' | 'applied'>('all');
    const [statusFilter, setStatusFilter] = useState<string>('all');
    const [typeFilter, setTypeFilter] = useState<'all' | 'channel' | 'campaign'>('all');
    const [sortOrder, setSortOrder] = useState<'newest' | 'oldest'>('newest');
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

    const filteredDeals = deals.filter(deal => {
        if (roleFilter === 'owner' && deal.owner_telegram_id !== user?.id) return false;
        if (roleFilter === 'applied' && deal.applicant_telegram_id !== user?.id) return false;

        if (statusFilter !== 'all') {
            if (statusFilter === 'active') {
                if (['pending', 'rejected', 'completed', 'cancelled', 'released'].includes(deal.state)) return false;
            } else if (statusFilter === 'completed') {
                if (!['released', 'completed'].includes(deal.state)) return false;
            } else {
                if (deal.state !== statusFilter) return false;
            }
        }

        if (typeFilter === 'channel' && deal.is_campaign_application) return false;
        if (typeFilter === 'campaign' && !deal.is_campaign_application) return false;

        return true;
    }).sort((a, b) => {
        if (sortOrder === 'newest') {
            return b.id - a.id;
        } else {
            return a.id - b.id;
        }
    });



    return (
        <div className="space-y-6 pb-24">
            <div className="flex flex-col gap-4 pt-4">
                <div className="flex justify-between items-center px-1">
                    <h2 style={{
                        fontSize: '20px',
                        fontWeight: 'bold',
                        color: 'var(--tg-theme-text-color)'
                    }}>My Deals</h2>
                    <button
                        onClick={fetchDeals}
                        className="w-10 h-10 flex items-center justify-center glass rounded-xl active:rotate-180 transition-all duration-500"
                        style={{ color: 'var(--tg-theme-text-color)' }}
                    >
                        <RefreshCw size={18} className={loading ? 'animate-spin' : ''} />
                    </button>
                </div>


                <div className="flex gap-2 overflow-x-auto pb-2 -mx-4 px-4 scrollbar-hide">
                    <select
                        value={roleFilter}
                        onChange={(e) => setRoleFilter(e.target.value as any)}
                        className="h-9 bg-white/5 border border-white/10 rounded-lg px-3 text-xs font-bold focus:outline-none focus:border-blue-500/50 appearance-none whitespace-nowrap"
                        style={{ color: 'var(--tg-theme-text-color)' }}
                    >
                        <option value="all">All Roles</option>
                        <option value="owner">Owned / Received</option>
                        <option value="applied">Applied</option>
                    </select>

                    <select
                        value={sortOrder}
                        onChange={(e) => setSortOrder(e.target.value as 'newest' | 'oldest')}
                        className="h-9 bg-white/5 border border-white/10 rounded-lg px-3 text-xs font-bold focus:outline-none focus:border-blue-500/50 appearance-none whitespace-nowrap"
                        style={{ color: 'var(--tg-theme-text-color)' }}
                    >
                        <option value="newest">Newest First</option>
                        <option value="oldest">Oldest First</option>
                    </select>

                    <select
                        value={statusFilter}
                        onChange={(e) => setStatusFilter(e.target.value)}
                        className="h-9 bg-white/5 border border-white/10 rounded-lg px-3 text-xs font-bold focus:outline-none focus:border-blue-500/50 appearance-none whitespace-nowrap"
                        style={{ color: 'var(--tg-theme-text-color)' }}
                    >
                        <option value="all">All Status</option>
                        <option value="pending">Pending</option>
                        <option value="active">Active (Processing)</option>
                        <option value="awaiting_payment">Awaiting Payment</option>
                        <option value="drafting">Drafting</option>
                        <option value="scheduled">Scheduled</option>
                        <option value="published">Published</option>
                        <option value="completed">Completed</option>
                        <option value="rejected">Rejected</option>
                        <option value="cancelled">Cancelled</option>
                    </select>

                    <select
                        value={typeFilter}
                        onChange={(e) => setTypeFilter(e.target.value as any)}
                        className="h-9 bg-white/5 border border-white/10 rounded-lg px-3 text-xs font-bold focus:outline-none focus:border-blue-500/50 appearance-none whitespace-nowrap"
                        style={{ color: 'var(--tg-theme-text-color)' }}
                    >
                        <option value="all">All Types</option>
                        <option value="channel">Channel Deals</option>
                        <option value="campaign">Campaigns</option>
                    </select>
                </div>
            </div>

            {error && (
                <div style={{
                    padding: '16px',
                    backgroundColor: 'rgba(239, 68, 68, 0.1)',
                    borderRadius: '16px',
                    border: '1px solid rgba(239, 68, 68, 0.2)',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '12px',
                    color: '#ef4444',
                    fontSize: '14px'
                }}>
                    <AlertCircle size={20} />
                    {error}
                </div>
            )}

            {loading ? (
                <div className="space-y-4">
                    {[1, 2, 3].map(i => (
                        <div key={i} className="h-24 glass rounded-2xl animate-pulse" />
                    ))}
                </div>
            ) : filteredDeals.length === 0 ? (
                <div style={{
                    textAlign: 'center',
                    padding: '80px 0',
                    color: 'var(--tg-theme-hint-color)'
                }}>
                    <MessageSquare size={48} style={{ margin: '0 auto 16px' }} />
                    <p style={{ color: 'var(--tg-theme-text-color)' }}>
                        {deals.length === 0 ? 'No active deals yet' : 'No deals match your filters'}
                    </p>
                    {deals.length === 0 && (
                        <p style={{ fontSize: '10px', marginTop: '8px', color: 'var(--tg-theme-hint-color)' }}>
                            Check the Explorer to start collaborating
                        </p>
                    )}
                </div>
            ) : (
                <div className="space-y-3">
                    {filteredDeals.map((deal) => {
                        const isOwner = user && deal.owner_telegram_id === user.id;
                        const isApplicant = user && deal.applicant_telegram_id === user.id;

                        return (
                            <div
                                key={deal.id}
                                onClick={() => handleViewDeal(deal)}
                                className="deal-card p-4 flex items-center justify-between group active:scale-[0.98] transition-all cursor-pointer"
                            >
                                <div className="flex items-center gap-4">
                                    <div style={{
                                        width: '40px',
                                        height: '40px',
                                        backgroundColor: 'rgba(59, 130, 246, 0.1)',
                                        borderRadius: '12px',
                                        display: 'flex',
                                        alignItems: 'center',
                                        justifyContent: 'center',
                                        color: '#3b82f6',
                                        border: '1px solid rgba(59, 130, 246, 0.2)'
                                    }}>
                                        <Clock size={20} />
                                    </div>
                                    <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                                        <div style={{
                                            fontSize: '12px',
                                            fontWeight: 'bold',
                                            color: 'var(--tg-theme-text-color)'
                                        }}>
                                            {deal.channel_username ? `@${deal.channel_username}` : (deal.channel_title || 'Channel')}
                                        </div>
                                        <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flexWrap: 'wrap' }}>
                                            {isOwner && (
                                                <span style={{
                                                    padding: '2px 8px',
                                                    borderRadius: '9999px',
                                                    fontSize: '9px',
                                                    fontWeight: 'bold',
                                                    border: '1px solid rgba(16, 185, 129, 0.2)',
                                                    display: 'flex',
                                                    alignItems: 'center',
                                                    gap: '4px',
                                                    color: '#10b981',
                                                    backgroundColor: 'rgba(16, 185, 129, 0.1)'
                                                }}>
                                                    <User size={10} /> Owner
                                                </span>
                                            )}
                                            {isApplicant && (
                                                <span style={{
                                                    padding: '2px 8px',
                                                    borderRadius: '9999px',
                                                    fontSize: '9px',
                                                    fontWeight: 'bold',
                                                    border: '1px solid rgba(14, 165, 233, 0.2)',
                                                    display: 'flex',
                                                    alignItems: 'center',
                                                    gap: '4px',
                                                    color: '#0ea5e9',
                                                    backgroundColor: 'rgba(14, 165, 233, 0.1)'
                                                }}>
                                                    <Briefcase size={10} /> Applied
                                                </span>
                                            )}
                                            <span style={{
                                                padding: '2px 8px',
                                                borderRadius: '9999px',
                                                fontSize: '9px',
                                                fontWeight: 'bold',
                                                border: deal.is_campaign_application
                                                    ? '1px solid rgba(168, 85, 247, 0.2)'
                                                    : '1px solid rgba(59, 130, 246, 0.2)',
                                                display: 'flex',
                                                alignItems: 'center',
                                                gap: '4px',
                                                color: deal.is_campaign_application ? '#a855f7' : '#3b82f6',
                                                backgroundColor: deal.is_campaign_application
                                                    ? 'rgba(168, 85, 247, 0.1)'
                                                    : 'rgba(59, 130, 246, 0.1)'
                                            }}>
                                                {deal.is_campaign_application ? (
                                                    <><Megaphone size={10} /> Campaign</>
                                                ) : (
                                                    <><Hash size={10} /> Channel</>
                                                )}
                                            </span>
                                            <span style={{
                                                padding: '2px 8px',
                                                borderRadius: '9999px',
                                                fontSize: '9px',
                                                fontWeight: 'bold',
                                                border: '1px solid var(--card-border)',
                                                textTransform: 'uppercase',
                                                letterSpacing: '0.05em',
                                                color: 'var(--tg-theme-hint-color)',
                                                backgroundColor: 'var(--card-bg)'
                                            }}>
                                                {deal.state.replace('_', ' ')}
                                            </span>
                                        </div>
                                    </div>
                                </div>
                                <div style={{ textAlign: 'right', display: 'flex', flexDirection: 'column', gap: '4px' }}>
                                    <div style={{
                                        fontSize: '14px',
                                        fontWeight: 'bold',
                                        color: '#10b981'
                                    }}>
                                        {deal.price_ton ? `${Number(deal.price_ton).toFixed(3)} TON` : 'TBD'}
                                    </div>
                                    <div style={{
                                        fontSize: '10px',
                                        color: 'var(--tg-theme-hint-color)'
                                    }}>View Details</div>
                                </div>
                            </div>
                        );
                    })}
                </div>
            )}

            {selectedDeal && (
                <DealDetailsModal
                    deal={selectedDeal}
                    onClose={() => setSelectedDeal(null)}
                    onRefresh={fetchDeals}
                />
            )}
        </div>
    );
}
