# LayerStack v2, FracturedHalo y reacciones del cursor

Esta revisión conserva Rust, Win32, Direct2D, DirectWrite, la consola y la ruta de WindowFocus existente. La prueba manual de WindowFocus anterior fue confirmada por el usuario. La revisión nueva necesita la validación manual siguiente; las pruebas propias del ejecutable no sustituyen esa validación.

## Ejecutar esta revisión

Una instancia anterior bloqueó `target\release\navi-app.exe`. Se conservó ese proceso y se compiló en una carpeta independiente:

```powershell
cd F:\navi
cargo build --release --offline --target-dir target\parallel-build
.\target\parallel-build\release\navi-app.exe
```

Antes de abrirla para uso cotidiano, salir de la instancia anterior desde su bandeja. Dos instancias compiten por los atajos y pueden reaccionar a las ventanas de la otra. No se terminó la instancia del usuario durante el desarrollo.

## Contratos y responsabilidades

Los contratos se establecieron antes de delegar. Cada subagente trabajó en archivos separados del árbol compartido.

| Responsable | Implementado e integrado |
|---|---|
| A | `layer_stack_v2.rs`: planos fragmentados, profundidad, parallax, construcción/disolución y contraste. `placement.rs`: monitor proporcional, DPI, NearCursor, WindowAnchor y SafeRandom reproducible. V1 se conserva. |
| B | `fractured_halo.rs`: anillos 3D incompletos proyectados a 2D, rotaciones independientes, fragmentos, desincronización/reconstrucción, intensidades. También `manifestation_qa.rs`, diagnóstico nativo propio. |
| C | `cursor.rs`: clasificación pura de gestos. `cursor_observer.rs`: Raw Input pasivo, botones izquierdo/derecho/central, umbral según DPI y movimientos limitados a 33 ms. |
| Integrador | Contratos `visual.rs`, ReactionEngine, SurfaceManager, adaptador de eventos, render de pulso/conectores, menús/atajos, integración, pruebas, release y documentación. |

`DesktopObserver → ReactionEngine → SurfaceManager → overlay` sigue siendo la ruta de foco. Para mouse: `WM_INPUT → CursorObserver → CursorGesture → adaptador → ReactionEngine → SurfaceManager → overlay`.

Los renderers reciben `EffectFrame` y `VisualStyle { intensity, seed }`; no observan Windows. SurfaceManager decide posición, DPI, tamaño, puntos y vencimiento, sin dibujar. Un único HWND decorativo admite todas las familias: ningún HWND por partícula o plano. Se usa el compositor alpha premultiplicado existente; Direct2D resulta suficiente para esta geometría 2.5D, sin nuevas dependencias ni motor 3D.

## Opciones y demos

Todas las opciones nuevas aparecen en el menú de bandeja y el menú contextual de NAVI. Son locales a la ejecución; no se persisten entre reinicios.

- Intensidad: Subtle → Experimental → Anomalous. Predeterminada: **Experimental**.
- Foco: LayerStack v2 → FracturedHalo → LayerStack v1.
- Posición: WindowAnchor → NearCursor → SafeRandom. NearCursor usa la posición real al recibir el evento; SafeRandom se mantiene estable durante toda la manifestación.
- Semilla reproducible: 42 o variación derivada del evento. La misma semilla reproduce geometría y posición para iguales datos de entrada.
- Clics y arrastres: activación independiente. Clic: ClickPulse o v1 de debug; arrastre: DragConnector o v2. Las alternativas complejas usan un viewport proporcional, no el del microefecto.
- FracturedHalo al invocar NAVI: asociación opcional a una acción importante, desactivada inicialmente. También puede asociarse a WindowFocus. No se agregó un observador de creación de ventanas.
- **Ctrl+Alt+5**: FracturedHalo aislado. **Ctrl+Alt+6**: LayerStack v1 de debug.
- **F11 / Ctrl+Alt+3**: LayerStack v2. F9, F10, F12, Ctrl+Alt+1/2/4, consola, demos de estados y opciones anteriores se conservan.

La demo aislada se muestra en el monitor del cursor, respeta sus límites y vuelve al estado previo sin borrar un borrador o contraer una consola. La aparición decorativa no activa la ventana; invocar el editor por teclado sí solicita foco deliberadamente.

## Política y recursos

