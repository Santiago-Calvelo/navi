# Arquitectura de NAVI

Fecha: 2026-10-22. Refleja el frontend existente, núcleo textual, panel independiente y proyección de acciones. Los resultados de verificación se consolidan en progress.md.

## Workspace existente

Rust 2021, Windows 10/11 x64 y bindings `windows` 0.54.0. No hay runtime web, servidor, dependencias de IA ni runtime async. Los crates son:

| Crate | Contrato |
|---|---|
| navi-app | Entrada de escritorio y selección de diagnósticos por argumentos |
| navi-agent | Mensajes, contexto, router, registro, validación y respuestas; sin dependencias externas |
| navi-state | Session/Event/Phase y políticas puras de reacciones y gestos |
| navi-platform | HWND, mensajes, observadores, bandeja, foco y recursos de Windows |
| navi-render | Direct2D/DirectWrite; DIB BGRA con alpha premultiplicado |
| navi-ui | Mensajes de presentación, tamaños y hit testing |
| navi-demo | Escenarios y señales deterministas |

`navi-platform` coordina estado y render. `SurfaceManager` es una política sin APIs de Windows: consume geometría, DPI y estado, y devuelve planes. `ReactionEngine` decide aceptación, cooldown y duración. El renderer no observa eventos ni decide permisos.

## Flujos existentes

### Variación compartida (Hito 1)

`navi-state::visual::ResolvedVariation` selecciona parámetros puros y reproducibles. `monitor_catalog` aporta área útil física, DPI, exclusiones y cobertura fullscreen; actualiza la topología por mensajes, sin polling. `organic::OrganicPlanner` resuelve ocho candidatos por monitor y recencia acotada. `SurfaceManager` almacena la selección de LayerStack v2 para foco/preview; `ReactionEngine.active` recibe la misma duración. `FocusEffectPlan` transmite variación, escala final y movimiento al overlay; el renderer consume la transformación y el frame sin observar escritorio. No hay selección por frame. Cambios de pantalla/DPI cancelan, no recolocan aleatoriamente. Cursor, otras familias y ActionProjection conservan su camino anterior. Contrato y límites: [ADR 0004](decisions/0004-variacion-organica-por-manifestacion.md).

### Evolución temporal y Sutura (Hito 2)

`TemporalMotion` conserva cinco perfiles con fase, frecuencia (0.7–1.7 ciclos/vida), amplitud, dirección, torsión y desfases de construcción/disolución; limita rotación interna a ±2.5°, desplazamientos a ±8/5 DIP y skew a ±6 DIP antes de ponderar por intensidad. `ResolvedVariation.temporal` almacena parámetros inmutables muestreados al admitir la manifestación. `FocusEffectPlan.style.temporal` los lleva al renderer; éste sólo evalúa progreso y no consulta eventos ni sortea nuevos valores. Las rutas de ActionProjection y cursor usan `None`, conservando su comportamiento. LayerStack libre animado usa 1080–1320 ms; Sutura diagnóstico 1620–1980 ms. Progreso, vencimiento y temporizador 4 consumen la misma duración resuelta. Sin movimiento hay un frame fijo y un despertar de vencimiento, sin ticks de animación.

`OrganicPlanner` compara zonas dentro de cada monitor y luego elige entre monitores elegibles por rango pseudoaleatorio separado, derivado de seed e ID estable; proximidad y recencia ya no priorizan monitores. `Family::Sutura` comparte canvas 540x300 DIP, matriz centrada y overlay existentes. La bandeja permite un preview explícito; `ReactionConfig` y ciclos de familias no la activan cotidianamente. No se importan host/demo ni bucles del laboratorio. La ruta sin catálogo sigue siendo el fallback previo de diagnósticos sintéticos. Véase [ADR 0005](decisions/0005-organicidad-temporal-y-monitores.md).

```text
Compacto: WM_CHAR → KeyboardEntry/Session.input → Enter
Consola: WM_CHAR → Event::Character → Session.input
Enter → TextBackend → Event::BeginRequest → Processing
Worker → Agent → ToolHost → Response → WM_APP+41 → CompleteRequest
CompleteRequest → Responding/Error/Invoked (si existe un borrador nuevo)
Diagnósticos/demos: Event::Submit → Tick (24 frames) → Responding

WinEvent foreground/restauración → DesktopObserver → debounce 120 ms → ReactionEngine
Raw Input → CursorObserver → CursorGesture → ReactionEngine
ReactionEngine → SurfaceManager → overlay Direct2D
```

