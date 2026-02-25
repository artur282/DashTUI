//! Comando `init` — Scaffold de proyectos integrados en el TUI.
//!
//! Genera proyectos completos con Docker Compose, Dockerfile, Makefile,
//! .gitignore y README para múltiples lenguajes/frameworks.
//! Todos los templates están basados en Docker Compose como estándar.

use std::fs;
use std::path::Path;

use crate::error::ScaffoldError;

/// Catálogo de templates disponibles para scaffold de proyectos
pub const AVAILABLE_TEMPLATES: &[&str] = &[
    "rust-cli",
    "rust-api",
    "python-fastapi",
    "python-django",
    "node-express",
];

/// Ejecuta el scaffold de un proyecto basándose en el template seleccionado.
///
/// # Argumentos
/// * `template` - Nombre del template a utilizar
/// * `name` - Nombre opcional del proyecto (si no se da, usa el nombre del template)
///
/// # Retorna
/// Los logs de los pasos realizados durante la creación del proyecto
pub fn execute(template: &str, name: Option<&str>) -> Result<Vec<String>, ScaffoldError> {
    if !AVAILABLE_TEMPLATES.contains(&template) {
        return Err(ScaffoldError::TemplateNotFound(template.to_string()));
    }

    let project_name = name.unwrap_or(template);
    let project_path = Path::new(project_name);

    if project_path.exists() {
        return Err(ScaffoldError::DirectoryExists(project_name.to_string()));
    }

    let mut logs = vec![format!(
        "📦 Creando proyecto '{}' ({})",
        project_name, template
    )];

    // Despachar al generador correspondiente según el template elegido
    match template {
        "rust-cli" => create_rust_cli_project(project_path, project_name, &mut logs)?,
        "rust-api" => create_rust_api_project(project_path, project_name, &mut logs)?,
        "python-fastapi" => create_python_fastapi_project(project_path, project_name, &mut logs)?,
        "python-django" => create_django_project(project_path, project_name, &mut logs)?,
        "node-express" => create_node_express_project(project_path, project_name, &mut logs)?,
        _ => return Err(ScaffoldError::TemplateNotFound(template.to_string())),
    }

    // Inicializar repositorio Git en el proyecto generado
    initialize_git(project_path, &mut logs)?;

    logs.push(format!(
        "✅ ¡Proyecto '{}' creado exitosamente!",
        project_name
    ));
    Ok(logs)
}

// ─── Generadores de templates ────────────────────────────────────────────────

/// Genera un proyecto Rust CLI con clap, serde y thiserror.
fn create_rust_cli_project(
    project_path: &Path,
    project_name: &str,
    logs: &mut Vec<String>,
) -> Result<(), ScaffoldError> {
    create_directories(project_path, &["src", "tests"])?;

    let cargo_toml = format!(
        r#"[package]
name = "{project_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = {{ version = "4", features = ["derive"] }}
thiserror = "2"
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
"#
    );
    write_project_file(project_path, "Cargo.toml", &cargo_toml, logs)?;

    let main_rs = r#"use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[arg(short, long)]
    name: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    let name = cli.name.unwrap_or_else(|| "World".to_string());
    println!("Hello, {name}!");
}
"#;
    write_project_file(project_path, "src/main.rs", main_rs, logs)?;

    // Generar archivos comunes con Docker Compose
    create_common_files(project_path, project_name, "rust-cli", logs)?;

    Ok(())
}

/// Genera un proyecto Rust API con actix-web y tokio.
fn create_rust_api_project(
    project_path: &Path,
    project_name: &str,
    logs: &mut Vec<String>,
) -> Result<(), ScaffoldError> {
    create_directories(project_path, &["src", "tests"])?;

    let cargo_toml = format!(
        r#"[package]
name = "{project_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4"
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
tokio = {{ version = "1", features = ["full"] }}
thiserror = "2"
"#
    );
    write_project_file(project_path, "Cargo.toml", &cargo_toml, logs)?;

    let main_rs = r#"use actix_web::{web, App, HttpResponse, HttpServer, Responder};

async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(health))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
"#;
    write_project_file(project_path, "src/main.rs", main_rs, logs)?;
    create_common_files(project_path, project_name, "rust-api", logs)?;

    Ok(())
}

