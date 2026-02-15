#!/bin/bash
set -e

echo "Deploying Telegram Ad Mini App..."

echo "Pulling latest code..."
git pull

echo "Rebuilding and restarting containers..."
docker-compose down
docker-compose up -d --build

echo "Deployment complete! Application should be running on port 3000."
echo "Check logs with: docker-compose logs -f app"
