# Dirección visual

Fecha: 2026-10-22. Fuente: referencias y decisiones del usuario, AGENTS.md y código implementado.

NAVI debe sentirse como una presencia de la computadora integrada en el escritorio real de Windows. La referencia principal es Serial Experiments Lain (PS1); Ex Machina aporta sobriedad y Blade Runner 2049 inspira interferencias breves. Rainmeter/JaxCore orientan la modularidad futura. Estas referencias no implican copiar activos o implementar skins existentes.

## Implementado

- Palimpsesto de inicio (2026-10-26): una frase entre 48 originales, con alusiones sutiles a Lain mediante señal, cables, ecos, reflejos y capas. Letras serif claras con registros desplazados verde pálido; reunión, lectura quieta y dispersión durante seis segundos. Sin leyendas del laboratorio ni panel de fondo. El experimento original se preserva. La rotación evita repetir frases hasta agotar cada ciclo; lectura y ritmo pendientes de evaluación del usuario. [ADR 0007](decisions/0007-palimpsesto-de-inicio.md).

- Negro azulado, verde desaturado y tipografía técnica solicitada a DirectWrite: Cascadia Mono con sustitución del sistema.
- Indicador persistente de 88 × 28 DIP, tarjeta inicial de 320 × 104 DIP y consola conservada.
- SignalField, ContextFragment, LayerStack v1/v2, FracturedHalo y microreacciones del cursor, mediante geometría real y alpha por píxel.
- Intensidades Subtle, Experimental y Anomalous; Experimental predeterminada en las pruebas.
- Construcción/disolución finitas, sin animación permanente en reposo; efectos desactivables y decoraciones sin captura de clics.

La revisión estética completa, skins y la validación visual final no están aprobadas como terminadas. Las reacciones pueden usar WindowAnchor, NearCursor o SafeRandom dentro del monitor; v1 sigue como debug.

## Revisión de interacción aprobada

La consolidación posterior sustituye Tu espacio: se escribe en el compacto. Al invocar por teclado aparece «¿Qué necesitás?» durante dos segundos; luego `_` titila cada 500 ms mientras espera texto y tiene foco. La escritura no usa ondas de audio. Se preservan estética principal, indicador y consola. La futura invocación por voz tendrá su tratamiento propio; no está implementada.

LayerStack v2 sólo foco; FracturedHalo sólo resultado de acciones, rojo si falla, durante 1,8 segundos con posición/ángulo resueltos por aparición en monitor enfocado. ClickPulse sólo clic y DragConnector sólo arrastre con origen fijo entre monitores. ActionProjection y v1 se retiran del catálogo activo. Sutura y SignalField siguen sin función cotidiana; ContextFragment con apps reales queda por definir, no se simula como dato real. La valoración física y estética de esta consolidación está pendiente. Véase ADR 0006.

Véanse [arquitectura de efectos](architecture.md) y [validación](validation.md).

## Hito 2 — identidad temporal

El usuario confirmó visualmente Hito 1. Para LayerStack libre se amplía moderadamente la vida (1080–1320 ms) y se añaden desfases, torsión y oscilaciones suaves internas por capa. Cada aparición conserva parámetros durante toda su ejecución; no se gira constantemente el conjunto ni se añaden saltos aleatorios. La intensidad acota el movimiento. Construcción y disolución deben completar su curva antes de liberar la superficie.

Sutura adapta la membrana reglada plegada, filamentos claros, bandas oscuras parciales y costuras con acento rojo del original. No incluye textos de demostración ni paneles del laboratorio. Expresa continuidad/recomposición, y permanece sólo en diagnóstico explícito de bandeja; el foco ordinario no justifica su significado. Su vida animada prevista es 1620–1980 ms. El original se conserva en `experiments/visual-genesis/`.

Las manifestaciones libres eligen monitor al azar entre destinos elegibles, sin prioridad por foco, DPI o recencia del monitor. La recencia sigue variando zonas dentro de cada destino. Esto permite repeticiones naturales y no garantiza porcentajes idénticos en una muestra breve. Pantallas suprimidas o sin espacio seguro quedan excluidas.

La revisión visual de estos cambios nuevos sigue pendiente del usuario. No se avanzó a otras piezas de Astra, Margen ni cambios de FracturedHalo.
