# Estado operativo de NAVI

## Bandeja y defaults visuales — 2026-10-22

El menú contextual de la bandeja conserva el propietario en primer plano lógico y recibe `WM_NULL` después de `TrackPopupMenu`, evitando el cierre inmediato del menú con el HWND de herramienta oculto. Los defaults de reacción son `Intensity::Anomalous` y `Placement::SafeRandom`. Build/reinicio oficial terminó en `OK`; validación manual del clic derecho pendiente.

## Bandeja y defaults visuales — 2026-10-22

Se añadió el mensaje nulo requerido después de `TrackPopupMenu` para mantener abierto el menú contextual de la bandeja con el propietario oculto. Los defaults de `ReactionConfig` y `SurfaceManager` son ahora `Intensity::Anomalous` y `Placement::SafeRandom`.

## Aplicaciones abiertas y CPU bajo demanda — 2026-10-22

Fuente: reporte visual del usuario e inspección del cálculo anterior. Se retiró el filtro CPU > 0,1 %, la división por 80 ms fijos y la lista indiscriminada de procesos. El backend enumera aplicaciones con ventanas visibles (también minimizadas), excluye NAVI y ventanas decorativas, conserva aplicaciones en 0 %, agrega procesos relacionados y usa una muestra de un segundo con tiempo real por PID y comprobación de fecha de creación. Nombres habituales legibles, hasta seis resultados ordenados por CPU; errores de acceso se muestran como no disponibles. Sin lectura de títulos/contenido, polling ni dependencias nuevas. La medición no replica exactamente el Administrador de tareas; agrupaciones UWP/especiales y ventanas de muestreo pueden diferir. Módulo nuevo: crates/navi-platform/src/system_status.rs; integración vía WindowsHost y Response.data. Mantiene el contexto independiente del reposo y sin halo. Build release/reinicio oficial terminó en OK; ejecutable canónico actualizado y NAVI iniciado. QA manual pendiente a cargo del usuario, sin tests ni QA ejecutados por el agente.

## Contexto independiente del retorno a reposo — 2026-10-22

Inspección: el retorno al indicador restaura el foco previo y la reacción de foco borraba ContextFragment. Ahora los eventos de foco no sustituyen un contexto activo; al iniciar la respuesta contextual se retiran las manifestaciones de foco/acción anteriores. La superficie principal puede permanecer en reposo mientras el contexto completa su duración. Build/reinicio release oficial finalizó en OK, con NAVI iniciado; validación visual a cargo del usuario, sin QA adicional ejecutado.

## Tiempo de lectura contextual — 2026-10-22

ContextFragment de estado usa 5600 ms: entrada de 200 ms, 5200 ms con opacidad completa y salida de 200 ms. Los datos se calculan en WindowsHost (adaptador del backend en navi-platform), se entregan en Response.data y el renderer sólo dibuja nombres y porcentajes formateados. La agrupación es por ejecutable: no reproduce exactamente los grupos ni la métrica del Administrador de tareas. Build/reinicio oficial ejecutado con salida OK; ejecutable canónico actualizado el 22/10/2026 a las 23:24:26 y NAVI iniciado. Validación manual a cargo del usuario, sin tests ni QA ejecutados en esta revisión.

## ContextFragment legible — 2026-10-22

La respuesta textual de «estado del sistema» ahora es breve y no duplica los datos mostrados en el fragmento. El panel compacto se oculta durante la consulta y ContextFragment presenta hasta seis aplicaciones agrupadas por ejecutable, con CPU en formato legible. La medición se asocia por PID para evitar porcentajes incorrectos cuando existen varios procesos con el mismo nombre. Esta consulta no dispara FracturedHalo; sólo muestra el contexto temporal y luego vuelve al indicador mínimo. `cargo check --offline -p navi-app` terminó correctamente. El build release alcanzó los crates pero no pudo reemplazar `target/release/navi-app.exe` por acceso denegado porque NAVI seguía ejecutándose; el ejecutable oficial no se actualizó. Pendiente cerrar NAVI normalmente y ejecutar `./scripts/build-run.ps1`, además de la validación visual del usuario.

## ContextFragment real — 2026-10-22

