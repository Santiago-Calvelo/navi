# 0003 — Panel del usuario y acciones directas

Estado: aceptada por solicitud explícita del usuario.
Fecha: 2026-10-22.
Sustituye la confirmación obligatoria de calculadora y la entrada compacta cotidiana del [ADR 0002](0002-nucleo-textual-determinista.md); sus demás contratos siguen vigentes.

## Contexto y alternativas

El usuario considera confuso invocar NAVI sin un campo visible y escribir «confirmar 4» para una acción simple. Reutilizar la tarjeta mantiene la ambigüedad entre usuario y presencia; una ventana de escritura independiente permite un EDIT nativo con cursor, selección, pegado y botón Enviar. Un framework nuevo o un renderer de texto editable propio añadirían costo innecesario.

## Decisión

Ctrl+Alt+Espacio, Alt+Espacio o clic en el indicador abren «Tu espacio», un HWND independiente de 540 × 244 DIP, ajustado al área útil. Sólo una apertura explícita solicita foco. Escape oculta y conserva el borrador. El indicador, tarjeta de respuesta y consola permanecen separados. SurfaceManager evita superponer la respuesta al panel cuando hay espacio.

Abrir la calculadora con una frase completa registrada se ejecuta directamente por defecto. Se siguen validando permisos, parámetros y destino fijo. `Permissions.confirm_launch` permite una política más restrictiva; las herramientas que declaren confirmación obligatoria siguen exigiéndola. No se habilita shell, ejecución arbitraria ni herramientas destructivas.

Una respuesta de ejecución de calculadora inicia ActionVisual durante 1800 ms: reutiliza LayerStack v2 y FracturedHalo, con barrido e interferencia cromática breve. Tiene prioridad temporal sobre reacciones pasivas y utiliza el mismo overlay transparente sin activación ni captura de clics. La máquina de estados es pura; el posicionamiento pertenece a SurfaceManager y el dibujo a navi-render. El menú permite desactivar estas animaciones por separado o todos los efectos. Sin movimiento se muestra una proyección estática temporal; en reposo no hay temporizador de efecto.

## Consecuencias y validación

La ventana de escritura es opaca para alojar controles Win32 de texto fiables; las proyecciones de NAVI conservan alpha por píxel. Se añade sólo una feature Controls a los bindings Windows existentes, sin paquetes nuevos. La ejecución confirma aceptación del inicio por Windows, no disponibilidad de la calculadora.

Tests con host falso verifican ejecución directa, denegación y política estricta. El diagnóstico nativo usa sólo hora/estado y solicita/cancela calculadora bajo política estricta; reproduce la proyección sin lanzar aplicaciones. Uso físico, apertura de calculadora y valoración estética siguen siendo validaciones manuales. Véanse [validación](../validation.md) y [progreso](../progress.md).
