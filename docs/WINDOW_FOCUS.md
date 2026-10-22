> Actualización: el usuario confirmó manualmente la integración de foco anterior. LayerStack v2 y las reacciones nuevas se validan siguiendo [PARALLEL_EFFECTS.md](PARALLEL_EFFECTS.md); sus pruebas manuales siguen pendientes.

# Validación de WindowFocus en Windows

## Estado y alcance

Se completó la integración del observador, la política, el gestor de superficies y el renderer existente. El usuario confirmó la validación manual de esa integración. Este documento conserva el procedimiento; la revisión visual nueva y el mouse requieren la nueva validación indicada en PARALLEL_EFFECTS.md. No se utiliza Computer Use y no es necesario cambiar ninguna configuración del navegador.

DesktopObserver sólo escucha cambios reales de foreground. La revisión posterior incorpora clics y arrastres mediante CursorObserver, LayerStack v2 y FracturedHalo, conservando esta ruta y todos los controles demo. No hay automatización de aplicaciones.

## Compilar e iniciar

Usá una sola instancia. Si quedó otra ejecución de NAVI abierta, cerrala desde su bandeja antes de compilar, para que Windows no bloquee el ejecutable.

```powershell
Set-Location 'F:\navi'
cargo build --release --offline
.\target\release\navi-app.exe
```

Debe aparecer el indicador `navi_` junto a la esquina inferior derecha, sin invocar manualmente NAVI. El icono de bandeja tiene el tooltip `NAVI · presencia local`; puede estar dentro del área de iconos ocultos de Windows. El panel compacto no tiene que abrirse.

## Prueba manual esperada

1. Abrí Explorador de archivos, Firefox y VS Code en ventanas normales o maximizadas. Para la primera prueba evitá F11, videojuegos, presentaciones y modos de concentración.
2. En la bandeja de NAVI comprobá que **Reaccionar a cambios de foco** esté marcada. No confundas esa opción con **Efectos de sesión simulada**.
3. Alterná con Alt+Tab o clic entre las tres aplicaciones, esperando **al menos un segundo** entre cambios.
4. En cada cambio válido esperá unos planos translúcidos pequeños cerca del borde superior derecho de la ventana que acaba de recibir el foco. Duran aproximadamente **850 ms**, incluidas entrada y salida. El indicador debe permanecer visible y quedar como única presencia de NAVI si no había una sesión abierta.
5. Verificá que la aplicación enfocada siga respondiendo a teclado y clics. Podés usar un documento sin guardar o un área vacía para evitar modificar trabajo existente. La animación no debe activar NAVI ni impedir interactuar con el área que ocupa.
6. Hacé varias activaciones dentro de la misma ventana: no deben reiniciar el efecto. Si alternás entre ventanas en menos de **700 ms**, es normal que se omitan reacciones y se retire la anterior; no debe aparecer después un efecto sobre una ventana que ya perdió el foco.
7. Invocá NAVI con Alt+Espacio, escribí un borrador sin enviar y contraé con Esc. El indicador debe volver automáticamente. Al reinvocar, el texto debe seguir presente. Abrí la consola con Ctrl+Alt+N y comprobá que siga disponible. Las ventanas propias de NAVI no deben generar una reacción automática de foco.
8. Desmarcá **Reaccionar a cambios de foco** y repetí Alt+Tab: no deben aparecer estas manifestaciones. Volvé a marcarla para reactivarlas. **Desactivar todos los efectos** también debe cancelar la manifestación activa y conservar la interfaz funcional.
9. Si tenés varios monitores, repetí la prueba con ventanas en cada pantalla y con escalas diferentes. El efecto debe usar el DPI del monitor de la ventana enfocada, sin saltar al monitor del indicador, sin invadir la barra de tareas ni quedar fuera del área visible.
10. Probá opcionalmente Firefox en F11: la reacción automática debe suprimirse. Al salir de pantalla completa, cambiá a otra ventana normal y volvé a Firefox después de un segundo.

Marcá cada paso como aprobado o fallido durante tu prueba. Que un contador aumente no demuestra por sí mismo posición correcta, legibilidad o paso real de clics.

