# NAVI · Presencia compacta

Aplicación nativa en Rust, Win32, Direct2D y DirectWrite, con un núcleo textual determinista. El indicador y la tarjeta compacta son la experiencia inicial; el panel original se conserva como consola expandida. WindowFocus observa cambios reales de ventana activa y produce una manifestación breve. Las demos siguen siendo simuladas; el texto enviado normalmente usa herramientas locales explícitas. No hay modelo de IA ni reconocimiento de voz.

NAVI explora una interfaz que se integra con el escritorio de Windows como una presencia discreta. El proceso mantiene un indicador mínimo en reposo, muestra superficies transitorias sólo cuando hay una interacción o reacción válida y devuelve la presencia al estado mínimo al terminar.

Las reglas para investigar, aprobar alcance, dividir el trabajo y mantener diffs pequeños están en [AGENTS.md](AGENTS.md). La implementación no se inicia sólo por detectar una oportunidad: primero se analiza el problema, se comparan alternativas y se acuerda el alcance.

Esta revisión incorpora **LayerStack v2, FracturedHalo, clics y arrastres reales**. Contratos, responsabilidades de los tres subagentes, controles nuevos, límites y prueba manual: [docs/PARALLEL_EFFECTS.md](docs/PARALLEL_EFFECTS.md).

## Estado del proyecto

El prototipo actual compila para Windows x64 y conserva la consola expandida junto con la experiencia compacta. La ruta de foco, los renderers holográficos y el observador de cursor cuentan con pruebas automatizadas y diagnósticos nativos propios. La prueba manual entre aplicaciones de la revisión visual todavía debe ejecutarse con el usuario en Explorador, Firefox y VS Code; no se reemplaza por Computer Use ni por eventos sintéticos.

La etapa textual añade normalización, router, registro de herramientas, permisos, confirmación y contexto volátil. Una solicitud desconocida devuelve `NeedsProvider`; no hay proveedores conectados ni ejecución arbitraria de comandos. Arquitectura y estado actuales: [docs/architecture.md](docs/architecture.md) y [docs/progress.md](docs/progress.md).

Para escribir, hacé clic en el indicador o presioná **Ctrl+Alt+Espacio**. El panel compacto pregunta «¿Qué necesitás?»; tras dos segundos sin escribir muestra `_` titilando. Es espera de texto, sin micrófono. Escribí y usá Enter: `ayuda`, `qué hora es`, `estado del sistema` o `abrí la calculadora`. La calculadora se inicia sin código de confirmación. Su resultado dispara FracturedHalo; rojo si falla, posición y ángulo distintos en el monitor enfocado. Escape conserva el borrador. Ya no se abre «Tu espacio» ni se usa ActionProjection.

La entrada compacta y consola aceptan 120 caracteres; núcleo 512, respuesta visible 240 y contexto ocho turnos volátiles. Permisos, parámetros estrictos y confirmaciones opcionales se conservan. El resultado de calculadora confirma inicio aceptado por Windows, no ventana lista. Voz, IA y autonomía quedan fuera.

## Compilar y abrir

Requisitos: Windows 10/11 x64, Rust estable MSVC y Visual Studio Build Tools con Desktop development with C++ y Windows SDK. Desde PowerShell en `F:\navi`:

```powershell
cargo build --release --offline -p navi-app
.\target\release\navi-app.exe
```

El ejecutable oficial es únicamente `F:\navi\target\release\navi-app.exe`. Cerrar NAVI normalmente desde la bandeja antes de recompilar si Windows bloquea el archivo. No desviar una entrega de producción a otro `target-dir`. Inventario visual, diagnóstico y contrato de variación pendiente de aprobación: [docs/visual-recovery.md](docs/visual-recovery.md).

Arranca sin solicitar foco, mostrando un indicador `navi_` de **88 × 28 DIP**, a 20 DIP de los bordes inferior y derecho del área útil del monitor. La tarjeta mide **320 × 104 DIP**; el texto largo incrementa su altura hacia arriba. A escala 125%, la tarjeta inicial equivale a 400 × 130 píxeles físicos.

## WindowFocus automático

