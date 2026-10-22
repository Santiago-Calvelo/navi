# Visual Genesis

Laboratorio independiente. Cinco estudios nativos originales; no importa crates de NAVI ni modifica su aplicación. Selección y reproducción por CLI; **ningún teclado, clic, hotkey global, hook, captura, red, voz o LLM**. Las dos interfaces son prototipos escénicos: sus datos, selección y conversación están simulados.

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

- Implementados: paquete y lock independientes, cinco escenas, Win32 no activable/click-through, Direct2D software, DirectWrite, alpha premultiplicado, ajuste al área de trabajo/DPI, temporizadores acotados y liberación de recursos.
- Verificado: formato, `cargo check`, cuatro tests, Clippy `--all-targets -- -D warnings`, release offline. Los tests recorren 505 frames, comprueban coordenadas finitas, complejidad acotada, inmovilidad del modo reducido y CLI.
- Ejecución nativa: `--list`, `--all --seconds 4` y `--all --seconds 4 --reduced-motion` terminaron con código 0; los cinco estudios completaron ambas pasadas. Esto verifica creación, dibujo y cierre sin errores reportados, no aprobación visual ni interacción física.
- Pendiente manual: apariencia y tipografía sobre fondos claros/oscuros, fluidez y consumo, clic físico a través de geometría pintada, conservación de foco, monitores con origen negativo, cambio de DPI/hot-plug y pantalla completa. No se usó Computer Use.
- Límite: superficie única ajustada a un monitor; no cruza monitores ni se suprime automáticamente sobre fullscreen. Sin mediciones de CPU/GPU; la rasterización es software. Cerrar la prueba antes de una actividad en pantalla completa.

No actualizar la documentación principal de NAVI: el aislamiento solicitado mantiene este estado únicamente en el laboratorio.
