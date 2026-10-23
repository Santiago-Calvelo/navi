---
name: navi-subagent-orchestration
description: Coordina subagentes en el repositorio NAVI mediante documentos persistentes y fases explícitas de análisis, propuesta, solución y validación. Usar cuando una tarea de NAVI pueda dividirse en unidades independientes, requiera investigación paralela, contratos entre módulos o revisión separada, y siempre que el orquestador deba delegar trabajo sin depender de outputs conversacionales de los subagentes.
---

# Orquestación de subagentes para NAVI

Esta skill es específica del repositorio NAVI. No utilizarla para otros proyectos.

Aplicar los principios de `AGENTS.md`: cambios mínimos, alcance aprobado, propietarios exclusivos, evidencia verificable y responsabilidad centralizada.

## 1. Regla central

El orquestador, identificado mediante el alias **Lain**, dirige la tarea, administra los contratos y mantiene el estado consolidado.

Los subagentes no entregan resultados sustantivos por el canal de salida. Escriben análisis, propuestas, implementaciones documentadas y evidencia en archivos persistentes asignados.

Lain lee esos documentos, inspecciona los cambios reales, resuelve conflictos y comunica al usuario un único resultado consolidado.

Si la herramienta exige una respuesta textual del subagente, solicitar únicamente una confirmación mínima de escritura, preferentemente con la ruta del documento generado.

No transportar análisis extensos, diffs ni conclusiones mediante el output conversacional.

**Los documentos son el mecanismo de coordinación; no sustituyen la inspección del código ni las pruebas.**

## 2. Preparación obligatoria

Antes de delegar:

1. Leer `AGENTS.md`, `docs/progress.md`, `docs/architecture.md`, los ADR relevantes y las instrucciones cercanas a los archivos afectados.
2. Revisar `git status` y el estado real del repositorio.
3. Identificar el alcance autorizado y las exclusiones.
4. Separar los cambios existentes que no pertenecen a la tarea.
5. Determinar si la delegación aporta una ventaja real frente a la ejecución directa.
6. Elegir la modalidad de trabajo.
7. Asignar propietarios exclusivos de archivos y contratos.

Crear un directorio persistente por tarea:

`docs/agent-work/<identificador-de-tarea>/`

El identificador debe ser único, descriptivo y estable durante la tarea.

Ejemplo:

`docs/agent-work/2026-10-22-organic-motion/`

Estructura:

```text
docs/agent-work/<identificador>/
├── 00-brief.md
├── 01-analysis/
├── 02-proposal/
├── 03-solution/
└── 04-validation/
```

Lain debe crear previamente los directorios necesarios y asignar rutas de documentos individuales.

Los subagentes no deben modificar la estructura compartida ni crear archivos fuera de sus rutas autorizadas.

### Contenido de `00-brief.md`

Registrar:

- Objetivo.
- Problema y evidencia disponible.
- Alcance aprobado.
- Exclusiones.
- Modalidad de trabajo.
- Subagentes y modelos seleccionados.
- Archivos propietarios.
- Contratos entre módulos.
- Dependencias entre tareas.
- Criterios de aceptación.
- Pruebas previstas.
- Limitaciones conocidas.
- Condiciones para detener el trabajo.

Lain es el único propietario de `00-brief.md`.

## 3. Selección eficiente de modelos

Preferir subagentes más económicos que el orquestador cuando la tarea pueda resolverse adecuadamente con ellos.

Evaluar:

- Modelo disponible.
- Nivel de razonamiento necesario.
- Complejidad del código.
- Costo esperado de tokens y herramientas.
- Necesidad de investigación.
- Riesgo de errores y retrabajo.

No asumir que un modelo es más económico únicamente por su nombre.

No delegar tareas triviales si la coordinación cuesta más que la ejecución directa.

Si no existe un modelo claramente adecuado y más económico, Lain puede:

1. Resolver la tarea directamente.
2. Reducir el alcance delegado.
3. Utilizar un modelo de costo equivalente si está justificado y autorizado por las restricciones del entorno.

No detener automáticamente todo el trabajo por falta de una variante inferior.

No cambiar configuraciones de modelos o permisos que el entorno no permita.

Registrar en `00-brief.md` la elección y su justificación breve.

## 4. Fases y alias

Los alias son identificadores operativos. No representan personajes, identidades independientes ni firmas.

| Fase | Alias | Responsabilidad | Documento |
|---|---|---|---|
| Análisis | Arisu | Inspeccionar evidencia, contratos, riesgos y unidades separables | `01-analysis/<tema>.md` |
| Propuesta | Taro | Diseñar el cambio mínimo, alternativas e interfaces | `02-proposal/<tema>.md` |
| Solución | Eiri | Implementar el alcance aprobado y documentar resultados | `03-solution/<tema>.md` |
| Validación | Mika | Revisar independientemente el resultado integrado | `04-validation/<tema>.md` |

