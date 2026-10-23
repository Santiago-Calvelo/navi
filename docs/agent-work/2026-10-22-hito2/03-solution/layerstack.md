# Eiri — LayerStack v2 / movimiento temporal

## Archivos

- `crates/navi-state/src/visual.rs`
- `crates/navi-render/src/layer_stack_v2.rs`

## Implementación

- Añadido `Family::Sutura`.
- `VisualStyle.temporal: Option<TemporalMotion>` conserva `Default` en `None`.
- `ResolvedVariation.temporal` se calcula una sola vez con seed, intensidad y disponibilidad de movimiento; `identity` da movimiento neutral.
- API pública de modelo: `TemporalMotion::{sample, layer, envelope, construct, dissolve}`, `LayerTemporal` y `LayerMotion`. El muestreo genera parámetros por capa (fase, frecuencia de 0,7–1,7 ciclos, amplitud, sesgos temporales y torsión/dirección) mediante streams deterministas. No hay estado mutable ni RNG por frame; índices mayores que cuatro usan módulo cinco.
- `envelope` aplica entrada y salida suaves con endpoints cero y ventanas levemente variadas por seed. `construct` y `dissolve` son curvas monótonas, individualizadas por capa y completas en p=0/1. La deformación por capa es cero exacto en ambos extremos y en movimiento reducido.
- LayerStack v2 evalúa construct/dissolve y desplazamientos/rotación suaves cuando hay `Some`; neutraliza la deformación interna cuando `frame.motion == false`. `None` conserva las curvas/tiempos y la geometría anterior. Para la ruta temporal se extrae `geometry_point`, compartido entre dibujo y prueba; aplica rotación sin clamp ni reducción global. El host aplica `envelope` a `frame.opacity`; el renderer no la duplica.

## Evidencia y validación

- Ejecutado `rustfmt crates/navi-state/src/visual.rs crates/navi-render/src/layer_stack_v2.rs` (éxito).
- Añadidas pruebas deterministas de repetibilidad, diferencias por seed/capa/tiempo, modo reducido, límites de rotación, continuidad local, monotonía de curvas construct/dissolve, endpoints y cobertura densa de vértices reales de shards, trazos y colas con seeds, intensidades, capas y tiempos variados. No ejecutadas por la instrucción de no ejecutar `cargo test` mientras hay trabajo paralelo.
- No ejecutados `cargo test`, build, Clippy ni validación visual. `git status` no pudo ejecutarse: Git rechazó el directorio por `dubious ownership`; no se alteró configuración global.

## Límites / integración

- Se requiere integrar `VisualStyle.temporal` en los literales/constructores afectados por los demás propietarios, conservando `None` para acciones y cursor.
- No se añadió ajuste global: el tamaño aprobado queda intacto. La cobertura de coordenadas reales está añadida para confirmar contención antes de integrar; pendiente de ejecutar junto con la suite del integrador. Validación visual pendiente.
- El ejecutable oficial y pruebas finales corresponden al integrador.
