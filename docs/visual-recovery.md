# Recuperación visual y variación orgánica

Fecha: 2026-10-22. Evidencia: inspección del árbol de trabajo, Cargo, procesos, hashes y pruebas actuales. Estado operativo y ejecutable: [progress.md](progress.md); comandos y cobertura: [validation.md](validation.md).

## Alcance y resultado

Fase 1 e inventario completados. Después de su aprobación, Hito 1 implementado con LayerStack v2: variación compartida y ejecutable canónico actualizado, pendientes de validación manual. El frontend y los triggers existentes se conservan. Integraciones de Astra, cambios funcionales de FracturedHalo y retirada de Margen requieren aprobación separada. Este documento conserva el diagnóstico y la dirección incremental; el estado actual está en progress.md.

## Ejecutable: causa comprobada

El workspace resuelve `target_directory = F:\navi\target`; paquete/binario `navi-app`, Rust 2021, rustc/cargo 1.99.1, host `x86_64-pc-windows-msvc`, release con LTO. No se encontró una redirección efectiva por variables o configuración Cargo. Sí hay comandos explícitos `--target-dir` en documentación y entregas anteriores:

| Artefacto observado antes del mantenimiento | Modificación, Argentina | Bytes |
|---|---|---:|
| `target/release/navi-app.exe` | 2026-10-21 21:06:20 | 401408 |
| `target/parallel-build/release/navi-app.exe` | 2026-10-21 21:37:34 | 468992 |
| `target/text-core/release/navi-app.exe` | 2026-10-21 22:46:29 | 710656 |
| `target/user-panel/release/navi-app.exe` | 2026-10-22 09:02:49 | 770048 |

Las entregas recientes se dirigieron a otras carpetas, mientras el usuario ejecutaba el archivo antiguo de la ruta canónica. Su hash era `0205B4FDB5162FE966ADBFA08DB0A2C0DFBA27218E35A4D95344A053532F628B`. Además, el proceso 13712 ejecutaba esa misma ruta desde las 09:40:46; el primer build canónico falló al retirar el archivo, con «Acceso denegado (os error 5)». No se terminó ningún proceso. Cuando dejó de aparecer, se reintentó el mismo comando y terminó correctamente. No se atribuye el contenido de cada binario antiguo exclusivamente a su tamaño o fecha.

La política canónica ya está incorporada en [AGENTS.md](../AGENTS.md), sección 8. Las rutas anteriores quedan como evidencia histórica, no como entregas ni alternativas recomendadas. No se eliminaron esos artefactos.

## Inventario preservado

Los cinco originales están en `experiments/visual-genesis/src/experiments/`. Comparten `mod.rs`, el catálogo/contrato de `../experiment.rs`, el host pasivo `../platform.rs` y `../render.rs`. Usan el paquete independiente `windows = 0.54.0`, Win32, Direct2D software, DirectWrite y DIB BGRA premultiplicado presentado mediante `UpdateLayeredWindow`. No dependen de crates de NAVI. Herbario y Umbral son escenas de interfaz con datos simulados, no funcionalidades operativas.

| Pieza / archivo | Implementación y costo relativo | Funciones propuestas, sujetas a contexto |
|---|---|---|
| [Sutura](../experiments/visual-genesis/src/experiments/sutura.rs) | Membrana proyectada, 34 filamentos y bandas oclusivas; adaptación media. Geometría construida por frame. | Retomar un hilo o conectar transitoriamente dos contextos después de una acción del usuario. No reaccionar a todo cambio de ventana. |
| [Palimpsesto](../experiments/visual-genesis/src/experiments/palimpsesto.rs) | Convergencia tipográfica, profundidad y disolución; adaptación media. La frase actual es fija. | Revelar una síntesis breve solicitada o un resultado confirmado. Sustituir el texto exige validar composición, longitud y legibilidad; no inventar pensamientos ni resultados. |
| [Nacre](../experiments/visual-genesis/src/experiments/nacre.rs) | 58 secciones de hasta 97 puntos, relleno y oclusión; adaptación media-alta. | Revelar una ausencia después de una exploración explícita sin resultado; reservar para manifestaciones excepcionales. Un evento de búsqueda real aún no existe: no implementar herramientas para justificar el efecto. |
| [Herbario](../experiments/visual-genesis/src/experiments/herbario.rs) | Cuatro especímenes y detalle, selección guionada; alta como función interactiva. | Materializar elementos que el usuario haya fijado explícitamente. Conservar primero como laboratorio; necesita contrato de datos e interacción, sin registrar actividad del escritorio. |
| [Umbral](../experiments/visual-genesis/src/experiments/umbral.rs) | Frontera plegada entre intención y respuesta; media como efecto, alta como interfaz. | Acompañar envío, espera y recepción reales junto a las superficies actuales. Derivar fases de esos estados, sin fingir porcentaje de progreso ni reemplazar el editor. |
| [FracturedHalo](../crates/navi-render/src/fractured_halo.rs) | Ya implementado en producción: 3/5/7 bandas y 5/10/15 fragmentos según intensidad. No necesita recuperación de código; revisar uso tiene costo bajo-medio. | Recomposición excepcional al recuperar continuidad tras interrupción o ante una acción de alta relevancia realmente completada. No justificar su intensidad con cada apertura de calculadora. |

