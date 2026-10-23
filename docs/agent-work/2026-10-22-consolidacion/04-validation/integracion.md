# Integración y evidencia consolidada

Cambios inspeccionados directamente; no se aceptaron sólo por los informes de agentes. 96 tests workspace aprobados (17 agent, 35 platform, 13 render, 30 state, 1 ui); una prueba render nativa ignorada por defecto se ejecutó separadamente y pasó. Formato y Clippy con todos los targets/-D warnings aprobados.

La revisión independiente encontró tres puntos: error persistente, fallback de halo sin monitor fiable y previews de capas. Resueltos: Error ahora expira sin protección con test dedicado; halo se omite si no hay monitor y cancela al cambiar monitor; preview STACK sólo runner explícito demo_submission, teclas F de diagnóstico bloqueadas en compacto normal y atajos globales de efectos no se registran durante uso normal. Family::StackV1/ActionProjection y módulos legacy retirados del catálogo compilado. Las fuentes históricas permanecen sin ejecutar.

Diagnóstico oficial --input-check terminó exit 0, ocho checks true: foco/entrada compacta, prompt a caret sin audio, Escape a Beacon, borrador, Enter/reloj Windows real, halo alpha/noactivación, vencimiento/liberación y desactivación. Sin captura compuesta, input externo ni apertura real de calculadora. Reporte artifacts/consolidation/input-check.json. Frame del halo guardado como DIB propio por helper histórico; no es captura de escritorio.

Primeros intentos de compilación detectaron referencias QA a UserPanel retirado; migradas al compacto. Test previo esperaba alternativas de clic: actualizado al contrato fijo. Clippy detectó rama redundante legacy: corregida. Estas fallas se revalidaron con workspace/Clippy exitosos. Build canónico exitoso, sin bloqueo ni terminación de procesos. No se lanzaron ejecutables alternativos.

Pendientes: visual/teclado físicos, apertura de calculadora, drag entre pantallas/DPI mixto y rendimiento. Canvas virtual aumenta consumo activo de memoria; no hay medición que permita prometer costo bajo. Inactividad visual de drag 1500 ms conserva origen; continúa al mover. ContextFragment/apps reales no implementado por carácter tentativo del pedido.