- Una manifestación reactiva como máximo, sin cola. Foco válido reemplaza la decoración anterior; un microclic no reemplaza una manifestación de ventana activa.
- Foco: 850 ms, cooldown 700 ms y agrupación de duplicados. Clic: 260 ms, cooldown 250 ms. Arrastre: inicio limitado a 350 ms; movimientos del adaptador a un máximo de aproximadamente 30/s.
- El arrastre es un gesto genérico, sin inferir archivos. La liberación siempre termina el conector, incluso sobre NAVI. El estado de gesto se limita a 60 s y se comprueba al recibir entrada.
- Un conector estático no necesita animación periódica. Tras 1500 ms sin movimiento se libera; mover de nuevo durante el mismo gesto lo reconstruye. La liberación lo elimina inmediatamente si sigue visible.
- El conector usa un viewport reutilizable de hasta 640 × 360 DIP alrededor del extremo actual, limitado al monitor. Un segmento largo se recorta; no se asigna un bitmap del escritorio entero por cada movimiento. Al cambiar de DPI/monitor se reconstruye el recurso cuando cambia su tamaño físico.
- Al terminar se destruye el overlay y su renderer. Reposo sin temporizador de efectos. Si se desactivan clics y arrastres se desregistra Raw Input. Activado, recibe paquetes de movimiento reales, sin consultar posiciones ni procesos cuando no hay un botón seguido.
- `WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE`, `HTTRANSPARENT`, `MA_NOACTIVATE` y `SWP_NOACTIVATE`. Raw Input no usa NOLEGACY, captura, inyección, hooks ni sustitución del cursor.
- Se ignoran gestos iniciados en NAVI; los puntos que caen sobre sus superficies no crean una nueva manifestación. La decoración deja pasar el hit test a la aplicación subyacente.
- Supresión conservadora en pantalla completa y estados de presentación/ocupado notificados por Windows. Un fallo del observador o renderer desactiva efectos sin impedir el funcionamiento del cursor ni cerrar la UI funcional.
- No hay lectura de contenido de aplicaciones, títulos, historial, backend, reconocimiento de voz ni automatización.

## Validación manual pendiente de esta revisión

1. Salir de la versión anterior desde su bandeja y abrir el ejecutable indicado. Verificar presencia mínima inmediata, sin tomar foco ni animarse continuamente.
2. Alternar Explorador, Firefox y VS Code dejando un segundo entre cambios. Esperar LayerStack v2 breve anclado a la ventana; después debe permanecer solamente el indicador. Teclear durante la animación y comprobar que escribe en la aplicación activa.
3. Probar NearCursor y SafeRandom; repetir con semilla 42. En un segundo monitor, incluido uno a la izquierda y con otra escala de Windows, verificar límites y tamaño. Repetir cerca de los bordes y con una ventana pequeña.
4. En una misma aplicación, después de terminar la reacción de foco, hacer clics: esperar pulsos pequeños. Los clics rápidos pueden omitirse por cooldown. No se espera un halo en cada clic.
5. Arrastrar una selección o mover una ventana: esperar conectores entre origen y cursor. No se presupone que sea un archivo. Pausar dos segundos manteniendo el botón y mover de nuevo: el conector debe reaparecer. Soltar, incluso sobre NAVI: debe desaparecer. Comprobar que la aplicación sigue recibiendo el arrastre normalmente.
6. Probar clic/arrastre sobre NAVI: no deben crear nuevas reacciones. Repetir un arrastre entre monitores y observar el recorte del conector largo.
7. Ctrl+Alt+5: inspeccionar profundidad, reconstrucción, estabilización y disolución de FracturedHalo en las tres intensidades. Asociarlo a foco desde la bandeja o activar su aparición al invocar NAVI. Luego volver al v2 predeterminado.
8. Comparar v1 con Ctrl+Alt+6; probar F9/F10/F11/F12, controles de consola, texto sin enviar y Escape. La decoración no debe borrar texto ni cerrar una interacción activa.
9. Desactivar cada evento por separado y después todos los efectos. La UI debe seguir funcionando y reposar sin efectos. Activar de nuevo desde las opciones de eventos.
10. Probar supresión sobre una aplicación a pantalla completa. La comprobación real de pantalla completa exclusiva y distintos controladores gráficos sigue pendiente; los tests sólo verifican la política y superficies propias.

## Comprobaciones reproducibles

```powershell
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo fmt --all -- --check
cargo build --release --offline --target-dir target\parallel-build
.\target\parallel-build\release\navi-app.exe --snapshot F:\navi\artifacts\parallel\compact.bmp
powershell -ExecutionPolicy Bypass -File .\scripts\measure-effects.ps1 -Executable F:\navi\target\parallel-build\release\navi-app.exe -OutputDirectory F:\navi\artifacts\parallel\performance
```

