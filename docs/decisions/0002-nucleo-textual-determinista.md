# 0002 — Núcleo textual determinista y herramientas explícitas

Estado: aceptada por el usuario; implementación pendiente al registrar este ADR.
Fecha: 2026-10-21.

## Contexto

El frontend recibe texto pero Submit/Tick generan respuestas simuladas. El usuario aprobó comenzar un backend local ligero, con derivación futura a IA únicamente cuando sea necesario. Esta etapa excluye modelos, red, voz y autonomía.

## Alternativas

- Añadir dominio textual a navi-state: menos archivos, pero mezcla la sesión visual con herramientas y conversación.
- Añadirlo a navi-platform: reutiliza entrada, pero acopla routing a Win32 y dificulta tests independientes.
- Crear navi-agent: crate puro sin dependencias externas, con adaptador de herramientas en la plataforma. Elegido por aislamiento, latencia acotada y pruebas deterministas.

## Decisión y contratos

Normalización → contexto acotado → router → registro/permisos → ejecutor → respuesta estructurada. Mensaje original y forma normalizada se distinguen; sólo se enrutan frases completas conocidas. Las órdenes desconocidas devuelven NeedsProvider y las ambiguas solicitan aclaración.

Herramientas iniciales: ayuda, hora local y estado básico de sólo lectura; apertura de calculadora mediante destino fijo de Windows y confirmación explícita por solicitud. Un ToolHost tipado aporta primitivas Windows y permite hosts falsos. Nunca se deriva una línea de shell del texto del usuario.

Una confirmación es de un solo uso, ligada al ID, herramienta y parámetros, con vencimiento. Una nueva solicitud invalida la pendiente. Contexto volátil limitado a ocho turnos y un referente explícito; no hay historial persistente ni inspección de otras aplicaciones.

La plataforma tendrá una única solicitud en vuelo en un worker bajo demanda. Un mensaje propio despierta al hilo UI al completar; no hay polling en reposo ni espera del worker en el hilo Win32. Los estados se traducen a Phase existentes sin cambiar los renderers.

La respuesta contiene estado, texto, datos, herramienta, error/confirmación y tiempos separados de routing/ejecución. NeedsProvider es el contrato futuro; no se implementa un proveedor, runtime async, framework de agentes ni serialización externa.

## Consecuencias

Un pequeño vocabulario explícito ofrece seguridad y baja latencia, pero no entiende lenguaje arbitrario. El éxito al lanzar calculadora no prueba que la aplicación esté lista o visible. Las pruebas usan hosts falsos y la apertura física se valida manualmente. Los límites y permisos se validan dentro del ejecutor, aunque el router ya haya reconocido la frase.

## Validación prevista

Tests de normalización, matching completo, parámetros, registro, permisos, confirmaciones, contexto, fallos y regresiones del frontend. Medición de routing y ejecución por separado; no retrasos artificiales para mostrar animaciones.

Referencias: [arquitectura](../architecture.md), [progreso](../progress.md), [validación](../validation.md).

## Evidencia de implementación — 2026-10-21

Implementado en `navi-agent`, `navi-platform/src/text_backend.rs` y los adaptadores Session/UI. Las herramientas de lectura usan Windows real; la calculadora se lanza desde el directorio de sistema fijo. Las pruebas ejecutan confirmaciones mediante hosts falsos y el diagnóstico nativo sólo solicita/cancela esa acción. Resultados y límites actuales en progress.md; este apartado no cambia las decisiones ni el alcance aprobado de este ADR.