`ContextFragment` ahora recibe datos reales sólo para la consulta «estado del sistema». `WindowsHost` toma dos muestras breves de procesos locales mediante ToolHelp/GetProcessTimes, calcula el porcentaje aproximado por núcleo y conserva las tres aplicaciones con mayor actividad. El resultado estructurado se entrega al renderer; el fragmento muestra nombre y porcentaje, sin títulos de ventana, contenido ni historial. La consulta permanece bajo demanda y no inicia polling en reposo. El build/reinicio oficial terminó en `OK`; no se ejecutaron pruebas ni QA adicional.

## Organización de cambios en Git — 2026-10-22

Por pedido del usuario, los cambios pendientes se agrupan en commits con gitmoji: núcleo textual; integración compacta y composición orgánica; corrección de foco; corrección de arrastre; halo y respuestas operativas; laboratorio visual independiente; skill de orquestación; documentación; build y reinicio automático con política de agentes. Las correcciones se separaron mediante el índice de Git, conservando los archivos de trabajo. No se modifica la implementación ni se repiten compilaciones o QA por esta tarea de organización; el último build/reinicio exitoso sigue siendo el documentado debajo. Sin publicación remota.

## Build y reinicio automatizados — 2026-10-22

Estado actual: `scripts/build-run.ps1` creado y ejecutado desde PowerShell; salida única **OK**, código 0. Cerró ordenadamente las instancias de la ruta oficial, compiló release offline y dejó NAVI iniciado. El bloqueo de compilación registrado debajo queda resuelto por esta ejecución; las correcciones anteriores están incluidas en el release actual. Ejecutable: `F:\navi\target\release\navi-app.exe`.

El script solicita WM_QUIT al bucle de mensajes de NaviSessionWindow sólo para procesos con esa ruta; espera hasta 10 s y aborta si no terminan. No fuerza procesos ni cambia el código de la aplicación. Cargo guarda stderr en `artifacts/build-run/build.log` y stdout en `stdout.log`; sólo se consultan si falla. Tras el build inicia el ejecutable y comprueba que no termine durante el primer segundo, sin QA funcional. Instrucción de uso registrada en AGENTS.md. Validación visual/manual a cargo del usuario, sin nuevas pruebas ejecutadas.

Comando habitual desde el repositorio: `.\scripts\build-run.ps1`. Próximo paso: uso y validación de las correcciones por el usuario.

## Correcciones posteriores — 2026-10-22

Fuente: fallos visuales reportados por el usuario e inspección de código. Alcance implementado, entrega release **bloqueada por acceso denegado al reemplazar el ejecutable oficial**.

- DragConnector: el observador no considera al overlay pasivo una superficie interactiva propia; el foco de otra ventana no sustituye un conector activo. Canvas que incluye origen y extremo, crece por bloques de 128 píxeles dentro del escritorio virtual; evita copiar todas las pantallas para un gesto corto. Sin límite artificial de distancia. Un fallo gráfico ya no desactiva todos los efectos y el observador del mouse para toda la sesión.
- FracturedHalo: todos los resultados Success/Error del backend, incluidas hora y estado, en lugar del filtro exclusivo para calculadora. Monitor, variación y error rojo conservados.
- Foco: observa también restauración de minimizados y espera 120 ms antes de resolver el foreground actual. Maximizado no se trata como fullscreen sólo por su rectángulo; el catálogo no bloquea un monitor por el overlay propio.
- Respuesta de calculadora: «Iniciando calculadora», sin primera persona ni afirmar una ventana visible aún no comprobada.
- Módulos: cursor_observer, pointer_reaction, desktop_observer, monitor_catalog, lib, holograms, surface, focus_reaction, cursor_qa, reactions, text_backend y tools.
- Compilación intentada: `cargo build --release --offline -p navi-app`, seguida de `OK` sólo con salida exitosa. Falló al retirar `F:\navi\target\release\navi-app.exe`: acceso denegado (os error 5); **no terminó en OK**. Cargo compiló los crates y alcanzó navi-app; no se declara entrega release actualizada.
- Sin tests, diagnósticos, QA ni validación manual ejecutados en esta corrección, según AGENTS.md actualizado. Se adaptaron expectativas antiguas del canvas en pruebas existentes, sin ejecutarlas. Las evidencias que siguen corresponden al hito anterior.
- Próximo paso: el usuario debe cerrar NAVI normalmente desde la bandeja y se reintentará el build canónico. La validación funcional y visual queda a cargo del usuario. No se finalizaron procesos, alteraron permisos ni crearon ejecutables alternativos.

