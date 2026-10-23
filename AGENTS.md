# AGENTS.md — NAVI

Este archivo rige cualquier intervención de agentes en el repositorio NAVI. Las instrucciones específicas de un módulo pueden ampliar estas reglas, pero no contradecirlas sin aprobación explícita.

## 1. Propósito y visión

NAVI es una presencia computacional nativa para Windows: la computadora debe sentirse capaz de comunicarse con el usuario, no como un chatbot superpuesto.

Su estética principal se inspira en *Serial Experiments Lain* (juego PS1, 1998); *Ex Machina* orienta la credibilidad y sobriedad; *Blade Runner 2049* aporta efectos selectivos de interferencia y proyección; Rainmeter/JaxCore inspiran skins y modularidad.

Evitar HUDs espaciales genéricos, neón excesivo y animación permanente.

La prioridad actual es validar y refinar el frontend, recuperar las animaciones aprovechables y desarrollar reacciones visuales a eventos cotidianos.

Voz, LLM y automatización quedan fuera de las tareas visuales salvo aprobación explícita.

### Principio de identidad

**NAVI no es un programa que controla la computadora. NAVI es la computadora comunicándose con el usuario.**

Las interfaces, animaciones y respuestas deben respetar esta identidad.

Las respuestas operativas deben ser directas, breves y representar resultados reales.

Preferir «Calculadora abierta» frente a «Solicité abrir la calculadora», siempre que la apertura haya sido confirmada.

No confundir acciones solicitadas con acciones completadas.

## 2. Lectura obligatoria al iniciar

Antes de proponer cambios:

1. Leer este `AGENTS.md` y las instrucciones más cercanas a los archivos afectados.
2. Revisar `git status`, cambios pendientes y estructura real del repositorio. No sobrescribir trabajo ajeno.
3. Leer `docs/progress.md`, `docs/architecture.md` y los ADR relevantes de `docs/decisions/`.
4. Revisar código, contratos, pruebas y documentación específicos del área afectada.
5. Identificar si la tarea afecta la aplicación principal o un experimento independiente.
6. Si faltan documentos, están desactualizados o contradicen el código, informarlo.

El código y las pruebas observables prevalecen como evidencia del estado implementado; los documentos registran intención y decisiones.

No asumir que el resumen de una sesión anterior equivale a una validación actual.

## 3. Principios de cambio

### Cambios mínimos

Si una modificación puede resolverse en una línea, resolverla en una línea.

Conservar código, nombres, formatos y estructura existentes salvo necesidad directa.

Evitar refactorizaciones oportunistas, cambios masivos de formato y dependencias no justificadas.

Cada diff debe tener un propósito verificable.

### Alcance aprobado

Antes de editar, presentar brevemente:

- Problema y evidencia disponible.
- Archivos y módulos previstos.
- Comportamiento esperado y criterios de aceptación.
- Alternativas y costos relevantes.
- Validaciones posibles y limitaciones del entorno.
- Exclusiones explícitas.

Esperar aprobación para tareas que implican cambios, salvo que el usuario ya haya autorizado inequívocamente un alcance concreto.

Si surge una ampliación no imprescindible, detenerla y solicitar aprobación.

Reparaciones indispensables para compilar o preservar el contrato aprobado pueden realizarse, explicando su causa.

### Investigar antes de implementar

Distinguir claramente:

- **Verificado:** comprobado mediante inspección o prueba.
- **Inferido:** conclusión razonable que todavía requiere comprobación.
- **Pendiente:** no investigado o no validado.

La validación visual, manual, de QA, GPU, multimonitor e integración sobre Windows corresponde al usuario. Los agentes no deben ejecutarla ni presentarla como realizada.

El agente sólo debe comprobar que el código compile y que la última línea del comando de compilación sea `OK`. Esto confirma compilación, no comportamiento funcional.

### Divide y vencerás

Dividir en unidades pequeñas con contratos explícitos y pruebas aisladas cuando sea posible.

