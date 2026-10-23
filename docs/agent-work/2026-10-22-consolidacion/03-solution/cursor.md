# Eiri — consolidación de cursor

Estado: implementación terminada en archivos propietarios; validación integrada corresponde a Lain/Mika.

## Archivos modificados

- `crates/navi-state/src/cursor.rs`: el origen físico inicial nunca cambia; watchdog de 60 s ahora mide ausencia de movimientos aceptados, no duración total del gesto.
- `crates/navi-state/src/reactions.rs`: política fija LayerStack v2 para foco, ClickPulse para clic y DragConnector para arrastre, ignorando selectores obsoletos. Cooldown de inicio y actualización del watchdog separados; los movimientos continuos no caducan a los 60 s.
- `crates/navi-platform/src/pointer_reaction.rs`: eventos de drag reciben límites físicos del escritorio virtual real con GetSystemMetrics; DPI 96 para que una unidad del canvas sea un píxel físico, estable al cruzar monitores de distinto DPI. Clic conserva DPI/área útil de su monitor.
- `crates/navi-platform/src/surface.rs`: sustituido viewport 640x360 móvil por canvas estable del escritorio virtual para drag. Coordenadas de ambos extremos se trasladan al canvas sin clamp ni límite de distancia. El canvas se reutiliza en movimientos y se libera por la ruta previa al soltar/vencer.
- `crates/navi-platform/src/cursor_qa.rs`: chequeo sintético nativo de reutilización del canvas y reconstrucción exacta del origen/extremo después de un desplazamiento hasta el borde del escritorio. No inyecta entrada externa.

## Contratos para integración

No requiere campos nuevos en App ni modificaciones de overlay. `DesktopEvent.work_area` significa para drag la envolvente física del escritorio virtual; para clic/foco mantiene semántica anterior. El adaptador fija drag DPI 96. SurfaceManager es puro y sigue soportando otros DPI en eventos sintéticos. El HWND decorativo, passthrough/noactivate y watchdog de 1500 ms sin actualización se conservan.

Lain debe retirar los selectores `click_family`/`drag_family`/`focus_family` del menú; se mantienen campos de configuración para minimizar ruptura de contratos, pero el motor ya no permite familias alternativas. Pruebas antiguas en **focus_reaction.rs, propiedad de Lain**, requieren adaptar `alternate_pointer_family_uses_aspect_correct_placement` (ahora debe exigir ClickPulse, no aspecto 1.8) y el final de `cursor_anchors_remain_physical_even_with_a_remote_catalog` (ahora DragConnector aunque config pida StackV2). El test del viewport fijo sigue pasando al conservar tamaño para la misma work_area.

## Evidencia

- `rustfmt --edition 2021` sólo los cinco archivos propietarios: ejecutado, exitoso.
- Primera corrida `cargo test --offline -p navi-state`: 27/28; detectó que refrescar last_drag alteraba cooldown. Corregido con `drag_updated` independiente.
- Segunda corrida `cargo test --offline -p navi-state`: **30 tests aprobados**, incluyendo nuevas pruebas de origen negativo, movimiento de 11 250 px, gesto continuo de 120 s y rechazo de selectores antiguos. Incluye dos pruebas de entrada compacta incorporadas concurrentemente por el otro propietario.
- `cargo test --offline -p navi-platform surface::tests`: bloqueado temporalmente por integración concurrente en holograms.rs (ActionProjection retirada), reaction_options.rs (invocation_halo retirada) y verification.rs (user_panel retirada). No es evidencia de ejecución de las pruebas de superficie; volver a correr al integrar.

## Riesgos y pendientes reales

- Canvas BGRA del escritorio virtual completo sólo durante drag: coste de memoria y transferencia proporcional a su envolvente (por ejemplo 7680x2160 ≈63 MiB por bitmap), sin trabajo ni recursos retenidos en reposo. Permite ancla ilimitada respecto de la distancia dentro del escritorio real, con una sola superficie estable. Direct2D/Windows mantienen sus límites de tamaño/asignación; no se eluden ni se promete soporte de geometrías mayores que el backend. No se midió rendimiento GPU/CPU/RAM.
- Geometría entre monitores disjuntos incluye huecos transparentes. El rectángulo virtual incluye barras de tareas, necesario para no perder un origen situado allí. Fullscreen/notificaciones se siguen evaluando para el monitor del extremo; no se añadió observación invasiva.
- Foco, transparencia y multimonitor físico no se validaron manualmente. La prueba nativa modificada aún no ejecutada en esta fase. No se lanzó GUI ni se utilizó Computer Use.
- Release oficial y documentación consolidada corresponden exclusivamente a Lain.
