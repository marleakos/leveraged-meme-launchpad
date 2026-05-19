#!/bin/bash
# Script to push to GitHub and trigger build

echo "Setting up GitHub repository..."

# Initialize git if not already done
if [ ! -d .git ]; then
    git init
    git add .
    git commit -m "Initial commit - Leveraged Meme Token Launchpad"
fi

echo ""
echo "Next steps:"
echo "1. Create a new repository on GitHub (https://github.com/new)"
echo "2. Run: git remote add origin https://github.com/YOUR_USERNAME/leveraged-meme-launchpad.git"
echo "3. Run: git push -u origin main"
echo "4. Go to Actions tab on GitHub to see build progress"
echo "5. Download the artifact when build completes"
echo ""
echo "The workflow will automatically build your program using the backpackapp/build:v0.30.0 image"
