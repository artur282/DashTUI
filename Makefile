# Makefile de RustCLI (herramienta de alto rendimiento)

.PHONY: all build check test clean run install clippy check-format doc

# Directorio del target
TARGET_DIR = target/release
BIN_NAME = rustcli

# Compilación default optimizada (LTO + Strip en Cargo.toml)
all: build

build:
	@echo "🦀 Construyendo RustCLI en modo release..."
	@cargo build --release

# Revisión rápida (sin compilación)
check:
	@echo "🔍 Chequeando sintaxis..."
	@cargo check

# Pruebas unitarias completas
test:
	@echo "🧪 Ejecutando suite de test..."
	@cargo test --all-features

# Limpieza del proyecto
clean:
	@echo "🧹 Limpiando artifacts..."
	@cargo clean

# Correr proyecto local (Desarrollo)
run:
	@echo "🏃 Corriendo RustCLI localmente..."
	@cargo run --

# Linter Clippy de las best practices
clippy:
	@echo "🧐 Pasando Clippy Linter..."
	@cargo clippy --all-targets --all-features -- -D warnings

# Verificador de formato
check-format:
	@echo "✨ Chequeando con rustfmt..."
	@cargo fmt --all -- --check

# Documentación del crate
doc:
	@echo "📚 Generando RustDocs..."
	@cargo doc --no-deps --open

# Instalar tool globalmente (~/.cargo/bin)
install:
	@echo "📥 Instalando rustcli en el environment path..."
	@cargo install --path . --force
