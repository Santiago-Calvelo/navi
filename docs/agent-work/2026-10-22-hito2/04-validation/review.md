# Revisión independiente del Hito 2

Fecha: 2026-10-23. Revisor: Mika. Alcance: inspección de código integrado y pruebas existentes. No ejecuté Cargo, Clippy, build ni validación visual; la suite y la prueba nativa indicada abajo se toman del brief integrado.

## Dictamen

**Aceptar la implementación inspeccionada; no cerrar todavía la entrega del hito.** No encontré una regresión demostrable en el contrato temporal, la selección de monitor, Sutura de diagnóstico, los triggers existentes ni la restauración de transformaciones. El cierre sigue condicionado al build release oficial y a la validación visual/manual indicada como pendiente. Esta revisión no sustituye esas comprobaciones.

## Evidencia verificada en código

- **Muestreo una vez y continuidad temporal:** `crates/navi-platform/src/organic.rs:69-72` deriva una seed de la raíz de sesión y del contador de manifestaciones exitosas y llama una vez a `ResolvedVariation::sample`. Sólo incrementa la secuencia al resolver un plan (`:220-229`). `SurfaceManager` conserva el `ResolvedManifestation` mientras vive el efecto (`crates/navi-platform/src/surface.rs:149-157,182-216`); las consultas posteriores sólo calculan progreso y tiempo restante. El host transmite el mismo `TemporalMotion` a cada frame, aplica el envelope en `frame.opacity` y usa el timer 4 para el siguiente frame o vencimiento (`crates/navi-platform/src/holograms.rs:163-197,240-275`). No vi muestreo aleatorio por frame.
- **Frame estático y endpoints:** `TemporalMotion::layer` devuelve movimiento cero si está deshabilitado o en `p=0/1`; `construct` y `dissolve` limitan el progreso y alcanzan 0/1 en sus endpoints (`crates/navi-state/src/visual.rs:211-244`). LayerStack fija `t=0.5` y omite desplazamiento interno cuando `frame.motion=false` (`crates/navi-render/src/layer_stack_v2.rs:42-73`). Sutura reemplaza el temporal por `default`, usa `p=0.5`, y no vuelve a multiplicar el envelope que ya aplicó el host (`crates/navi-render/src/sutura.rs:134-139,186-196`). La política estática conserva 850 ms para StackV2 y 1800 ms para Sutura; la animada usa las duraciones resueltas, compartidas por motor y superficie (`crates/navi-platform/src/surface.rs:139-157`; tests en `focus_reaction.rs:439-474`).
- **Selección justa de monitores y sesgos:** el planner busca primero el mejor candidato local por monitor, aplicando proximidad/recencia sólo dentro de ese monitor (`organic.rs:74-208`), y luego compara un rango determinista separado derivado de seed e ID estable (`:209-218`). Monitores bloqueados o sin geometría se omiten antes del sorteo. La prueba `monitor_lottery_is_balanced_and_independent_of_event_order_dpi_and_recency` cubre orden, cambio del área de evento, distribución sobre 4096 raíces y repetición permitida (`organic.rs:442-495`). La igualdad estadística documentada cubre dos monitores igualmente elegibles; no representa una prueba sobre hardware multimonitor real.
- **Triggers, cursor y Halo:** sólo StackV2 y Sutura de `WindowFocus` pasan por el planner orgánico (`surface.rs:139-160`); las familias de puntero conservan `variation: None`, DPI y anclaje físico, cubiertos por pruebas (`focus_reaction.rs:403-437`). Sutura no es familia por defecto y el único comando nuevo es el preview explícito de bandeja id 63 (`reaction_options.rs:69-74,105-116`); el test de configuración confirma que foco, clic y arrastre no la eligen (`focus_reaction.rs:477-484`). Los comandos y rutas existentes de FracturedHalo permanecen presentes (`reaction_options.rs:69-72,113-115`; `holograms.rs:304-312`). El archivo experimental original `experiments/visual-genesis/src/experiments/sutura.rs` no aparece modificado en el estado Git inspeccionado; `crates/navi-render/src/sutura.rs` es el renderer de producción separado y omite textos/etiquetas del experimento.
- **Alpha, matriz y recuperación:** el renderer rodea el dibujo con `BeginDraw`/`EndDraw`, restaura la transformación previa antes de propagar los errores y presenta el overlay con alpha per-pixel (`crates/navi-render/src/holograms.rs:291-319`). La prueba nativa `native_temporal_alpha_and_transform_restore` cubre StackV2 y Sutura y prueba además el caso de error (`:401-557`); el brief reporta que se ejecutó con ventana oculta y pasó. La geometría densa de Sutura y los puntos de LayerStack se comprueban contra el canvas sin clamps/reducción global, en tests del renderer (`sutura.rs:277-305`, `layer_stack_v2.rs` módulo de tests). No repetí esos tests.

## Limitaciones y pasos pendientes

- `docs/agent-work/2026-10-22-hito2/00-brief.md` informa 88 tests aprobados y la prueba nativa oculta aprobada; los considero evidencia reportada por integración, no una ejecución independiente mía. Clippy seguía en curso al redactar este informe.
- El brief indica que el release canónico aún no está compilado porque se detectó el proceso oficial 12636 y se pidió su cierre normal. La inspección no confirma un binario actualizado. Debe compilarse y verificarse `F:\navi\target\release\navi-app.exe` antes de declarar entregado el cambio de producción.
- No hice una inspección visual manual en Windows ni comprobé monitores/DPI/GPU físicos o rendimiento. Esas validaciones siguen pendientes aun si pasan las pruebas automatizadas y el release.
- El árbol ya tenía numerosos cambios sin confirmar antes de esta revisión; este dictamen se limita a los contratos Hito 2 y no atribuye al hito todos los archivos del diff.

## Resultado por criterio

| Criterio | Resultado | Evidencia |
|---|---|---|
| Seed fijada al admitir efecto y plan congelado | Aceptar | `organic.rs:69-72,220-229`; `surface.rs:182-216` |
| Movimiento temporal, frame estático, endpoints y duración | Aceptar | `visual.rs:211-244`; `layer_stack_v2.rs:42-75`; `sutura.rs:134-196`; tests de duración |
| Elección de monitor independiente de evento/orden/recencia | Aceptar por código y tests sintéticos | `organic.rs:161-218,442-495` |
| Cursor, FracturedHalo y triggers cotidianos preservados | Aceptar | `surface.rs:139-160`; `focus_reaction.rs:403-484`; `reaction_options.rs:105-116` |
| Sutura diagnóstica y experimento original | Aceptar | preview id 63; original sin cambios en estado Git observado |
| Alpha y restauración de matriz | Aceptar según evidencia reportada | `holograms.rs:291-319,401-557`; prueba nativa reportada por el brief |
| Cierre de producción | Pendiente | Clippy, build release oficial y validación visual/manual |
