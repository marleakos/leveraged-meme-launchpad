#!/bin/bash
set -e

echo "=========================================="
echo "Building Leveraged Meme Token Launchpad"
echo "=========================================="
echo ""

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Error: Docker is not running"
    echo ""
    echo "Please start Docker:"
    echo "  - macOS: Open Docker Desktop app"
    echo "  - Linux: sudo systemctl start docker"
    echo "  - Windows: Open Docker Desktop"
    exit 1
fi

echo "✅ Docker is running"
echo ""

# Check disk space
AVAILABLE=$(docker system df --format '{{.Size}}' | head -1)
echo "📦 Docker disk usage: $AVAILABLE"
echo ""

# Build
echo "🔨 Building program..."
echo "This may take 5-10 minutes..."
echo ""

docker run --rm -v "$(pwd):/workspace" backpackapp/build:v0.30.0 bash -c "cd /workspace && anchor build"

echo ""
echo "=========================================="
echo "✅ Build complete!"
echo "=========================================="
echo ""
echo "Program location: target/deploy/leveraged_meme.so"
echo ""
echo "Next steps:"
echo "1. Upload to VPS:"
echo "   scp target/deploy/leveraged_meme.so root@YOUR_VPS_IP:/root/.openclaw2/workspace/leveraged-meme-launchpad/target/deploy/"
echo ""
echo "2. Deploy to devnet:"
echo "   solana program deploy target/deploy/leveraged_meme.so --program-id 9siEsegivtASLpuRHzMC9UEBcCuzeKe8iREadFEZqCAP"
echo ""
