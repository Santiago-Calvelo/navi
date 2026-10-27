# 0001 — Superficies nativas y composición alpha

Estado: aceptada, reconstrucción documental de la implementación existente.
Fecha de registro: 2026-10-21.

## Contexto

NAVI debe integrarse al escritorio Windows con poco consumo y conservar Rust, Win32, Direct2D y DirectWrite. El código ya implementa presencia compacta y decoración independiente.

## Alternativas

- Geometría nativa en DIB premultiplicado y UpdateLayeredWindow: infraestructura existente y alpha verificable.
- DirectComposition/D3D/DXGI: prueba técnica aislada existente; mayor complejidad para la misma geometría actual.
- Framework web o motor de juegos: excluidos por decisión explícita del usuario.

## Decisión

Conservar la ruta Direct2D habitual y un único overlay decorativo por manifestación. SurfaceManager controla planes; ReactionEngine decide eventos; render sólo dibuja. No HWND por línea/plano/partícula y ningún dibujo continuo en reposo.

## Consecuencias y validación

La rasterización actual puede consumir CPU durante los efectos; no se afirma render GPU en toda la ruta. Alpha, transparencia al clic, foco y liberación se verifican mediante checks nativos propios; DPI mixto físico y fullscreen exclusivo siguen pendientes. Cambiar esta ruta requiere justificar un ADR que sustituya esta decisión.

Referencias vigentes: [arquitectura](../architecture.md) y [validación](../validation.md). Los informes históricos enlazados originalmente se retiraron por limpieza explícita del usuario el 2026-10-26; la decisión permanece sin cambios.