Los costos son estimaciones de adaptación y riesgo, **no mediciones CPU/GPU ni plazos**. Los cinco originales permanecen intactos. Su build release offline y sus nueve tests pasan; los 505 frames muestreados prueban finitud/complejidad acotada, no aspecto ni rendimiento.

Para integración futura: adaptar geometría a `navi-render` y estados reales, retirando los rótulos del laboratorio de la ruta de producción. No importar el host experimental, sus conversaciones simuladas ni sus bucles de reproducción. Conservar los originales como referencia ejecutable; no crear un crate compartido ni modificar dependencias sin aprobarlo.

## FracturedHalo: disponibilidad frente a uso

- `navi-render/src/lib.rs` incluye el módulo; `holograms.rs::draw_manifestation` despacha `Family::FracturedHalo`. Compila y sus tres pruebas están dentro de las cinco de render actuales.
- **Demo:** Ctrl+Alt+5 y menú 59. Demuestran reproducción; no cuentan como integración cotidiana.
- **Evento real configurable:** menú «Foco» permite elegirlo para `WindowFocus`; por defecto la familia es StackV2. La asignación es fija durante esa sesión, sin selector orgánico.
- **Uso real dentro de otra composición:** `text_backend.rs::complete` inicia ActionVisual ante Success/Error de `open_calculator`; `action_projection.rs` dibuja FracturedHalo junto a LayerStack v2. Existe esa conexión, pero no una función autónoma cuidadosamente calibrada. El éxito actual confirma inicio aceptado por Windows, no ventana externa lista.
- **Opción engañosa:** «FracturedHalo al invocar» empieza desactivada. Además, `lib.rs::invoke` deriva y retorna hacia `user_panel::show` en ejecución normal antes de llegar al bloque `invocation_halo`; el atajo también deriva directamente a `user_panel::toggle`. Activar la opción no hace que acompañe la escritura cotidiana. Corregirla sería un cambio funcional y queda pendiente de alcance.

No se reescribió ni reconectó la pieza. Tampoco se verificó visualmente la diferencia entre las versiones antiguas.

## Alcance aprobado y estado del Hito 1

Inventario y dirección aprobados por el usuario. Implementado el sistema compartido con LayerStack v2 como primera familia, manteniendo triggers y frontend. Varían monitor elegible, zona/posición, escala, alpha, rotación, ritmo y duración, con recencia y plan estable. Los detalles efectivos y límites están en el [ADR 0004](decisions/0004-variacion-organica-por-manifestacion.md); pruebas y estado oficial en [progreso](progress.md). Pendiente validación manual. Astra, Margen y los usos/triggers de FracturedHalo no se modificaron. Hito 2 no autorizado.

## Contrato de organicidad (dirección; implementación incremental)

**Una manifestación, una decisión estable.** Primero decidir si una acción o transición real merece manifestación; después elegir entre familias compatibles y la opción de silencio. Ponderar significado, intensidad y recencia; no asignar una familia única por evento ni sortear uniformemente todo el catálogo. Separar el resultado operativo legible de la decoración.

| Responsable | Contrato propuesto |
|---|---|
| `navi-state` | `ManifestationId`, capacidades por familia/evento y `ResolvedVariation { seed, scale, opacity_gain, rotation, duration_ms, rhythm, geometry_variant }`. Selección pura, sin Windows ni reloj propio. |
| `navi-platform` | Instantánea de monitores elegibles: identificador opaco, área útil física y DPI; ventana relacionada, puntos de cursor, superficies propias que evitar y supresiones. Actualizar catálogo al inicio y ante cambios de pantalla/DPI, no por polling. |
| `placement` / `SurfaceManager` | Evaluar pocos candidatos, elegir monitor/zona, resolver envolvente transformada y guardar un plan estable. Recencia en anillo fijo, sin títulos, HWND persistidos ni historial de aplicaciones. |
| `navi-render` | Consumir frame temporal y transformación resuelta. Multiplicar opacidad una vez, transformar alrededor de un pivote y restaurar matrices incluso al fallar. Sin observar Windows. |

Reutilizar la mezcla determinista de 64 bits existente, sin `rand`: seed de sesión suministrada, contador de manifestaciones y streams independientes por propiedad. Misma seed, secuencia, contexto y recencia reproducen la elección. Limitar rangos según familia e intensidad; para texto, rotación y escala deben conservar lectura. Definir los rangos visuales finales después de aprobar el contrato.

**Todos los monitores pueden ser candidatos para efectos libres.** El foco es un factor de contexto, no una restricción. Ponderar espacio, uso reciente de monitor/zona, escala física/DPI y relación con la acción. Una manifestación puede nacer en otra pantalla; su monitor se elige al inicio y no cambia aleatoriamente durante la ejecución. En una sola pantalla se aplica la misma política entre zonas. No introducir proyecciones simultáneas por pantalla inicialmente. Respetar supresiones de pantalla completa/presentación sin utilizar otra pantalla para eludirlas.