/// Genera un proyecto Python con FastAPI y uvicorn.
fn create_python_fastapi_project(
    project_path: &Path,
    project_name: &str,
    logs: &mut Vec<String>,
) -> Result<(), ScaffoldError> {
    create_directories(project_path, &["app", "tests"])?;

    let pyproject = format!(
        r#"[project]
name = "{project_name}"
version = "0.1.0"
requires-python = ">=3.11"

[project.dependencies]
fastapi = "*"
uvicorn = "*"

[build-system]
requires = ["setuptools"]
build-backend = "setuptools.build_meta"
"#
    );
    write_project_file(project_path, "pyproject.toml", &pyproject, logs)?;

    let main_py = r#"from fastapi import FastAPI

app = FastAPI(title="API", version="0.1.0")


@app.get("/health")
async def health():
    return {"status": "ok"}


if __name__ == "__main__":
    import uvicorn
    uvicorn.run("app.main:app", host="0.0.0.0", port=8000, reload=True)
"#;
    write_project_file(project_path, "app/main.py", main_py, logs)?;
    write_project_file(project_path, "app/__init__.py", "", logs)?;

    create_common_files(project_path, project_name, "python-fastapi", logs)?;

    Ok(())
}

/// Genera un proyecto Python con Django, Gunicorn y PostgreSQL.
///
/// Incluye configuración para docker-compose con servicio de base de datos
/// PostgreSQL, variables de entorno y estructura Django estándar.
fn create_django_project(
    project_path: &Path,
    project_name: &str,
    logs: &mut Vec<String>,
) -> Result<(), ScaffoldError> {
    // Estructura de directorios Django
    let app_dirs = &[
        &format!("{project_name}"),
        "static",
        "templates",
        "apps",
    ];
    create_directories(project_path, app_dirs)?;

    // manage.py — punto de entrada principal de Django
    let manage_py = format!(
        r#"#!/usr/bin/env python
"""Django management script para {project_name}."""
import os
import sys


def main():
    """Punto de entrada para tareas administrativas de Django."""
    os.environ.setdefault("DJANGO_SETTINGS_MODULE", "{project_name}.settings")
    try:
        from django.core.management import execute_from_command_line
    except ImportError as exc:
        raise ImportError(
            "No se pudo importar Django. Asegúrate de que esté instalado "
            "y disponible en la variable de entorno PYTHONPATH."
        ) from exc
    execute_from_command_line(sys.argv)


if __name__ == "__main__":
    main()
"#
    );
    write_project_file(project_path, "manage.py", &manage_py, logs)?;

    // settings.py — configuración principal de Django
    let settings_py = format!(
        r#""""
Configuración de Django para {project_name}.

Usa variables de entorno para configuración sensible (DB, SECRET_KEY).
"""
import os
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
SECRET_KEY = os.environ.get("DJANGO_SECRET_KEY", "change-me-in-production")
DEBUG = os.environ.get("DJANGO_DEBUG", "True").lower() in ("true", "1", "yes")

ALLOWED_HOSTS = os.environ.get("DJANGO_ALLOWED_HOSTS", "*").split(",")

INSTALLED_APPS = [
    "django.contrib.admin",
    "django.contrib.auth",
    "django.contrib.contenttypes",
    "django.contrib.sessions",
    "django.contrib.messages",
    "django.contrib.staticfiles",
]

MIDDLEWARE = [
    "django.middleware.security.SecurityMiddleware",
    "django.contrib.sessions.middleware.SessionMiddleware",
    "django.middleware.common.CommonMiddleware",
    "django.middleware.csrf.CsrfViewMiddleware",
    "django.contrib.auth.middleware.AuthenticationMiddleware",
    "django.contrib.messages.middleware.MessageMiddleware",
    "django.middleware.clickjacking.XFrameOptionsMiddleware",
]

ROOT_URLCONF = "{project_name}.urls"

TEMPLATES = [
    {{
        "BACKEND": "django.template.backends.django.DjangoTemplates",
        "DIRS": [BASE_DIR / "templates"],
        "APP_DIRS": True,
        "OPTIONS": {{
            "context_processors": [
                "django.template.context_processors.debug",
                "django.template.context_processors.request",
                "django.contrib.auth.context_processors.auth",
                "django.contrib.messages.context_processors.messages",
            ],
        }},
    }},
]

WSGI_APPLICATION = "{project_name}.wsgi.application"

# Base de datos PostgreSQL vía Docker Compose
DATABASES = {{
    "default": {{
        "ENGINE": "django.db.backends.postgresql",
        "NAME": os.environ.get("POSTGRES_DB", "{project_name}_db"),
        "USER": os.environ.get("POSTGRES_USER", "postgres"),
        "PASSWORD": os.environ.get("POSTGRES_PASSWORD", "postgres"),
        "HOST": os.environ.get("POSTGRES_HOST", "db"),
        "PORT": os.environ.get("POSTGRES_PORT", "5432"),
    }}
}}

LANGUAGE_CODE = "es"
TIME_ZONE = "America/Caracas"
USE_I18N = True
USE_TZ = True

STATIC_URL = "/static/"
STATIC_ROOT = BASE_DIR / "staticfiles"
STATICFILES_DIRS = [BASE_DIR / "static"]

DEFAULT_AUTO_FIELD = "django.db.models.BigAutoField"
"#
    );
    write_project_file(
        project_path,
        &format!("{project_name}/settings.py"),
        &settings_py,
        logs,
    )?;

    // urls.py — rutas principales
    let urls_py = format!(
        r#""""Configuración de URLs para {project_name}."""
from django.contrib import admin
from django.urls import path
from django.http import JsonResponse


def health(request):
    """Endpoint de salud para verificar que la app está corriendo."""
    return JsonResponse({{"status": "ok"}})


urlpatterns = [
    path("admin/", admin.site.urls),
    path("health/", health, name="health"),
]
"#
    );
    write_project_file(
        project_path,
        &format!("{project_name}/urls.py"),
        &urls_py,
        logs,
    )?;

    // wsgi.py — WSGI entry point para Gunicorn
    let wsgi_py = format!(
        r#""""Configuración WSGI para {project_name}."""
import os
from django.core.wsgi import get_wsgi_application

os.environ.setdefault("DJANGO_SETTINGS_MODULE", "{project_name}.settings")
application = get_wsgi_application()
"#
    );
    write_project_file(
        project_path,
        &format!("{project_name}/wsgi.py"),
        &wsgi_py,
        logs,
    )?;

    // __init__.py del módulo principal
    write_project_file(
        project_path,
        &format!("{project_name}/__init__.py"),
        "",
        logs,
    )?;

    // requirements.txt para pip
    let requirements = "\
django>=5.0
gunicorn>=22.0
psycopg2-binary>=2.9
django-environ>=0.11
";
    write_project_file(project_path, "requirements.txt", requirements, logs)?;

    // .env.example con las variables de entorno
    let env_example = format!(
        r#"# Variables de entorno para {project_name}
DJANGO_SECRET_KEY=change-me-in-production
DJANGO_DEBUG=True
DJANGO_ALLOWED_HOSTS=*
POSTGRES_DB={project_name}_db
POSTGRES_USER=postgres
POSTGRES_PASSWORD=postgres
POSTGRES_HOST=db
POSTGRES_PORT=5432
"#
    );
    write_project_file(project_path, ".env.example", &env_example, logs)?;

    // Generar archivos comunes con Docker Compose (incluye PostgreSQL)
    create_common_files(project_path, project_name, "python-django", logs)?;

    Ok(())
}

