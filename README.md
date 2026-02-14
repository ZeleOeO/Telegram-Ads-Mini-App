# Telegram Ad Mini App

---

A comprehensive platform for managing Telegram ad campaigns, channel analytics, and secure escrow payments on the TON blockchain.

## Technologies

---

![Rust](https://img.shields.io/badge/Rust-1.81%2B-orange?logo=rust&logoColor=white)
![React](https://img.shields.io/badge/React-18.3-blue?logo=react&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-5.5-blue?logo=typescript&logoColor=white)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-15-blue?logo=postgresql&logoColor=white)
![Docker](https://img.shields.io/badge/Docker-24%2B-blue?logo=docker&logoColor=white)
![TON](https://img.shields.io/badge/TON-Blockchain-blue?logo=ton&logoColor=white)

## Project Overview

---

This application bridges the gap between advertisers and Telegram channel owners by providing a trustless, transparent marketplace for ad deals.

### Architecture & Key Decisions
- **Unified Monolith**: Built as a single deployable unit where the high-performance Rust backend (Axum) serves the static React frontend. This simplifies deployment and reduces latency.
- **MTProto Integration**: Unlike standard bots, we use the `grammers` library to connect as a user client. This unlocks deep analytics (views, shares, languages) unavailable to regular bots.
- **Real On-Chain Escrow**: Payments are secured via custodial wallets on the TON blockchain. Authenticated via `ed25519` keypairs, funds are verified on-chain before any service is rendered.
- **Event-Driven Workflow**: The deal flow utilizes a state machine pattern to manage complex transitions (Draft -> Review -> Scheduled -> Published -> Verified).

### Features

---

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
- **Percentage of Code Written by AI**: ~99%
- This entire codebase, including backend logic, frontend UI, database schema, and deployment configuration, was generated through iterative prompting with an agentic AI assistant.

## Demo

---

![Dashboard Preview](/Users/zele/.gemini/antigravity/brain/55825d83-cbbe-434d-983e-00fb1e16f11d/telegram_mini_app_dashboard_mockup_1770981321684.png)

Try the test bot deployed on Telegram:
[**@YourTestBotUsername**](https://t.me/YourTestBotUsername) *(Replace with actual bot link)*

## Prerequisites

---

- Docker & Docker Compose (Recommended)
- **Optional** - Rust 1.81+ & Node.js 20+ (For local dev)
- A Telegram App `api_id` and `api_hash` from [my.telegram.org](https://my.telegram.org)

## Installation (With Git)

---

1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/telegram-ad-mini-app.git
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

---

### Deployment Guide

#### 1. Choose a VPS Provider
- **Free Options**:
  - **Oracle Cloud** (Always Free Tier: 4 ARM Cores, 24GB RAM - Highly Recommended)
  - **Google Cloud** (e2-micro: Very limited, good for small tests)
- **Budget Options**:
  - **Hetzner** (~€5/mo: Best performance/price ratio in Europe/US)
  - **DigitalOcean** ($4/mo Droplets)
  - **Vultr** ($2.50/mo IPv6 only, or $5 regular)

#### 2. Server Setup (One-Command)
SSH into your new server and run:
```bash
# Install Docker & Git automatically
curl -sSL https://raw.githubusercontent.com/your-username/telegram-ad-mini-app/main/setup_server.sh | bash
```

#### 3. Deploy Application
```bash
# 1. Clone your repo
git clone https://github.com/your-username/telegram-ad-mini-app.git
cd telegram-ad-mini-app

# 2. Configure Environment
cp env.example .env
nano .env  # Add your API_ID, API_HASH, BOT_TOKEN, etc.

# 3. Launch
./deploy.sh
```
The app will be live at `http://YOUR_SERVER_IP:3000`.

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

---

1. Open the Mini App in Telegram via the bot menu or direct link.
2. **Channel Owners**: Register your channel, view analytics, and accept deals.
3. **Advertisers**: Browse the explorer, create campaigns, and fund deals with TON.

## Testing

---

Run backend tests with:

```shell
cargo test
```

## Steps to Contribute

---

Contributions are welcome!

1. Open an issue first to discuss the change.
2. Fork the Repository
3. Clone your fork
4. Create a new branch (`git checkout -b feature/amazing-feature`)
5. Make your changes
6. Commit your changes
7. Push to the branch
8. Open a Pull Request