El diagnóstico de snapshot abre exclusivamente ventanas propias y genera JSON/BMP. Comprueba geometría dibujada, alpha real, paso de clics sobre píxeles pintados, foco y destrucción de recursos. Las entradas de foco y gestos de las pruebas de lógica son sintéticas; no se declara validado un clic físico ni Alt+Tab por esas pruebas. No se utiliza Computer Use.

Las cargas de rendimiento repiten ráfagas acotadas durante nueve segundos por etapa; el arrastre se actualiza aproximadamente cada 33 ms. Son cargas diagnósticas sintéticas, no captura de una interacción externa. CPU se informa por núcleo y por máquina, RAM por proceso; GPU de NAVI se distingue de DWM, que incluye todo el escritorio. GPU nula significa contador no disponible, no cero medido.

## Resultados obtenidos

- `cargo test --workspace --offline`: **41 pruebas unitarias aprobadas** (18 plataforma, 5 renderer, 18 estado), más doc-tests sin fallos.
- Clippy con `--all-targets -- -D warnings`, formato y compilación release: aprobados.
- `artifacts/parallel/compact.json`: **56 comprobaciones nativas aprobadas**. Incluyen registro Raw Input, alpha real de cuatro familias en tres intensidades, hit test sobre píxeles pintados, foco, clic finito, arrastre estático, reutilización de HWND y liberación inmediata al soltar sobre NAVI.
- El registro del observador es real. Los eventos de foco y gestos del harness son sintéticos: no validan físicamente Firefox, Explorador o VS Code.
- Una instancia anterior siguió abierta durante las comprobaciones. El reporte de atajos iniciales fue `[true, false, false]`: la invocación alternativa estuvo disponible; consola y demo estaban ocupados por esa instancia. Cerrarla desde su bandeja antes de la prueba manual.

Medición en una máquina de cuatro procesadores lógicos, Experimental, 11–12 muestras por etapa. CPU es porcentaje de la máquina completa; RAM es el pico de working set de NAVI; GPU es media de la suma de motores atribuidos a NAVI, no porcentaje global del adaptador.

| Carga | CPU máquina | RAM MiB | GPU motores NAVI |
|---|---:|---:|---:|
| Reposo diagnóstico | 0,000% | 15,57 | 0,000% |
| SignalField | 0,557% | 20,56 | 0,250% |
| ContextFragment | 0,711% | 25,82 | 0,000% |
| LayerStack v2 | 1,385% | 25,34 | 0,273% |
| Combinación de las tres familias originales | 2,967% | 28,88 | 0,833% |
| FracturedHalo | 1,089% | 23,67 | 0,167% |
| ClickPulse | 0,899% | 21,21 | 0,083% |
| DragConnector | 1,059% | 28,01 | 1,545% |

Informes: `artifacts/parallel/performance/metrics.json` y `native.json`. Las ocho etapas confirmaron liberación de superficies y conservaron el píxel de esquina muestreado. El benchmark desactiva observadores para controlar las cargas; su reposo no mide el costo de Raw Input al mover físicamente el mouse. La comparación del foreground al final de cada etapa contra el inicio global dio `false`; este diagnóstico no distingue cambios externos de foco, por lo que **no se toma como prueba aprobada de foco durante cargas largas**. Las comprobaciones nativas cortas sí conservaron su foreground; la comprobación manual sigue pendiente.

## Archivos de esta revisión

- Estado y contratos: `crates/navi-state/src/{lib.rs,reactions.rs,visual.rs,cursor.rs}`.
- Render: `crates/navi-render/src/{lib.rs,holograms.rs,layer_stack_v2.rs,fractured_halo.rs,cursor_effects.rs}`. `layer_stack.rs` v1 permanece intacto.
- Plataforma: `crates/navi-platform/Cargo.toml` y `src/{lib.rs,desktop_observer.rs,focus_reaction.rs,surface.rs,holograms.rs,verification.rs,placement.rs,cursor_observer.rs,pointer_reaction.rs,reaction_options.rs,manifestation_qa.rs,cursor_qa.rs}`.
- Documentación: `README.md`, `VALIDATION.md`, `docs/WINDOW_FOCUS.md`, `docs/PARALLEL_EFFECTS.md`.

No hay tareas de los subagentes pendientes de integrar. Quedan pendientes la aprobación estética manual, mouse físico entre aplicaciones, monitores físicos con DPI mixto, pantalla completa exclusiva y mediciones comparativas en otras máquinas. La asociación de FracturedHalo a creación real de ventanas no se implementó; está disponible por demo, foco o invocación de NAVI.