La interfaz principal alterna Beacon/Compact/Console en un HWND. La decoración comparte otro HWND layered, transparente al hit test y sin activación. No hay HWND por partícula. Los temporizadores de efectos vencen; el reposo no tiene dibujo continuo. El conector de arrastre reutiliza un viewport acotado. Hay una prueba técnica DirectComposition/D3D/DXGI aislada; la ruta habitual sigue siendo Direct2D y UpdateLayeredWindow.

## Núcleo textual implementado

El crate `navi-agent` contiene normalización, contexto, routing, registro y política de ejecución. `ToolHost` tipado aporta las primitivas Windows y permite hosts falsos en tests. El núcleo no importa HWND, renderer ni Windows. `Status`, `Response`, `Action`, `Parameters`, `ToolError`, `Confirmation` y `Timings` definen sus contratos; `NeedsProvider` permite derivar solicitudes futuras sin implementar IA.

```text
Entrada → normalización/contexto → router determinista
    → registro/validación/permisos → ToolHost → respuesta estructurada
    → NeedsProvider (sin proveedor conectado)
```

`text_backend.rs` mantiene una sola solicitud en vuelo. La tarea posee el Agent y lo devuelve por un canal acotado junto con la respuesta. La terminación notifica al HWND mediante WM_APP+41. El temporizador de Processing ya existente sirve como respaldo de recepción durante una tarea; no hay polling en reposo ni espera del worker en el hilo Win32. Al cerrar el proceso se descarta la recepción; no se fuerza la terminación de aplicaciones iniciadas por el usuario. Las herramientas actuales son rápidas; no se ofrece cancelación forzada de una herramienta bloqueada.

`BeginRequest` vacía la entrada enviada y evita que los ticks de demo completen una tarea real. `CompleteRequest` conserva un borrador nuevo y no reabre una superficie contraída explícitamente. Una confirmación protege el panel sin animación ni temporizador de cierre; la vigencia de 60 segundos se valida al responder. `caption`, `headline` y `compact_status` usan el resultado local.

El núcleo limita a 512 caracteres; compacto y consola conservan 120. Contexto de ocho turnos, respuesta visible de 240 y datos estructurados de hasta 16 pares acotados. Permisos y confirmaciones estrictas opcionales se conservan. Calculadora usa destino fijo GetSystemDirectoryW/calc.exe, sin shell ni argumentos derivados del texto; Success significa inicio aceptado, no ventana visible comprobada.

## Entrada compacta y resultados (ADR 0006)

`keyboard_entry.rs` reutiliza el HWND compacto, en el monitor del foco previo. Session.KeyboardEntry mantiene plazo de pregunta y fase de cursor; el renderer dibuja texto y `_`. Timer 5 sólo existe con espera vacía, compacto enfocado; se suspende al perder foco, escribir, enviar, contraer o expandir. Ctrl+Alt+Espacio/Alt+Espacio/clic invocan; Escape conserva borrador. No hay HWND de escritura separado, audio ni onda de escucha en esta ruta.

`action_surface.rs` amplía SurfaceManager con resolución pura de límites/escala/ángulo; el adaptador obtiene sólo área útil y DPI del monitor enfocado. ActionVisual incrementa seed por resultado y mantiene duración finita de 1800 ms. TextBackend inicia FracturedHalo en todos los resultados Success/Error, incluidas hora y estado; Error usa rojo. Se omite si no se resuelve el monitor y se cancela si el foco pasa a otro. Se reutiliza el overlay con prioridad temporal sobre foco/cursor, sin activación ni captura de clics. Al terminar se libera; estático temporal y desactivación se conservan.

ReactionEngine fija LayerStack v2/foco, ClickPulse/clic y DragConnector/arrastre. El drag conserva Down absoluto a DPI 96 y amplía su canvas por bloques de 128 píxeles para incluir ambos extremos, dentro del escritorio virtual. No recorta el origen por distancia ni al cruzar monitores; recursos liberados al finalizar/vencer. El foco no sustituye un drag activo; el observador distingue el HWND decorativo de las superficies propias interactivas. Un error de dibujo cancela el efecto, sin desactivar permanentemente los observadores. Las ventanas maximizadas no se clasifican como fullscreen por sus bordes extendidos. Watchdogs distinguen inactividad y duración total. Sutura/SignalField/ContextFragment siguen sólo diagnóstico. ActionProjection, v1 y Tu espacio tienen fuentes históricas desconectadas de módulos compilados. Véase [ADR 0006](decisions/0006-consolidacion-compacto-y-efectos.md).

Véanse [ADR gráfico](decisions/0001-superficies-nativas.md), [ADR textual](decisions/0002-nucleo-textual-determinista.md), [panel y acciones](decisions/0003-panel-usuario-y-acciones-directas.md) y [progreso](progress.md).
