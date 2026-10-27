# 0007 — Palimpsesto al iniciar NAVI

Estado: aceptada por pedido del usuario de usar las letras en el aire al iniciar NAVI, con texto variable y referencias sutiles a Serial Experiments Lain.
Fecha: 2026-10-26 (Argentina).

## Contexto

El laboratorio independiente contiene Palimpsesto: letras dispersas que se alinean y se disuelven. Su frase, composición y duración de estudio eran fijas. Se necesita una aparición de inicio con variedad, conservando el experimento original y la interfaz principal.

## Alternativas

- Una frase fija: no cumple la variedad solicitada.
- Aleatoriedad sin memoria: permite repeticiones inmediatas entre ejecuciones.
- Generación por LLM: introduce un proveedor, latencia y alcance no aprobados.
- Repertorio original y rotación persistente: variedad local, acotada y sin dependencia nueva.

## Decisión

Adaptar la composición de letras en un renderer de producción separado. Mostrar una frase original de un repertorio de 48 al inicio normal, una sola vez por proceso, durante 6000 ms. Las letras se reúnen hasta el 32 % de la vida, permanecen alineadas hasta el 76 % y se dispersan al terminar. El movimiento reducido usa un frame alineado y sólo un despertar de vencimiento.

La rotación usa una bolsa barajada sin reposición; al agotarla comienza otro ciclo evitando repetir la última frase en su frontera. `navi-state::startup` contiene repertorio, selección y serialización; `navi-platform::startup` guarda sólo índices en `%LOCALAPPDATA%\NAVI\startup-phrases.txt`. No contiene texto del usuario, actividad ni marcas temporales. Un estado ausente o inválido inicia otra bolsa. Si no se puede escribir, se permite la aparición y se registra el fallo en `%TEMP%\navi-startup-error.log`; en ese caso no se garantiza la continuidad de la rotación entre arranques.

Las frases son originales, con alusiones a señal, cables, reflejos, presencia y capas de realidad; no son citas ni afirman acciones del sistema. La selección ocurre una vez, nunca por frame.

Reutilizar el único overlay decorativo y el timer 4. Elegir el monitor del foreground al arrancar y centrar un canvas de hasta 800×460 DIP, reducido para caber en el área útil. No hay activación ni captura de clics. Se omite si el monitor está cubierto por fullscreen o Windows informa presentación/ocupado/no presente. Se excluyen las rutas diagnósticas.

Durante la lectura, las reacciones de foco y cursor no sustituyen el inicio. Invocar el compacto o la consola, ocultar NAVI, desactivar efectos, pedir un preview explícito o cambiar pantalla/DPI cancela la manifestación. El vencimiento libera el overlay y deja el indicador habitual. La cancelación no reprograma otra aparición.

## Consecuencias

Sin nuevos observadores, dependencias, voz, LLM ni alteración del laboratorio. El repertorio es finito: no se promete texto inédito ilimitado. El render usa DirectWrite/Direct2D y DIB premultiplicado existentes; tiene actividad sólo durante la aparición. No se afirma rendimiento medido ni legibilidad visual validada.

## Validación

Compilación release mediante `scripts/build-run.ps1`; resultado vigente en [progress.md](../progress.md). El agente no ejecuta tests, QA ni inspección visual. Guía del usuario en [validation.md](../validation.md).
