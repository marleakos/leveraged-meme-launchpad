# Leveraged Meme Launchpad Makefile

.PHONY: help build test deploy devnet mainnet frontend clean

# Default target
help:
	@echo "Leveraged Meme Launchpad - Available Commands:"
	@echo ""
	@echo "  make build       - Build the Anchor program"
	@echo "  make test        - Run test suite"
	@echo "  make deploy      - Deploy to devnet"
	@echo "  make devnet      - Full devnet setup"
	@echo "  make mainnet     - Deploy to mainnet (production)"
	@echo "  make frontend    - Start frontend dev server"
	@echo "  make clean       - Clean build artifacts"
	@echo ""

# Build the program
build:
	@echo "🔨 Building program..."
	anchor build
	@echo "✅ Build complete"

# Run tests
test:
	@echo "🧪 Running tests..."
	anchor test
	@echo "✅ Tests complete"

# Deploy to devnet
deploy:
	@echo "🚀 Deploying to devnet..."
	./scripts/deploy.sh devnet
	@echo "✅ Deployed to devnet"

# Full devnet setup
devnet: build test deploy
	@echo "🎉 Devnet setup complete!"
	@echo ""
	@echo "Next steps:"
	@echo "  1. Update app/.env with program ID"
	@echo "  2. Run 'make frontend' to start UI"
	@echo ""

# Deploy to mainnet (BE CAREFUL!)
mainnet:
	@echo "⚠️  WARNING: Deploying to MAINNET!"
	@read -p "Are you sure? (yes/no): " confirm && [ $$confirm = yes ] || exit 1
	./scripts/deploy.sh mainnet
	@echo "✅ Deployed to mainnet"

# Start frontend
frontend:
	@echo "🎨 Starting frontend..."
	cd app && npm install && npm run dev

# Clean build artifacts
clean:
	@echo "🧹 Cleaning..."
	cargo clean
	rm -rf target
	rm -rf app/node_modules
	@echo "✅ Clean complete"

# Generate IDL
idl:
	@echo "📄 Generating IDL..."
	anchor idl init --filepath target/idl/leveraged_meme.json <program_id>
	@echo "✅ IDL generated"

# Verify deployment
verify:
	@echo "🔍 Verifying deployment..."
	solana program show <program_id>
	@echo "✅ Verification complete"

# Format code
fmt:
	@echo "📝 Formatting code..."
	cargo fmt
	cd app && npm run lint -- --fix
	@echo "✅ Formatting complete"

# Check code
lint:
	@echo "🔍 Linting..."
	cargo clippy
	cd app && npm run lint
	@echo "✅ Linting complete"

# Update dependencies
update:
	@echo "📦 Updating dependencies..."
	cargo update
	cd app && npm update
	@echo "✅ Dependencies updated"

# Full CI pipeline
ci: fmt lint build test
	@echo "✅ CI pipeline complete"

# Quick start for new developers
setup:
	@echo "🚀 Setting up development environment..."
	@echo ""
	@echo "1. Installing Rust..."
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
	@echo ""
	@echo "2. Installing Solana CLI..."
	sh -c "$$(curl -sSfL https://release.solana.com/v1.18.0/install)"
	@echo ""
	@echo "3. Installing Anchor..."
	npm install -g @coral-xyz/anchor-cli
	@echo ""
	@echo "4. Installing Node dependencies..."
	cd app && npm install
	@echo ""
	@echo "✅ Setup complete! Run 'make devnet' to deploy."
