# 0006 — Entrada compacta y funciones únicas de efectos

Estado: aceptada por indicaciones explícitas del usuario.
Fecha: 2026-10-22 (Argentina).
Sustituye el panel independiente y composición de acciones del [ADR 0003](0003-panel-usuario-y-acciones-directas.md), y las alternativas de familias del mouse. Mantiene organicidad de foco de ADR 0004/0005.

## Contexto y alternativas

Tu espacio duplicaba la superficie de entrada; el menú mezclaba demos, familias antiguas y funciones reales. El drag usaba un viewport móvil de 640×360 que recortaba el origen. Se descartaron conservar dos editores, mezclar halos con capas para cada acción y agregar un motor gráfico para resolver el conector.

## Decisión

- Escribir directamente en el compacto existente. KeyboardInvoke presenta la pregunta por 2000 ms; después aparece `_` alternando cada 500 ms mientras el campo vacío tiene foco. No es audio ni Listening; no usa waveform. BeginRequest/CompleteRequest y cierre cancelan el temporizador. Se conservan borradores y consola.
- LayerStack v2 únicamente responde a cambios de foco reales durante uso normal. Los previews de capas se restringen al runner de diagnóstico explícito.
- FracturedHalo acompaña Success/Error de la única acción actual, open_calculator; error usa rojo en todos sus pinceles. Posición y ángulo se resuelven una vez por resultado, con seed distinta por evento, en el monitor enfocado. Sin monitor fiable se omite; si el foco cambia de monitor se cancela. No se presupone apertura visible de Calculator: el backend sólo confirma creación del proceso solicitado.
- ClickPulse es la única familia de clic, DragConnector la única de arrastre, incluso si una configuración antigua solicita otra. Los selectores se retiran.
- El origen del drag conserva el Down físico. Se dibujan ambos extremos en un canvas estable del escritorio virtual a DPI 96 (píxeles físicos), sin clamp por distancia ni cambio de monitor. Un único HWND decorativo. Watchdogs por inactividad y supresión fullscreen siguen vigentes.
- ActionProjection y LayerStack v1 quedan fuera del catálogo compilado; sus fuentes históricas se conservan sin conexión. Sutura, SignalField y ContextFragment conservan diagnósticos identificados, sin nuevos disparadores. Apps reales en ContextFragment queda propuesta, no implementada.
- Respuesta/error sin borrador ni confirmación vuelven al indicador después de cuatro segundos. La espera de teclado y la consola no se cierran automáticamente.

## Consecuencias

Sin dependencias nuevas, voz, LLM ni herramientas adicionales. La edición compacta conserva el límite de 120 caracteres y append/backspace; no se añadió selección o pegado. El canvas de arrastre consume memoria y transferencia proporcionales a la envolvente del escritorio sólo durante el efecto: 7680×2160 requiere aproximadamente 63 MiB por DIB. Hay límites de asignación/tamaño del backend; no se promete longitud fuera del escritorio real. Sin medición nueva de CPU/RAM/GPU ni validación de DPI mixto físico.

## Validación

### Corrección posterior (2026-10-22)

El usuario reportó cortes de drag, ausencia del halo en consultas y ausencia de foco al restaurar. Se elimina el filtro exclusivo de calculadora: cualquier resultado Success/Error puede iniciar el halo. El canvas de arrastre pasa a envolver ambos extremos y crece por bloques de 128 píxeles, conservando el Down absoluto; evita el coste de rasterizar el escritorio completo para gestos cortos. El foco se resuelve tras 120 ms y escucha también restauraciones; se distingue maximizado de fullscreen y el overlay propio de una ventana interactiva. No se ejecuta QA por agentes según la política actual; estado de compilación en progress.md.

Pruebas de origen a más de 11000 píxeles y duración continua de 120 s, familias fijas, pregunta/cursor/borrador, posición/ángulo reproducibles, expiración de error y permisos existentes. Diagnóstico `--input-check` opera únicamente HWND propios, reloj real y halo sintético, sin captura compuesta ni apertura de aplicaciones. Resultados actuales en [progreso](../progress.md) y [validación](../validation.md).