/// Genera un proyecto Node.js con Express.
fn create_node_express_project(
    project_path: &Path,
    project_name: &str,
    logs: &mut Vec<String>,
) -> Result<(), ScaffoldError> {
    create_directories(project_path, &["src", "tests"])?;

    let package_json = format!(
        r#"{{
  "name": "{project_name}",
  "version": "0.1.0",
  "main": "src/index.js",
  "scripts": {{
    "start": "node src/index.js",
    "dev": "node --watch src/index.js",
    "test": "node --test tests/"
  }},
  "dependencies": {{
    "express": "^4.18.0"
  }}
}}"#
    );
    write_project_file(project_path, "package.json", &package_json, logs)?;

    let index_js = r#"const express = require('express');
const app = express();
const PORT = process.env.PORT || 3000;

app.use(express.json());

app.get('/health', (req, res) => {
  res.json({ status: 'ok' });
});

app.listen(PORT, '0.0.0.0', () => {
  console.log(`Server running on port ${PORT}`);
});
"#;
    write_project_file(project_path, "src/index.js", index_js, logs)?;
    create_common_files(project_path, project_name, "node-express", logs)?;

    Ok(())
}

// ─── Utilidades de scaffold ──────────────────────────────────────────────────

/// Crea múltiples directorios dentro del path base del proyecto.
fn create_directories(base: &Path, dirs: &[&str]) -> Result<(), ScaffoldError> {
    for dir in dirs {
        fs::create_dir_all(base.join(dir))?;
    }
    Ok(())
}

