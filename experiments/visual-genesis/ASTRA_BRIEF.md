# NAVI — Brief técnico para Astra Pro

## 1. Objetivo y límites

Construir un laboratorio visual nativo y experimental para Windows que permita diseñar e implementar nuevas familias visuales de NAVI sin modificar la aplicación existente. El laboratorio debe probar comportamiento espacial, temporal e interacción con el escritorio, no convertirse en una nueva pantalla de NAVI ni en un refactor del frontend actual.

Todo el trabajo experimental queda dentro de `experiments/visual-genesis/`. No modificar `Cargo.toml`, `Cargo.lock`, `crates/`, `docs/`, `README.md`, `VALIDATION.md` ni ningún otro archivo existente. No reutilizar ni imitar `SignalField`, `ContextFragment`, `LayerStack`, `FracturedHalo` ni sus animaciones. No usar WebView, Electron, Tauri, motores de juego o frameworks web. No agregar voz, LLM, automatización ni observación invasiva del escritorio.

La dirección artística queda abierta: Astra decide tesis visual, formas, paleta, ritmo, composición, nombres y algoritmos. Este documento fija únicamente el contrato técnico.

Preflight de la skill efectiva: `C:\Users\Santi\.codex\skills\navi-visual-genesis\SKILL.md`. Sus reglas aplicables quedan incorporadas aquí: novedad respecto del frontend existente, prototipos nativos ejecutables, transparencia, click-through, DPI, bajo trabajo en reposo y distinción entre triggers reales y simulados.

## 2. Arquitectura técnica seleccionada

Usar un paquete Cargo independiente en `experiments/visual-genesis/` con un binario nativo Rust 2021. Su `Cargo.toml` debe declarar un `[workspace]` propio para que Cargo no intente incorporarlo al workspace raíz. No depender de crates `navi-*`; la independencia debe permitir compilar, ejecutar y eliminar el laboratorio sin afectar NAVI.

La ruta gráfica recomendada es una ventana Win32 `WS_POPUP` layered, tool window y no activable, con una única superficie de dibujo para el laboratorio. Renderizar con Direct2D sobre un DIB de 32 bits BGRA con alpha premultiplicado y presentar mediante `UpdateLayeredWindow`. Usar DirectWrite sólo si el experimento necesita texto. No crear un HWND por partícula, línea, capa o elemento visual.

Inicializar COM en el hilo UI, establecer `DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2` antes de crear ventanas y ejecutar un message loop Win32 normal. Mantener el renderizador, la ventana y los recursos gráficos en el hilo UI; no bloquearlo con esperas, sleeps ni trabajo de simulación pesado.

No usar DirectComposition, Direct3D, DXGI o shaders en la primera implementación. La evidencia del repositorio muestra que existe una prueba aislada para esa ruta, pero Direct2D + DIB + `UpdateLayeredWindow` es la solución mínima y reutilizable para validar ideas visuales. Sólo introducir otra ruta si el experimento no puede expresarse con esta arquitectura y Astra documenta la razón dentro del laboratorio.

## 3. Estructura de archivos permitida

Crear exclusivamente esta estructura, ampliándola sólo cuando una necesidad del laboratorio lo justifique:

```text
experiments/visual-genesis/
├── ASTRA_BRIEF.md
├── Cargo.toml
├── Cargo.lock                  # generado por Cargo, si corresponde
├── src/
│   ├── main.rs
│   ├── platform.rs             # ventana, mensajes, DPI y ciclo de vida
│   ├── render.rs               # Direct2D/DirectWrite y presentación alpha
│   ├── experiment.rs            # contrato y selección de experimentos
│   └── experiments/             # una implementación por experimento
└── assets/                     # sólo recursos locales indispensables
```

No agregar archivos fuera de esta ruta. No copiar código del frontend existente; consultar el repositorio sólo como referencia de límites técnicos.

## 4. APIs y dependencias

Dependencia única recomendada: `windows = 0.54.0`, con las features estrictamente necesarias para `Win32_Foundation`, `Win32_Graphics_Direct2D`, `Win32_Graphics_Direct2D_Common`, `Win32_Graphics_DirectWrite`, `Win32_Graphics_Dxgi_Common`, `Win32_Graphics_Gdi`, `Win32_System_Com`, `Win32_System_LibraryLoader`, `Win32_UI_HiDpi` y `Win32_UI_WindowsAndMessaging`.

APIs esperadas: `CreateWindowExW`, `DefWindowProcW`, `GetMessageW`/`DispatchMessageW`, `SetTimer` o `InvalidateRect`, `D2D1CreateFactory`, `CreateDCRenderTarget`, `DWriteCreateFactory`, `CreateDIBSection`, `UpdateLayeredWindow`, `SetWindowPos`, `GetMonitorInfoW`, `MonitorFromWindow`, `GetDpiForWindow` y `WM_DPICHANGED`. Usar RAII/`Drop` para liberar DC, bitmap, brushes, formatos, targets y la ventana.

