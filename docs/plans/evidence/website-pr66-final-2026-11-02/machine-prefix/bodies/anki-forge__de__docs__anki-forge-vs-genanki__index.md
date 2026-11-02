# Anki Forge oder genanki: passend zu deinem Ablauf wählen

Canonical page: https://morehardy.github.io/anki-forge/de/docs/anki-forge-vs-genanki/
Language: de
Description: Vergleiche die Erstellung von Anki-APKGs, Laufzeitanforderungen, Aktualisierungsprüfungen und Migrationsaufwand.

Ziehe Anki Forge in Betracht, wenn du einen wiederholbaren Veröffentlichungsablauf entwickelst und Medien mit eigenem Datenbesitz, strukturierte Diagnosen, Vergleiche sowie eine Richtlinienprüfung vor der Veröffentlichung von Aktualisierungen benötigst. Behalte einen bewährten genanki-Ablauf bei, wenn dessen Python-Skripte, Vorlagen und Medien deine Anforderungen bereits erfüllen und die Migration mehr Aufwand als Nutzen bringt.

| Entscheidung | Anki Forge | genanki |
| --- | --- | --- |
| Programmiersprache | Rust-Kern; Schnittstellen für Rust, Python und Node / TypeScript | Python |
| Gemeinsame Grundlagen | Basic, Cloze, eigene Vorlagen, Medien und Offline-Erzeugung von APKGs | Basic, Cloze, eigene Vorlagen, Medien und Offline-Erzeugung von APKGs |
| Identität | Ausdrücklicher Namensraum und Notizschlüssel; ursprüngliches APKG für Aktualisierungen aufbewahren | Konfigurierbare GUIDs und numerische Modell-/Stapel-IDs; Aktualisierungen beim erneuten Import |
| Vor der Veröffentlichung | Vergleichsbefunde, Risikorichtlinie und geprüfter vorbereiteter Kandidat | Dein Skript und deine Tests legen die Veröffentlichungsprüfung fest |
| Besitz von Mediendaten | Typisierte Bild- und Audioobjekte verwalten unveränderliche Snapshots der Mediendaten | Das Skript liefert Mediendateien und Referenzen für das Paket |
| Bildverdeckung per Code | Rechteckmasken mit Schlüsseln über die öffentliche API | Prüfe ein für deinen Ablauf geeignetes Modell oder eine Erweiterung |
| Laufzeitaufwand | Native Abhängigkeiten; unterstützte Plattform und Interpreter verwenden | Eine bestehende Python-Bereitstellung beizubehalten kann günstiger sein |
| Migrationsaufwand | Alte Identitätszuordnung, Vorlagen und Importe ausdrücklich validieren | Kein Migrationsaufwand bei einer bereits funktionierenden genanki-Pipeline |

## Dieselbe kleine Aufgabe

Um aus drei JSON-Fragen einen Stapel mit einem Bild und einem Ton zu erstellen, lässt du bei beiden Bibliotheken Python das JSON einlesen und Notizen erzeugen. Mit genanki wählst du Modell- und Stapel-IDs, weist Feldwerte und GUID-Regeln zu, fügst dem Paket Mediendateien hinzu und schreibst das APKG. Mit Anki Forge wählst du einen Namensraum und dauerhafte fachliche Schlüssel, kombinierst Medien mit eigenem Datenbesitz zu Notizen, erzeugst v1, vergleichst v2 mit dem ursprünglichen v1-APKG und bereitest v2 auf dieser Grundlage für die Veröffentlichung vor. Erst danach veröffentlichst du v2. Die [vollständige JSON-Anleitung](https://morehardy.github.io/anki-forge/de/docs/json-to-anki/) enthält Implementierungen für Rust, Python und TypeScript samt Ausführungsbefehlen.

## Vorhandene genanki-Stapel

[genanki unterstützt eigene GUIDs](https://github.com/kerrickstaley/genanki#note-guids), Medien und Aktualisierungen beim erneuten Import. Stabile Identität ist keine exklusive Fähigkeit von Anki Forge. Die Standard-GUID-Regel kann von Feldern abhängen; wähle eine bewusste GUID-Regel, wenn sich Felder ändern werden.

Anki Forge stellt beliebige numerische genanki-IDs oder GUIDs nicht automatisch wieder her. Behandle deine erste Anki-Forge-Verteilung ohne separat validierte Zuordnung als Identitätswechsel. Weder dieser Vergleich noch ein erfolgreicher APKG-Build verspricht eine verlustfreie Migration oder den Erhalt des gesamten Wiederholungsverlaufs. Lies die [Migrationsanleitung](https://morehardy.github.io/anki-forge/de/docs/genanki-migration/), bevor du einen bestehenden Stapel umstellst.

## Historische Leistungsmessung

Der Vergleich vom 2026-10-08 maß die Rust-Project-API gegen genanki 0.13.1 auf einem Apple M1 Pro. Bei 1.000 Basic-Notizen benötigten die fünf gemessenen Rust-Lastfälle 45,7–53,9 % weniger Exportzeit; einige Medienlastfälle benötigten mehr Spitzenspeicher. Siehe [Methode, Rohdaten und Einschränkungen](https://github.com/morehardy/anki-forge/blob/bef4aeb653fc875f216614e73d8a617be039b9cf/benchmarks/results/20261008-latest-commit-genanki/report.md). Die Messung erfasste nicht die aktuelle Leistung der Python- oder Node-Hosts und garantiert diesen SDKs oder anderen Lastfällen keine entsprechende Geschwindigkeit.

Anki-Forge-Maintainer · am 2026-10-25 anhand der öffentlichen APIs von Anki Forge 0.3.1, der [genanki-Dokumentation](https://github.com/kerrickstaley/genanki) und der archivierten Messung geprüft. Prüfe die [Laufzeitkompatibilität](https://morehardy.github.io/anki-forge/de/docs/compatibility/) und [wähle eine Programmiersprache](https://morehardy.github.io/anki-forge/de/docs/languages/), um eine neue Aufgabe zu beginnen.