Evitar que dos agentes editen simultáneamente el mismo archivo.

Integrar sólo resultados comprobables.

## 4. Límites arquitectónicos

- `navi-state`: estados, políticas y modelos puros; sin Win32 ni renderer.
- `navi-platform`: observación de eventos, ventanas, foco, superficies y recursos de Windows.
- `navi-render`: dibujo y recursos gráficos; no observa aplicaciones ni decide política.
- `navi-ui`: presentación, composición y geometría de interfaz.
- `navi-demo`: escenarios deterministas y datos simulados.
- `navi-app`: composición de módulos, entrada, ejecución y diagnósticos.

Mantener Rust, Win32, Direct2D y DirectWrite.

DirectComposition, DXGI, Direct3D o shaders requieren justificación técnica y aprobación cuando amplíen la arquitectura.

No introducir WebView, Electron, Tauri o motores de juego para la aplicación nativa.

Preservar el indicador persistente, la consola expandida, los borradores y los controles de diagnóstico.

Las superficies decorativas deben ser transparentes donde corresponda, no capturar clics ni robar foco.

Evitar HWND por partícula o línea.

No bloquear el hilo de mensajes ni renderizar continuamente en reposo.

Considerar DPI mixto, multimonitor, pantalla completa y liberación de recursos.

Los observadores de escritorio deben ser locales, mínimos y no intrusivos.

No capturar texto, credenciales ni historiales de actividad; no añadir inyección o hooks invasivos sin autorización específica.

## 5. Flujo de trabajo obligatorio

1. Inspeccionar estado, documentación e instrucciones.
2. Presentar diagnóstico, alternativas y alcance.
3. Confirmar aprobación del alcance aplicable.
4. Definir o comprobar contratos entre módulos.
5. Implementar el cambio mínimo.
6. Ejecutar la compilación disponible y terminar la comprobación con una última línea `OK` si el comando tuvo éxito.
7. Integrar sólo cambios que compilen y respeten los contratos.
8. Informar que la validación visual, manual y de QA queda a cargo del usuario; no bloquear la entrega por esas comprobaciones.
9. Para cambios de producción, actualizar y verificar el ejecutable oficial según la sección 8.
10. Actualizar la documentación afectada, especialmente `docs/progress.md`.
11. Entregar archivos cambiados, resultados, evidencia, limitaciones y siguiente paso.

No crear commits, descartar cambios o alterar ramas ajenas sin autorización del usuario, salvo instrucciones explícitas del entorno.

## 6. Documentación viva (`docs/`)

La documentación es parte de la definición de terminado; debe ser útil para que otro agente continúe sin contexto conversacional.

### `docs/progress.md` — obligatorio

Es el **estado operativo actual**, no un diario exhaustivo ni una lista de promesas.

Debe existir y mantenerse al día tras cada hito o interrupción significativa.

Mantener, en este orden:

1. Fecha de actualización y fuente de evidencia.
2. Objetivo o fase actual.
3. Estado por componente: implementado, verificado, parcial, pendiente o bloqueado.
4. Último hito y archivos o módulos relevantes.
5. Pruebas ejecutadas, resultados y pruebas pendientes, distinguiendo entorno real de simulación.
6. Estado del ejecutable oficial de NAVI: ruta, última compilación comprobada y limitaciones.
7. Riesgos, problemas conocidos y decisiones abiertas.
8. Próximo paso concreto y criterio de aceptación.
9. Historial breve de hitos con fechas, sin duplicar documentación extensa.

**Nunca inventar resultados** ni cambiar un estado reportado a «verificado» sin evidencia.

Si el agente no puede editar el archivo por una interrupción, dejar un resumen de reanudación en su respuesta; el siguiente agente debe reconciliarlo con el repositorio.

### `docs/architecture.md`

Registrar arquitectura vigente, dependencias, contratos, flujo de eventos, ciclo de vida de superficies, rutas de renderizado y límites técnicos.