/// Escribe un archivo dentro del proyecto y registra el log correspondiente.
fn write_project_file(
    base: &Path,
    filename: &str,
    content: &str,
    logs: &mut Vec<String>,
) -> Result<(), ScaffoldError> {
    let path = base.join(filename);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, content)?;
    logs.push(format!("✔ Creado: {}", filename));
    Ok(())
}

/// Genera los archivos comunes del proyecto: Dockerfile, docker-compose.yml,
/// Makefile, .gitignore y README.
///
/// Todos los templates están basados en Docker Compose como estándar de
/// orquestación de contenedores.
fn create_common_files(
    base: &Path,
    project_name: &str,
    template: &str,
    logs: &mut Vec<String>,
) -> Result<(), ScaffoldError> {
    // Dockerfile específico por lenguaje/framework
    let dockerfile = generate_dockerfile(template, project_name);
    write_project_file(base, "Dockerfile", &dockerfile, logs)?;

    // Docker Compose con servicios específicos por template
    let compose = generate_docker_compose(template, project_name);
    write_project_file(base, "docker-compose.yml", &compose, logs)?;

    // Makefile con targets estandarizados + docker compose
    let makefile = generate_makefile(template);
    write_project_file(base, "Makefile", &makefile, logs)?;

    // .gitignore específico por lenguaje
    let gitignore = generate_gitignore(template);
    write_project_file(base, ".gitignore", &gitignore, logs)?;

    // README básico del proyecto
    let readme = format!(
        "# {project_name}\n\n\
         ## Inicio rápido\n\n\
         ```bash\n\
         docker compose up --build\n\
         ```\n\n\
         ## Desarrollo\n\n\
         ```bash\n\
         make dev\n\
         ```\n"
    );
    write_project_file(base, "README.md", &readme, logs)?;

    Ok(())
}

