# Validación reproducible

## Consolidación compacta — vigente, 2026-10-22

Esta sección sustituye los recorridos anteriores de Tu espacio, ActionProjection y selectores de familias. Los apartados de etapas previas se conservan como evidencia histórica, no como controles actuales.

Resultados actuales: 96 tests workspace aprobados; una prueba nativa render ignorada por defecto ejecutada separadamente/aprobada; formato y Clippy con todos los targets/-D warnings aprobados. Release canónico compilado y ejecutado con `--input-check`: exit 0, ocho checks true en `artifacts/consolidation/input-check.json`. Ese runner sólo opera el HWND compacto, consulta reloj Windows y reproduce halo sintético. No captura escritorio, pulsa teclado físico ni abre calculadora. La matriz/alpha de Hito 2 se comprobó con la prueba nativa oculta; no equivale a verificación visual del halo de error o drag físico.

```powershell
cargo fmt --all -- --check
cargo test --workspace --offline --quiet
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test -p navi-render --offline native_temporal_alpha_and_transform_restore -- --ignored
cargo build --release --offline -p navi-app
New-Item -ItemType Directory -Force artifacts\consolidation
$check = Start-Process -FilePath 'F:\navi\target\release\navi-app.exe' -ArgumentList '--input-check','F:\navi\artifacts\consolidation\input-check.json' -WindowStyle Hidden -Wait -PassThru
$check.ExitCode
Get-Content artifacts\consolidation\input-check.json
```

Prueba manual **pendiente**:

1. Cerrar normalmente instancias previas e iniciar `F:\navi\target\release\navi-app.exe`. Invocar con Ctrl+Alt+Espacio/clic: sólo compacto. Pregunta dos segundos; después `_` titilando, sin onda ni audio. Escribir antes/después del plazo: texto conservado.
2. Escape y reinvocar conservan borrador. Enter vacío no ejecuta demo; `hora` muestra reloj real y luego Beacon. Perder foco suspende cursor; consola y borradores siguen disponibles.
3. `abrí la calculadora`: iniciar sin código; FracturedHalo aislado, nunca capas combinadas. Repetir: posición/ángulo distintos y siempre monitor enfocado. Probar cambio de monitor durante efecto: cancela. Un fallo se prueba con host falso automáticamente; no provocar daños para forzarlo. Verificar color rojo con fallo controlado si hay entorno apropiado; no se afirma probado físicamente.
4. Cambios reales de ventana sólo LayerStack v2. Mouse sólo ClickPulse y DragConnector. Arrastrar desde punto fijo a más de 640 px, volver y cruzar monitores/DPI: origen estable, extremos bajo coordenadas físicas, sin saltos ni recorte. Soltar sobre NAVI u otra app: desaparece. Pausa de 1,5 s oculta la línea; al continuar recupera origen.
5. Desactivar animaciones de resultado y todos los efectos: texto sigue funcionando. Beacon sin timers de espera ni efectos; la espera de teclado tiene un timer de 500 ms sólo si está vacía y enfocada. Sutura/SignalField/ContextFragment permanecen diagnósticos y los datos contextuales son simulados.
6. Medir CPU/RAM/GPU durante drag largo y reposo en el equipo real. Canvas virtual activo aumenta memoria/transferencia; 7680×2160 son ≈63 MiB por DIB por cálculo, no medición. Probar límites reales de backend, fullscreen y desconexión de monitores antes de afirmar cobertura.

Estado oficial/metadata en [progress.md](progress.md); decisión [ADR 0006](decisions/0006-consolidacion-compacto-y-efectos.md). No se reejecutó el runner histórico de captura compuesta antes denegado.

Fecha: 2026-10-22. Windows 10/11 x64, Rust MSVC estable, Windows SDK y Visual Studio Build Tools para C++. Shell: PowerShell. Los resultados históricos están en [VALIDATION.md](../VALIDATION.md); esta guía define cómo repetirlos sin confundirlos con pruebas de la etapa nueva.

## Comandos

Desde la raíz del repositorio:

