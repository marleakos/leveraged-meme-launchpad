#!/bin/bash

# Leveraged Meme Launchpad Deployment Script

set -e

echo "🚀 Leveraged Meme Launchpad Deployment"
echo "======================================"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if anchor is installed
if ! command -v anchor &> /dev/null; then
    echo -e "${RED}❌ Anchor CLI not found. Please install it first.${NC}"
    echo "   npm install -g @coral-xyz/anchor-cli"
    exit 1
fi

# Check if solana CLI is installed
if ! command -v solana &> /dev/null; then
    echo -e "${RED}❌ Solana CLI not found. Please install it first.${NC}"
    exit 1
fi

# Get network from argument
NETWORK=${1:-devnet}

echo -e "${YELLOW}📡 Deploying to: $NETWORK${NC}"

# Set Solana config
solana config set --url $NETWORK

# Check wallet balance
echo -e "${YELLOW}💳 Checking wallet balance...${NC}"
BALANCE=$(solana balance)
echo "   Balance: $BALANCE"

if (( $(echo "$BALANCE < 2" | bc -l) )); then
    echo -e "${RED}❌ Insufficient balance. Need at least 2 SOL.${NC}"
    
    if [ "$NETWORK" = "devnet" ]; then
        echo "   Requesting airdrop..."
        solana airdrop 2
    else
        exit 1
    fi
fi

# Build the program
echo -e "${YELLOW}🔨 Building program...${NC}"
cd programs/leveraged_meme
cargo build-bpf

cd ../..

# Deploy
echo -e "${YELLOW}📤 Deploying program...${NC}"
anchor deploy --provider.cluster $NETWORK

# Get program ID
PROGRAM_ID=$(solana address -k target/deploy/leveraged_meme-keypair.json)
echo -e "${GREEN}✅ Program deployed!${NC}"
echo "   Program ID: $PROGRAM_ID"

# Update Anchor.toml
echo -e "${YELLOW}📝 Updating Anchor.toml...${NC}"
sed -i "s/leveraged_meme = \"[^\"]*\"/leveraged_meme = \"$PROGRAM_ID\"/g" Anchor.toml

# Initialize program
echo -e "${YELLOW}🔧 Initializing program...${NC}"
anchor run initialize --provider.cluster $NETWORK || true

echo ""
echo -e "${GREEN}🎉 Deployment complete!${NC}"
echo ""
echo "Program ID: $PROGRAM_ID"
echo "Network: $NETWORK"
echo ""
echo "Next steps:"
echo "   1. Update frontend with program ID: $PROGRAM_ID"
echo "   2. Run tests: anchor test"
echo "   3. Start frontend: cd app && npm run dev"