## Evidencia del hito anterior

Actualización: 2026-10-22 (Argentina). Evidencia: inspección del árbol real, pedido explícito de consolidación, informes por archivo, revisión independiente, 96 tests, una prueba nativa render separada, diagnóstico --input-check de ocho comprobaciones y build release canónico. El usuario reporta completado Hito 2; la consolidación actual está implementada, pendiente de validación visual/manual. [Evidencia](agent-work/2026-10-22-consolidacion/04-validation/integracion.md), [ADR 0006](decisions/0006-consolidacion-compacto-y-efectos.md).

## Objetivo y fase

Consolidar funciones de animaciones y volver a escribir directamente en el compacto. Alcance implementado: entrada por teclado, resultados con halo, familias fijas por evento y drag con origen estable a través del escritorio virtual. No se implementan voz, LLM, nuevas herramientas ni apps reales en ContextFragment.

## Estado por componente

| Componente | Estado actual |
|---|---|
| Entrada compacta | Implementada y probada: pregunta 2 s, luego `_` cada 500 ms mientras vacía/enfocada; Enter real, Escape conserva borrador |
| Indicador / consola | Conservados; reposo sin animación; consola y borradores preservados |
| Tu espacio | Retirado de ejecución; fuentes históricas sin módulo compilado |
| LayerStack v2 | Sólo cambios de foco en uso normal; organicidad de Hitos 1/2 conservada; previews sólo runner de diagnóstico |
| FracturedHalo | Resultado de open_calculator, éxito/error rojo, 1800 ms; posición/ángulo por evento en monitor enfocado; omite sin monitor fiable y cancela al cambiar monitor |
| ClickPulse / DragConnector | Únicas familias de clic/arrastre; no selectores alternativos |
| Drag entre monitores | Implementado y probado sintéticamente; origen Down fijo, canvas físico virtual, sin límite artificial de 640×360; físico/DPI mixto pendiente |
| ActionProjection / LayerStack v1 | Retirados del catálogo y módulos compilados; fuentes históricas conservadas sin uso |
| Sutura / SignalField | Sólo diagnóstico explícito, sin nueva función cotidiana |
| ContextFragment | Sólo diagnóstico con datos simulados; apps abiertas reales propuesto, pendiente de alcance |
| Núcleo textual | Conservado: permisos/router/confirmación estricta opcional; calc confirma inicio aceptado, no ventana visible |
| Ejecutable oficial | Release canónico exitoso; metadata final registrada abajo; valoración visual pendiente |

## Último hito y módulos

KeyboardEntry puro en Session, adaptación `keyboard_entry.rs`, UI/renderer compacto y timer 5 sólo para espera vacía enfocada. `action_surface.rs` resuelve posición/ángulo/monitor; `action_visual.rs` conserva plazo e incrementa seed. Backend inicia FracturedHalo; menú consolidado y rutas obsoletas retiradas. Error/respuesta sin protección expiran tras cuatro segundos.

`cursor.rs` conserva origen y separa inactividad de duración; `reactions.rs` fija familias; `pointer_reaction.rs` aporta escritorio virtual/DPI físico estable; `surface.rs` conserva ambos extremos sin clamp y reutiliza canvas; cursor QA adaptado. Los experimentos no se editaron en este hito; el árbol ya contenía modificaciones ajenas y previas, que se preservaron. No hubo commits ni descarte de cambios.

## Pruebas y evidencia

