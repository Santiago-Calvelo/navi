# Validación · LayerStack v2, FracturedHalo y cursor · 21 de octubre de 2026

La revisión actual integra el trabajo de los tres subagentes y conserva el foco previamente validado por el usuario. Informe completo, archivos, controles, mediciones y procedimiento manual: [docs/PARALLEL_EFFECTS.md](docs/PARALLEL_EFFECTS.md).

- **41 pruebas unitarias aprobadas**: plataforma 18, renderer 5, estado 18; doc-tests sin fallos.
- `cargo clippy --workspace --all-targets --offline -- -D warnings` y formato: aprobados.
- Release MSVC x64: `target/parallel-build/release/navi-app.exe`. Se usó otro directorio porque la instancia anterior del usuario mantiene bloqueado `target/release/navi-app.exe`; no se terminó ese proceso.
- **56 comprobaciones nativas aprobadas**, `all_passed: true`, en [artifacts/parallel/compact.json](artifacts/parallel/compact.json). El harness prueba ventanas reales propias, alpha, hit test sobre geometría pintada, foco, vencimientos y recursos. Los eventos de política son sintéticos.
- Medidas de CPU/RAM/GPU de ocho etapas en [metrics.json](artifacts/parallel/performance/metrics.json). Todas liberaron las superficies. El reposo diagnóstico dio 0% de CPU y 15,57 MiB de working set; excluye observadores. DWM incluye el escritorio completo.
- La comparación global de foco del benchmark largo dio `false`; no permite atribuir el cambio a NAVI o a una acción externa. No se declara superada esa comprobación. Las pruebas nativas cortas conservaron su foreground.
- La instancia anterior ocupó algunos atajos globales. Antes de probar esta versión, salir de la anterior desde su bandeja.
- **Pendiente manual:** clics/arrastres físicos, apariencia en aplicaciones, DPI mixto real y pantalla completa exclusiva. No se usó Computer Use.

## Registro histórico: integración WindowFocus anterior

Los resultados siguientes corresponden al hito anterior, no al conteo actual. La prueba manual de aquella integración fue confirmada después por el usuario.

## Resultado de esta revisión

- `cargo test --workspace --offline`: **18 pruebas unitarias correctas**, 11 en `navi-platform` y 7 en `navi-state`; sin fallos. Los demás crates y doc-tests también finalizaron correctamente.
- `cargo clippy --workspace --offline -- -D warnings`: correcto.
- `cargo fmt --all -- --check`: correcto.
- `cargo build --release --offline`: correcto, Windows MSVC x64. Ejecutable: `target/release/navi-app.exe`.
- Diagnóstico `--snapshot F:\navi\artifacts\window-focus.bmp`: **40 comprobaciones correctas**, `all_passed: true`. Informe completo en `artifacts/window-focus.json`.
- Arranque `--observer-probe`: instalación real confirmada (`observer_installed: true`). Durante sus doce segundos no hubo eventos admitidos (`events_received: 0`, `reactions_emitted: 0`), por lo que no prueba alternancia entre aplicaciones. Informe: `artifacts/window-focus-observer-startup.json`.

## Qué estaba conectado y qué faltaba

El código anterior ya instalaba `SetWinEventHook` para `EVENT_SYSTEM_FOREGROUND`, encolaba mensajes hacia NAVI y llamaba a `ReactionEngine`. Una reacción admitida iniciaba el `LayerStack` existente mediante el reproductor de demos.

La conexión tenía límites importantes: utilizaba el monitor/DPI del indicador, sobrescribía el anclaje de demos, no retiraba necesariamente la superficie cuando el motor cancelaba una reacción y conservaba reacciones expiradas en el modelo. El fallo de instalación del observador se descartaba silenciosamente.

Esta revisión separa la presentación temporal de foco, completa cancelación y vencimiento, usa la geometría/DPI del destino y deja intactos sesión, indicador y anclaje de las demos. Una notificación obsoleta o de otro tipo no puede iniciar el efecto. La bandeja permite desactivar/rearmar el foco automático, y un fallo del observador queda registrado sin cerrar NAVI.

## Cobertura automatizada

Las pruebas nuevas de lógica cubren:

- Filtrado del tipo de WinEvent, ventana nula, objetos internos y avisos de una ventana que ya perdió el foreground.
- Duplicados sin reiniciar vida útil, cooldown de 700 ms, cancelación del dibujo anterior y ausencia de disparos diferidos.
- Eventos propios que no producen efectos ni consumen cooldown, desactivación/rearme y supresión por pantalla completa.
- Ruta `ReactionEngine → coordinador de foco → SurfaceManager`, sin depender del renderer ni de HWND reales.
- Anclaje y DPI del monitor de destino, coordenadas negativas, ventana pequeña, geometría inválida, indicador persistente y anclaje de demos conservado.
- Sesión/borrador conservados y retorno desde una ventana propia de NAVI a la aplicación anterior.

El diagnóstico nativo añade ocho comprobaciones específicas:

1. Registro y liberación real del WinEvent hook.
2. Evento sintético que dibuja un overlay real en los límites calculados por `SurfaceManager`, con píxeles semitransparentes.
3. Foreground sin cambios, estilos de no activación/transparencia y `WindowFromPoint` que no selecciona el overlay sobre su geometría.
4. Duplicado sin nuevo disparo.
5. Cambio durante cooldown que retira el efecto obsoleto.
6. Nueva reacción admitida después del cooldown.
7. Supresión que elimina la superficie activa.
8. Vencimiento real del temporizador: overlay/renderer liberados, modelo sin reacción activa, indicador visible y ningún temporizador pendiente.

También pasaron las 32 comprobaciones existentes de ventana, texto, borradores, demos, familias visuales, transparencia, liberación de recursos y funcionalidad con efectos desactivados. Se generaron de nuevo las superficies compactas a 96/120/144/192 DPI. No se introdujeron nuevas familias ni observación de clics/arrastres.

## Lo que no se verificó manualmente

**Pendiente:** alternar físicamente entre Explorador, Firefox y VS Code y confirmar apariencia, ubicación percibida, legibilidad y continuidad de la interacción real. No se usó Computer Use, no se eludió su política de URLs y no se modificó el navegador.

La inyección sintética ocurre dentro del coordinador de NAVI; no constituye un cambio real de foco entre procesos. El registro exitoso del hook tampoco demuestra que las aplicaciones concretas del usuario disparen efectos correctos. La comprobación con `WindowFromPoint` y estilos no sustituye una prueba manual de clics sobre todas las aplicaciones.

Los escenarios físicos de múltiples monitores, DPI mixto, apps elevadas, juegos exclusivos y modos de concentración/presentación también quedan pendientes. No se repitieron mediciones de rendimiento en esta revisión; las métricas anteriores dentro de `artifacts/` pertenecen a otros hitos.

Las instrucciones y los resultados esperados están en [docs/WINDOW_FOCUS.md](docs/WINDOW_FOCUS.md).
