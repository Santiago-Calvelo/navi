# 0004 - Variación orgánica por manifestación

Estado: aceptada para Hito 1 por autorización explícita del usuario.
Fecha: 2026-10-22.

## Contexto

Las manifestaciones libres repiten monitor, posición y tamaño. El usuario aprobó el contrato de recuperación visual, limitado inicialmente a una familia existente, conservando frontend y triggers. El informe de investigación enlazado originalmente se retiró por limpieza explícita del usuario el 2026-10-26; el contrato se conserva en esta decisión.

## Alternativas

Sortear por frame produciría saltos e impediría reproducir fallos. Restringir todo al monitor enfocado impediría la presencia distribuida aprobada. Añadir un generador externo o un servicio periódico es innecesario.

## Decisión

`navi-state::visual::ResolvedVariation` selecciona materialidad y tiempo con mezcla de 64 bits y streams separados. `monitor_catalog` captura geometría Windows al inicio y ante mensajes de cambio; en la admisión consulta cobertura fullscreen y exclusiones propias. `organic::OrganicPlanner` evalúa ocho candidatos por monitor con costo de proximidad y recencia en anillo de ocho entradas, con decaimiento de 60 segundos. No registra títulos ni actividad de aplicaciones.

`SurfaceManager` conserva el resultado completo hasta vencer o cancelarse. Primera integración: **LayerStack v2 ante WindowFocus y su preview explícita**. Las demás familias y los eventos del cursor siguen sus rutas existentes. No se introduce selección semántica de familias, identidad nueva de gestos ni cambios en ActionProjection/FracturedHalo.

Todos los monitores elegibles pueden recibir un efecto libre. Se encaja su envolvente rotada de 540×300 DIP con margen de 16 DIP y padding de 8 DIP; se excluyen las superficies propias y la franja superior aproximada de la ventana relacionada. Si ningún candidato cabe, se omite. Sin catálogo se conserva la ruta anterior, utilizada por los diagnósticos sintéticos.

Rangos Subtle / Experimental / Anomalous: escala 0,96–1,04 / 0,92–1,07 / 0,88–1,10; ganancia alpha 0,90–1 / 0,82–1 / 0,75–1; rotación ±3° / ±6° / ±9°. Duración animada ±10%, limitada a 200–4000 ms, y curva temporal monótona seleccionada al inicio. La escala final también se ajusta al espacio disponible. La seed alimenta además las variantes internas existentes.

La duración seleccionada se propaga al estado activo, progreso y vencimiento. Sin movimiento se conserva el tiempo base, imagen estática y rotación cero. Cambios de pantallas/DPI y de movimiento cancelan el plan, sin sortear de nuevo desde un temporizador. El renderer centra, escala y rota; restaura la matriz incluso ante error. No consulta Windows para decidir política.

## Consecuencias y validación

Sin dependencias nuevas, HWND adicionales por elemento, polling ni renderizado nuevo en reposo. La recencia vive sólo en memoria. Misma seed, secuencia y contexto producen el mismo resultado; la opción existente de seed 42 reproduce la secuencia, no una imagen única repetida.

La protección de controles externos es geométrica, no semántica. La cobertura fullscreen del destino es una instantánea conservadora, no observación permanente. Los gestos alternativos conservan su presentación cercana al cursor; su identidad estable entre DragMove se difiere y no recibe variación libre en este hito.

Pruebas y build comprobados en [validación](../validation.md); estado del ejecutable y validaciones manuales en [progreso](../progress.md). No equivale a validación visual o de hardware.