```powershell
cargo fmt --all -- --check
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo build --release --offline -p navi-app
Get-Item 'F:\navi\target\release\navi-app.exe' | Select-Object FullName, LastWriteTime, Length
New-Item -ItemType Directory -Path artifacts\canonical -Force
```

`--offline` requiere las dependencias ya disponibles. La única ruta oficial es `F:\navi\target\release\navi-app.exe`; no usar `--target-dir` para producción. Si Windows bloquea el archivo, cerrar NAVI normalmente desde la bandeja y repetir, sin terminar procesos a la fuerza ni producir otra entrega. Los directorios alternativos mencionados en evidencia histórica ya no son rutas de entrega.

Diagnóstico nativo, exclusivamente sobre ventanas propias:

```powershell
$qa = Start-Process -FilePath '.\target\release\navi-app.exe' -ArgumentList '--snapshot','F:\navi\artifacts\canonical\compact.bmp' -WindowStyle Hidden -Wait -PassThru
$qa.ExitCode
Get-Content .\artifacts\canonical\compact.json
```

Los diagnósticos pueden fallar al registrar atajos si hay otra instancia. Cerrar la anterior desde su bandeja para la prueba física. Nunca terminar procesos ajenos automáticamente.

## Cobertura y límites

Los tests puros cubren estado, gestos, filtros, geometría y ciclo de vida. Los diagnósticos nativos comprueban alpha, hit testing, foco y recursos de HWND propios. No prueban Alt+Tab físico, otras aplicaciones, voz, GPU en otra máquina ni pantalla completa exclusiva. No se requiere Computer Use.

Etapa textual: los tests cubren frases completas, parámetros, desconocidos/ambigüedad, herramientas no registradas, permisos, ejecución directa, confirmación bajo política estricta, caducidad/cancelación, contexto y errores. Las herramientas con efectos usan un host falso en tests; el harness nativo solicita/cancela calculadora bajo política estricta y nunca la abre. El diagnóstico nuevo abre el panel propio, envía hora desde el botón nativo y reproduce ActionVisual de forma sintética: no demuestra por sí solo una apertura real de calculadora. Los checks de atajo envían WM_HOTKEY al HWND propio; no son pulsaciones físicas.

Routing y ejecución se miden por separado con `Instant`: routing incluye normalización y decisión del router; ejecución cubre la función de herramienta y la construcción inicial de sus datos. No incluyen validación posterior de confirmaciones, registro del turno, creación del worker, canal/Win32, render ni tiempo hasta que una aplicación externa está lista. No se usa un tiempo objetivo como medición ni se atribuye toda la latencia de interfaz al router.

El diagnóstico genera `text-latency.json` junto a la captura solicitada (actualmente `artifacts/user-panel/`): build release/debug explícito, 20 consultas de calentamiento y 500 muestras por herramienta (`current_time`, `system_status`), percentiles 50 y 95 en nanosegundos. Usa las primitivas Windows reales de sólo lectura, sin sleeps ni salida por muestra; el reloj puede devolver intervalos menores que su resolución efectiva. Es un microbenchmark del núcleo, no latencia de extremo a extremo ni un resultado extrapolable a otra máquina.

## Prueba manual textual prevista

Resultado físico aún pendiente. Si existe una instancia anterior, cerrarla desde su bandeja y ejecutar `F:\navi\target\release\navi-app.exe`.

