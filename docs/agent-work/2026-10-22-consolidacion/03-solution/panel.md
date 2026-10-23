# Eiri — entrada compacta

Fecha: 2026-10-22. Aplicación principal, alcance aprobado en 00-brief.md.

## Archivos implementados

- `crates/navi-state/src/lib.rs`: `KeyboardEntry` puro, eventos KeyboardInvoke/KeyboardTick/KeyboardCancel; invocación textual sin Listening, pregunta 2000 ms, cursor alterna cada 500 ms. Inicio/final de solicitud y reset cancelan la entrada temporal. Borrador y solicitud activa/confirmación conservados.
- `crates/navi-ui/src/lib.rs`: headline presenta pregunta, `_` o blanco; mientras haya borrador presenta texto. Estado «escribí / enter»; tamaño compacto estable al ocultar cursor.
- `crates/navi-render/src/lib.rs`: pie «esc / ocultar» durante entrada; no waveform ni demo en esa ruta. Retirados módulos compilados action_projection y user_panel por instrucción de Lain; sus fuentes históricas permanecen sin uso.
- `crates/navi-platform/src/keyboard_entry.rs`: nuevo adaptador para HWND principal, UTF-16, envío real, foco sólo explícito, timer 5 fuera del vector histórico 1..4. Resuelve monitor del foco anterior al invocar; sin HWND nuevo ni dependencia.

## Integración requerida (Lain, archivos fuera de propiedad)

1. `mod keyboard_entry;` en plataforma lib.rs.
2. Invocación cotidiana llama `keyboard_entry::invoke(hwnd, focus)` en vez de user_panel::show; diagnóstico legacy puede conservar invoke anterior. Hotkey normal oculta si ya está Compact con foco, invoca de lo contrario.
3. En sync de la superficie, después de calcular `plan` y **antes de return por invisibilidad**: `keyboard_entry::sync(hwnd, &app.session, plan.kind, app.now());`. Este helper mata timer 5 y agenda sólo si Compact, foreground propio, phase Invoked, input vacío y keyboard_entry activo. Usa tiempo real para evitar que otros redraws posterguen el deadline.
4. Antes del guard de timers 1..4 en WM_TIMER: si `wp.0 == 5`, llamar `keyboard_entry::tick(hwnd)` y devolver LRESULT(0).
5. En WM_CHAR normal (no demo_submission), ruta compacta llama `keyboard_entry::character(hwnd, wp)` y devuelve. Consola conserva decoder existente; Enter llama backend, jamás demo si entrada vacía.
6. WM_SETFOCUS/WM_KILLFOCUS: commit noop para reprogramar o suspender timer. Al perder foco no debe haber polling. `sync` usa GetForegroundWindow: WM_KILLFOCUS puede preceder actualización final del foreground; por ello matar timer 5 explícitamente en WM_KILLFOCUS, sin borrado de draft. WM_SETFOCUS commit noop reanuda.
7. `keyboard_entry::cancel(hwnd)` antes de hide, toggle_console, destruir; conserva input y answer, cancela timer. En hide/console commit ya existente dibuja estado final. BeginRequest/CompleteRequest/Reset cancelan puro automáticamente.
8. Retirar footer compacto que dispara Demo al hacer clic: `surface_hit` hoy devuelve Hit::Demo en esquina inferior; cambiarlo a Input para toda área compacta si se desea coherencia completa. No lo modifiqué porque QA viejo puede depender y Lain migra diagnósticos.
9. Mantener respuesta ya visible si await_confirmation/executing: KeyboardInvoke sólo cambia Mode, no borra answer ni phase en esos casos. Una respuesta completada normal se reemplaza al invocar nueva entrada; un borrador existente permanece.

API: `invoke(HWND,bool)`, `tick(HWND)`, `cancel(HWND)`, `character(HWND,WPARAM)`, `sync(HWND,&Session,SurfaceKind,u64)`; todas pub(super) unsafe.

## Pruebas ejecutadas

- `cargo test -p navi-state --offline keyboard_ --quiet`: 2 aprobadas nuevas.
- `cargo test -p navi-state -p navi-ui --offline --quiet`: 30 state + 1 UI, todas aprobadas en ejecución final. Ejecución inicial simultánea a cambios de otro agente observó fallo transitorio de reactions::dragging_resumes_after_inactivity_and_release_over_navi_always_ends, luego repetida satisfactoriamente.
- `cargo test -p navi-render --offline --quiet`: 13 aprobadas; 1 nativa ignorada.
- `cargo clippy -p navi-state -p navi-ui -p navi-render --all-targets --offline -- -D warnings`: aprobado.
- `rustfmt --check --edition 2021 --config skip_children=true` sobre los cuatro archivos Rust propios: aprobado.

## Límites y revisión propuesta

La compilación plataforma requiere hookups por Lain y eliminación de dependencias de QA hacia paint_user_panel; no se afirma compilada esa integración ni release. La entrega oficial/release final y progress.md son responsabilidad del integrador. Sin captura de escritorio, prueba manual, audio, CPU/GPU ni monitores físicos.

El modelo de edición conserva límites y decoder existentes (120 caracteres, append/backspace, no selección/portapapeles nuevos). El cursor titila sólo durante espera vacía y con ventana enfocada; al escribir se muestra borrador y se suspende timer. Las demos Listening siguen disponibles en consola/diagnóstico, no se usan para invocación de teclado.

Pendiente manual: invocar sin texto, observar pregunta→cursor sin ondas; escribir antes de 2 s y comprobar que el texto no se borra; Esc→Beacon→invocar conserva draft; Enter hora muestra resultado real y termina cursor; consola y retorno conservan draft; perder foco suspende timer; dos monitores/DPI conservan geometría viable.
