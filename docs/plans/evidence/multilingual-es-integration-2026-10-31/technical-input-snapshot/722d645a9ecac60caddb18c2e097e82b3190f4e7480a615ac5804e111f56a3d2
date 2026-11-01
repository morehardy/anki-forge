# Compila los SDK desde el código fuente

Usa esta guía para desarrollar en el repositorio y probar cambios aún no publicados. Para los paquetes publicados de `0.3.1`, utiliza la guía de instalación de [Rust](/docs/installation/), [Node](/docs/node-quickstart/) o [Python](/docs/python-quickstart/).

Requisitos: Git, Rust 1.92.0 con Cargo y Node.js 22.13 o posterior con npm, o CPython 3.11/3.12 para el SDK correspondiente. El repositorio fija la versión de la cadena de herramientas de Rust; instálala con `rustup toolchain install 1.92.0` si hace falta.

<!-- section: source-builds -->
## Compilación desde el código fuente

Para compilar la versión actual del repositorio:

```sh
git clone https://github.com/morehardy/anki-forge.git
cd anki-forge
```

Las aplicaciones de Rust pueden usar `ankiforge = { path = "/absolute/path/to/anki-forge/anki_forge" }` en su manifiesto. Ejecuta los ejemplos con `cargo run --locked -p ankiforge --example target_api_basic`.

Compila el complemento nativo de Node desde esta copia del repositorio:

```sh
cd bindings/node
npm run setup
npm run build
npm run pack:local
```

Instala en una aplicación independiente los archivos tar del paquete de interfaz y del paquete nativo para tu plataforma que muestre `pack:local`, con `npm install --offline --ignore-scripts --omit=optional /absolute/path/facade.tgz /absolute/path/native.tgz`. Son compilaciones del código fuente, no instalaciones desde el registro. La configuración de TypeScript está en la [API de Node](/docs/node-api/#typescript).

Para Python, activa un entorno virtual de CPython 3.11/3.12, instala Maturin y ejecuta lo siguiente desde la raíz del repositorio:

```sh
python -m pip install maturin==1.15.0
maturin develop --manifest-path bindings/python/native/Cargo.toml --locked
```

Los wheels instalados normalmente no necesitan Cargo durante la ejecución. No mezcles las capas de interfaz del código fuente con binarios nativos de versiones anteriores del registro.