## Pruebas automatizadas

```powershell
cargo test --workspace --offline
cargo clippy --workspace --offline -- -D warnings
cargo fmt --all -- --check
New-Item -ItemType Directory -Path artifacts -Force
$probe = Start-Process -FilePath 'F:\navi\target\release\navi-app.exe' -ArgumentList '--snapshot', 'F:\navi\artifacts\window-focus.bmp' -WorkingDirectory 'F:\navi' -WindowStyle Hidden -Wait -PassThru
if ($probe.ExitCode -ne 0) { throw 'Falló el diagnóstico; revisar artifacts/window-focus.json y navi-error.log' }
Get-Content .\artifacts\window-focus.json
```

Los tests unitarios cubren filtro de mensajes, duplicados, cooldown, cancelación, eventos propios, supresión, monitor secundario con coordenadas negativas, DPI, ventanas pequeñas, anclaje de demos y conservación de la superficie de sesión.

El diagnóstico nativo registra y libera un WinEvent hook, inyecta **eventos sintéticos en el coordinador de NAVI** y dibuja un overlay real. Comprueba posición, píxeles con alpha, foreground sin cambios, estilos de no activación y transparencia, `WindowFromPoint`, vencimiento y liberación de recursos. No envía entrada a aplicaciones externas, no cambia sus ventanas y no prueba Alt+Tab físico. El observador real se mantiene desactivado durante `--snapshot` para que el resultado sea determinista.

Para una prueba opcional de observación real durante doce segundos:

```powershell
$probe = Start-Process -FilePath 'F:\navi\target\release\navi-app.exe' -ArgumentList '--observer-probe', 'F:\navi\artifacts\observer-manual.json' -WorkingDirectory 'F:\navi' -WindowStyle Hidden -PassThru
# Durante los próximos 12 segundos, alterná manualmente entre las aplicaciones.
$probe.WaitForExit()
Get-Content .\artifacts\observer-manual.json
```

Ese modo termina automáticamente y guarda sólo estado de instalación y contadores agregados, sin títulos ni historial de actividad. Un contador `reactions_emitted` positivo significa que la política admitió eventos; no certifica la comprobación visual manual.

## Diagnóstico y límites conocidos

- Si la opción de foco aparece deshabilitada, revisá `%TEMP%\navi-observer-error.log`. No requiere privilegios de administrador, DLL inyectadas ni cambios de seguridad. Una instalación fallida deja la interfaz disponible.
- Un fallo gráfico decorativo desactiva los efectos y se registra en `%TEMP%\navi-effects-error.log`. Un fallo general del ejecutable se guarda en `%TEMP%\navi-error.log`. Los archivos pueden corresponder a una ejecución anterior: comprobá su fecha.
- Sólo reaccionan ventanas principales visibles, no minimizadas, no ocultas por DWM, sin propietario, sin estilo tool window y con tamaño mínimo de 120 × 80 píxeles. Diálogos, tooltips y algunas ventanas auxiliares se omiten deliberadamente.
- Los avisos obsoletos de la cola se descartan si la ventana ya no tiene el foco. La supresión usa estado de notificaciones de Windows y geometría de pantalla completa; puede ser conservadora ante modos de concentración o presentación.
- Se lee únicamente HWND, PID, estado, límites de ventana, área útil del monitor y DPI. No se inspecciona texto, título, documentos ni contenido privado. No se persisten eventos normales.
- El efecto dura 850 ms y tiene cooldown fijo de 700 ms en este hito. Las opciones de duración del menú se aplican a las demos; la opción de animación también permite presentar estáticamente el efecto de foco durante su breve duración.
- El anclaje se toma al recibir el foco: no sigue movimientos o arrastres posteriores. La compatibilidad física entre varios monitores, apps elevadas y pantallas completas exclusivas queda pendiente de prueba manual.
- Una sesión o consola que el usuario dejó abierta no se contrae por acabar una reacción de escritorio. Si NAVI estaba en reposo, el indicador se mantiene durante y después del efecto.
