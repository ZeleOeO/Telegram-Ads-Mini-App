export interface Channel {
    id: number;
    owner_id: number;
    telegram_channel_id: number;
    title?: string;
    username?: string;
    description?: string;
    subscribers: number;
    reach: number;
    language?: string;
    premium_percentage: number;
    enabled_notifications?: number;
    shares_per_post?: number;
    reactions_per_post?: number;
    status: 'active' | 'pending' | 'suspended';
    verified: boolean;
    category?: string;
}

export interface Campaign {
    id: number;
    advertiser_id: number;
    title: string;
    brief: string;
    budget_ton: string; // Decimal comes as string from JSON often
    target_subscribers_min?: number;
    target_subscribers_max?: number;
    target_languages?: string;
    media_urls?: string;
    status: 'active' | 'completed' | 'paused' | 'cancelled';
    categories?: string[];
    created_at: string;
}

export interface Deal {
    id: number;
    campaign_id?: number | null;
    channel_id: number;
    advertiser_id: number;
    channel_owner_id: number;
    owner_id: number;
    applicant_id: number;
    owner_telegram_id: number;
    applicant_telegram_id: number;
    advertiser_telegram_id: number;
    channel_owner_telegram_id: number;
    is_campaign_application: boolean;
    ad_format_id?: number | null;
    state: string;
    deal_type: string;
    price_ton: string | number;
    post_content?: string;
    creative_status?: string;
    payment_status?: string;
    escrow_address?: string;
    created_at: string;
    creative_submitted_at?: string;
    scheduled_post_time?: string;
    actual_post_time?: string;
    post_link?: string;
    rejection_reason?: string;
    edit_request_reason?: string;
    channel_title?: string;
    channel_username?: string;
    advertiser_username?: string;
    channel_owner_username?: string;
    campaign_title?: string;
}


export interface User {
    telegram_id: number;
    username?: string;
    first_name?: string;
    last_name?: string;
    is_advertiser: boolean;
    is_channel_owner: boolean;
    balance_ton: string;
    created_at: string;
}

export interface AdFormat {
    id: number;
    channel_id: number;
    format_name: string;
    price_ton: string | number;
    description?: string;
    duration?: string;
    format_description?: string;
}

export interface CampaignApplication {
    id: number;
    campaign_id: number;
    channel_id: number;
    price_ton: string;
    message?: string;
    status: 'pending' | 'accepted' | 'rejected';
    created_at: string;
    deal_id?: number;
    // Enriched fields
    channel_title?: string;
    channel_username?: string;
    subscribers?: number;
}

export interface BackendError {
    response?: {
        data?: {
            error?: string;
        };
    };
    message?: string;
}

export interface ChannelSuggestion {
    telegram_id: number;
    title: string;
    username?: string;
}
