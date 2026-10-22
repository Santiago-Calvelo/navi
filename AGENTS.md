# AGENTS.md — NAVI

Este archivo rige cualquier intervención de agentes en el repositorio NAVI. Las instrucciones específicas de un módulo pueden ampliar estas reglas, pero no contradecirlas sin aprobación explícita.

## 1. Propósito y visión

NAVI es una presencia computacional nativa para Windows: la computadora debe sentirse capaz de comunicarse con el usuario, no como un chatbot superpuesto. Su estética principal se inspira en *Serial Experiments Lain* (juego PS1, 1998); *Ex Machina* orienta la credibilidad y sobriedad; *Blade Runner 2049* aporta efectos selectivos de interferencia y proyección; Rainmeter/JaxCore inspiran skins y modularidad. Evitar HUDs espaciales genéricos, neón excesivo y animación permanente.

La prioridad actual es validar el frontend y las reacciones visuales a eventos cotidianos. Voz, LLM y automatización quedan fuera salvo aprobación explícita.

## 2. Lectura obligatoria al iniciar

Antes de proponer cambios:
1. Leer este `AGENTS.md` y las instrucciones más cercanas a los archivos afectados.
2. Revisar `git status`, cambios pendientes y estructura real del repositorio. No sobrescribir trabajo ajeno.
3. Leer `docs/progress.md`, `docs/architecture.md` y los ADR relevantes de `docs/decisions/`.
4. Revisar código, contratos, pruebas y documentación específicos del área afectada.
5. Si faltan documentos, están desactualizados o contradicen el código, informarlo. El código y las pruebas observables prevalecen como evidencia del estado implementado; los documentos registran intención y decisiones.

No asumir que el resumen de una sesión anterior equivale a una validación actual.

## 3. Principios de cambio

### Cambios mínimos
Si una modificación puede resolverse en una línea, resolverla en una línea. Conservar código, nombres, formatos y estructura existentes salvo necesidad directa. Evitar refactorizaciones oportunistas, cambios masivos de formato y dependencias no justificadas. Cada diff debe tener un propósito verificable.

### Alcance aprobado
Antes de editar, presentar brevemente:
- Problema y evidencia disponible.
- Archivos/módulos previstos.
- Comportamiento esperado y criterios de aceptación.
- Alternativas y costos relevantes.
- Validaciones posibles y limitaciones del entorno.
- Exclusiones explícitas.

Esperar aprobación para tareas que implican cambios, salvo que el usuario ya haya autorizado inequívocamente un alcance concreto. Si surge una ampliación no imprescindible, detenerla y solicitar aprobación. Reparaciones indispensables para compilar o preservar el contrato aprobado pueden realizarse, explicando su causa.

### Investigar antes de implementar
Distinguir **verificado**, **inferido** y **pendiente**. No declarar superadas pruebas visuales, manuales, de GPU, multimonitor o integración que no se ejecutaron. No confundir éxito de compilación con éxito funcional.

### Divide y vencerás
Dividir en unidades pequeñas con contratos explícitos y pruebas aisladas cuando sea posible. Evitar que dos agentes editen simultáneamente el mismo archivo. Integrar sólo resultados comprobables.

## 4. Límites arquitectónicos

- `navi-state`: estados, políticas y modelos puros; sin Win32 ni renderer.
- `navi-platform`: observación de eventos, ventanas, foco, superficies y recursos de Windows.
- `navi-render`: dibujo y recursos gráficos; no observa aplicaciones ni decide política.
- `navi-ui`: presentación, composición y geometría de interfaz.
- `navi-demo`: escenarios deterministas y datos simulados.
- `navi-app`: composición de módulos, entrada, ejecución y diagnósticos.

Mantener Rust, Win32, Direct2D y DirectWrite. DirectComposition/DXGI/Direct3D requieren justificación técnica y aprobación cuando amplíen la arquitectura. No introducir WebView, Electron, Tauri o motores de juego para la aplicación nativa.

Preservar indicador persistente, consola expandida, borradores y controles de diagnóstico. Las superficies decorativas deben ser transparentes donde corresponda, no capturar clics ni robar foco. Evitar HWND por partícula o línea. No bloquear el hilo de mensajes; no renderizar continuamente en reposo. Considerar DPI mixto, multimonitor, pantalla completa y liberación de recursos.

Los observadores de escritorio deben ser locales, mínimos y no intrusivos. No capturar texto, credenciales ni historiales de actividad; no añadir inyección o hooks invasivos sin autorización específica.

## 5. Flujo de trabajo obligatorio

1. Inspeccionar estado, documentación e instrucciones.
2. Presentar diagnóstico, alternativas y alcance.
3. Confirmar aprobación del alcance aplicable.
4. Definir o comprobar contratos entre módulos.
5. Implementar el cambio mínimo.
6. Ejecutar formato, pruebas enfocadas y compilación en el entorno disponible.
7. Integrar sólo cambios que compilen y respeten los contratos.
8. Ejecutar validaciones finales apropiadas; separar automatizadas de manuales.
9. Actualizar la documentación afectada, especialmente `docs/progress.md`.
10. Entregar archivos cambiados, resultados, evidencia, limitaciones y siguiente paso.

