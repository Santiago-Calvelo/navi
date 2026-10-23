# NAVI

NAVI es una presencia computacional nativa para Windows. Su objetivo es que la computadora pueda comunicarse con la persona mediante señales visuales, contexto y respuestas directas, sin convertirse en un dashboard que cubra el escritorio ni en un chatbot superpuesto.

La aplicación mantiene una presencia mínima en reposo y despliega superficies breves cuando existe una interacción, un cambio relevante del entorno o una respuesta que necesita representación visual. Las superficies se integran con el escritorio real, no capturan clics cuando son decorativas y no roban el foco salvo que el usuario invoque explícitamente la interfaz.

## Principios

- La computadora se expresa como NAVI: las respuestas describen resultados reales y evitan lenguaje de intermediario.
- La interfaz es nativa, compacta y transitoria; la consola expandida queda disponible para tareas complejas.
- La animación tiene significado contextual y duración finita. El reposo no ejecuta un bucle visual permanente.
- El estado, la observación del escritorio, la gestión de superficies y el renderizado permanecen separados.
- Las acciones son explícitas, registradas y validadas. No se ejecutan comandos arbitrarios derivados del texto.
- La información del escritorio se consulta bajo demanda y con metadatos mínimos; NAVI no lee títulos, contenido, credenciales ni historiales de las aplicaciones.

## Tecnología

NAVI está construido con Rust 2021, Win32, Direct2D y DirectWrite. No utiliza WebView, Electron, Tauri, motores de juego, servidor ni runtime de JavaScript.

El workspace se divide en estos crates:

| Crate | Responsabilidad |
| --- | --- |
| `navi-app` | Arranque, entrada y diagnósticos |
| `navi-agent` | Mensajes, contexto, router determinista, herramientas y respuestas |
| `navi-state` | Estados, eventos y políticas puras |
| `navi-platform` | HWND, bandeja, foco, observadores, superficies y Windows |
| `navi-render` | Direct2D, DirectWrite y composiciones con alpha |
| `navi-ui` | Geometría, tamaños y hit testing |
| `navi-demo` | Escenarios deterministas para demostración |

El flujo textual es:

```text
Entrada → normalización y contexto → router determinista
       → herramienta registrada → validación y permisos
       → ToolHost → respuesta estructurada → presentación visual
```

Cuando una solicitud no puede resolverse de forma determinista, NAVI devuelve un resultado explícito para una futura integración con un proveedor de inteligencia. La IA, la voz y la autonomía en segundo plano no forman parte del núcleo actual.

## Superficies y reacciones

El indicador `navi_`, la tarjeta compacta y la consola comparten la experiencia principal. Las superficies decorativas usan un overlay transparente y click-through, sin crear un HWND por partícula o línea.

- `LayerStack v2`: cambios reales de ventana activa.
- `FracturedHalo`: resultado de acciones; usa rojo cuando la acción falla.
- `ClickPulse`: clics.
- `DragConnector`: arrastres, conservando el ancla entre monitores.
- `ContextFragment`: información contextual bajo demanda, como aplicaciones abiertas y actividad de CPU.
- `SignalField` y `Sutura`: composiciones disponibles para diagnóstico y futuras decisiones de producto.

Los estudios Astra —Palimpsesto, Nacre, Herbario, Umbral y el laboratorio de Sutura— se conservan en `experiments/visual-genesis/` como investigación visual aislada. No son funcionalidades operativas de NAVI.

## Uso

Requisitos: Windows 10/11 x64, Rust estable con toolchain MSVC, Visual Studio Build Tools y Windows SDK.

Desde PowerShell:

```powershell
Set-Location F:\navi
.\scripts\build-run.ps1
```

El script cierra la instancia oficial de NAVI de forma ordenada, compila en release offline, inicia la aplicación y termina con `OK` cuando todo salió correctamente.

El ejecutable oficial es:

```text
F:\navi\target\release\navi-app.exe
```

Para invocar la interfaz compacta:

- `Ctrl+Alt+Espacio` o `Alt+Espacio` abre y oculta la entrada.
- Clic en el indicador o en la bandeja abre el modo de escritura.
- `Ctrl+Alt+N` alterna la consola expandida.
- `Esc` contrae la superficie sin cerrar NAVI.
- El menú contextual de la bandeja permite configurar reacciones, diagnósticos y salida.

La entrada compacta pregunta «¿Qué necesitás?». Después de un breve intervalo muestra `_` como señal de espera. Las herramientas deterministas actuales cubren ayuda, hora, estado del sistema y apertura segura de la calculadora.

## Desarrollo

Antes de modificar código se revisan `AGENTS.md`, la arquitectura y las decisiones registradas. Los cambios se dividen por funcionalidad, mantienen diffs pequeños y se validan con compilación release. La validación visual, manual y de hardware se realiza sobre Windows real.

Documentación de referencia:

- [AGENTS.md](AGENTS.md): reglas de trabajo y límites de arquitectura.
- [docs/architecture.md](docs/architecture.md): contratos y flujo interno.
- [docs/visual-direction.md](docs/visual-direction.md): dirección estética.
- [docs/validation.md](docs/validation.md): comandos y cobertura de validación.
- [docs/decisions/](docs/decisions/): decisiones arquitectónicas duraderas.