Hasta ocho candidatos por monitor elegible; comparar con la misma función de costo, sin favorecer monitores por tener más muestras. Descartar geometría inválida, recortes y solapamientos de superficies propias; penalizar posiciones y familias recientes; ponderar después relación espacial y desempate determinista. Encajar la envolvente **rotada**, incluyendo grosor de trazos y excursión animada. Reducir hasta el mínimo permitido u omitir si no cabe. Sin inspección del contenido de aplicaciones no puede garantizarse conocer todos sus controles: usar exclusiones conocidas y márgenes conservadores, sin afirmar detección semántica.

**Cursor es la excepción obligatoria, incluso al elegir otra familia.** Clic conserva el origen físico; arrastre conserva origen y extremo actuales. Traslación orgánica cero, escala de anclajes uno y ninguna rotación del segmento. Sólo variar materialidad o geometría interna. Al cruzar monitores cambia el viewport/DPI por movimiento real, no por sorteo; el punto exterior se recorta, no se desplaza al borde. Conversión: `(punto_px - origen_viewport_px) × 96 / DPI_destino`.

`DragMove` hoy crea nuevas reacciones con otro `start`; debe convertirse en actualización de la misma identidad y variante hasta el fin/cancelación/watchdog, separando inactividad de duración visual. Para foco/acciones, congelar posición y parámetros al iniciar. Una alteración real de pantallas o espacio invalida/cancela el plan; no re-sortear desde el timer.

Progreso, duración elegida, vencimiento del estado, SurfaceManager y temporizador deben usar la misma fuente temporal. Ritmo: una curva normalizada seleccionada al inicio. Mantener modo estático temporal y desactivación. Selección/recencia sólo al comenzar; ningún cálculo nuevo periódico en reposo. Esto no afirma que la CPU total del proceso sea exactamente cero.

## Secuencia propuesta (alcance vigente: sólo Hito 1 del ADR 0004)

1. **Identidad y contrato:** `navi-state/src/{visual,reactions,action_visual}.rs`; pruebas de semilla, inicio único, actualización de arrastre, tiempo y cancelación. Registrar un ADR propuesto de contrato antes de implementarlo; actualizar arquitectura sólo cuando cambie realmente.
2. **Selección espacial y monitores:** `navi-platform/src/{placement,surface,focus_reaction,pointer_reaction,desktop_observer,holograms,lib,reaction_options}.rs`; contexto acotado, catálogo de monitores, recencia y plan almacenado. `text_backend.rs` sólo para pasar contexto visual de eventos ya existentes, sin cambiar herramientas. Probar un monitor, varios, orígenes negativos, DPI 96/144/192, espacios mínimos y exclusiones completas.
3. **Materialidad y calibración:** `navi-render/src/{holograms,cursor_effects}.rs`, más adaptación aislada por pieza aprobada. Primero variación exterior de una familia existente; después un solo candidato de Astra, recomendado Sutura para continuidad. Validar matrices/alpha y anclajes antes de multiplicar efectos.
4. **FracturedHalo y selección semántica:** aprobar su función excepcional y revisar el desvío de invocación y el uso en ActionProjection. Evaluar si reducir su frecuencia antes de añadir disparos. No activar todas las piezas automáticamente.
5. **Validación y cierre:** adaptar `manifestation_qa.rs`, `cursor_qa.rs`, `verification.rs`, escenarios deterministas en `navi-demo`, documentación y build canónico. Comprobar paso de clics/foco/recursos en ventanas propias; después interacción física entre aplicaciones, DPI mixto, fullscreen y desconexión de monitor. Medir CPU/RAM/GPU con método/duración, sin extrapolar resultados históricos.

No se cambiaron crates ni dependencias. Herbario y las demás piezas de Astra permanecen en el laboratorio. La primera integración de variación corresponde únicamente a LayerStack v2, según ADR 0004.

## Margen: retirada posible, no ejecutada

Exclusivos: `experiments/visual-genesis/src/experience/{mod,host,model,view}.rs`; allí están superficies, conversación y recorrido simulado. No hay activos externos exclusivos identificados. No están conectados a la aplicación principal.

No es seguro borrar sólo esa carpeta: `src/main.rs` declara y llama al módulo y contiene flags/campos `experience`, `tour`, `smoke`. Una retirada aprobada debe desacoplarlos y ajustar ayuda/README. Podrían retirarse después las features `Win32_System_DataExchange`, `Win32_System_Memory` y `Win32_UI_Input_KeyboardAndMouse`, comprobando compilación y lock del laboratorio.

**Conservar:** los cinco archivos de piezas, `experiments/mod.rs`, catálogo/Frame/Mark/primitivas, `platform.rs`, `render.rs`, manifiesto y lock. `Clip`/`Unclip` sólo se usan en Margen, pero limpiar ese soporte es opcional. `Renderer::new` llama a `Renderer::canvas`: no eliminar canvas como si fuera exclusivo. Mantener inicialmente toda infraestructura compartida reduce el riesgo. Nada fue eliminado.
