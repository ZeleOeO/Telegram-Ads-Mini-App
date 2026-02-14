import { useNavigate } from 'react-router-dom';
import { Clock, CheckCircle2, AlertCircle } from 'lucide-react';
import { cn } from '../lib/utils';
import type { Deal } from '../types';

interface RecentCampaignsListProps {
    deals: Deal[];
    isLoading?: boolean;
}

export function RecentCampaignsList({ deals, isLoading }: RecentCampaignsListProps) {
    const navigate = useNavigate();

    if (isLoading) {
        return (
            <div className="space-y-3">
                {[1, 2, 3].map((i) => (
                    <div key={i} className="h-16 rounded-2xl bg-white/5 animate-pulse" />
                ))}
            </div>
        );
    }

    if (deals.length === 0) {
        return (
            <div className="text-center py-8 text-white/40 text-sm">
                No active campaigns found.
            </div>
        );
    }

    const getStatusColor = (status: string) => {
        switch (status) {
            case 'completed':
            case 'published':
                return 'text-green-400 bg-green-400/10 border-green-400/20';
            case 'pending':
            case 'negotiating':
                return 'text-yellow-400 bg-yellow-400/10 border-yellow-400/20';
            case 'rejected':
            case 'cancelled':
                return 'text-red-400 bg-red-400/10 border-red-400/20';
            default:
                return 'text-blue-400 bg-blue-400/10 border-blue-400/20';
        }
    };

    const getStatusIcon = (status: string) => {
        switch (status) {
            case 'completed':
            case 'published':
                return CheckCircle2;
            case 'rejected':
            case 'cancelled':
                return AlertCircle;
            default:
                return Clock;
        }
    };

    return (
        <div className="space-y-3">
            <h3 className="text-xs font-bold text-white/60 uppercase tracking-wider mb-2 px-1">Recent Activity</h3>

            {deals.slice(0, 5).map((deal) => {
                const StatusIcon = getStatusIcon(deal.state);

                return (
                    <button
                        key={deal.id}
                        onClick={() => navigate('/deals', { state: { openDealId: deal.id } })}
                        className="w-full group relative flex items-center justify-between p-4 glass-card hover:bg-white/10 transition-all rounded-2xl border border-white/5"
                    >
                        <div className="flex items-center gap-4">
                            {/* Icon Placeholder or Channel Logo */}
                            <div className={cn(
                                "w-10 h-10 rounded-xl flex items-center justify-center text-lg font-bold border border-white/10",
                                "bg-gradient-to-br from-white/10 to-transparent text-white"
                            )}>
                                {deal.channel_title ? deal.channel_title.charAt(0).toUpperCase() : '#'}
                            </div>

                            <div className="text-left">
                                <h4 className="font-bold text-sm text-white group-hover:text-blue-400 transition-colors">
                                    {deal.campaign_title || deal.channel_title || `Deal #${deal.id}`}
                                </h4>
                                <div className="text-[10px] text-white/50 flex items-center gap-1">
                                    {deal.channel_title || 'Unknown Channel'}
                                    <span className="w-1 h-1 rounded-full bg-white/30" />
                                    {new Date(deal.created_at).toLocaleDateString()}
                                </div>
                            </div>
                        </div>

                        <div className={cn(
                            "px-2.5 py-1 rounded-lg text-[10px] font-bold border flex items-center gap-1.5",
                            getStatusColor(deal.state)
                        )}>
                            <StatusIcon size={10} />
                            <span className="uppercase">{deal.state.replace('_', ' ')}</span>
                        </div>
                    </button>
                );
            })}
        </div>
    );
}
