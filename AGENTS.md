# AGENTS.md — NAVI

## 1. Reglas fundamentales

Estas reglas son obligatorias para cualquier agente o subagente que intervenga en el repositorio.

- **Nunca implementar sin aprobación explícita del usuario.**
- **Nunca ampliar el alcance aprobado sin solicitar autorización.**
- **Aplicar siempre el cambio mínimo posible.** Si algo puede resolverse modificando una línea, modificar únicamente esa línea.
- **Prohibido reformatear código existente:** no cambiar indentación, espacios, saltos de línea ni estilos ajenos al cambio.
- **Prohibido eliminar y reinsertar líneas para modificarlas.** Editar únicamente el fragmento necesario.
- **Prohibidos los cambios masivos entre clases, funciones o archivos**, las refactorizaciones oportunistas y las modificaciones cosméticas.
- **Usar exclusivamente la herramienta de edición de archivos** para modificar código o documentación. Nunca editar mediante terminal, scripts, redirecciones, reemplazos masivos o comandos equivalentes.
- Producir código limpio, legible por humanos y consistente con el proyecto.
- Mantener documentación inline mínima, únicamente cuando aporte contexto que el código no expresa por sí mismo.
- No crear commits, modificar ramas, descartar cambios ni sobrescribir trabajo existente sin autorización.

Estas reglas prevalecen sobre cualquier sugerencia de optimización, refactorización o simplificación del agente.

## 2. Investigación obligatoria

Antes de proponer cualquier implementación:

1. Leer `AGENTS.md` y las instrucciones aplicables.
2. Revisar `docs/`, especialmente `docs/progress.md`, `docs/architecture.md` y las decisiones relevantes.
3. Inspeccionar el código existente y comprobar si la funcionalidad solicitada ya existe, total o parcialmente.
4. Identificar cómo está implementada actualmente, sus dependencias, contratos y limitaciones.
5. Revisar el estado de Git para evitar sobrescribir cambios ajenos.

No asumir comportamientos ni arquitecturas sin comprobarlos.

Distinguir claramente entre información verificada, inferencias y aspectos pendientes de investigar.

**Investigar no autoriza implementar.**

## 3. Propuesta obligatoria antes de editar

Finalizada la investigación, presentar al usuario:

### Diagnóstico
- Estado actual y comportamiento observado.
- Causa del problema o necesidad identificada.
- Implementaciones existentes que puedan reutilizarse.

### Alternativas de implementación
Presentar al menos dos alternativas técnicamente viables cuando existan.

Para cada alternativa, especificar:
- Qué se hará y cómo funcionará.
- Archivos concretos que se crearán o modificarán.
- Líneas o rangos exactos previstos, obtenidos del código actual.
- Cambios puntuales que se realizarán.
- Ventajas, desventajas, riesgos y costos.
- Beneficio frente a las demás alternativas.

Si solo existe una alternativa razonable, justificarlo. No inventar opciones artificiales.

### Recomendación
Indicar qué alternativa recomienda el agente y por qué, priorizando:
1. Menor cantidad de líneas modificadas.
2. Reutilización de implementaciones existentes.
3. Menor impacto arquitectónico.
4. Legibilidad y mantenibilidad.
5. Menor riesgo de regresiones.

### Aprobación
**Detenerse y esperar la aprobación explícita del usuario.**

No interpretar una solicitud de análisis, diagnóstico o propuesta como autorización para implementar.

La aprobación de una alternativa autoriza exclusivamente el alcance descrito. Cualquier desviación requiere nueva aprobación.

## 4. Implementación autorizada

Una vez aprobado el alcance:

1. Modificar exclusivamente los archivos, líneas y comportamientos autorizados.
2. Utilizar herramientas de edición de archivos, nunca comandos de terminal para escribir.
3. Preservar estructura, formato, nombres y comportamiento no relacionados.
4. Evitar abstracciones, dependencias y archivos nuevos innecesarios.
5. Mantener comentarios inline mínimos y útiles.
6. Revisar el diff para detectar cambios accidentales.

**Reglas estrictas del diff:**
- No reindentar bloques existentes.
- No mover código sin necesidad aprobada.
- No eliminar y recrear líneas para producir el mismo resultado.
- No reformatear clases o archivos completos.
- No modificar código adyacente por conveniencia.
- No introducir mejoras fuera del alcance.

Si aparece un problema que requiere cambios adicionales, detener la implementación afectada, explicar el problema y solicitar autorización.

## 5. Arquitectura e identidad NAVI

NAVI es una presencia computacional nativa de Windows.

**NAVI no es una aplicación que controla la computadora; es la computadora comunicándose con el usuario.**

- Mantener Rust, Win32, Direct2D y DirectWrite.
- Respetar los límites entre `navi-state`, `navi-platform`, `navi-render`, `navi-ui` y `navi-app`.
- Evitar interfaces convencionales, HUDs genéricos y animaciones permanentes innecesarias.
- Priorizar reacciones visuales orgánicas, sutiles y vinculadas a eventos reales.
- Preservar transparencia, foco, interacción con ventanas, DPI y soporte multimonitor.
- No introducir nuevas tecnologías gráficas, dependencias o cambios arquitectónicos sin aprobación.
- Los experimentos de `experiments/` permanecen independientes hasta que se apruebe su integración.


## 6. Compilación y validación

**La única validación autorizada para el agente es comprobar que el script oficial de compilación devuelva `OK`.**

Para cambios de código de producción:

1. Ejecutar exclusivamente `scripts/build-run.ps1`.
2. Comprobar que la última línea de su salida sea exactamente `OK`.
3. Si devuelve `OK`, informar compilación exitosa.
4. Si no devuelve `OK`, informar el error sin declarar éxito.

**Prohibiciones:**
- No realizar validaciones visuales, manuales ni funcionales.
- No ejecutar tests, Clippy, benchmarks ni comprobaciones adicionales.
- No inspeccionar visualmente la aplicación ni interactuar con su interfaz.
- No realizar verificaciones de GPU, DPI, multimonitor o rendimiento.
- No sustituir el script oficial por comandos de compilación alternativos.
- No interpretar `OK` como garantía de funcionamiento correcto.

El único ejecutable oficial es `F:\navi\target\release\navi-app.exe`.

Los experimentos independientes quedan excluidos de la compilación oficial cuando no modifican producción.

**La validación funcional y visual corresponde exclusivamente al usuario.**


## 7. Documentación y entrega

Actualizar únicamente la documentación afectada por los cambios aprobados.

- `docs/progress.md`: estado real, hitos y pendientes.
- `docs/architecture.md`: cambios arquitectónicos aprobados.
- `docs/decisions/`: decisiones técnicas duraderas.
- `docs/validation.md`: procedimientos y evidencia de validación.

No reescribir documentos completos ni generar documentación redundante.

Al finalizar, informar brevemente:
- Archivos y líneas efectivamente modificados.
- Cambios realizados y correspondencia con el alcance aprobado.
- Resultado de compilación y validaciones realmente ejecutadas.
- Problemas, limitaciones y pendientes.

**Flujo obligatorio: investigar → proponer alternativas → esperar aprobación → editar mínimamente → ejecutar script de compilación → leer únicamente la última línea (`OK`) → informar.**

El agente debe recibir únicamente `OK` como resultado de una compilación exitosa. La salida completa debe permanecer redirigida a los archivos de log, sin mostrarse ni leerse.

Solo si la compilación falla, podrá consultar los logs necesarios para diagnosticar el error. No ejecutar validaciones adicionales.
