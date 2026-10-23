# Mika — revisión independiente de consolidación

Fecha: 2026-10-22. Inspección estática del árbol compartido durante integración activa de Lain; no constituye validación de un release final. Sólo se escribe este documento. Aplicación principal; sin GUI, Computer Use, capturas, procesos externos ni cambios a código.

## Hallazgos concretos

### P2 — El resultado de error no vuelve automáticamente a presencia mínima

Verificado en `crates/navi-platform/src/text_backend.rs:127-143`: una respuesta Error lleva Session a Phase::Error y presenta el compacto; FracturedHalo se inicia correctamente en rojo. Sin embargo, `crates/navi-platform/src/surface.rs:367` y `:405` sólo asignan vencimiento a Responding. El halo libera sus recursos a los 1800 ms pero la tarjeta de error queda abierta indefinidamente, sin borrador, ejecución ni confirmación que la proteja. Esto conserva un límite anterior, pero no satisface la garantía histórica de regresar a presencia mínima al terminar una interacción. Propuesta mínima: incluir Error en la política de respuesta transitoria no protegida y probar éxito/error, borrador y confirmación. Si se decide conservar errores persistentes, documentarlo expresamente como excepción UX pendiente.

### P2 — Fallback de halo permite dibujar fuera del monitor de foco si falla su resolución

Verificado en `crates/navi-platform/src/text_backend.rs:143`: `action_surface::focused()` puede devolver None tras GetMonitorInfoW fallido, pero ActionVisual ya está iniciado. `crates/navi-platform/src/holograms.rs:167` sólo cancela ante cambio de monitor cuando existe action_surface; `:210` usa app.work/app.dpi como fallback. En ese caso el halo puede aparecer en el monitor de NAVI aunque no sea el del foco. No se ha reproducido fallo real de API; el camino estático es verificable. Para cumplir estrictamente «sólo aparece en la pantalla de foco», omitir la manifestación cuando no se puede resolver su monitor; conservar fallback exclusivamente para diagnóstico sintético si necesita ese contrato.

### P2 transitorio — Todavía hay rutas de preview de LayerStack fuera de foco

Estado observado durante limpieza activa del integrador: `lib.rs::preview_effect(STACK)` llama `reaction_options::preview(StackV2)`; WM_HOTKEY 10..=13 y WM_KEYDOWN F11/F12 aún pueden llegar ahí, y preview construye un WindowFocus simulado. No es un cambio real de foco. El menú ya retiró la opción visible, pero quitar sólo etiquetas no satisface «LayerStack v2: sólo cambios de foco». Restringir previews a ejecución explícita de diagnósticos o deshabilitar esos atajos cotidianos. No basta que el evento sintético se llame WindowFocus.

## Verificado por inspección

- `keyboard_entry.rs` reutiliza HWND principal, backend real y edición existente; la ruta normal ya no crea Tu espacio. Módulos históricos user_panel/action_projection permanecen como fuentes sin registro compilado.
- KeyboardInvoke establece Text/Invoked; pregunta por 2000 ms y alterna `_` cada 500 ms. No activa Listening, micrófono ni waveform. Borrador y solicitud/confirmación vigentes se preservan; Enter vacío no dispara demo. El cursor sólo agenda timer 5 con compacto enfocado y vacío; WM_KILLFOCUS lo mata.
- `ReactionEngine` fija StackV2 para foco, ClickPulse para clic y DragConnector para arrastre aunque configuración pida familias anteriores. El menú ya no presenta selectores alternativos. La eliminación de referencias StackV1 se estaba integrando mientras se revisaba; comprobar árbol final y compilación, no este snapshot.
- `text_backend::complete` utiliza FracturedHalo para éxito/error de la única herramienta de acción actual, open_calculator. Error cambia a ERROR.0 en todos los pinceles del halo. ActionVisual cambia seed por evento; ActionSurface resuelve posición y ángulo una vez y conserva monitor durante la vida, cancelándolo si el foco pasa a otro monitor.
- Origen de CursorGesture se conserva desde Down; la duración total del gesto ya no es límite mientras llegan movimientos. El adaptador usa envolvente física del escritorio virtual y DPI 96; el canvas no aplica clamp a los extremos. El renderer dibuja longitud completa y marcas del origen. Transparencia/hit test/noactivate permanecen en el overlay único.
- Sutura queda diagnóstico; SignalField/ContextFragment siguen demos identificadas sin nuevas funciones reales. No se incorporan voz, proveedores ni herramientas nuevas. Los originales experimentales están fuera del diff aprobado; el árbol ya contenía cambios anteriores en experiments, por lo que no se atribuye su autoría ni se declara todo el laboratorio idéntico a HEAD.

## Riesgos y validación pendiente

- El canvas de arrastre ocupa la envolvente completa del escritorio virtual, no sólo la línea. Coste BGRA de 7680×2160: aproximadamente 63 MiB por bitmap; memoria/transferencia aumentan con monitores y huecos. El efecto se libera al terminar/vencer; no hay costo continuo en reposo, pero no se midió CPU/RAM/GPU ni se prueban límites de tamaño Direct2D.
- El watchdog visual de 1500 ms sin movimientos sigue retirando la línea al detener el mouse; el origen se conserva y reaparece al continuar. El watchdog de gesto de 60 s sin movimientos puede finalizarlo. Son políticas de inactividad, no recorte de distancia; consignarlas en validación manual.
- La confirmación de apertura sigue siendo aceptación de creación de proceso calc.exe, no presencia comprobada de una ventana Calculator. No afirmar «Calculadora abierta» sin esa evidencia; no se amplió backend en esta revisión.
- Compilación, Clippy, release canónico y QA final corresponden a Lain una vez estabilizado el árbol. Los documentos progress/architecture/ADR aún describen el hito anterior al momento de esta revisión y requieren actualización consolidada.
- No se validaron teclado físico, foco/clics reales, arrastre entre monitores o DPI mixto, valoración estética ni rendimiento. Tests puros aprobados; resultado abajo.

## Comandos

- Lecturas: AGENTS.md, 00-brief.md, 03-solution/cursor.md, 03-solution/panel.md, docs/progress.md, docs/architecture.md, ADR 0003/0005; git status y búsquedas dirigidas del código de integración.
- `cargo test -p navi-state -p navi-ui --offline --quiet`: **30 state + 1 UI aprobados**, 0 fallos, exit code 0; doc-tests sin casos. No reemplaza QA nativo/manual.