1. Al iniciar sólo debe verse el indicador, sin quitar el foco. Hacer clic en él o usar Ctrl+Alt+Espacio (también Alt+Espacio): debe abrirse «Tu espacio» con campo, cursor y ejemplo. Enviar `ayuda`, `qué hora es` y `estado del sistema` con Enter y con Enviar. La respuesta de NAVI aparece separada y vuelve al indicador tras cuatro segundos; la escritura queda abierta hasta ocultarla.
2. Escribir un borrador, usar Escape y volver a invocar: el borrador debe conservarse. Probar Tab, selección, Ctrl+V y arrastre del encabezado. Escape oculta sin cerrar el proceso.
3. Enviar `abrí la calculadora`: debe abrir directamente, sin pedir un código. NAVI proyecta capas/fractura/barrido durante 1,8 segundos; debe dejar pasar clics y no activar ninguna superficie decorativa. El panel no debe recuperar foco después de que Windows abra la calculadora. Tras la respuesta queda el indicador.
4. Bandeja → Animaciones de acciones: desactivar y repetir. La calculadora debe seguir abriendo sin proyección. Repetir con Desactivar todos los efectos y con movimiento desactivado (proyección estática temporal si las animaciones de acciones están activadas). `abrila` sólo debe ejecutar cuando el referente inmediato sea inequívoco.
5. Enviar `ejecutá powershell`, una orden compuesta o una solicitud desconocida: debe informar necesidad de proveedor sin ejecutar comandos. `abrí` debe pedir aclaración.
6. Si se logra escribir mientras una tarea está activa, enviar otro borrador: no debe perderse ni enviarse por una segunda pulsación de Enter durante la tarea. Los tests nativos cubren este caso de forma controlada, ya que las herramientas actuales suelen terminar enseguida.
7. Contraer NAVI explícitamente mientras trabaja: el resultado no debe reabrir su tarjeta. La consola conserva sus controles de diagnóstico F1–F12 y Ctrl+Alt+1–6; el panel de escritura recibe texto normal. Probar los efectos y cambios de aplicación: indicador persistente y paso de clics. Permiso denegado, fallos y confirmaciones estrictas se comprueban con host falso.

## Evidencia histórica

La etapa visual reportó 41 tests y 56 checks nativos aprobados, más mediciones CPU/RAM/GPU. El usuario confirmó la ruta de foco anterior; la revisión visual nueva y mouse físico permanecen pendientes según [PARALLEL_EFFECTS.md](PARALLEL_EFFECTS.md). Los nuevos resultados se consolidarán en [progress.md](progress.md).

## Resultado de etapa 1 — 2026-10-21

59 tests de lógica/estado/geometría aprobados; formato, Clippy de todos los targets con `-D warnings` y release offline aprobados. El [reporte nativo](../artifacts/text-core/compact.json) registra 64 checks aprobados: 56 anteriores y ocho de submit real, worker/mensajes, hora, confirmación/cancelación, derivación, borradores y contracción. La captura `text-confirmation.png` también se inspeccionó: texto y leyenda de confirmación visibles dentro de la tarjeta.

Microbenchmark release de las herramientas reales de sólo lectura, 500 resultados correctos por herramienta. Valores convertidos de nanosegundos a microsegundos del [JSON generado](../artifacts/text-core/text-latency.json):

| Herramienta | Routing p50 | Routing p95 | Ejecución p50 | Ejecución p95 |
|---|---:|---:|---:|---:|
| Hora local | 0,6 µs | 0,6 µs | 0,5 µs | 0,6 µs |
| Estado básico | 0,8 µs | 1,0 µs | 1,7 µs | 2,4 µs |

Son mediciones de esta máquina y de los límites descritos arriba, no objetivos ni latencia completa de interfaz. Los artefactos se regeneran con el diagnóstico y permanecen ignorados por Git. No se midió la latencia de apertura real de calculadora ni se realizó la prueba manual de uso entre aplicaciones. Una instancia anterior del usuario siguió abierta; no se alteró su proceso.

## Panel y acciones — 2026-10-22

63 tests aprobados (17 núcleo, 21 estado, 20 plataforma y 5 render); formato, Clippy con `-D warnings` y build release offline aprobados. El diagnóstico nativo terminó con código 0 y [71 comprobaciones aprobadas](../artifacts/user-panel/compact.json): mantiene las 64 anteriores y añade siete sobre panel separado, foco, Escape/indicador, borrador, botón/worker/reloj, alpha/estilos pasivos, vencimiento y desactivación. Las capturas de ventanas propias se inspeccionaron en `artifacts/user-panel/user-panel.png` y `action-projection.png`.

