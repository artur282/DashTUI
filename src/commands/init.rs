//! Comando `init` — Scaffold de proyectos integrados en el TUI.
//!
//! Excluye el uso directo de console prints para respetar Ratatui.
//! Devuelve listas formatadas de los pasos exitosos.

use std::fs;
use std::path::Path;

use crate::error::ScaffoldError;

pub const AVAILABLE_TEMPLATES: &[&str] =
    &["rust-cli", "rust-api", "python-fastapi", "node-express"];

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

    match template {
        "rust-cli" => create_rust_cli_project(project_path, project_name, &mut logs)?,
        "rust-api" => create_rust_api_project(project_path, project_name, &mut logs)?,
        "python-fastapi" => create_python_fastapi_project(project_path, project_name, &mut logs)?,
        "node-express" => create_node_express_project(project_path, project_name, &mut logs)?,
        _ => return Err(ScaffoldError::TemplateNotFound(template.to_string())),
    }

    initialize_git(project_path, &mut logs)?;

    logs.push(format!(
        "✅ ¡Proyecto '{}' creado exitosamente!",
        project_name
    ));
    Ok(logs)
}

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
    create_common_files(project_path, project_name, "rust", logs)?;

    Ok(())
}

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
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
"#;
    write_project_file(project_path, "src/main.rs", main_rs, logs)?;
    create_common_files(project_path, project_name, "rust", logs)?;

    Ok(())
}

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

    create_common_files(project_path, project_name, "python", logs)?;

    Ok(())
}

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

app.listen(PORT, () => {});
"#;
    write_project_file(project_path, "src/index.js", index_js, logs)?;
    create_common_files(project_path, project_name, "node", logs)?;

    Ok(())
}

fn create_directories(base: &Path, dirs: &[&str]) -> Result<(), ScaffoldError> {
    for dir in dirs {
        fs::create_dir_all(base.join(dir))?;
    }
    Ok(())
}

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

fn create_common_files(
    base: &Path,
    project_name: &str,
    language: &str,
    logs: &mut Vec<String>,
) -> Result<(), ScaffoldError> {
    let dockerfile = match language {
        "rust" => {
            "FROM rust:1.83-slim AS builder\nWORKDIR /app\nCOPY . .\nRUN cargo build --release\n\nFROM debian:bookworm-slim\nCOPY --from=builder /app/target/release/app /usr/local/bin/app\nCMD [\"app\"]\n".to_string()
        }
        "python" => {
            "FROM python:3.12-slim\nWORKDIR /app\nCOPY pyproject.toml .\nRUN pip install -e .\nCOPY . .\nCMD [\"uvicorn\", \"app.main:app\", \"--host\", \"0.0.0.0\", \"--port\", \"8000\"]\n".to_string()
        }
        "node" => {
            "FROM node:20-slim\nWORKDIR /app\nCOPY package*.json .\nRUN npm ci --production\nCOPY . .\nCMD [\"node\", \"src/index.js\"]\n".to_string()
        }
        _ => String::new(),
    };
    write_project_file(base, "Dockerfile", &dockerfile, logs)?;

    let compose = format!(
        r#"version: '3.8'
services:
  app:
    build: .
    container_name: {project_name}
    ports:
      - "8080:8080"
    restart: unless-stopped
"#
    );
    write_project_file(base, "docker-compose.yml", &compose, logs)?;

    let makefile = match language {
        "rust" => ".PHONY: build dev test clean\n\nbuild:\n\tcargo build --release\n\ndev:\n\tcargo run\n\ntest:\n\tcargo test\n\nclean:\n\tcargo clean\n\nlint:\n\tcargo clippy --all-targets --all-features -- -D warnings\n".to_string(),
        "python" => ".PHONY: dev test lint\n\ndev:\n\tuvicorn app.main:app --reload --port 8000\n\ntest:\n\tpytest tests/\n\nlint:\n\truff check .\n".to_string(),
        "node" => ".PHONY: dev test lint\n\ninstall:\n\tnpm install\n\ndev:\n\tnpm run dev\n\nstart:\n\tnpm start\n\ntest:\n\tnpm test\n".to_string(),
        _ => String::new(),
    };
    write_project_file(base, "Makefile", &makefile, logs)?;

    fn gitreq(l: &str) -> String {
        let common = "# IDE\n.vscode/\n.idea/\n*.swp\n*.swo\n\n# OS\n.DS_Store\nThumbs.db\n\n";
        let specific = match l {
            "rust" => "/target\nCargo.lock\n",
            "python" => "__pycache__/\n*.pyc\n.venv/\n*.egg-info/\ndist/\n",
            "node" => "node_modules/\ndist/\n.env\n",
            _ => "",
        };
        format!("{common}{specific}")
    }
    write_project_file(base, ".gitignore", &gitreq(language), logs)?;
    write_project_file(base, "README.md", &format!("# {project_name}\n"), logs)?;

    Ok(())
}

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
}
