# Anki Forge in deine Rust-Anwendung integrieren

Canonical page: https://ankiforge.dev/de/docs/installation/
Language: de
Description: Installiere das veröffentlichte Rust-Crate, konfiguriere Cargo-Abhängigkeiten und wähle die nächste Anleitung.

Verwende Rust ab Version 1.92. Installiere das veröffentlichte Crate in deiner bestehenden Cargo-Anwendung:

```sh
cargo add ankiforge@0.3.1
```

Für einen gewöhnlichen APKG-Export muss Anki nicht installiert sein. Ein vollständiges erstes Programm und die Schritte für `cargo run` findest du im [Rust-Schnellstart](https://ankiforge.dev/de/docs/quickstart/).

## Cargo-Abhängigkeiten

Das veröffentlichte Crate `ankiforge` umfasst Basic, Cloze, eigene Notiztypen, Medien mit eigenem Datenbesitz, Bildverdeckung, Vergleiche und vorbereitete Veröffentlichungen. Standardmäßig sind keine Features aktiviert; gewöhnlicher Anwendungscode benötigt keine nur im Repository verwendeten Features.

Das Crate ist eine native Rust-Bibliothek. Installiere den Linker und die Build-Werkzeuge, die deine Rust-Toolchain für deine Plattform benötigt. Beachte bei der Wahl einer Bereitstellungsumgebung die [geprüften Umgebungen](https://ankiforge.dev/de/docs/compatibility/). Rust 1.92 ist die Mindestversion. Eine aufgeführte Plattform ist keine Garantie für jedes Zielsystem oder jede Konfiguration einer externen Toolchain.

Füge für JSON-Eingaben die Parser-Abhängigkeiten in deinem Anwendungscode ausdrücklich hinzu:

```sh
cargo add serde --features derive
cargo add serde_json
```

Die [JSON-zu-APKG-Anleitung](https://ankiforge.dev/de/docs/json-to-anki/#rust) enthält den vollständigen Parser sowie den Medien- und Aktualisierungsablauf. Das SDK stellt keinen allgemeinen JSON- oder CSV-Konverter bereit.

## Die nächste Anleitung wählen

-   [Rust-Anleitung](https://ankiforge.dev/de/docs/rust-guide/) für den Aufbau von Projekten und Modellen
-   [Rust-API](https://ankiforge.dev/de/docs/rust-api/) für Methoden, Berichte und Fehler
-   [Medien](https://ankiforge.dev/de/docs/media/) für Bilder und Audio mit eigenem Datenbesitz
-   [Aktualisieren und verteilen](https://ankiforge.dev/de/docs/updates/) für Basisversionen und Veröffentlichungsbedingungen
-   [Aus dem Quellcode bauen](https://ankiforge.dev/de/docs/development/) für die Entwicklung im Repository
