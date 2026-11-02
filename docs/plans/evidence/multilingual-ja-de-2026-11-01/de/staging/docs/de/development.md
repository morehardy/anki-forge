# SDKs aus dem Quellcode bauen

Diese Anleitung gilt für die Entwicklung im Repository und noch nicht veröffentlichte Änderungen. Für die veröffentlichten Pakete der Version `0.3.1` verwende die Installationsanleitung für [Rust](/docs/installation/), [Node](/docs/node-quickstart/) oder [Python](/docs/python-quickstart/).

Voraussetzungen: Git, Rust 1.92.0 mit Cargo und für das jeweilige SDK Node.js ab Version 22.13 mit npm oder CPython 3.11/3.12. Das Repository legt die Rust-Toolchain fest; installiere sie bei Bedarf mit `rustup toolchain install 1.92.0`.

<!-- section: source-builds -->
## Aus dem Quellcode bauen

So baust du den aktuellen Stand des Repositorys:

```sh
git clone https://github.com/morehardy/anki-forge.git
cd anki-forge
```

Rust-Anwendungen können `ankiforge = { path = "/absolute/path/to/anki-forge/anki_forge" }` in ihrem Manifest verwenden. Beispiele führst du mit `cargo run --locked -p ankiforge --example target_api_basic` aus.

Baue das Node-Addon aus diesem Repository:

```sh
cd bindings/node
npm run setup
npm run build
npm run pack:local
```

Installiere die tatsächlich von `pack:local` ausgegebenen Tarballs der Fassade und des nativen Pakets für dein System mit `npm install --offline --ignore-scripts --omit=optional /absolute/path/facade.tgz /absolute/path/native.tgz` in einer separaten Anwendung. Das sind Builds aus dem Quellcode, keine Installationen aus der Paketregistrierung. Die TypeScript-Konfiguration findest du in der [Node-API](/docs/node-api/#typescript).

Aktiviere für Python eine virtuelle Umgebung mit CPython 3.11/3.12, installiere Maturin und führe dann im Stammverzeichnis des Repositorys Folgendes aus:

```sh
python -m pip install maturin==1.15.0
maturin develop --manifest-path bindings/python/native/Cargo.toml --locked
```

Gewöhnliche installierte Wheels benötigen zur Laufzeit kein Cargo. Kombiniere Wrapper aus dem Quellcode nicht mit älteren nativen Binärdateien aus der Paketregistrierung.