Lain conserva la responsabilidad de integración y entrega.

### Modalidades permitidas

**Modalidad A — Investigación**

`Arisu → Taro`

Utilizar para diagnóstico, exploración arquitectónica, inventarios y decisiones todavía no aprobadas.

No implementar código de producción.

**Modalidad B — Implementación acotada**

`Eiri → Mika`

Utilizar cuando el alcance, los contratos y los criterios de aceptación ya están definidos y aprobados.

No repetir investigaciones innecesarias.

**Modalidad C — Flujo completo**

`Arisu → Taro → aprobación → Eiri → Mika`

Utilizar para cambios complejos, nuevas interfaces entre crates, modificaciones de contratos compartidos o integraciones de mayor riesgo.

La aprobación se interpreta según `AGENTS.md`: si el usuario ya autorizó inequívocamente el alcance, no es necesario solicitar una segunda aprobación para ese mismo alcance.

Si aparece una ampliación no imprescindible, detenerse y solicitar autorización.

### Fases opcionales

No ejecutar fases por formalidad.

Lain puede resolver directamente análisis o propuesta sencillos y documentarlos en el brief.

La validación independiente es preferible cuando hay cambios relevantes de código, pero no reemplaza las comprobaciones finales del orquestador.

## 5. Contrato obligatorio de cada subagente

Cada encargo debe incluir:

1. Alias y fase.
2. Modelo seleccionado, cuando sea configurable.
3. Ruta exacta del documento de salida.
4. Archivos de código que puede modificar.
5. Archivos expresamente prohibidos.
6. Documentación que debe leer.
7. Contratos y dependencias.
8. Criterios de aceptación.
9. Pruebas autorizadas.
10. Exclusiones.
11. Condiciones de bloqueo.
12. Instrucción de comunicar resultados mediante documentos, no mediante respuestas extensas.

### Propiedad de archivos

Un archivo de código sólo puede tener un propietario de escritura activo.

Los demás subagentes pueden leerlo, pero no modificarlo.

Los contratos compartidos deben estabilizarse antes de paralelizar implementaciones dependientes.

Si una tarea requiere editar archivos de otro propietario, el subagente debe documentar la necesidad y detener esa modificación.

Lain decide cómo resolverla.

### Documentación protegida

Los subagentes no pueden modificar directamente:

- `AGENTS.md`.
- `docs/progress.md`.
- `docs/architecture.md`.
- ADR compartidos.
- `00-brief.md`.
- Documentos asignados a otros subagentes.

Pueden proponer modificaciones en su documento de salida.

Lain integra los cambios aprobados.

## 6. Contenido de los documentos

### Arisu — Análisis

Debe registrar:

- Evidencia inspeccionada.
- Hechos verificados.
- Inferencias.
- Información pendiente.
- Archivos y dependencias relevantes.
- Riesgos.
- Unidades potencialmente independientes.
- Preguntas abiertas.

No implementar.

### Taro — Propuesta

Debe registrar:

- Objetivo.
- Alternativa recomendada.
- Alternativas descartadas y motivos.
- Contratos propuestos.
- Propietarios de archivos.
- Secuencia mínima.
- Criterios de aceptación.
- Pruebas necesarias.
- Riesgos y exclusiones.

No implementar.

### Eiri — Solución

Debe registrar:

- Archivos modificados.
- Comportamiento implementado.
- Contratos utilizados.
- Comandos ejecutados.
- Resultados observados.
- Pruebas no ejecutadas.
- Limitaciones.
- Decisiones pendientes.

No declarar como aprobadas pruebas que no ejecutó.

No ampliar el alcance sin autorización.

### Mika — Validación

Debe registrar:

- Alcance revisado.
- Contratos comprobados.
- Diff y archivos inspeccionados.
- Pruebas ejecutadas directamente.
- Resultados.
- Fallos y regresiones.
- Evidencia disponible.
- Validaciones manuales pendientes.
- Recomendación: aceptar, corregir o bloquear.

Mika no debe corregir silenciosamente los problemas encontrados.

Debe documentarlos para que Lain decida la corrección.

## 7. Independencia de la validación

La documentación de Eiri no constituye prueba suficiente de corrección.

Mika debe evaluar el resultado utilizando:

- El brief aprobado.
- Los contratos establecidos.
- El árbol integrado.
- El diff real.
- Los criterios de aceptación.
- Las pruebas reproducibles.

Las conclusiones de Eiri pueden consultarse como evidencia secundaria, pero no deben sustituir una revisión independiente.

Cuando sea viable, Mika debe ejecutar sus propias comprobaciones.

No repetir pruebas costosas sin justificación.

No afirmar validación visual, física, multimonitor o de GPU si no se realizó.

Si Mika detecta una regresión, debe documentarla sin modificar código.