## 5. Contratos mínimos de experimentos y renderizado

Cada experimento debe declarar: identificador estable, descripción de una línea, trigger, estado inicial, actualización temporal, dibujo y condición de finalización. El experimento recibe un contexto acotado con tamaño físico de superficie, DPI, tiempo monotónico, estado de entrada y modo de movimiento reducido. No debe observar procesos, títulos, texto, credenciales ni historial de actividad.

El renderizador debe recibir un frame inmutable y devolver `Result<()>`. Separar actualización de estado, selección de experimento y dibujo. El frame debe poder dibujarse a una superficie transparente, con alpha premultiplicado, sin asumir una resolución fija. Redimensionar o recrear recursos cuando cambien el monitor, el DPI o el tamaño. Liberar la ventana y todos los recursos al cerrar.

El reposo no debe producir trabajo continuo: animar sólo mientras exista un experimento activo o una transición pendiente, usando timer o invalidación acotada. Al terminar, detener timers y dejar la superficie oculta o sin composición activa.

## 6. Reglas de interacción

La selección inicial se hace por línea de comandos: `--list`, `--experiment <id>`, `--reduced-motion` y `--monitor <n>` si Astra necesita fijar un monitor para pruebas. Dentro de una ejecución, permitir alternar experimentos con teclas simples sólo cuando la ventana tenga foco explícito; `Esc` debe cerrar y `R` reiniciar el experimento actual. No registrar hotkeys globales en la primera versión.

La ventana debe ser `WS_EX_NOACTIVATE` y click-through por defecto (`WM_NCHITTEST` devuelve `HTTRANSPARENT`). Si un experimento necesita interacción directa, debe habilitarla sólo durante una fase explícita, documentar el cambio y restaurar click-through al terminar. Nunca robar foco ni impedir el uso de otras aplicaciones. Los triggers de escritorio pueden ser simulados; indicar claramente en la ejecución cuáles son simulados y cuáles son reales.

## 7. Comandos de ejecución

Desde PowerShell en `F:\navi`:

```powershell
cargo fmt --manifest-path .\experiments\visual-genesis\Cargo.toml -- --check
cargo check --manifest-path .\experiments\visual-genesis\Cargo.toml
cargo run --manifest-path .\experiments\visual-genesis\Cargo.toml -- --list
cargo run --manifest-path .\experiments\visual-genesis\Cargo.toml --release -- --experiment <id>
cargo run --manifest-path .\experiments\visual-genesis\Cargo.toml --release -- --experiment <id> --reduced-motion
```

Si se requiere aislar artefactos de una instancia NAVI en uso, agregar `--target-dir .\experiments\visual-genesis\target` a los comandos Cargo. Para builds offline, usar `--offline` sólo si `windows 0.54.0` ya está disponible en la caché local; no asumir conectividad ni descargar dependencias durante la sesión creativa.

## 8. Criterios de aceptación

- Compila como paquete independiente sin modificar el workspace raíz.
- Arranca y termina limpiamente en Windows 10/11 x64 con Rust estable MSVC, Windows SDK y Visual Studio Build Tools disponibles.
- Presenta alpha correcto, no captura clics ni roba foco en modo pasivo.
- Reacciona a cambio de tamaño, monitor y DPI sin coordenadas fijas ni recursos obsoletos.
- No trabaja continuamente cuando no hay experimento activo.
- `--list`, selección por identificador, alternancia, `Esc`, `R` y `--reduced-motion` funcionan o quedan documentados como limitación concreta.
- Cada experimento incluye trigger, ejecución, distinción real/simulada y limitaciones breves.
- La validación visual/manual se reporta como pendiente hasta ejecutarse en Windows real; compilar no equivale a validarla.

## 9. Restricciones conocidas

El workspace actual usa Rust 2021 y `windows 0.54.0`; `rustc` verificado: `1.99.1`, MSVC estable esperado. La ruta existente usa DIB BGRA premultiplicado y `UpdateLayeredWindow`; su soporte para DPI mixto y pantalla completa exclusiva no debe considerarse validado automáticamente. El entorno actual puede bloquear la consulta normal de `git status` por `safe.directory`; no cambiar esa configuración desde el laboratorio. No hay autorización para modificar NAVI ni para afirmar pruebas de GPU, multimonitor o interacción física no ejecutadas.

## 10. Instrucción para Astra

No repitas esta investigación ni diseñes una arquitectura alternativa. Acepta este contrato como preflight técnico, concentra el presupuesto en diseñar e implementar la exploración visual original y deja el laboratorio ejecutable bajo `experiments/visual-genesis/`. Mantén breves las explicaciones y prioriza el comportamiento renderizado observable sobre documentación especulativa.
