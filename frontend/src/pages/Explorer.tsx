import { useCallback, useEffect, useState } from 'react';
import { Search, Globe, Users, MessageSquare, Megaphone, CheckCircle2, Eye, Sparkles } from 'lucide-react';
import { api } from '../lib/api';
import { useTelegram } from '../hooks/useTelegram';
import { ChannelDetailsModal } from '../components/ChannelDetailsModal';
import { ApplyCampaignModal } from '../components/ApplyCampaignModal';
import type { Channel, Campaign, BackendError } from '../types';

export function Explorer() {
    const [activeTab, setActiveTab] = useState<'channels' | 'campaigns'>('channels');
    const [searchQuery, setSearchQuery] = useState('');
    const [channels, setChannels] = useState<Channel[]>([]);
    const [campaigns, setCampaigns] = useState<Campaign[]>([]);
    const [loading, setLoading] = useState(true);
    const [showFilters, setShowFilters] = useState(false);
    const [filters, setFilters] = useState<Record<string, string | number | undefined>>({});
    const [sortBy, setSortBy] = useState<string>('newest');

    // Modal states
    const [selectedChannelId, setSelectedChannelId] = useState<number | null>(null);
    const [selectedCampaign, setSelectedCampaign] = useState<{ id: number, title: string } | null>(null);

    const { tg } = useTelegram();

    const loadData = useCallback(async () => {
        setLoading(true);
        try {
            const [channelsRes, campaignsRes] = await Promise.all([
                api.get('/channels', { params: { ...filters, sort: sortBy } }),
                api.get('/campaigns')
            ]);
            setChannels(channelsRes.data);
            setCampaigns(campaignsRes.data);
        } catch (err) {
            console.error('Failed to fetch explorer data:', err);
        } finally {
            setLoading(false);
        }
    }, [filters, sortBy]);

    useEffect(() => {
        loadData();
    }, [loadData]);

    const filteredChannels = channels.filter(c =>
        c.title?.toLowerCase().includes(searchQuery.toLowerCase()) ||
        c.username?.toLowerCase().includes(searchQuery.toLowerCase())
    );

    const filteredCampaigns = campaigns.filter(c =>
        c.title?.toLowerCase().includes(searchQuery.toLowerCase()) ||
        c.brief?.toLowerCase().includes(searchQuery.toLowerCase())
    );

    const handleViewChannel = (id: number) => {
        tg?.HapticFeedback.impactOccurred('light');
        setSelectedChannelId(id);
    };

    const handleApplyCampaign = (id: number, title: string) => {
        tg?.HapticFeedback.impactOccurred('medium');
        setSelectedCampaign({ id, title });
    };

    const handleStartDealFromChannel = async (formatId: number) => {
        if (!selectedChannelId) return;

        try {
            await api.post('/deals', {
                channel_id: selectedChannelId,
                ad_format_id: formatId
            });
            tg?.HapticFeedback.notificationOccurred('success');
            tg?.showAlert('Success! Deal started. Check the Deals tab for updates.');
            setSelectedChannelId(null);
        } catch (err: unknown) {
            tg?.HapticFeedback.notificationOccurred('error');
            const error = err as BackendError;
            tg?.showAlert(`Failed to start deal: ${error.response?.data?.error || error.message}`);
        }
    };

    return (
        <div className="space-y-6 pb-24">
            {/* Header & Tabs */}
            <div className="space-y-4 pt-4">
                <div className="flex justify-between items-center px-2">
                    <h2 className="text-2xl font-bold">Explore</h2>
                    <select
                        value={sortBy}
                        onChange={(e) => setSortBy(e.target.value)}
                        className="appearance-none bg-white/5 border border-white/10 rounded-xl px-3 py-1.5 text-xs font-bold text-white/60 focus:outline-none focus:border-blue-500/50"
                    >
                        <option value="newest">Newest</option>
                        <option value="subscribers_desc">Top Subs</option>
                        <option value="reach_desc">Top Reach</option>
                        <option value="premium_desc">Top Premium</option>
                        <option value="price_asc">Best Price</option>
                    </select>
                </div>

                <div className="flex bg-white/5 p-1 rounded-2xl border border-white/5">
                    <button
                        onClick={() => { setActiveTab('channels'); tg?.HapticFeedback.impactOccurred('light'); }}
                        className={`flex-1 h-10 rounded-xl text-sm font-bold flex items-center justify-center gap-2 transition-all ${activeTab === 'channels' ? 'bg-blue-500 text-black shadow-lg shadow-blue-500/20' : 'text-white/40'}`}
                    >
                        <MessageSquare size={16} /> Channels
                    </button>
                    <button
                        onClick={() => { setActiveTab('campaigns'); tg?.HapticFeedback.impactOccurred('light'); }}
                        className={`flex-1 h-10 rounded-xl text-sm font-bold flex items-center justify-center gap-2 transition-all ${activeTab === 'campaigns' ? 'bg-blue-500 text-black shadow-lg shadow-blue-500/20' : 'text-white/40'}`}
                    >
                        <Megaphone size={16} /> Campaigns
                    </button>
                </div>
            </div>

            {/* Search */}
            <div className="flex gap-2">
                <div className="relative group flex-1">
                    <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-white/30 group-focus-within:text-blue-400 transition-colors" size={20} />
                    <input
                        type="text"
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        placeholder={activeTab === 'channels' ? "Search channels..." : "Search campaigns..."}
                        className="w-full h-14 pl-12 pr-4 bg-white/5 border border-white/10 rounded-2xl text-sm focus:outline-none focus:border-blue-500/50 transition-all placeholder:text-white/20"
                    />
                </div>
                <button
                    onClick={() => {
                        tg?.HapticFeedback.impactOccurred('light');
                        setShowFilters(!showFilters);
                    }}
                    className={`h-14 w-14 flex items-center justify-center rounded-2xl border transition-all ${showFilters ? 'bg-blue-500 border-blue-500 text-black' : 'bg-white/5 border-white/10 text-white/40'}`}
                >
                    <Search size={22} className={showFilters ? '' : 'rotate-45'} />
                </button>
            </div>

            {/* Filter Panel */}
            {showFilters && (
                <div className="p-1 bg-white/5 rounded-2xl border border-white/5 overflow-hidden animate-in slide-in-from-top-2 duration-300">
                    <div className="p-4 space-y-6">
                        <div className="flex justify-between items-center px-1">
                            <h4 className="text-[10px] font-bold text-blue-400 uppercase tracking-widest flex items-center gap-2">
                                <Search size={14} /> Advanced Discovery
                            </h4>
                            <button
                                onClick={() => { setFilters({}); loadData(); }}
                                className="text-[10px] font-bold text-white/30 uppercase tracking-widest hover:text-white transition-colors"
                            >
                                Reset All
                            </button>
                        </div>

                        <div className="grid grid-cols-2 gap-4">
                            <div className="space-y-1.5">
                                <label className="text-[10px] text-white/40 uppercase font-bold ml-1">Min Subscribers</label>
                                <div className="relative">
                                    <input
                                        type="number"
                                        value={filters.min_subscribers || ''}
                                        onChange={(e) => setFilters({ ...filters, min_subscribers: parseInt(e.target.value) || undefined })}
                                        placeholder="0"
                                        className="w-full h-11 bg-black/40 border border-white/10 rounded-xl px-4 text-xs text-white focus:outline-none focus:border-blue-500/50 transition-all"
                                    />
                                    <Users size={12} className="absolute right-4 top-1/2 -translate-y-1/2 text-white/20" />
                                </div>
                            </div>
                            <div className="space-y-1.5">
                                <label className="text-[10px] text-white/40 uppercase font-bold ml-1">Min Reach</label>
                                <div className="relative">
                                    <input
                                        type="number"
                                        value={filters.min_reach || ''}
                                        onChange={(e) => setFilters({ ...filters, min_reach: parseInt(e.target.value) || undefined })}
                                        placeholder="0"
                                        className="w-full h-11 bg-black/40 border border-white/10 rounded-xl px-4 text-xs text-white focus:outline-none focus:border-blue-500/50 transition-all"
                                    />
                                    <Globe size={12} className="absolute right-4 top-1/2 -translate-y-1/2 text-white/20" />
                                </div>
                            </div>
                        </div>

                        <div className="grid grid-cols-2 gap-4">
                            <div className="space-y-1.5">
                                <label className="text-[10px] text-white/40 uppercase font-bold ml-1">Category</label>
                                <select
                                    value={filters.category || ''}
                                    onChange={(e) => setFilters({ ...filters, category: e.target.value || undefined })}
                                    className="w-full h-11 bg-black/40 border border-white/10 rounded-xl px-3 text-xs text-white focus:outline-none focus:border-blue-500/50 appearance-none transition-all"
                                >
                                    <option value="">All Categories</option>
                                    <option value="crypto">Crypto</option>
                                    <option value="forex">Forex</option>
                                    <option value="tech">Technology</option>
                                    <option value="news">News</option>
                                    <option value="health">Health</option>
                                    <option value="sports">Sports</option>
                                    <option value="business">Business</option>
                                    <option value="other">Other</option>
                                </select>
                            </div>
                            <div className="space-y-1.5">
                                <label className="text-[10px] text-white/40 uppercase font-bold ml-1">Language</label>
                                <select
                                    value={filters.language || ''}
                                    onChange={(e) => setFilters({ ...filters, language: e.target.value || undefined })}
                                    className="w-full h-11 bg-black/40 border border-white/10 rounded-xl px-3 text-xs text-white focus:outline-none focus:border-blue-500/50 appearance-none transition-all"
                                >
                                    <option value="">Any Language</option>
                                    <option value="EN">English</option>
                                    <option value="RU">Russian</option>
                                    <option value="UZ">Uzbek</option>
                                </select>
                            </div>
                        </div>

                        <button
                            onClick={() => {
                                tg?.HapticFeedback.notificationOccurred('success');
                                loadData();
                            }}
                            className="w-full h-12 bg-blue-500 hover:bg-blue-400 text-black rounded-xl text-xs font-bold active:scale-[0.98] transition-all shadow-lg shadow-blue-500/20"
                        >
                            Apply Selection
                        </button>
                    </div>
                </div>
            )}

            {loading ? (
                <div className="text-center py-20 opacity-30 animate-pulse">Scanning the ecosystem...</div>
            ) : (
                <div className="space-y-4">
                    {activeTab === 'channels' ? (
                        filteredChannels.length === 0 ? (
                            <div className="text-center py-10 text-white/20">No channels found</div>
                        ) : (
                            filteredChannels.map((channel) => (
                                <div key={channel.id} className="glass p-4 rounded-2xl border border-white/5 hover:border-white/10 transition-all group active:scale-[0.98]">
                                    <div className="flex justify-between items-start mb-4">
                                        <div className="flex gap-3">
                                            <div className="w-12 h-12 bg-blue-500 rounded-xl flex items-center justify-center text-xl font-bold text-black shadow-lg shadow-blue-500/10">
                                                {channel.title?.[0] || 'T'}
                                            </div>
                                            <div>
                                                <h4 className="font-bold text-white/90 group-hover:text-blue-400 transition-colors">{channel.title}</h4>
                                                <p className="text-xs text-white/30">@{channel.username || 'private'}</p>
                                            </div>
                                        </div>
                                        <button
                                            onClick={() => handleViewChannel(channel.id)}
                                            className="h-9 px-4 bg-white/5 border border-white/10 rounded-xl text-xs font-bold hover:bg-white/10 transition-all"
                                        >
                                            View
                                        </button>
                                    </div>
                                    <div className="grid grid-cols-2 gap-2 mt-2">
                                        <div className="flex items-center gap-2 bg-white/5 p-1.5 rounded-lg">
                                            <Users size={12} className="text-blue-400" />
                                            <div>
                                                <p className="text-[10px] text-white/40 font-bold uppercase">Subs</p>
                                                <p className="text-xs font-bold">
                                                    {channel.subscribers
                                                        ? (channel.subscribers >= 1000
                                                            ? (channel.subscribers / 1000).toFixed(1) + 'k'
                                                            : channel.subscribers)
                                                        : 0}
                                                </p>
                                            </div>
                                        </div>
                                        <div className="flex items-center gap-2 bg-white/5 p-1.5 rounded-lg">
                                            <Eye size={12} className="text-green-400" />
                                            <div>
                                                <p className="text-[10px] text-white/40 font-bold uppercase">Avg Views</p>
                                                <p className="text-xs font-bold">
                                                    {channel.reach
                                                        ? (channel.reach < 1000
                                                            ? channel.reach
                                                            : (channel.reach / 1000).toFixed(1) + 'k')
                                                        : 0}
                                                </p>
                                            </div>
                                        </div>
                                        {(channel.premium_percentage !== undefined && channel.premium_percentage !== null) && (
                                            <div className="flex items-center gap-2 bg-white/5 p-1.5 rounded-lg">
                                                <Sparkles size={12} className="text-yellow-400" />
                                                <div>
                                                    <p className="text-[10px] text-white/40 font-bold uppercase">Premium Users</p>
                                                    <p className="text-xs font-bold text-yellow-500">{channel.premium_percentage}%</p>
                                                </div>
                                            </div>
                                        )}
                                        <div className="flex items-center gap-2 bg-white/5 p-1.5 rounded-lg">
                                            <Globe size={12} className="text-purple-400" />
                                            <div>
                                                <p className="text-[10px] text-white/40 font-bold uppercase">Lang</p>
                                                <p className="text-xs font-bold">{channel.language || 'Global'}</p>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            ))
                        )
                    ) : (
                        filteredCampaigns.length === 0 ? (
                            <div className="text-center py-10 text-white/20">No active campaigns found</div>
                        ) : (
                            filteredCampaigns.map((campaign) => (
                                <div key={campaign.id} className="glass p-4 rounded-2xl border border-white/5 hover:border-white/10 transition-all">
                                    <div className="flex justify-between items-start mb-3">
                                        <h4 className="font-bold text-lg">{campaign.title}</h4>
                                        <div className="px-3 py-1 bg-green-500/10 text-green-400 rounded-full text-[10px] font-bold border border-green-500/20">
                                            {Number(campaign.budget_ton).toFixed(3)} TON
                                        </div>
                                    </div>
                                    <p className="text-sm text-white/50 mb-4 line-clamp-2">
                                        {campaign.brief}
                                    </p>
                                    <div className="flex justify-between items-center">
                                        <div className="flex items-center gap-2 text-[10px] text-white/30 uppercase font-bold">
                                            <CheckCircle2 size={12} /> Min {campaign.target_subscribers_min?.toLocaleString() || 0} subs
                                        </div>
                                        <button
                                            onClick={() => handleApplyCampaign(campaign.id, campaign.title)}
                                            className="h-9 px-6 bg-blue-500 text-black rounded-xl text-xs font-bold active:scale-95 transition-all shadow-lg shadow-blue-500/10"
                                        >
                                            Apply
                                        </button>
                                    </div>
                                </div>
                            ))
                        )
                    )}
                </div>
            )}

            {/* Modals */}
            {selectedChannelId && (
                <ChannelDetailsModal
                    channelId={selectedChannelId}
                    onClose={() => setSelectedChannelId(null)}
                    onStartDeal={handleStartDealFromChannel}
                />
            )}

            {selectedCampaign && (
                <ApplyCampaignModal
                    campaignId={selectedCampaign.id}
                    campaignTitle={selectedCampaign.title}
                    onClose={() => setSelectedCampaign(null)}
                />
            )}
        </div>
    );
}

