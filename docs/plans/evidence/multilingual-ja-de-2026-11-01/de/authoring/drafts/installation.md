# Anki Forge in eine Rust-Anwendung integrieren

Verwende Rust ab Version 1.92. Installiere das veröffentlichte Crate in deiner bestehenden Cargo-Anwendung:

[[CODE0]]

Für einen gewöhnlichen APKG-Export muss Anki nicht installiert sein. Ein vollständiges erstes Programm und die Schritte für `cargo run` findest du im [Rust-Schnellstart](/docs/quickstart/).

<!-- section: cargo-dependencies -->
## Cargo-Abhängigkeiten

Das veröffentlichte Crate `ankiforge` umfasst Basic, Cloze, eigene Notiztypen, Medien mit eigenem Datenbesitz, Bildverdeckung, Vergleiche und vorbereitete Veröffentlichungen. Standardmäßig sind keine Features aktiviert; gewöhnlicher Anwendungscode benötigt keine nur im Repository verwendeten Features.

Das Crate ist eine native Rust-Bibliothek. Installiere den Linker und die Build-Werkzeuge, die deine Rust-Toolchain für deine Plattform benötigt. Beachte bei der Wahl einer Bereitstellungsumgebung die [geprüften Umgebungen](/docs/compatibility/). Rust 1.92 ist die Mindestversion. Eine aufgeführte Plattform ist keine Garantie für jedes Zielsystem oder jede Konfiguration einer externen Toolchain.

Füge für JSON-Eingaben die Parser-Abhängigkeiten in deinem Anwendungscode ausdrücklich hinzu:

[[CODE1]]

Die [JSON-zu-APKG-Anleitung](/docs/json-to-anki/#rust) enthält den vollständigen Parser sowie den Medien- und Aktualisierungsablauf. Das SDK stellt keinen allgemeinen JSON- oder CSV-Konverter bereit.

<!-- section: choose-the-next-guide -->
## Die nächste Anleitung wählen

- [Rust-Anleitung](/docs/rust-guide/) für den Aufbau von Projekten und Modellen
- [Rust-API](/docs/rust-api/) für Methoden, Berichte und Fehler
- [Medien](/docs/media/) für Bilder und Audio mit eigenem Datenbesitz
- [Aktualisieren und verteilen](/docs/updates/) für Basisversionen und Veröffentlichungsbedingungen
- [Aus dem Quellcode bauen](/docs/development/) für die Entwicklung im Repository
