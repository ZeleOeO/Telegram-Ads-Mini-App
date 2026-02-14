#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
NC='\033[0m' # No Color

echo -e "${GREEN}Starting Server Setup for Telegram Ad Mini App...${NC}"

# 1. Update System
echo -e "${GREEN}Updating system packages...${NC}"
sudo apt-get update && sudo apt-get upgrade -y

# 2. Install Git and Curl
echo -e "${GREEN}Installing Git and Curl...${NC}"
sudo apt-get install -y git curl

# 3. Install Docker
if ! command -v docker &> /dev/null; then
    echo -e "${GREEN}Installing Docker...${NC}"
    curl -fsSL https://get.docker.com -o get-docker.sh
    sudo sh get-docker.sh
    # Add current user to docker group to avoid using sudo
    sudo usermod -aG docker $USER
    echo "Docker installed successfully."
else
    echo "Docker is already installed."
fi

# 4. Install Docker Compose
echo -e "${GREEN}Installing Docker Compose...${NC}"
sudo apt-get install -y docker-compose-plugin

# 5. Summary
echo -e "${GREEN}Server Setup Complete!${NC}"
echo "You may need to log out and log back in for docker group changes to take effect."
echo "Next steps:"
echo "1. Clone the repository: git clone https://github.com/your-username/telegram-ad-mini-app.git"
echo "2. CD into it: cd telegram-ad-mini-app"
echo "3. Copy env: cp env.example .env"
echo "4. Edit .env with your credentials"
echo "5. Run: ./deploy.sh"
