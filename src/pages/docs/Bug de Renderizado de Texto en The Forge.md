# 🐛 Postmortem Técnico — Bug de Renderizado de Texto en The Forge

## Resumen

Se corrigió un bug crítico en el outline de **Chapters** donde el nombre visible **no se actualizaba** tras un rename, **solo cuando el nuevo título superaba ~8 caracteres**.

Ejemplos observados:
- `"Chapt 1"` ✅ funciona
- `"Chapter 1"` ❌ no se actualiza visualmente
- `"hola ali"` ✅
- `"emily"` ✅

El estado interno y la lógica de `update()` **sí se ejecutaban correctamente**.  
El fallo era exclusivamente **visual**.

---

## Impacto

- El usuario podía renombrar capítulos correctamente en el modelo de datos.
- La UI mostraba un valor **obsoleto**, causando confusión y pérdida de confianza.
- El bug era **no determinista** desde el punto de vista lógico, pero **determinista por longitud del string**.

---

## Diagnóstico Final

### ❌ Qué NO era el bug

- No era un problema de `update()`
- No era un problema de `view()`
- No era un problema de `HashMap` / state
- No era un problema de UTF-8
- No era un problema de foco, eventos o double-click
- No era un problema de IDs o keys de Iced
- No era un problema de `TextInput`

### ✅ Qué SÍ era el bug

Un **bug de glyph caching / text batching** en el backend gráfico  
(**Iced 0.14 + wgpu en Windows 11**).

El renderer:
- Cacheaba incorrectamente un **único text-run largo**
- A partir de ~8 caracteres, el run **no se invalidaba**
- Reutilizaba primitivas antiguas aunque el contenido cambiara
- Ignoraba cambios legítimos del string renderizado

Este tipo de bug:
- No genera errores
- No crashea
- No deja logs
- No responde a invalidaciones normales
- Solo se manifiesta bajo ciertos patrones (longitud del texto)

---

## Intentos Previos (Fallidos)

Se probaron sin éxito:

- Keys versionadas (`stable_key_v`)
- Invalidación de scroll (`Id` alternante)
- Nudges invisibles (`\u{200B}`, `\u{200C}`)
- Forzar re-render con contadores visuales
- Cambio de pipeline a `TextInput` (descartado: rompe double-click)
- Prefijos invisibles
- Rebuild completo del outline

Todos fallaron porque **el problema no era el frame**, sino el **batching interno de texto**.

---

## Solución Final (Correcta)

### 🎯 Estrategia

Evitar que el renderer procese **un solo text-run largo**.

### 🛠 Implementación

El título del capítulo se renderiza como **múltiples widgets `text()`**, divididos dinámicamente en **chunks de 8 caracteres**, usando **slices `&str`**:

- Sin límites artificiales de longitud
- Sin modificar el modelo de datos
- Sin `clone()`
- Sin `format!()`
- Sin allocations
- Sin romper interacción (double-click intacto)

Cada chunk es un run independiente → el bug del atlas **no se dispara**.

---

## Código (Concepto Simplificado)

```rust
const CHUNK: usize = 8;

let mut row = Row::new();

let mut start = 0;
let mut count = 0;

for (i, _) in title.char_indices() {
    if count == CHUNK {
        row = row.push(text(&title[start..i]));
        start = i;
        count = 0;
    }
    count += 1;
}

row = row.push(text(&title[start..]));
