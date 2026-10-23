# Solución Sutura (Arisu)

## Archivos modificados

- `crates/navi-render/src/sutura.rs` (módulo nuevo; sin cableado, según el alcance).
- `docs/agent-work/2026-10-22-hito2/03-solution/sutura.md` (este informe).

No se modificó el experimento original ni `lib.rs`, `visual.rs`, `layer_stack_v2.rs` u otros archivos compartidos.

## Comportamiento implementado

- `pub(super) unsafe fn draw(r: &Renderer, frame: EffectFrame, style: VisualStyle) -> Result<()>` dibuja una membrana reglada en un canvas lógico de 540×300 DIP.
- Adapta el pliegue paramétrico y la proyección perspectiva del experimento `experiments/visual-genesis/src/experiments/sutura.rs`, a menor resolución: 24 filamentos × 57 muestras longitudinales, 18 posiciones de trama y hasta 18 costuras. La proyección usa la profundidad del pliegue; el margen interno de 8 DIP se comprueba en pruebas densas.
- Conserva papel claro con acento rojo sobrio, filamentos de opacidad variada, bandas oscuras estrechas y costuras de recorrido temporal finito. Se eliminaron todas las etiquetas y texto anfitrión del experimento.
- `frame.progress` sólo gobierna geometría cuando `frame.motion` es verdadero; el modo estático usa p=0.5 y `TemporalMotion::default()`, incluso si el estilo contiene variación. En modo animado, `style.temporal.unwrap_or_default()` aporta desplazamiento de capa y fase persistente desde `layers[index % 5]`; se normaliza `phase / TAU` donde las curvas de superficie requieren fase unidad. No se accede a campos raíz `phase` o `amplitude`. No se vuelve a aplicar el envelope a `frame.opacity`, que lo recibe del host. No hay RNG ni muestreo por frame.
- `path` es un helper privado que crea geometría Direct2D por trazo, rellena opcionalmente y dibuja una línea opcional. Los helpers puros de geometría son compartidos por el dibujo y las pruebas.

## Evidencia y validaciones ejecutadas

- Inspeccionados el brief actualizado del hito, `AGENTS.md`, `docs/progress.md`, `docs/architecture.md`, ADR 0004, el skill de orquestación, el contrato estabilizado `TemporalMotion.layers[5]`, `EffectFrame`, los helpers privados de `Renderer` y el experimento Sutura.
- `rustfmt --edition 2021 crates/navi-render/src/sutura.rs`: ejecutado sin error.
- Se añadieron pruebas unitarias del módulo para 101 muestras temporales × cinco semillas anomalous en filamentos, bandas, trama y costuras; continuidad entre pasos de progreso 0.0001; cambio geométrico efectivo entre semillas; invariancia de modo estático aunque reciba temporal; y endpoints de apertura, longitud y costuras.
- No ejecuté Cargo, pruebas, build, Clippy ni validación visual: el brief reserva esas comprobaciones y el build final al integrador.
- En la revisión posterior, `git -c safe.directory=F:/navi status --short` permitió observar el árbol sin cambiar la configuración global. Contiene numerosos cambios preexistentes; los dos archivos de este encargo son el módulo nuevo y este informe.

## Limitaciones y pendientes

- Los tests están escritos pero pendientes de ejecución por el integrador; el módulo aún requiere confirmación/cableado en `lib.rs` y pruebas de integración con el contrato compartido.
- Falta validar visualmente composición/opacidad en Windows, interacción con escala DPI y presupuesto real de GPU/CPU. El número de trayectorias reduce la densidad frente al experimento, pero no se midió rendimiento.
- La firma `draw` y los helpers son internos al crate. No se propone una API compartida nueva.