/// Genera el contenido del Dockerfile según el template.
fn generate_dockerfile(template: &str, project_name: &str) -> String {
    match template {
        "rust-cli" => format!(
            r#"# ── Build stage ──
FROM rust:1.83-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# ── Runtime stage ──
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/{project_name} /usr/local/bin/app
CMD ["app"]
"#
        ),
        "rust-api" => format!(
            r#"# ── Build stage ──
FROM rust:1.83-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# ── Runtime stage ──
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/{project_name} /usr/local/bin/app
EXPOSE 8080
CMD ["app"]
"#
        ),
        "python-fastapi" => r#"FROM python:3.12-slim

WORKDIR /app

# Instalar dependencias del sistema
RUN apt-get update && apt-get install -y --no-install-recommends \
    && rm -rf /var/lib/apt/lists/*

# Instalar dependencias Python
COPY pyproject.toml .
RUN pip install --no-cache-dir -e .

COPY . .

EXPOSE 8000
CMD ["uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8000"]
"#
        .to_string(),
        "python-django" => r#"FROM python:3.12-slim

WORKDIR /app

# Instalar dependencias del sistema para PostgreSQL
RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq-dev gcc \
    && rm -rf /var/lib/apt/lists/*

# Instalar dependencias Python
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

# Recolectar archivos estáticos
RUN python manage.py collectstatic --noinput || true

EXPOSE 8000
CMD ["gunicorn", "--bind", "0.0.0.0:8000", "--workers", "3", "--timeout", "120"]
"#
        .to_string(),
        "node-express" => r#"FROM node:20-slim

WORKDIR /app

# Instalar dependencias
COPY package*.json .
RUN npm ci --production

COPY . .

EXPOSE 3000
CMD ["node", "src/index.js"]
"#
        .to_string(),
        _ => String::new(),
    }
}

/// Genera el docker-compose.yml con servicios específicos por template.
///
/// Cada template incluye los servicios necesarios (app, db, redis, etc.)
/// con redes, volúmenes y healthchecks configurados.
fn generate_docker_compose(template: &str, project_name: &str) -> String {
    match template {
        "rust-cli" => format!(
            r#"services:
  app:
    build: .
    container_name: {project_name}
    restart: unless-stopped
    volumes:
      - .:/app
"#
        ),
        "rust-api" => format!(
            r#"services:
  app:
    build: .
    container_name: {project_name}
    ports:
      - "8080:8080"
    restart: unless-stopped
    environment:
      - RUST_LOG=info
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
"#
        ),
        "python-fastapi" => format!(
            r#"services:
  app:
    build: .
    container_name: {project_name}
    ports:
      - "8000:8000"
    restart: unless-stopped
    volumes:
      - .:/app
    environment:
      - PYTHONDONTWRITEBYTECODE=1
      - PYTHONUNBUFFERED=1
    command: uvicorn app.main:app --host 0.0.0.0 --port 8000 --reload
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
"#
        ),
        "python-django" => format!(
            r#"services:
  # ── Aplicación Django ──
  app:
    build: .
    container_name: {project_name}_app
    ports:
      - "8000:8000"
    restart: unless-stopped
    volumes:
      - .:/app
      - static_data:/app/staticfiles
    environment:
      - DJANGO_SECRET_KEY=${{DJANGO_SECRET_KEY:-change-me-in-production}}
      - DJANGO_DEBUG=${{DJANGO_DEBUG:-True}}
      - DJANGO_ALLOWED_HOSTS=${{DJANGO_ALLOWED_HOSTS:-*}}
      - POSTGRES_DB=${{POSTGRES_DB:-{project_name}_db}}
      - POSTGRES_USER=${{POSTGRES_USER:-postgres}}
      - POSTGRES_PASSWORD=${{POSTGRES_PASSWORD:-postgres}}
      - POSTGRES_HOST=db
      - POSTGRES_PORT=5432
    depends_on:
      db:
        condition: service_healthy
    command: >
      sh -c "python manage.py migrate &&
             python manage.py runserver 0.0.0.0:8000"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/health/"]
      interval: 30s
      timeout: 10s
      retries: 5

  # ── Base de datos PostgreSQL ──
  db:
    image: postgres:16-alpine
    container_name: {project_name}_db
    restart: unless-stopped
    volumes:
      - postgres_data:/var/lib/postgresql/data
    environment:
      - POSTGRES_DB=${{POSTGRES_DB:-{project_name}_db}}
      - POSTGRES_USER=${{POSTGRES_USER:-postgres}}
      - POSTGRES_PASSWORD=${{POSTGRES_PASSWORD:-postgres}}
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
    driver: local
  static_data:
    driver: local
"#
        ),
        "node-express" => format!(
            r#"services:
  app:
    build: .
    container_name: {project_name}
    ports:
      - "3000:3000"
    restart: unless-stopped
    volumes:
      - .:/app
      - /app/node_modules
    environment:
      - NODE_ENV=development
      - PORT=3000
    command: npm run dev
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
"#
        ),
        _ => String::new(),
    }
}

/// Genera el Makefile con targets de Docker Compose estandarizados.
///
/// Todos los templates incluyen targets `up`, `down`, `build`, `logs`
/// además de los targets específicos del lenguaje.
fn generate_makefile(template: &str) -> String {
    // Targets comunes de Docker Compose para todos los templates
    let docker_targets = r#"
# ── Docker Compose ──
up:
	docker compose up -d --build

down:
	docker compose down

logs:
	docker compose logs -f

restart:
	docker compose restart

clean-docker:
	docker compose down -v --rmi local
"#;

    let specific = match template {
        "rust-cli" | "rust-api" => {
            r#".PHONY: build dev test lint up down logs restart clean-docker

# ── Desarrollo local ──
build:
	cargo build --release

dev:
	cargo run

test:
	cargo test

lint:
	cargo clippy --all-targets --all-features -- -D warnings

clean:
	cargo clean
"#
        }
        "python-fastapi" => {
            r#".PHONY: dev test lint up down logs restart clean-docker

# ── Desarrollo local ──
dev:
	uvicorn app.main:app --reload --port 8000

test:
	pytest tests/

lint:
	ruff check .
"#
        }
        "python-django" => {
            r#".PHONY: dev test lint migrate shell up down logs restart clean-docker

# ── Desarrollo local ──
dev:
	python manage.py runserver 0.0.0.0:8000

test:
	python manage.py test

lint:
	ruff check .

migrate:
	python manage.py migrate

makemigrations:
	python manage.py makemigrations

shell:
	python manage.py shell

createsuperuser:
	python manage.py createsuperuser

collectstatic:
	python manage.py collectstatic --noinput
"#
        }
        "node-express" => {
            r#".PHONY: install dev start test up down logs restart clean-docker

# ── Desarrollo local ──
install:
	npm install

dev:
	npm run dev

start:
	npm start

test:
	npm test
"#
        }
        _ => "",
    };

    format!("{specific}{docker_targets}")
}

/// Genera el .gitignore específico por lenguaje/framework.
fn generate_gitignore(template: &str) -> String {
    let common =
        "# IDE\n.vscode/\n.idea/\n*.swp\n*.swo\n\n# OS\n.DS_Store\nThumbs.db\n\n# Docker\n.docker/\n\n# Environment\n.env\n\n";

    let specific = match template {
        "rust-cli" | "rust-api" => "/target\nCargo.lock\n",
        "python-fastapi" => "__pycache__/\n*.pyc\n.venv/\n*.egg-info/\ndist/\n",
        "python-django" => {
            "__pycache__/\n*.pyc\n.venv/\n*.egg-info/\ndist/\nstaticfiles/\ndb.sqlite3\nmedia/\n"
        }
        "node-express" => "node_modules/\ndist/\n",
        _ => "",
    };

    format!("{common}{specific}")
}

/// Inicializa un repositorio Git en el directorio del proyecto.
fn initialize_git(project_path: &Path, logs: &mut Vec<String>) -> Result<(), ScaffoldError> {
    if git2::Repository::init(project_path).is_ok() {
        logs.push("🔀 Repositorio Git inicializado".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn execute_should_reject_invalid_template() {
        assert!(execute("invalid", Some("t")).is_err());
    }

    #[test]
    fn execute_should_reject_existing_directory() {
        let temp = TempDir::new().expect("x");
        assert!(execute("rust-cli", Some(temp.path().to_str().unwrap())).is_err());
    }

    #[test]
    fn available_templates_should_include_django() {
        assert!(AVAILABLE_TEMPLATES.contains(&"python-django"));
    }
}
