# 0005 — Organicidad temporal y selección de monitor

Estado: aceptada por autorización explícita del Hito 2.
Fecha: 2026-10-22 (Argentina).
Sustituye parcialmente [ADR 0004](0004-variacion-organica-por-manifestacion.md) en política de elección de monitor, duración y contrato temporal; conserva sus garantías de colocación, exclusiones y ciclo de vida.

## Contexto

El usuario validó visualmente Hito 1 y reportó una evolución interna demasiado estática y sesgo hacia el monitor que pierde foco. LayerStack tenía una base de 850 ms, construcción hasta aproximadamente .27 y disolución desde .73: unos 391 ms de desarrollo central. La comparación global de candidatos mezclaba proximidad y recencia entre monitores. Sutura existe como membrana reglada con costuras, en un laboratorio independiente.

## Alternativas

- Girar todo el conjunto: no aporta evolución por capas y fue descartado expresamente.
- Sortear valores por frame: genera discontinuidad y pierde reproducibilidad.
- Equilibrar monitores forzando alternancia: hace previsible la secuencia e impide repeticiones orgánicas.
- Sutura en cada cambio de foco: no corresponde a continuidad/recomposición; no se aprobó una nueva función cotidiana.

## Decisión

Ampliar `ResolvedVariation` con `TemporalMotion`, resuelto una sola vez al admitir la manifestación. `VisualStyle.temporal` lo transmite al renderer; `None` conserva las rutas previas de acciones y cursor. Las capas evalúan funciones continuas de progreso normalizado con parámetros inmutables. La semilla reproduce la trayectoria completa, sin valores nuevos por frame.

LayerStack libre usa base animada de 1200 ms y ±10% (1080–1320); modo sin movimiento conserva 850 ms. Sutura usa 1800 ms ±10% (1620–1980), o 1800 ms estáticos. La misma duración resuelta gobierna reacción activa, superficie, progreso y temporizador existente. No se añade un bucle independiente ni trabajo en reposo. La ruta sintética sin catálogo conserva la colocación y el contrato anteriores.

Elegir primero un candidato válido por monitor según costos locales de proximidad/recencia. Comparar después rangos pseudoaleatorios independientes por ID estable para elegir monitor: igual probabilidad entre monitores elegibles, sin priorizar monitor enfocado, anterior, DPI, tamaño o cantidad de zonas válidas. Permitir repeticiones. Monitores bloqueados o sin geometría viable no participan; no se eluden supresiones.

Adaptar sólo geometría de Sutura al canvas de producción y compartir el overlay transparente existente. Exponerla únicamente mediante «Sutura — diagnóstico visual» en la bandeja; no incluirla en el ciclo de familias de foco/cursor ni cambiar otros triggers. Los eventos actuales observan foco y gestos físicos, pero no representan una recomposición significativa de contexto. Su activación cotidiana requiere otra decisión explícita.

## Consecuencias

Sin dependencias, observadores ni HWND adicionales por elemento. El experimento original permanece intacto. El renderer conserva transformaciones y alpha por píxel; la envolvente canónica se comprueba con puntos reales antes de la transformación externa estable. Cambios de pantalla/DPI o movimiento cancelan el plan existente.

## Validación

Comandos, resultados y limitaciones en [validation.md](../validation.md); estado de release y pruebas físicas pendientes en [progress.md](../progress.md). La geometría y selección se prueban sintéticamente: no equivalen a validación visual, multimonitor físico ni medición de rendimiento.