Los tests prueban ejecución directa y errores con host falso; el harness conserva su política estricta para solicitar/cancelar calculadora sin abrirla. ActionProjection se reproduce sintéticamente. No se probó el atajo con teclado físico ni se abrió calculadora realmente. No se midieron CPU/RAM/GPU de este efecto nuevo. La prueba manual de esta guía sigue pendiente.

## Recuperación y ejecutable canónico — 2026-10-22

Ejecución actual: formato aprobado; `cargo test --workspace --offline --quiet`: 63 aprobadas, sin fallos, más doc-tests; `cargo clippy --workspace --all-targets --offline -- -D warnings`: aprobado. Incluye las tres pruebas de FracturedHalo. El laboratorio independiente aprobó build release y nueve tests (505 frames muestreados de los cinco originales más pruebas de CLI y Margen):

```powershell
cargo build --release --offline --manifest-path experiments/visual-genesis/Cargo.toml --target-dir experiments/visual-genesis/target
cargo test --offline --manifest-path experiments/visual-genesis/Cargo.toml --target-dir experiments/visual-genesis/target
```

Primer `cargo build --release --offline -p navi-app`: fallo al retirar el ejecutable oficial, acceso denegado, mientras PID 13712 estaba abierto desde esa ruta. No se terminó el proceso ni se creó otra entrega. Tras dejar de aparecer la instancia, el mismo comando terminó exitosamente. Ruta comprobada: `F:\navi\target\release\navi-app.exe`, modificación **2026-10-22 13:15:46 -03:00**, 770048 bytes; hash y estado actual en [progress.md](progress.md). Se preservaron fuentes/configuración y las piezas originales; los cambios de esta investigación son documentales.

Diagnóstico nativo ejecutado con el archivo oficial y `--snapshot F:\navi\artifacts\canonical\compact.bmp`: **código 1**, log `%TEMP%\navi-error.log`: `Acceso denegado. (0x80070005)`. Se generaron DIB de efectos —incluido `fractured-halo-native.bmp`—, capturas de texto y `text-latency.json`. No se generó `compact.json`; no hay resultado consolidado actual de los 71 checks. El flujo posterior a las capturas textuales entra en `verify_user_panel`, cuya captura compuesta precede a `user-panel.bmp`; por secuencia y artefactos se infiere interrupción en ese bloque. La API exacta que devuelve el error no se instrumentó, por lo que no se afirma su causa concreta ni un fallo del renderer a partir de ese HRESULT solo.

No se reintentó la operación denegada ni se modificaron permisos, capturas o código del harness. No se ejecutó Computer Use ni prueba física, apertura real de calculadora o medición CPU/RAM/GPU. Las comprobaciones nativas históricas permanecen válidas como antecedentes; no se declaran revalidadas. La compilación canónica es exitosa y la validación nativa completa queda pendiente.

## Hito 1: variación orgánica - 2026-10-22

Implementado, pendiente de validación manual. Primera familia: LayerStack v2 para foco de ventanas y su preview. No se integraron las piezas de Astra ni se modificaron Margen o los usos/triggers de FracturedHalo. Contrato efectivo: [ADR 0004](decisions/0004-variacion-organica-por-manifestacion.md).

Comandos ejecutados, todos exitosos:

```powershell
cargo test --workspace --offline --quiet
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo build --release --offline -p navi-app
```

79 tests aprobados: 17 navi-agent, 26 navi-state, 28 navi-platform y 8 navi-render, más doc-tests sin fallos. Los 16 tests nuevos cubren rangos y streams, curvas, movimiento reducido, matrices/pivote, elección determinista, recencia mono/multimonitor, área útil con orígenes negativos y DPI 96/144/192, exclusiones, destinos bloqueados, estabilidad entre frames, duración compartida, supresión y anclajes físicos del cursor. Son pruebas de lógica/matrices, no renderizado nativo ni validación visual.

Release oficial verificado en `F:\navi\target\release\navi-app.exe`, modificado **2026-10-22 23:38:14.6172947 UTC**, 792576 bytes. SHA-256: `E7F08534A11B16841B36151B3659D55D6B8EAD99C11C6B6DA674457FDC30B61E`. No hubo redirección de salida ni proceso anterior detectado al comprobar el build. Se compiló el árbol integrado existente, sin crear otra entrega.

