# Visual Genesis

Laboratorio independiente; no importa crates de NAVI ni modifica su aplicación. Incluye **Margen**, una experiencia interactiva completa, y los cinco estudios pasivos anteriores. No utiliza backend, red, voz, LLM, hooks ni observadores del escritorio.

## Margen — experiencia completa

```powershell
Set-Location F:\navi\experiments\visual-genesis
.\target\release\navi-visual-genesis.exe --experience
```

Una incisión permanece en el borde derecho. Al hacer clic se abre un manuscrito continuo anclado al borde: presencia → escritura → actividad → decisión contextual → conversación → retirada. El foco se solicita sólo al abrir explícitamente; el recorrido automático no lo solicita. El borrador y el hilo sobreviven al cierre de la superficie durante la sesión.

- **Escribir:** clic en «Tu escritura», Enter para continuar, Shift+Enter para nueva línea. Flechas, Inicio/Fin, selección con Shift o arrastre, Ctrl+A/C/X/V. Límite de 4096 caracteres; editor con desplazamiento al cursor.
- **Leer:** rueda, PageUp/PageDown, Ctrl+Inicio/Fin, barra lateral o marcas de intervención. «Copiar hilo» copia la conversación completa. Una respuesta nueva conserva la posición de lectura.
- **Decidir:** escribí «guardar este hilo» y elegí «sí, continuar» o «dejarlo así». No cambia ningún archivo ni ventana. Durante actividad/confirmación podés preparar un borrador; se envía después de resolver el paso pendiente.
- **Retirar:** Esc o «retirar». Volvé con un clic en la presencia. × termina el proceso. Tab/Shift+Tab recorren las acciones y Enter las activa.
- **Ver todos los estados:** `--tour` o «recorrer la experiencia». Dura unos 28 segundos y entrega el control con un borrador intacto; cualquier clic, tecla o rueda lo interrumpe. `--reduced-motion` elimina movimiento; `--monitor 1` elige otro monitor.

Los textos y decisiones son un guion local. «Hilo extenso» agrega material de lectura. Todo es volátil. No hay integración con acciones reales de NAVI. La escritura directa es la excepción interactiva prevista en el brief; los estudios pasivos siguen sin recibir entrada.

Implementación: `src/experience/` (modelo, vista y host Win32). Reutiliza únicamente el transporte gráfico del laboratorio, ampliado con canvas y clipping; no usa las composiciones anteriores. Sin trabajo continuo en reposo; caret estable. Edición Unicode por caracteres, sin UI Automation, composición IME avanzada ni deshacer/rehacer.

## Estudios pasivos anteriores

Desde PowerShell:

```powershell
Set-Location F:\navi\experiments\visual-genesis
cargo build --release --offline --target-dir .\target
.\target\release\navi-visual-genesis.exe --all
```

| ID | Pieza | Trigger ensayado y comportamiento |
|---|---|---|
| `sutura` | Animación: Sutura | Cambio de contexto simulado: una costura se abre en una membrana suspendida que gira y se recoge. |
| `palimpsesto` | Animación: Palimpsesto | Aparición de una presencia simulada: letras dispersas coinciden en una frase y vuelven a perder su alineación. |
| `nacre` | Animación: Nacre | Recuperación simulada: un volumen de cortes se abre alrededor de un núcleo ausente y vuelve a cerrarse. |
| `herbario` | Interfaz: Herbario de ausencias | Consulta de memoria simulada: cuatro especímenes y un margen de lectura; la selección cambia automáticamente. |
| `umbral` | Interfaz: Umbral de escucha | Conversación escrita simulada: una frontera plegada se abre entre intención y respuesta; no ejecuta acciones. |

Ejecutar cualquiera con `--experiment ID`. Cada estudio dura 16 segundos, seguido de un segundo de silencio; después cierra o pasa al siguiente. `--seconds 30` cambia la duración. `--reduced-motion` muestra una composición estática sin refresco continuo. `--monitor 0` selecciona el principal; `1`, `2`, etc. los restantes en orden de enumeración Windows. `--list` muestra el catálogo. Repetir el comando reinicia; Ctrl+C **en la terminal de lanzamiento** interrumpe. No hay Esc/R en el overlay pasivo.

## Estado / 2026-10-22

Margen: nueve tests aprobados, Clippy sin warnings y release offline compilado. Verificados por tests: edición Unicode, cancelación, confirmación de un solo uso, borrador, posición de lectura y clipping de los estados. `--smoke` ejecutó el recorrido nativo durante 29 segundos y cerró con código 0, sin errores de dibujo reportados. Retirar intenta devolver el foco a la ventana anterior; su comportamiento físico sigue pendiente. La validación visual, interacción física, DPI mixto y rendimiento medido permanecen pendientes. El estado se registra aquí para respetar el aislamiento del prototipo.

- Implementados: paquete y lock independientes, cinco escenas, Win32 no activable/click-through, Direct2D software, DirectWrite, alpha premultiplicado, ajuste al área de trabajo/DPI, temporizadores acotados y liberación de recursos.
- Verificado: formato, `cargo check`, cuatro tests, Clippy `--all-targets -- -D warnings`, release offline. Los tests recorren 505 frames, comprueban coordenadas finitas, complejidad acotada, inmovilidad del modo reducido y CLI.
- Ejecución nativa: `--list`, `--all --seconds 4` y `--all --seconds 4 --reduced-motion` terminaron con código 0; los cinco estudios completaron ambas pasadas. Esto verifica creación, dibujo y cierre sin errores reportados, no aprobación visual ni interacción física.
- Pendiente manual: apariencia y tipografía sobre fondos claros/oscuros, fluidez y consumo, clic físico a través de geometría pintada, conservación de foco, monitores con origen negativo, cambio de DPI/hot-plug y pantalla completa. No se usó Computer Use.
- Límite: superficie única ajustada a un monitor; no cruza monitores ni se suprime automáticamente sobre fullscreen. Sin mediciones de CPU/GPU; la rasterización es software. Cerrar la prueba antes de una actividad en pantalla completa.

No actualizar la documentación principal de NAVI: el aislamiento solicitado mantiene este estado únicamente en el laboratorio.
