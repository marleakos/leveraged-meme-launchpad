# Build Locally with Docker

## Quick Start

### 1. Clone the repo on your local machine
```bash
git clone https://github.com/marleakos/leveraged-meme-launchpad.git
cd leveraged-meme-launchpad
```

### 2. Build with Docker
```bash
docker run --rm -v "$(pwd):/workspace" backpackapp/build:v0.30.0 bash -c "cd /workspace && anchor build"
```

### 3. Get the compiled program
```bash
ls -la target/deploy/leveraged_meme.so
```

### 4. Upload to your VPS
```bash
scp target/deploy/leveraged_meme.so user@your-vps-ip:/path/to/workspace/
```

---

## Prerequisites

- Docker installed on your machine
- Git installed
- ~2GB free disk space for Docker image

---

## Troubleshooting

### Docker not installed?
**macOS:**
```bash
brew install --cask docker
```

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install docker.io
sudo usermod -aG docker $USER
# Log out and back in
```

**Windows:**
Download from https://docs.docker.com/desktop/install/windows-install/

### Build fails?
Make sure you have enough disk space:
```bash
docker system df
```

Clean up if needed:
```bash
docker system prune -a
```

---

## Alternative: Build Script

Save this as `build.sh` and run it:

```bash
#!/bin/bash
set -e

echo "Building Leveraged Meme Token Launchpad..."

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "Error: Docker is not running"
    exit 1
fi

# Build
docker run --rm -v "$(pwd):/workspace" backpackapp/build:v0.30.0 bash -c "cd /workspace && anchor build"

echo ""
echo "✅ Build complete!"
echo "Program: target/deploy/leveraged_meme.so"
echo ""
echo "Next steps:"
echo "1. Upload to VPS: scp target/deploy/leveraged_meme.so your-vps:/path/"
echo "2. Deploy: solana program deploy target/deploy/leveraged_meme.so"
```

---

## Deploy to Devnet

Once you have the `.so` file on your VPS:

```bash
solana config set --url devnet
solana program deploy target/deploy/leveraged_meme.so --program-id 9siEsegivtASLpuRHzMC9UEBcCuzeKe8iREadFEZqCAP
```

---

**Need help?** The code is ready, just need to compile it!
