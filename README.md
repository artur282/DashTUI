# 🦀 DashTUI

<div align="center">
  <p><strong>Herramienta de línea de comandos todo-en-uno con un dashboard interactivo (TUI) de primer nivel para potenciar tu productividad de desarrollo.</strong></p>
  
  [![Crates.io MSRV](https://img.shields.io/crates/msrv/dashtui?style=flat-square&color=blue)](https://crates.io/crates/dashtui)
  [![Build Status](https://img.shields.io/github/actions/workflow/status/artur282/DashTUI/rust.yml?style=flat-square)](https://github.com/artur282/DashTUI/actions)
  [![Licencia MIT](https://img.shields.io/badge/License-MIT-green.svg?style=flat-square)](https://opensource.org/licenses/MIT)
  [![Hecho con Rust](https://img.shields.io/badge/Made_with-Rust-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
</div>

<br/>

DashTUI es una aplicación puramente TUI-céntrica. Ha sido construida fusionando las herramientas indispensables para cualquier ingeniero de software en un solo entorno manejable por teclado.

Desde la generación de proyectos con mejores prácticas nativos con **Docker Compose**, hasta gestión de snippets, análisis del repositorio local, pomodoro y hasta integración nativa para descubrir **AI Skills**, todo está a una pulsación de tecla.

---

## 🚀 Características Principales

DashTUI ofrece un set completo de herramientas distribuidas a través de sus pestañas, navegables con las flechas `Izquierda` / `Derecha`:

### 📊 1. General
Vista general del entorno de trabajo, métricas generales de las tareas, estado del proyecto y recuento de las sesiones pomodoro completadas.

### ✅ 2. Tareas
Gestor de "to-do" list integrado. Agrega, completa y elimina recordatorios pendientes de tu jornada.
- `a`: Añadir nueva tarea
- `x`: Alternar estado completado
- `d`: Eliminar tarea

### ⏱️ 3. Pomodoro
Cronómetro de concentración nativo directo en tu terminal. Sigue periodos de enfoque con descansos automáticos.
- `s`: Iniciar / Pausar el temporizador
- `r`: Reiniciar la sesión

### 🐳 4. Scaffold (Docker Ready)
Potente generador de proyectos (`bootstrap`). Genera el código base con las mejores prácticas listas para producción.  
**Todos los templates son nativos con Docker Compose**, incluyendo archivo `docker-compose.yml`, `Dockerfile` multi-stage, `Makefile` unificado, y `.gitignore`.
- **Templates**: `rust-cli`, `rust-api`, `python-fastapi`, `python-django` (con PostgreSQL embebido), `node-express`.

### 📝 5. Snippets
Manager personal de fragmentos de código, conectado directamente a la base de datos local embebida y comunicándose con tu portapapeles.
- `a`: Añadir nuevo snippet. Todo el código **se lee automáticamente de tu portapapeles actual**.
- `c`: Copiar el fragmento al portapapeles.

### 🌿 6. Git
Monitor en tiempo real del estado de tu repositorio Git basado en Conventional Commits.
- Contribuciones: Muestra el top contributors y total de commits.
- Ramas: Detecta automáticamente ramas limpias/fuera de uso.
- `c`: Limpiar ramas locales ya mergeadas.

### 🔌 7. Skills (¡NUEVO!)
Integración nativa con **[skills.sh](https://skills.sh/)**. Extiende tu entorno integrando skills y comandos externos fácilmente.
- Realiza búsquedas directamente conectándose al _leaderboard_ del ecosistema Agent.
- `s`: Buscar una skill usando palabras clave o autor.
- `l`: Listar el Leaderboard global actual.
- `Enter`: Instalar inmediatamente mediante `npx skills add` la skill seleccionada.

---

## ⚙️ Instalación

Instalar DashTUI es muy simple, requiere [Rust 1.83+](https://www.rust-lang.org/tools/install) y Git instalados en tu sistema.

**Mediante Make (Recomendado):**
```bash
git clone https://github.com/artur282/DashTUI.git
cd DashTUI
make install
```

**Mediante Cargo directamente:**
```bash
cargo install --git https://github.com/artur282/DashTUI.git
```
*(Opcionalmente clona el repo y usa `cargo install --path .`)*

---

## 🎮 Uso

Simplemente ejecuta el comando principal en cualquier carpeta para abrir el dashboard interactivo:

```bash
dashtui
```

> **Consejo**: La herramienta leerá el nombre de la carpeta actual y cualquier información de Git si te encuentras dentro de un proyecto, así es que siéntete libre de abrir el comando donde estás desarrollando.

---

## 🤝 Soporta el Proyecto

El proyecto está en constante evolución y **buscamos a personas entusiastas del Open Source** y de la terminal para contribuir:

1. **⭐ Dale Estrella al repositorio:** Si la herramienta te ayuda a ser productivo, dar una estrella nos ayuda enormemente a ganar visibilidad.
2. **🐛 Reporta problemas (Issues):** Contribuye reportando bugs o sugiriendo nuevas herramientas que facilitarían tu día.
3. **💻 Abre un Pull Request:** Estamos muy felices de ver nuevos programadores de Rust metiendo las manos al código para agregar plantillas para más frameworks, mejorar el diseño UI/UX de [Ratatui](https://ratatui.rs/), u optimizar el código.
4. **💬 Difunde la herramienta:** Habla con tus colegas sobre DashTUI.

¡Cualquiera es bienvenido a ser parte del código libre!

---

## 📜 Licencia

Este proyecto está distribuido y cubierto bajo la licencia [MIT](https://opensource.org/licenses/MIT). Siéntase libre de usar, modificar y distribuir.
