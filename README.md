# DashTUI

![Crates.io MSRV](https://img.shields.io/crates/msrv/dashtui) ![Build Status](https://img.shields.io/github/actions/workflow/status/yourusername/dashtui/rust.yml)

DashTUI es una herramienta de línea de comandos todo-en-uno, diseñada con un dashboard interactivo (TUI) para gestionar proyectos, snippets, tareas y repositorios Git.

## Arquitectura

DashTUI ahora es una aplicación puramente TUI-céntrica. En lugar de ofrecer subcomandos tradicionales (`dashtui init`, etc.), se ha unificado todas sus funciones en un dashboard visual fácil de usar. Al invocar el comando `dashtui`, arranca directamente la interfaz visual interactiva.

## Instalación

Puedes instalar DashTUI directamente desde el repositorio de GitHub con el siguiente comando:

```bash
git clone https://github.com/artur282/DashTUI.git && cd DashTUI && make install
```

O si prefieres usar `cargo` directamente:

```bash
git clone https://github.com/artur282/DashTUI.git && cd DashTUI && cargo install --path .
```

## Desinstalación

Para desinstalar DashTUI de tu sistema, puedes usar el Makefile (si estás en el directorio del proyecto):

```bash
make uninstall
```

O usar `cargo` directamente desde cualquier lugar:

```bash
cargo uninstall dashtui
```

## Uso

Simplemente ejecuta el comando principal para abrir el dashboard:

```bash
dashtui
```

*(Opcional) Si necesitas pasar un nombre personalizado u otros argumentos CLI básicos (según disponibilidad actual), puedes hacerlo, pero la interfaz siempre será TUI:*

```bash
dashtui --name "Mi proyecto"
```

## Funcionalidades del Dashboard

El TUI tiene distintas pestañas para ayudarte en la productividad diaria. Utiliza las flechas `Izquierda` y `Derecha` para cambiar entre pestañas.

### 1. General

Muestra información clave sobre el entorno de trabajo: métricas generales, estado de las tareas y recuento de los pomodoros completados en la sesión actual.

### 2. Tareas

Agrega y da seguimiento a tus recordatorios y "todo" list diarios.

- **`a`**: Agregar nueva tarea
- **`x`**: Alternar estado completado/pendiente
- **`d`**: Eliminar tarea seleccionada
- **`↑` / `↓`**: Navegar entre tareas

### 3. Pomodoro

Cronómetro de concentración integrado para periodos de enfoque de 25 minutos seguidos de descansos cortos y largos.

- **`s`**: Iniciar o pausar el temporizador
- **`r`**: Reiniciar el temporizador

### 4. Scaffold

Potente generador de proyectos. Permite hacer un bootstrap de código base con las mejores prácticas listas para producción (Dockerfile, Makefile, CI).

- **Templates**: `rust-cli`, `rust-api`, `python-fastapi`, `node-express`.
- **`Enter`**: Elegir proyecto -> Solicita nombre de proyecto -> Genera los archivos localmente e inicializa un repo git local.

### 5. Snippets

Manager personal de piezas de código. Se guardan directamente en una DB local embebida y se comunican con tu portapapeles.

- **`a`**: Agregar nuevo snippet. Te pedirá Título, Lenguaje y Descripción, **el código lo leerá de tu portapapeles actual**.
- **`c`**: Copiar el snippet seleccionado al portapapeles.
- **`d`**: Eliminar el snippet seleccionado.
- **`↑` / `↓`**: Navegar entre snippets.

### 6. Git

Monitor y utilidades para tu repositorio local basado en convenciones de commits (Conventional Commits).

- Muestra el historial general, los top contributors y qué archivos están trackeados.
- Visualiza el changelog basado en tus commits locales.
- **`c`**: Limpiar ramas locales ya mergeadas.

## Próximas Funcionalidades

DashTUI sigue en desarrollo activo. Algunas de las características planeadas para futuras versiones incluyen:

- **Más Templates**: Ampliación del catálogo de plantillas para scaffolds con más lenguajes y frameworks populares.
- **Docker Compose**: Soporte nativo para la generación y gestión de proyectos que utilicen `docker-compose`.
- **Compatibilidad con Skills**: Integración con las skills de [skills.sh](https://skills.sh/) para extender la potencia del TUI con scripts y herramientas personalizadas.

## Requisitos

- Rust 1.83 o superior.
- Git (para las funcionalidades de análisis de ramas e inicialización).
- Portapapeles del sistema (X11, Wayland o Windows) configurado para arboard/xclip.

## Licencia

Este proyecto está bajo la licencia MIT.