Actualizar sólo cuando cambie un contrato o decisión arquitectónica.

Los diagramas deben reflejar el código real.

### `docs/decisions/` — ADR

Registrar decisiones duraderas o costosas de revertir, por ejemplo:

- Composición gráfica.
- Observación de Windows.
- Política de foco.
- Estrategia multimonitor.
- Elección de dependencias.
- Contratos compartidos de animaciones.
- Política de ejecutables y compilación.

Formato `NNNN-titulo-breve.md` con:

- Estado: propuesta, aceptada o sustituida.
- Fecha.
- Contexto.
- Alternativas.
- Decisión.
- Consecuencias.
- Validación.
- Vínculos relevantes.

Nunca reescribir silenciosamente una decisión histórica: crear un ADR nuevo que la sustituya.

Decisiones triviales no requieren ADR.

### `docs/visual-direction.md`

Registrar referencias aprobadas, principios visuales, skins, intensidad, estados, comportamiento de hologramas y restricciones de UX.

Distinguir inspiración de implementación; no copiar activos de terceros.

### `docs/validation.md` y guías específicas

Registrar comandos reales de compilación, requisitos de entorno y límites conocidos cuando la tarea lo requiera.

Las guías pueden documentar pruebas manuales para el usuario, pero los agentes no deben ejecutarlas ni marcarlas como aprobadas.

### Higiene documental

- Preferir enlaces relativos a archivos existentes.
- Una sola fuente de verdad por tema: progreso = estado; arquitectura = estructura; ADR = porqué; validación = evidencia; dirección visual = criterios estéticos.
- Actualizar secciones concretas, no reescribir documentos completos sin necesidad.
- Si el alcance sólo cambia documentación, verificar coherencia y enlaces; no modificar código.
- No registrar secretos, datos personales, telemetría sensible ni volcados innecesarios.

## 7. Subagentes y trabajo paralelo

El agente principal define contratos y asigna propietarios exclusivos de archivos o worktrees/ramas aisladas.

Cada subagente recibe alcance, interfaces, pruebas y exclusiones.

No se asume que los subagentes tienen contexto completo: deben leer los documentos aplicables.

Cada subagente entrega un resumen verificable:

- Archivos tocados.
- Cambios implementados.
- Comandos ejecutados y resultados.
- Limitaciones.
- Decisiones propuestas.

El agente principal integra, resuelve conflictos y **actualiza `docs/progress.md` una sola vez** con el estado consolidado.

Los subagentes no deben editar concurrentemente `docs/progress.md` ni ADR compartidos; proponen sus entradas al integrador.

La compilación final del ejecutable oficial corresponde al agente principal, después de integrar los cambios aprobados.

## 8. Ejecutable oficial y política de compilación

### 8.1. Regla fundamental

**NAVI debe tener una única ubicación canónica para su ejecutable de producción, siempre actualizada al finalizar cada entrega de código de producción.**

La ruta oficial es:

`F:\navi\target\release\navi-app.exe`

Esta ruta es la única referencia válida para ejecutar y validar la versión release de la aplicación principal.

No entregar versiones de producción desde rutas alternativas.

No crear copias con nombres como:

- `navi-new.exe`
- `navi-final.exe`
- `navi-v2.exe`
- `navi-test.exe`
- `navi-updated.exe`

No renombrar ni duplicar el ejecutable oficial como mecanismo de entrega.

La regla no prohíbe artefactos intermedios, binarios auxiliares de Cargo o herramientas de prueba: regula cuál es la versión oficial de NAVI.

### 8.2. Compilación obligatoria de producción

**Toda tarea que modifique código de producción debe finalizar intentando la compilación release del ejecutable oficial.**

El comando habitual del agente es el script de cierre, build y reinicio:

```powershell
Set-Location F:\navi
.\scripts\build-run.ps1
```