No crear commits, descartar cambios o alterar ramas ajenas sin autorización del usuario, salvo instrucciones explícitas del entorno.

## 6. Documentación viva (`docs/`)

La documentación es parte de la definición de terminado; debe ser útil para que otro agente continúe sin contexto conversacional.

### `docs/progress.md` — obligatorio
Es el **estado operativo actual**, no un diario exhaustivo ni una lista de promesas. Debe existir y mantenerse al día tras cada hito o interrupción significativa.

Mantener, en este orden:
1. Fecha de actualización y fuente de evidencia.
2. Objetivo/fase actual.
3. Estado por componente: implementado, verificado, parcial, pendiente o bloqueado.
4. Último hito y archivos/módulos relevantes.
5. Pruebas ejecutadas, resultados y pruebas pendientes, distinguiendo entorno real de simulación.
6. Riesgos, problemas conocidos y decisiones abiertas.
7. Próximo paso concreto y criterio de aceptación.
8. Historial breve de hitos con fechas, sin duplicar documentación extensa.

**Nunca inventar resultados** ni cambiar un estado reportado a «verificado» sin evidencia. Si el agente no puede editar el archivo por una interrupción, dejar un resumen de reanudación en su respuesta; el siguiente agente debe reconciliarlo con el repositorio.

### `docs/architecture.md`
Registrar arquitectura vigente, dependencias, contratos, flujo de eventos, ciclo de vida de superficies, rutas de renderizado y límites técnicos. Actualizar sólo cuando cambie un contrato o decisión arquitectónica. Los diagramas deben reflejar el código real.

### `docs/decisions/` — ADR
Registrar decisiones duraderas o costosas de revertir (p. ej. composición gráfica, observación de Windows, política de foco, estrategia multimonitor, elección de dependencias). Formato `NNNN-titulo-breve.md` con: estado (propuesta/aceptada/sustituida), fecha, contexto, alternativas, decisión, consecuencias, validación y vínculos. Nunca reescribir silenciosamente una decisión histórica: crear un ADR nuevo que la sustituya. Decisiones triviales no requieren ADR.

### `docs/visual-direction.md`
Registrar referencias aprobadas, principios visuales, skins, intensidad, estados, comportamiento de hologramas y restricciones de UX. Distinguir inspiración de implementación; no copiar activos de terceros.

### `docs/validation.md` y guías específicas
Registrar comandos reales, requisitos de entorno, procedimientos reproducibles, resultados y límites de cobertura. Las pruebas manuales deben incluir pasos y resultado esperado, sin marcarse aprobadas antes de su ejecución.

### Higiene documental
- Preferir enlaces relativos a archivos existentes.
- Una sola fuente de verdad por tema: progreso = estado; arquitectura = estructura; ADR = porqué; validación = evidencia; dirección visual = criterios estéticos.
- Actualizar secciones concretas, no reescribir documentos completos sin necesidad.
- Si el alcance sólo cambia documentación, verificar coherencia y enlaces; no modificar código.
- No registrar secretos, datos personales, telemetría sensible ni volcados innecesarios.

## 7. Subagentes y trabajo paralelo

El agente principal define contratos y asigna propietarios exclusivos de archivos o worktrees/ramas aisladas. Cada subagente recibe alcance, interfaces, pruebas y exclusiones. No se asume que los subagentes tienen contexto completo: deben leer los documentos aplicables.

Cada subagente entrega un resumen verificable: archivos tocados, cambios, comandos y resultados, limitaciones y decisiones propuestas. El agente principal integra, resuelve conflictos y **actualiza `docs/progress.md` una sola vez** con el estado consolidado. Los subagentes no deben editar concurrentemente `docs/progress.md` ni ADR compartidos; proponen sus entradas al integrador.

## 8. Validación y entrega

Usar comandos compatibles con el shell real (PowerShell, Git Bash, etc.) y no asumir disponibilidad de Computer Use. Distinguir:
- Pruebas unitarias y de lógica.
- Formato, Clippy y compilación.
- Diagnósticos nativos/sintéticos.
- Validación visual y manual sobre Windows real.
- Rendimiento medido (CPU, RAM, GPU), con método y duración.

Informar siempre qué se ejecutó, qué pasó, qué falló y qué no se pudo ejecutar. Una prueba sintética no reemplaza la interacción manual entre aplicaciones. Si una herramienta está restringida, respetar esa restricción y ofrecer pasos manuales; no intentar eludirla.

## 9. Criterio de terminado

Un hito sólo se considera cerrado si cumple el alcance aprobado, compila cuando corresponde, supera las pruebas ejecutables relevantes, preserva las garantías existentes y deja `docs/progress.md` actualizado. Si falta validación manual o de hardware, marcar el hito como **implementado, pendiente de validación**, no como completamente verificado.
