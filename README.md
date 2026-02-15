# Telegram Ad Mini App

A comprehensive platform for managing Telegram ad campaigns, channel analytics, and secure escrow payments on the TON blockchain.

## Technologies


![Rust](https://img.shields.io/badge/Rust-1.81%2B-orange?logo=rust&logoColor=white)
![React](https://img.shields.io/badge/React-18.3-blue?logo=react&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-5.5-blue?logo=typescript&logoColor=white)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-15-blue?logo=postgresql&logoColor=white)
![Docker](https://img.shields.io/badge/Docker-24%2B-blue?logo=docker&logoColor=white)
![TON](https://img.shields.io/badge/TON-Blockchain-blue?logo=ton&logoColor=white)

## Project Overview


This application bridges the gap between advertisers and Telegram channel owners by providing a trustless, transparent marketplace for ad deals.

### Architecture & Key Decisions
- **Unified Monolith**: Built as a single deployable unit where the high-performance Rust backend (Axum) serves the static React frontend. This simplifies deployment and reduces latency.
- **MTProto Integration**: Unlike standard bots, we use the `grammers` library to connect as a user client. This unlocks deep analytics (views, shares, languages) unavailable to regular bots.
- **Real On-Chain Escrow**: Payments are secured via custodial wallets on the TON blockchain. Authenticated via `ed25519` keypairs, funds are verified on-chain before any service is rendered.
- **Event-Driven Workflow**: The deal flow utilizes a state machine pattern to manage complex transitions (Draft -> Review -> Scheduled -> Published -> Verified).

### Features


- **Deep Analytics**: Fetch true reach, engagement rates, and audience demographics using MTProto.
- **Ad Marketplace**: Browse channels, filter by niche/price, and initiate deals.
- **Secure Escrow**: Automated TON wallet generation and payment verification.
- **Auto-Posting**: Schedule ads that are automatically posted by the bot.
- **Verification**: The system automatically verifies if the ad was posted and kept up for 24 hours before releasing funds.

### Future Thoughts & Limitations
- **Scaling MTProto**: Currently relies on a single session file. For high scale, a worker pool of sessions would be needed to avoid rate limits.
- **TON Integration**: Currently uses a custodial model. Future improvements could integrate non-custodial smart contracts (TON Connect) for even greater trustlessness.
- **Mobile Support**: The frontend is optimized for Telegram Mini Apps but could be expanded into a standalone PWA.

### AI Contribution
- **Percentage of Code Written by AI**: ~40%. Mostly the UI

## Demo




Try the test bot deployed on Telegram:
[**@ad_market_place_bot**](https://t.me/ad_market_place_bot)

## Prerequisites



- Docker & Docker Compose (Recommended)
- **Optional** - Rust 1.81+ & Node.js 20+ (For local dev)
- A Telegram App `api_id` and `api_hash` from [my.telegram.org](https://my.telegram.org)

## Installation (With Git)


1. Clone the repository:
   ```bash
   git clone https://github.com/ZeleOeO/telegram-ad-mini-app.git
   ```
2. Navigate to the project directory:
   ```bash
   cd telegram-ad-mini-app
   ```
3. Configure Environment:
   ```bash
   cp env.example .env
   # Edit .env with your credentials
   ```

## Running


### Deployment Guide (Railway)

**Recommended Method:**
This app is optimized for [Railway.app](https://railway.app).

1.  **Create Project**: "New Project" -> "Deploy from GitHub repo".
2.  **Add Database**: Add a PostgreSQL service.
3.  **Configure Env Vars** (In App Service -> Variables):
    *   `TELEGRAM_API_ID`, `TELEGRAM_API_HASH`, `TELOXIDE_TOKEN`
    *   `ESCROW_SECRET_KEY`, `TON_NETWORK`
    *   `DATABASE_URL`: **IMPORTANT** - Copy the "Connection URL" from the Postgres service and paste it here manually.
    *   `PORT`: `3000`
4.  **Done**: The app handles HTTPS and database migrations automatically.

### Manual VPS Deployment
If you prefer a VPS (DigitalOcean/Hetzner), use the provided scripts:
1.  `./setup_server.sh` (Installs Docker)
2.  `./deploy.sh` (Runs the app)

### Local Development

**Backend:**
```shell
cargo run
```

**Frontend:**
```shell
cd frontend
npm install
npm run build
```

## Usage


1. Open the Mini App in Telegram via the bot menu or direct link.
2. **Channel Owners**: Register your channel, view analytics, and accept deals.
3. **Advertisers**: Browse the explorer, create campaigns, and fund deals with TON.

## Testing


Run backend tests with:

```shell
cargo test
```

## Steps to Contribute


Contributions are welcome!

1. Open an issue first to discuss the change.
2. Fork the Repository
3. Clone your fork
4. Create a new branch (`git checkout -b feature/amazing-feature`)
5. Make your changes
6. Commit your changes
7. Push to the branch
8. Open a Pull Request