La única salida de éxito es `OK`: la instancia oficial terminó, el build release offline fue exitoso y NAVI quedó iniciado. La salida completa de Cargo se guarda en `artifacts/build-run/build.log` y `stdout.log`; leer esos logs sólo si falla. No seguir ni volcar toda la salida para comprobar un build exitoso.

No es necesario que el agente ejecute además:

- `cargo check`
- `cargo test`
- `cargo clippy`
- Una compilación debug.
- Una compilación aislada de un crate.
- Una compilación experimental.

Estas comprobaciones son opcionales y no sustituyen el intento de build release.

El script utiliza internamente el comando canónico, desde PowerShell:

```powershell
Set-Location F:\navi
cargo build --release --offline -p navi-app
```

Antes de utilizarlo, comprobar que coincide con la configuración real del workspace y que las dependencias están disponibles en la caché local.

Si `--offline` impide compilar por falta de dependencias, informar la causa. No modificar dependencias ni asumir autorización para descargarlas.

No cambiar el directorio de salida de producción mediante `--target-dir`, `CARGO_TARGET_DIR` o configuración equivalente sin aprobación.

Si el entorno contiene una configuración previa que redirige la salida, diagnosticarla y resolverla dentro del alcance autorizado antes de afirmar que se actualizó el ejecutable oficial.

### 8.3. Verificación del ejecutable

Después de cada intento de compilación release, basta con verificar:

1. Que la última línea del comando sea `OK`.
2. Si la compilación fue exitosa, informar la ruta canónica esperada.

No exigir al agente inspección visual, ejecución interactiva, tamaño, hash, fecha de modificación, proceso activo ni comparación de comportamiento del ejecutable.

### 8.4. Ejecutables bloqueados o procesos anteriores

El usuario autoriza que `scripts/build-run.ps1` cierre ordenadamente las instancias del ejecutable oficial antes de compilar y reinicie NAVI al terminar. El script identifica proceso por ruta y ventana por clase; solicita WM_QUIT al bucle de mensajes existente y espera hasta 10 segundos. Si no termina, aborta sin forzar el cierre. No afecta otros ejecutables ni experimentos.

Si el script falla por bloqueo, informar el error y consultar el log; no repetir indefinidamente ni crear otra entrega.

No finalizar procesos de forma forzada sin autorización.

No modificar mecanismos de seguridad, permisos o configuración global de Windows para evitar el bloqueo.

No generar un segundo ejecutable como solución temporal sin autorización explícita.

### 8.5. Excepción: experimentos independientes

Los experimentos desarrollados bajo `experiments/` pueden tener sus propios ejecutables y directorios de compilación.

Ejemplos:

