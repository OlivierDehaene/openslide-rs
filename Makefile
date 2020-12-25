build: ## Build Rust binaries
	cargo build --release

install-brew: ## Install dependencies with brew
	brew install cairo gdk-pixbuf glib jpeg libpng libtiff libxml2 openjpeg

install-apt: ## Install dependencies with apt
	apt install -y libcairo2-dev libgdk-pixbuf2.0-dev libglib2.0-dev libjpeg-turbo8-dev libopenjp2-7-dev libpng-dev libsqlite3-dev libtiff5-dev libxml2-dev libwebp-dev libzstd-dev
	apt install -y build-essential

format: ## Format code
	cargo fmt --all

format-check: ## Check that code is properly formatted
	cargo fmt --all -- --check

lint: ## Lint code
	cargo clippy --workspace

lint-check: ## Check that code is properly linted
	cargo clippy --workspace -- -D warnings

test: ## Run all tests
	cargo test --locked

bench: ## Run benchmarks
	cargo bench