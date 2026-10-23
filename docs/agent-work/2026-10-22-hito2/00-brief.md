# Hito 2 — alcance autorizado

Fuente: solicitud expresa del usuario y validación visual de Hito 1 reportada en esta sesión (2026-10-22 Argentina). Estado inicial: árbol con numerosos cambios previos; no descartarlos ni atribuirlos a este hito.

## Análisis y propuesta del integrador
LayerStack v2 vive 850 ms; geometría construye hasta .27 y disuelve desde .73. Base propuesta 1200 ms, variación existente ±10% (1080–1320). Sin movimiento conservar base 850 ms. Preview Sutura 1800 ms ±10% (1620–1980); estático base 1800. Recencia y proximidad comparadas entre monitores sesgan selección. Elegir monitor uniformemente entre monitores con candidato válido, zona por costo local. No inventar eventos: Sutura sólo preview explícito.

## Contrato compartido aprobado
- navi-state visual: agregar Family::Sutura.
- TemporalMotion Copy/Debug/PartialEq, sample(seed: u64,intensity: Intensity,motion: bool); parámetros inmutables, sin RNG por frame. Métodos layer(index: usize,p:f32)->LayerMotion con campos rotation_radians,offset_x,offset_y,skew; envelope(p), construct(index,p), dissolve(index,p). layer devuelve ceros sin movimiento; curvas continuas, finales completos a p=1. Reservar geometría dentro de canvas 540x300 DIP.
- VisualStyle agrega temporal: Option<TemporalMotion>; Default None. ResolvedVariation agrega temporal: TemporalMotion, muestreado una vez. identity neutral. Host propaga Some(variation.temporal) únicamente a manifestaciones libres; acciones y cursor preservan comportamiento anterior.
- Renderer Sutura: pub(super) unsafe fn draw(r:&Renderer,frame:EffectFrame,style:VisualStyle)->Result<()>; canvas 540x300, geometría y comportamiento del original, sin etiquetas/host/demo. Usa style.temporal opcional, no genera aleatoriedad en cada frame.
- Cualquier API auxiliar nueva queda documentada aquí por integrador antes de cruzar propietarios.

## Modalidad y propietarios
Flujo acotado: análisis/propuesta por integrador, implementaciones independientes Eiri y validación Mika posterior. Modelos gpt-6-luna, medium para código acotado, costo esperado inferior a modelo principal; revisión final por integrador. Comunicación por documentos, salida mínima.
- Eiri: crates/navi-state/src/visual.rs y crates/navi-render/src/layer_stack_v2.rs; documento 03-solution/layerstack.md.
- Arisu (solución Sutura): exclusivamente crates/navi-render/src/sutura.rs; documento 03-solution/sutura.md.
- Integrador: plataforma, wiring render, tests de integración, documentación compartida y release final.
- Mika: sólo 04-validation/review.md, lectura de código integrado, sin correcciones.
No builds concurrentes; agentes pueden rustfmt únicamente sus archivos, integrador corre pruebas tras integrar.

## Aceptación, límites y exclusiones
Pruebas deterministas/continuidad/variación real/envolventes/expiración, distribución independiente de evento/orden/DPI, modo estático y reducido, triggers y cursor preservados. fmt, test workspace offline, Clippy y release canónico, metadata/hash. Verificación visual y física/GPU pendiente, sin declarar éxito manual. No tocar experiments, Margen, otros efectos, usos de Halo, backend, dependencias, ramas/commits ni configuración global. Reutilizar overlay/timer existentes, sin actividad en reposo. Detener ampliaciones y documentar bloqueos. Ejecutable final sólo F:\navi\target\release\navi-app.exe.

## Revisión de integración
La primera solución temporal requiere corrección: fade final y variaciones independientes por capa, sin clamp de puntos. La primera Sutura requiere recuperar proyección en profundidad original, compilar Point correctamente y cubrir geometría animada completa. Se devolvieron ambas unidades a sus propietarios antes de ejecutar pruebas. TemporalMotion conserva phase/amplitude públicos para adaptar Sutura sin muestreo por frame. Los resultados iniciales no se aceptan como validados.

Contrato estabilizado tras corrección: TemporalMotion tiene enabled, layers: [LayerTemporal;5], entry, exit públicos. LayerTemporal expone phase, frequency, amplitude, construct_shift, dissolve_shift, direction,torsion. Para Sutura usar temporal.layers[index%5] (phase, amplitude, etc.); no campos phase/amplitude en el struct raíz. No mutar estos valores durante ejecución. layer endpoints deben ser cero exacto.

Integración: el HOST ya aplica temporal.envelope(progress) a frame.opacity. Renderers NO deben duplicarla. Modo frame.motion=false debe neutralizar desplazamiento/rotación interna aun si style contiene TemporalMotion.enabled=true. No reducir globalmente geometría 18% sin evidencia; preferir mantener tamaño aprobado, sólo ajustar margen si los tests REALES lo requieren.

## Evidencia integrada para revisión
Propietarios de código liberados: integración por principal completada. Suite workspace: 88 aprobadas, una prueba nativa ignorada por defecto. Prueba nativa de ventana propia oculta ejecutada explícitamente y aprobada: native_temporal_alpha_and_transform_restore (sin captura de escritorio). Se corrigieron umbrales inicialmente incorrectos de continuidad conforme a cota de derivada (ondas) y velocidad de extensión (Sutura), no mediante desactivar verificaciones. Contención real completa aprobada sin clamps ni reducción global de tamaño. Primer Clippy detectó let_and_return y helper13args; corregidos por integrador, repetición en curso. Primer fmt/test falló por codificación de nueva etiqueta bandeja; corregido a UTF8 sin cambiar labels existentes. No se ha compilado release todavía. Proceso oficial12636 detectado, usuario informado que debe cerrar normalmente. Revisión Mika puede leer todo, sólo escribe su informe. No repetir builds durante revisión.

Cierre técnico: Clippy corregido y aprobado. Disolución temporal normalizada hasta p=1 (no .98), alineada al vencimiento. La prueba nativa se amplió con comparación byte a byte de DIB estático en p=.1,.5,.9 aun recibiendo temporal animado, más foco conservado. Revalidación final en curso; Mika no debe ejecutar builds simultáneos.

## Ajuste final de fidelidad Sutura
La revisión geométrica del integrador recupera exactamente las proporciones x/y/z y proyección de la membrana original, con escala uniforme .65 y centrado canónico; la primera adaptación comprimía excesivamente los pliegues verticales. Barrido analítico conservador u/turn/v de bordes y desplazamientos: x22.28–492.19,y59.30–271.03 DIP; se repiten pruebas reales y release después de este ajuste. Primer release tras cierre normal fue exitoso; el release final debe compilar este ajuste posterior. Sin cambios al experimento original.

## Cierre consolidado
Suite final88 aprobadas, prueba nativa1 aprobada, fmt/Clippy aprobados; revisión Mika aceptó código (sin ejecutar pruebas propias). Primer release os error5 con proceso12636; cierre normal, dos builds posteriores exitosos, último incluye fidelidad 3D original Sutura. Canónico806400 bytes, 2026-10-22 21:31:53 -03:00, SHA2566DFB5965F888F56AFDEB1A25327CDDEE29CE1406B47BB502933C549B0D4B4B82. Documentación compartida consolidada por integrador. Estado implementado/compilado, pendiente valoración visual Hito2; Hito1 confirmado por usuario. Sin commits ni cambios a experimentos/deps/otros triggers.
