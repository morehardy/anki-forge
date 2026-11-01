# Integra Anki Forge en una aplicación de Rust

Usa Rust 1.92 o posterior. Instala el crate publicado en tu aplicación Cargo existente:

```sh
cargo add ankiforge@0.3.1
```

La exportación habitual de APKG no requiere instalar Anki. Sigue el [inicio rápido con Rust](/docs/quickstart/) para ver un primer programa completo y las instrucciones de `cargo run`.

<!-- section: cargo-dependencies -->
## Dependencias de Cargo

El crate publicado `ankiforge` incluye Basic, Cloze, tipos de nota personalizados, recursos multimedia cuyos datos conserva el proyecto, oclusión de imágenes, comparación y publicación de candidatos preparados. No hay características de Cargo activadas de forma predeterminada; el código normal de una aplicación no necesita las características exclusivas del repositorio.

El crate es una biblioteca nativa de Rust. Instala el enlazador y las herramientas de compilación de tu plataforma que requiera la cadena de herramientas de Rust. Consulta los [entornos verificados](/docs/compatibility/) al elegir dónde desplegar tu aplicación. Rust 1.92 es la versión mínima; que una plataforma aparezca en la lista no garantiza todos los destinos ni todas las configuraciones de herramientas de terceros.

Para leer JSON, añade explícitamente el análisis en tu aplicación:

```sh
cargo add serde --features derive
cargo add serde_json
```

El [tutorial de JSON a APKG](/docs/json-to-anki/#rust) incluye el analizador completo y el flujo con recursos multimedia y actualizaciones. El SDK no ofrece un conversor genérico de JSON o CSV.

<!-- section: choose-the-next-guide -->
## Elige la siguiente guía

- [Guía de creación con Rust](/docs/rust-guide/) para diseñar proyectos y modelos.
- [API de Rust](/docs/rust-api/) para métodos, informes y errores.
- [Recursos multimedia](/docs/media/) para imágenes y audio cuyos datos conserva el proyecto.
- [Actualización y distribución](/docs/updates/) para las condiciones de la versión base y la publicación.
- [Compilación desde el código fuente](/docs/development/) para desarrollar en el repositorio.