Al iniciar, las reacciones de foco están activadas. Sin invocar el panel, alterná entre Explorador de archivos, Firefox y VS Code, esperando al menos un segundo entre cambios. Una ventana principal válida dispara **LayerStack v2**, proporcional al monitor, durante unos **850 ms**. El anclaje inicial es junto a su borde superior derecho; puede cambiarse a NearCursor o SafeRandom. El indicador permanece visible durante la manifestación y al terminar; no se borra ni se contrae una sesión o consola abierta por el usuario.

La ruta es `EVENT_SYSTEM_FOREGROUND → DesktopObserver → ReactionEngine → SurfaceManager → overlay Direct2D`. El observador descarta ventanas propias, auxiliares, minimizadas y notificaciones que ya no corresponden al foreground actual. El motor agrupa duplicados, aplica un cooldown de **700 ms** y cancela dibujos obsoletos si el foco vuelve a cambiar. No hay disparos diferidos para ventanas que ya perdieron el foco.

El tamaño y anclaje usan el área útil y DPI del monitor de la ventana enfocada, sin cambiar la posición del indicador ni los anclajes de las demos. La ventana decorativa usa alpha por píxel, `WS_EX_TRANSPARENT`, `WS_EX_NOACTIVATE` y posicionamiento sin activación. Sus recursos y temporizador se liberan al terminar. No se leen títulos, texto, contraseñas ni contenido de aplicaciones, ni se guarda un historial de eventos.

Desde la bandeja o el menú contextual se puede marcar/desmarcar **Reaccionar a cambios de foco**. **Desactivar todos los efectos** también desactiva estas reacciones. Los efectos de sesión simulada son una opción independiente. Si no se pudo instalar el observador, NAVI sigue funcionando y registra el error en `%TEMP%\navi-observer-error.log`; la opción de foco queda deshabilitada.

La integración de foco anterior fue confirmada por el usuario en una prueba manual real. Esta revisión visual y las reacciones del mouse **requieren una nueva validación manual** según [docs/PARALLEL_EFFECTS.md](docs/PARALLEL_EFFECTS.md). No requiere Computer Use ni cambios en el navegador.

## Controles

| Acción | Control |
| --- | --- |
| Abrir / ocultar entrada compacta | Ctrl+Alt+Espacio; Alt+Espacio también disponible |
| Abrir con el mouse | Clic en el indicador o bandeja → Escribir |
| Consola original / compacta | Ctrl+Alt+N; Ctrl+N dentro de NAVI |
| Recorrer demo desde otra aplicación | Ctrl+Alt+D |
| Recorrer demo en consola | F6 o clic en su estado inferior |
| Estados simulados en consola | F1, F2, F3, F4, F5 |
| Reposo | F7 |
| Contraer sin terminar el proceso | Esc, F8, Alt+F4 o menú contextual; el indicador permanece |
| Reacciones automáticas de foco | Bandeja → Reaccionar a cambios de foco |
| Diagnósticos sin uso cotidiano | Bandeja → Sutura, SignalField o ContextFragment; éste usa datos simulados |
| Escucha simulada en consola | Ctrl+K; no activa audio |
| Texto compacto | Escribir, Backspace y Enter; Escape conserva borrador; sin selección/pegado nuevos |
| Resultado de acciones | Bandeja → FracturedHalo · resultados de acciones; duración 1,8 s |
| Terminar NAVI | Clic derecho → Salir de NAVI; Ctrl+Shift+Q dentro de NAVI |

El menú de clic derecho permite elegir los seis estados y configurar el indicador. Si Ctrl+Alt+Espacio y Alt+Espacio están ocupados, se conserva el acceso por clic y se registra el conflicto en `%TEMP%\navi-hotkeys.log`. Si el atajo global de demo está ocupado, se puede usar el menú o F6 en la consola.

Respuestas y errores sin borrador ni confirmación duran **cuatro segundos** y vuelven al indicador. Borradores, solicitudes activas, espera de teclado y consola no se cierran por este temporizador. Contraer explícitamente conserva el texto sin enviar.

Sólo una invocación explícita por teclado o clic solicita foco. El inicio y las reacciones pasivas no lo solicitan. Al contraer NAVI, se intenta devolverlo a la ventana anterior únicamente si NAVI todavía lo tenía. Las superficies son tool windows sin entrada en la barra de tareas ni Alt+Tab: se recuperan con atajos y desde la bandeja. Las preferencias duran esta ejecución.

