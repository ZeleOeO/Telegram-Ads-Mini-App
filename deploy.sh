#!/bin/bash
set -e

echo "Deploying Telegram Ad Mini App..."

# Pull latest changes
echo "Pulling latest code..."
git pull

# Build and restart containers clearly
echo "Rebuilding and restarting containers..."
docker-compose down
docker-compose up -d --build

echo "Deployment complete! Application should be running on port 3000."
echo "Check logs with: docker-compose logs -f app"
