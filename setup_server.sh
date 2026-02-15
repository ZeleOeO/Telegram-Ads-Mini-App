#!/bin/bash
set -e

GREEN='\033[0;32m'
NC='\033[0m' 

echo -e "${GREEN}Starting Server Setup for Telegram Ad Mini App...${NC}"

echo -e "${GREEN}Updating system packages...${NC}"
sudo apt-get update && sudo apt-get upgrade -y

echo -e "${GREEN}Installing Git and Curl...${NC}"
sudo apt-get install -y git curl

if ! command -v docker &> /dev/null; then
    echo -e "${GREEN}Installing Docker...${NC}"
    curl -fsSL https://get.docker.com -o get-docker.sh
    sudo sh get-docker.sh
    sudo usermod -aG docker $USER
    echo "Docker installed successfully."
else
    echo "Docker is already installed."
fi

echo -e "${GREEN}Installing Docker Compose...${NC}"
sudo apt-get install -y docker-compose-plugin

echo -e "${GREEN}Server Setup Complete!${NC}"
echo "You may need to log out and log back in for docker group changes to take effect."
echo "Next steps:"
echo "1. Clone the repository: git clone https://github.com/your-username/telegram-ad-mini-app.git"
echo "2. CD into it: cd telegram-ad-mini-app"
echo "3. Copy env: cp env.example .env"
echo "4. Edit .env with your credentials"
echo "5. Run: ./deploy.sh"