## Arquitectura

Se conservan los seis crates del prototipo y se añade el núcleo textual puro:

| Componente | Responsabilidad |
| --- | --- |
| `navi-app` | Entrada y diagnóstico |
| `navi-agent` | Mensajes, contexto, router, registro, permisos y respuesta estructurada; sin Windows ni dependencias externas |
| `navi-platform::surface::SurfaceManager` | Indicador persistente, visibilidad, respuestas y planes transitorios de foco con monitor y DPI propios |
| `navi-platform::desktop_observer` | WinEvents fuera de proceso y metadatos geométricos del foreground |
| `navi-state::reactions::ReactionEngine` | Duplicados, cooldown, supresión y ciclo de vida; independiente de Win32 y renderer |
| `navi-platform::focus_reaction` | Coordinación de política, superficie y cancelación sin mutar la sesión |
| `navi-platform` | HWND, mensajes, monitor, DPI, foco, menús y temporizadores |
| `navi-render` | Direct2D/DirectWrite, superficie BGRA premultiplicada y recursos gráficos |
| `navi-ui` | Presentación, tamaños según contenido y hit testing |
| `navi-state` | Estados y eventos tipados independientes de Windows |
| `navi-demo` | Escenarios y señal determinista |

`SurfaceManager` no importa renderer, HWND ni APIs de Windows. Consume geometría y DPI y devuelve planes de presentación. El anclaje temporal reactivo se almacena por separado del anclaje de las demos. LayerStack v2, FracturedHalo, los pulsos de clic y los conectores de arrastre comparten un único overlay decorativo; v1 se conserva como alternativa de debug.

El HWND de NAVI cambia entre indicador, tarjeta y consola; la escritura usa un HWND independiente con controles Win32. La tarjeta elimina barra de título y pestañas; conserva firma, mensaje y estado. Se conserva el dibujo original de consola con arrastre y fijación. La posición de consola dura la ejecución. El renderer crea componentes reales, sin imágenes de fondo; Windows compone el escritorio a través del alpha. Se solicita Cascadia Mono a DirectWrite, con sustitución del sistema si falta; el campo nativo usa Segoe UI.

En reposo y oculto no hay temporizadores ni dibujo continuo. Escucha/procesamiento usan 20 FPS. La interferencia tiene un vencimiento monotónico de unos 160 ms al invocar o entrar en error. La respuesta usa un temporizador de vencimiento. Los mensajes tardíos de temporizadores cancelados se ignoran. No se vuelve a posicionar una ventana cuyos límites no cambiaron. Ante un fallo de dibujo se recrea el renderer una vez.

## Validación reproducible

```powershell
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
New-Item -ItemType Directory -Path artifacts\canonical -Force
$probe = Start-Process -FilePath 'F:\navi\target\release\navi-app.exe' -ArgumentList '--snapshot', 'F:\navi\artifacts\canonical\compact.bmp' -WindowStyle Hidden -Wait -PassThru
if ($probe.ExitCode -ne 0) { throw 'Verificación nativa fallida: revisar artifacts/canonical/compact.json' }
powershell.exe -ExecutionPolicy Bypass -File .\scripts\export-preview.ps1 -Source 'F:\navi\artifacts\canonical\compact.bmp' -Destination 'F:\navi\artifacts\canonical\compact.png'
Get-Content .\artifacts\canonical\compact.json
```

La prueba abre una ventana real y dirige mensajes Win32 a su propio HWND. Espera dos vencimientos reales de cuatro segundos. Exporta indicador, tarjeta, texto largo, escucha, procesamiento, respuesta, error, consola y cuatro escalas DPI, con un informe JSON. No equivale a una prueba física de teclado/ratón ni a mover la ventana entre monitores. Los PNG son artefactos de revisión, no recursos utilizados en la interfaz.

Un fallo de arranque se guarda en `%TEMP%\navi-error.log` y termina con código 1. Ver [validación actual y pasos manuales](docs/validation.md) y [VALIDATION.md](VALIDATION.md) para antecedentes.