- `cargo test --workspace --offline --quiet`: **96 aprobadas**, 0 fallos (17 agent, 35 platform, 13 render, 30 state, 1 ui); una nativa ignorada por defecto y doc-tests sin fallos.
- `cargo test -p navi-render --offline native_temporal_alpha_and_transform_restore -- --ignored`: **1 aprobada**, ventana propia oculta; no captura externa.
- Formato y Clippy de todos los targets con `-D warnings`: aprobados.
- Release oficial `--input-check`: **8 checks aprobados**, exit 0. Prompt/cursor, foco, Escape/Beacon, borrador, Enter/reloj real, halo sintético con alpha/estilos pasivos, liberación y desactivación. Reporte `artifacts/consolidation/input-check.json`; no calculadora real ni entrada externa.
- Tests nuevos: drag de más de 11000 px, gesto continuo de 120 s, origen negativo, familias fijas, reloj de teclado, colocación de resultado y vencimiento de error protegido/no protegido.
- No se ejecutó prueba física de teclado/drag/multimonitor, apertura de calculadora ni medición CPU/RAM/GPU. No se repitió captura compuesta anteriormente denegada. No trasladar resultados nativos históricos a esta entrega.

## Ejecutable oficial

- Ruta exacta: `F:\navi\target\release\navi-app.exe`.
- `cargo build --release --offline -p navi-app`: exitoso desde el workspace real, sin redirección ni ejecutable alternativo; se recompila al finalizar para reflejar el árbol integrado.
- Metadata del rebuild final comprobado: **2026-10-23 01:09:52 UTC / 2026-10-22 22:09:52 -03:00**, **786432 bytes**, SHA-256 `CF888AD91C0125D6357FC8F5543546DF22EF8AAA846201F77F6A3066A9D1BBF1`.
- Este binario ejecutó satisfactoriamente el diagnóstico nativo. No se finalizaron procesos ni se alteraron permisos/configuración global. Compilado/implementado, pendiente de valoración física y visual.

## Riesgos y decisiones abiertas

- Canvas BGRA del escritorio virtual sólo durante drag: memoria/transferencia proporcionales al área envolvente. 7680×2160 ≈63 MiB por DIB, estimación aritmética, no medición. Límites de tamaño/asignación de Windows/Direct2D siguen vigentes; no se promete longitud fuera del escritorio real.
- Watchdog visual de arrastre: desaparece tras 1500 ms sin actualizaciones; el origen se conserva al continuar. Gesto abandona tras 60 s de inactividad, no por duración de movimiento continuo.
- Campo compacto conserva 120 caracteres y append/backspace; no añade selección/pegado. La espera sólo titila si vacío y enfocado, no en reposo.
- Halo limitado al monitor enfocado al manifestarse; no sigue una ventana. Su resultado real es aceptación del inicio por Windows. Apertura visible/foco de Calculator requiere prueba física.
- Sutura quizá acciones pesadas y ContextFragment con apps reales son propuestas pendientes; no se crean eventos/herramientas para justificarlas.
- Configuración de familias antiguas permanece tipada para compatibilidad de tests, pero motor fija las tres familias aprobadas y no hay selector cotidiano. Fuentes retiradas no se ejecutan.
- Se conservan límites previos: una solicitud textual en vuelo, sin proveedor, cancelación forzada ni automatización en segundo plano.

## Próximo paso y aceptación

Ejecutar el release oficial y seguir la sección Consolidación en [validation.md](validation.md): pregunta→cursor, escritura, Enter, borrador, halo success/error y monitor, cambios de foco y drag largo entre monitores/DPI. Aceptación: ancla de origen estable, cursor de espera claro, familias sin mezclas, efectos pasivos y liberación completa; registrar resultados físicos y costo activo del canvas antes de afirmar rendimiento validado.

## Historial breve

- 2026-10-21: frontend, familias y núcleo determinista; antecedentes 59 tests/64 checks en documentación histórica.
- 2026-10-22: Tu espacio y ActionProjection anteriores, 63 tests/71 checks históricos; ahora sustituidos por ADR 0006.
- 2026-10-22: recuperación del release canónico; Hito 1 validado visualmente por usuario; Hito 2 temporal/Sutura/monitores con 88 tests y una prueba nativa, reportado completado por usuario.
- 2026-10-22: consolidación compacta, halo y cursor; 96 tests, una nativa render y ocho checks propios; validación física pendiente.
- Skill local de orquestación creada en etapa previa; su validación Python/yaml previa no se repitió en este hito.