Lain decide si corresponde una corrección, una nueva validación o bloquear la entrega.

## 8. Paralelismo seguro

Permitir ejecución paralela únicamente cuando las tareas:

- No comparten archivos de escritura.
- No modifican simultáneamente contratos compartidos.
- No dependen de resultados todavía inexistentes.
- No compiten por recursos exclusivos.
- No interfieren con pruebas o builds de otros agentes.

Buenos candidatos:

- Inventarios de módulos independientes.
- Análisis de experimentos diferentes.
- Investigación de dependencias.
- Revisión de documentación separada.
- Pruebas de lectura sobre artefactos estables.

Evitar paralelizar:

- Cambios sobre el mismo contrato.
- Ediciones del mismo crate sin aislamiento suficiente.
- Actualizaciones de documentación compartida.
- Builds que utilicen simultáneamente los mismos artefactos de salida.
- La compilación release final.
- Integraciones que requieran resolver continuamente cambios cruzados.

La posibilidad de paralelizar no implica que siempre convenga hacerlo.

## 9. Secuencia de integración

Antes de delegar implementación, Lain debe comprobar:

1. Que el alcance está aprobado.
2. Que los contratos necesarios están definidos.
3. Que los propietarios no se superponen.
4. Que las dependencias entre tareas están resueltas.
5. Que no hay ampliaciones no autorizadas.

Después de recibir las soluciones:

1. Leer los documentos persistentes.
2. Inspeccionar el diff real.
3. Comprobar que los cambios respetan los contratos.
4. Integrar resultados compatibles.
5. Resolver personalmente conflictos y modificaciones cruzadas.
6. Ejecutar validación independiente sobre el árbol integrado.
7. Atender los fallos detectados.
8. Ejecutar formato, pruebas enfocadas y pruebas del workspace.
9. Compilar el ejecutable oficial cuando hubo cambios de producción.
10. Actualizar documentación compartida y `docs/progress.md`.
11. Entregar el resultado consolidado.

No aceptar una implementación únicamente porque su documento afirma que funciona.

## 10. Ejecutable oficial de NAVI

Toda modificación de código de producción debe respetar la sección 8 de `AGENTS.md`.

La ruta canónica es:

`F:\navi\target\release\navi-app.exe`

Desde PowerShell:

```powershell
Set-Location F:\navi
cargo build --release --offline -p navi-app
```

Comprobar que el comando corresponde al workspace real y que las dependencias están disponibles.

Después verificar:

```powershell
Get-Item 'F:\navi\target\release\navi-app.exe' |
    Select-Object FullName, LastWriteTime, Length
```

La compilación release final corresponde a Lain.

No crear ejecutables alternativos de producción.

Los experimentos independientes bajo `experiments/` pueden mantener sus propios binarios.

No considerar entregada una modificación de producción si el ejecutable oficial no pudo actualizarse.

No finalizar procesos abiertos de NAVI por la fuerza sin autorización.

## 11. Fallos, bloqueos y recuperación

Si un subagente:

- No escribe el documento asignado.
- Modifica archivos prohibidos.
- Amplía el alcance.
- Encuentra contratos incompatibles.
- No puede ejecutar pruebas esenciales.
- Detecta un conflicto de propiedad.

Debe registrar el bloqueo cuando sea posible y detener la operación afectada.

Lain no debe reconstruir silenciosamente el resultado de una fase utilizando una respuesta conversacional parcial.

Puede reintentar la tarea, reasignarla o ejecutarla directamente, dejando constancia de la decisión.

No borrar resultados parciales sin comprobar si contienen trabajo útil.

No sobrescribir documentos de otras tareas.

Si una tarea se interrumpe, el directorio persistente debe permitir reconstruir:

- Qué se autorizó.
- Qué se investigó.
- Qué se implementó.
- Qué se comprobó.
- Qué falta.

## 12. Documentación y cierre

Lain es responsable de actualizar una sola vez el estado consolidado en `docs/progress.md` al cerrar cada hito o interrupción significativa.

Actualizar `docs/architecture.md`, ADR y otras guías sólo cuando corresponda.

Los documentos de `docs/agent-work/` son evidencia de trabajo, no sustituyen la documentación permanente del proyecto.

No duplicar innecesariamente información extensa en `docs/progress.md`.

### Entrega al usuario

Informar:

1. Resultado consolidado.
2. Archivos y módulos modificados.
3. Pruebas ejecutadas y resultados.
4. Estado del ejecutable oficial, si corresponde.
5. Limitaciones y validaciones pendientes.
6. Próximo paso.

Mencionar los alias sólo cuando ayude a localizar evidencia.

No presentar la respuesta conversacional de un subagente como entrega independiente.

**Principio final: delegar para reducir trabajo y mejorar verificación, no para multiplicar agentes. Lain mantiene el control del alcance, la integración, la evidencia y la entrega.**