No se repitió el diagnóstico de captura que había fallado con acceso denegado. No se ejecutaron Computer Use, prueba física de clic/foco, multimonitor real ni mediciones CPU/RAM/GPU. Revisión del ciclo: no hay nuevo temporizador/observador periódico; selección y recencia sólo al inicio, y el timer de efecto existente se libera al vencer. Esto no constituye una medición de consumo.

Validación manual pendiente, ejecutar desde la ruta oficial:

1. **Una pantalla:** alternar entre dos aplicaciones con pausas de al menos un segundo, unas 12 veces. Esperar cambios de zona, tamaño, transparencia, rotación y ritmo sin saltos durante cada manifestación. Verificar bordes, ausencia de recortes y legibilidad del escritorio. Clic/foco deben seguir llegando a la aplicación.
2. **Varias pantallas:** mantener las aplicaciones alternadas en una pantalla; observar que algunas manifestaciones libres aparecen en las otras. Probar DPI distintos y monitor a la izquierda/arriba. Desconectar/reordenar o cambiar DPI durante un efecto: debe cancelarse y la siguiente manifestación usar la geometría nueva.
3. **Garantías:** clic/arrastre deben continuar sobre sus coordenadas reales, incluso al cruzar monitores. Pantalla completa/presentación deben suprimir las reacciones normales sin trasladarlas a otra pantalla. Sin movimiento: imagen estática temporal y sin rotación; al vencer, desaparece.
4. **Reposo:** sin interactuar durante 30–60 segundos no deben quedar efectos ni dibujo continuo; comprobar CPU/recursos con Administrador de tareas si se desea medir. No hay un umbral medido aprobado todavía.

Limitaciones: detección geométrica conservadora de fullscreen en destinos y franja superior de la ventana relacionada; no reconoce controles semánticamente. Si no cabe ningún candidato se omite. La variación nueva sólo corresponde a efectos libres de StackV2. Cursor y familias alternativas conservan el comportamiento anterior sin selección libre; estabilizar su identidad entre DragMove queda fuera de esta integración inicial. Hito 2 requiere aprobación.

## Hito 2: Sutura y evolución temporal — 2026-10-22 (Argentina)

Hito 1: el usuario confirmó su validación visual al autorizar esta tarea. Esa confirmación no constituye validación visual de los cambios nuevos.

Comprobaciones sobre el árbol integrado final:

```powershell
cargo test --workspace --offline --quiet
cargo test -p navi-render --offline native_temporal_alpha_and_transform_restore -- --ignored
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo fmt --all -- --check
cargo build --release --offline -p navi-app
```

- Suite: **88 pruebas aprobadas**, sin fallos; 17 agent, 26 state, 32 platform, 13 render. Una prueba nativa ignorada por defecto se ejecutó explícitamente: **aprobada** (89 comprobaciones automatizadas ejecutadas en total, más doc-tests).
- Nuevas invariantes: parámetros/capas/curvas reproducibles y diferentes por semilla; continuidad con cota de derivada para oscilaciones; vértices emitidos de fragmentos/trazos/colas dentro del canvas; filamentos/bandas/trama/costuras de Sutura con movimiento y proyección 3D original escalada .65; mismo deadline para reacción/superficie, tiempo restante de 1 ms y expiración; estático y triggers/cursor conservados. Monitor: 4096 raíces con dos destinos, distribución dentro del margen 45–55%, invariancia ante orden/monitor del evento, destinos excluidos y repeticiones permitidas. Estos son casos sintéticos, no mediciones físicas.
- Prueba nativa: ventana propia oculta y Direct2D software; ambas familias/intensidades con alpha parcial en desarrollo y DIB vacío en extremos; restauración de matriz previa y rechazo de transformación inválida; igualdad byte a byte de DIB estático en p=.1,.5,.9 aunque reciba un temporal animado; foco conservado. No captura escritorio, no sintetiza entradas externas ni valida percepción estética.
- Formato y Clippy con `-D warnings`: aprobados. Diff de módulos afectados sin errores al usar la configuración CRLF del repositorio. Forzar `core.autocrlf=false` trató finales CRLF como whitespace; no se normalizaron archivos ajenos.
- Primera ejecución de fmt/test: etiqueta nueva no UTF-8; corregida. Primera suite: dos umbrales de continuidad demasiado estrictos para velocidades continuas previstas; corregidos usando cota de derivada y extensión máxima subpíxel a delta .0001. Clippy detectó un retorno redundante y helper de 13 argumentos; corregidos sin desactivar lints. Se repitieron las comprobaciones tras la adaptación geométrica final.
- Release inicial bloqueado por `Acceso denegado (os error 5)` al reemplazar el canónico, mientras PID 12636 seguía abierto. No se finalizó a la fuerza ni se creó otra entrega. Se reintentó después del cierre normal y Cargo terminó correctamente. Metadata/hash finales en [progress.md](progress.md).
- Revisión independiente de código aceptada, sin ejecución duplicada de Cargo por el revisor: [informe](agent-work/2026-10-22-hito2/04-validation/review.md). La adaptación final de proporciones Sutura y su repetición de pruebas las comprobó el integrador.

