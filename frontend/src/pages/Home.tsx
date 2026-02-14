import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { Telescope, TrendingUp, PieChart, Wallet } from 'lucide-react';
import { useTelegram } from '../hooks/useTelegram';
import { api } from '../lib/api';
import { AnalyticsCard } from '../components/AnalyticsCard';
import { RecentCampaignsList } from '../components/RecentCampaignsList';
import type { Deal, Channel } from '../types';

export function Home() {
    const navigate = useNavigate();
    useTelegram();
    const [stats, setStats] = useState({
        totalViews: 0,
        activeDeals: 0,
        earnings: 0,
        balance: '0'
    });
    const [recentDeals, setRecentDeals] = useState<Deal[]>([]);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        const fetchData = async () => {
            try {
                const [dealsRes, channelsRes, meRes] = await Promise.all([
                    api.get('/deals/my'),
                    api.get('/channels/my'),
                    api.get('/me')
                ]);

                const deals = dealsRes.data as Deal[];
                const channels = channelsRes.data as Channel[];
                const userData = meRes.data;

                // Calculate Stats - Channel stats are flattened now (reach, subscribers)
                const totalViews = channels.reduce((acc, ch) => acc + (ch.reach || ch.subscribers || 0), 0);
                const activeDeals = deals.filter(d => !['completed', 'cancelled', 'rejected'].includes(d.state)).length;

                // Sort deals by date (newest first)
                const sortedDeals = deals.sort((a, b) =>
                    new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
                );

                setStats({
                    totalViews,
                    activeDeals,
                    earnings: 0, // TODO: Calculate from completed deals or fetch specific endpoint
                    balance: userData.balance_ton || '0'
                });
                setRecentDeals(sortedDeals);
            } catch (error) {
                console.error('Failed to fetch dashboard data:', error);
            } finally {
                setLoading(false);
            }
        };

        fetchData();
    }, []);

    const formatNumber = (num: number) => {
        if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M';
        if (num >= 1000) return (num / 1000).toFixed(1) + 'k';
        return num.toString();
    };

    return (
        <div className="flex flex-col min-h-screen pt-20 pb-24 px-4 space-y-6">

            {/* Analytics Cards Grid */}
            <div className="grid grid-cols-2 gap-4">
                {/* Total Views - Spans full width on small screens if needed, or just 1st */}
                <AnalyticsCard
                    title="Total Views"
                    value={formatNumber(stats.totalViews)}
                    trend="↑ 15% this week"
                    trendColor="green"
                    icon={TrendingUp}
                    color="blue"
                    className="col-span-2"
                />

                <AnalyticsCard
                    title="Active Deals"
                    value={stats.activeDeals}
                    trend={stats.activeDeals > 0 ? "Action needed" : "No active deals"}
                    trendColor={stats.activeDeals > 0 ? "green" : "neutral"}
                    icon={PieChart}
                    color="purple"
                />

                <AnalyticsCard
                    title="Balance"
                    value={`${Number(stats.balance).toFixed(2)} TON`}
                    trend="Wallet Balance"
                    trendColor="neutral"
                    icon={Wallet}
                    color="blue"
                />
            </div>

            {/* Recent Campaigns / Activity */}
            <div className="space-y-4">
                <RecentCampaignsList deals={recentDeals} isLoading={loading} />
            </div>

            {/* Quick Actions (Optional, but good for empty states) */}
            {recentDeals.length === 0 && !loading && (
                <div className="p-6 rounded-2xl glass border border-white/5 text-center space-y-3">
                    <div className="w-12 h-12 rounded-full bg-white/5 flex items-center justify-center mx-auto text-blue-400">
                        <Telescope size={24} />
                    </div>
                    <div>
                        <h3 className="font-bold text-white">Start Exploring</h3>
                        <p className="text-xs text-white/50 px-8">
                            Find channels to advertise on or register your own to start earning.
                        </p>
                    </div>
                    <button
                        onClick={() => navigate('/explorer')}
                        className="px-6 py-2 rounded-xl bg-blue-500 hover:bg-blue-600 text-white font-bold text-sm transition-colors"
                    >
                        Explore Channels
                    </button>
                </div>
            )}
        </div>
    );
}