`F:\navi\experiments\visual-genesis\`

`F:\navi\experiments\interface-genesis\`

Estos binarios son **prototipos experimentales, no versiones oficiales de NAVI**.

Deben cumplir estas reglas:

- Compilarse dentro del directorio experimental correspondiente.
- No reemplazar `navi-app.exe`.
- No modificar el workspace principal salvo aprobación explícita.
- No presentarse como entregas de producción.
- No mezclarse con los artefactos release oficiales.
- Documentar cómo ejecutarlos y qué partes son simuladas.

Una tarea exclusivamente experimental no requiere reconstruir el ejecutable oficial si no modificó código de producción.

Cuando un experimento se integre a NAVI, deja de ser una entrega exclusivamente experimental y debe aplicarse la compilación release oficial.

### 8.6. Fallos de compilación

Si la compilación release falla:

- No declarar el cambio como entregado.
- No afirmar que el ejecutable oficial está actualizado.
- Informar el error y los comandos ejecutados.
- Indicar que la compilación no terminó en `OK`.
- No generar ejecutables alternativos para ocultar el problema.

El ejecutable anterior puede permanecer disponible, pero debe identificarse como una versión previa, no como la implementación actual.

### 8.7. Entrega obligatoria

Al finalizar una tarea que modifica producción, el agente debe informar:

- Resultado del intento de compilación release y si la última línea fue `OK`.
- Ruta canónica del ejecutable que el usuario puede probar.
- Problemas de compilación conocidos.

El usuario debe poder ejecutar siempre la última entrega release exitosa desde:

```powershell
Set-Location F:\navi
.\target\release\navi-app.exe
```

### 8.8. Responsabilidad del agente principal

En trabajos con subagentes:

- Los subagentes pueden limitarse a compilar sus módulos.
- No deben crear versiones alternativas de producción para entregar.
- El agente principal integra los cambios.
- El agente principal ejecuta o coordina la compilación release final.
- El agente principal informa la ruta canónica esperada.
- El agente principal registra el resultado en `docs/progress.md`.

**Ningún subagente debe presentar validación visual o QA como realizada; su comprobación se limita a la compilación y la línea final `OK`.**

## 9. Animaciones y variación orgánica

Las animaciones de NAVI deben responder al contexto y evitar comportamientos visuales excesivamente repetitivos.

Cuando corresponda, permitir variaciones controladas de:

- Posición.
- Monitor y zona de aparición.
- Escala.
- Opacidad.
- Orientación.
- Ritmo.
- Duración.
- Geometría interna.

La variación debe respetar límites de pantalla, DPI, área disponible y legibilidad.

No utilizar aleatoriedad pura si perjudica la relación entre una acción y su manifestación.

Las animaciones vinculadas directamente al cursor deben mantener su anclaje espacial.

No desplazar arbitrariamente los efectos de clic, arrastre o selección.

La variación debe tener costo mínimo en reposo y permitir pruebas deterministas cuando sea necesario.

### Patrimonio visual experimental

Preservar los experimentos originales de Astra:

- Sutura.
- Palimpsesto.
- Nacre.
- Herbario.
- Umbral.

También investigar y conservar la animación existente `FracturedHalo`, si se confirma su implementación.

Estos experimentos no deben eliminarse ni reescribirse sin alcance aprobado.

El prototipo posterior de interfaz Margen no está aprobado como reemplazo del frontend principal.

Cualquier retirada de ese prototipo debe comprobar primero que no afecte a los experimentos gráficos que se desean conservar.

No integrar automáticamente todas las animaciones.

Cada integración debe justificar su función, costo, rendimiento y comportamiento esperado.

## 10. Validación y entrega

Usar comandos compatibles con el shell real (PowerShell, Git Bash, etc.) y no asumir disponibilidad de Computer Use.

Distinguir el intento de compilación de cualquier validación funcional que realizará el usuario.

Informar el comando de compilación y si terminó con `OK`. No ejecutar ni exigir pruebas unitarias, Clippy, diagnósticos nativos, validación visual/manual o mediciones de rendimiento salvo que el usuario las solicite expresamente.

Si una herramienta está restringida, respetar esa restricción y ofrecer pasos manuales; no intentar eludirla.

El ejecutable compilado queda listo para la validación del usuario; no presentarlo como visualmente validado por el agente.

## 11. Criterio de terminado

Un hito sólo se considera cerrado si:

1. Cumple el alcance aprobado.
2. Conserva los contratos arquitectónicos.
3. La compilación release se intentó y su última línea fue `OK`.
4. Preserva las garantías existentes.
5. Actualiza la documentación necesaria.
6. Deja `docs/progress.md` actualizado.
7. Si modificó código de producción, actualiza correctamente `F:\navi\target\release\navi-app.exe`.
8. Deja explícito que la validación manual, visual y de QA corresponde al usuario.

La validación manual, visual y de QA no forma parte del criterio de terminado del agente.

Si el ejecutable oficial no pudo actualizarse, marcar la entrega de producción como **bloqueada**, aunque el código compile en otro perfil o directorio.

**Regla final: NAVI debe mantener una única versión oficial release identificable, reproducible y confiable. Los experimentos pueden ser independientes, pero nunca deben confundirse con la aplicación principal.**