### Recorridos manuales pendientes

Ejecutar exclusivamente `F:\navi\target\release\navi-app.exe`, cerrando previamente cualquier instancia anterior desde la bandeja.

1. **LayerStack:** alternar entre dos aplicaciones con pausas de unos 2 segundos, 12–20 veces. Debe durar aproximadamente 1.08–1.32 s y presentar torsiones/orientaciones/oscilaciones internas distintas por capa, conservando identidad, transparencia y foco. Revisar aparición/desarrollo/disolución sin saltos, clipping ni restos. Cambios rápidos pueden cancelar una manifestación por la política previa de foco/cooldown; no extenderla artificialmente.
2. **Monitores:** mantener las aplicaciones en una pantalla y observar una muestra más larga (por ejemplo 60 apariciones) entre todos los destinos elegibles. Debe desaparecer la prioridad por monitor enfocado/anterior; permitir repeticiones, sin exigir porcentajes idénticos en una muestra breve. Probar DPI mixto y orígenes negativos. Destinos fullscreen/excluidos o sin espacio seguro no participan. Cambiar DPI/topología durante un efecto debe cancelarlo.
3. **Sutura:** bandeja → «Sutura — diagnóstico visual», repetir varias veces. Debe aparecer una membrana plegada con filamentos, bandas parciales y costuras, durante aproximadamente 1.62–1.98 s. Esperar profundidad, tensiones internas y variación sutil; sin etiquetas de laboratorio ni reproducción automática al cambiar de foco. Su activación cotidiana queda pendiente de aprobación.
4. **Sin movimiento:** bandeja → «Animación: activar/desactivar», repetir foco y diagnóstico. Debe producir imagen fija temporal, sin rotación ni oscilación, de 850 ms (LayerStack) o 1800 ms (Sutura); la superficie desaparece al vencer. Cambiar la opción durante una manifestación la cancela. Volver a activar permite movimiento en la siguiente.
5. **Garantías:** probar clic/arrastre sobre coordenadas reales y cruce de pantallas; comandos existentes de FracturedHalo; indicador/consola/borradores; paso de clics/foco y supresión fullscreen. Son recorridos físicos pendientes, no resultados aprobados por los tests.
6. **Reposo:** esperar 30–60 segundos sin interactuar: no deben quedar manifestaciones ni dibujo continuo. Medir CPU/RAM/GPU con método y duración si se evalúa rendimiento; no hay medición nueva realizada. Para reproducir una secuencia completa, reiniciar NAVI y activar seed 42 antes de los mismos eventos; mantener catálogo/exclusiones/secuencia iguales. La opción no promete repetir una sola imagen.

Límites: fallback sintético sin catálogo conserva colocación/tiempo anteriores; no prueba la nueva distribución. Sin validación visual física o rendimiento de Hito 2 todavía. No se repitió el harness anterior que fallaba al capturar el escritorio compuesto.
