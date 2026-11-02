# Grundbegriffe

<!-- section: project-and-note -->
## Project und Notiz

Ein `Project` steht für eine Veröffentlichung mit einem stabilen Namensraum. Es kann Notizen mehreren Anki-Stapeln zuordnen: `default_deck` legt den Standard fest, `Note::deck` überschreibt ihn. Ein Stapel ist ein Ziel für Karten und kein zusätzlicher Container für das Erstellen von Inhalten, Medien oder Builds.

Eine `Note` enthält ihr Modell und ihre Inhalte. `project.add(key, note)` weist ihr einen erforderlichen stabilen Quellschlüssel zu und sammelt Abhängigkeiten atomar. Doppelte Schlüssel, unbekannte Felder sowie widersprüchliche Modelle oder Medien führen zu einem Fehler, ohne den Zustand teilweise zu verändern. Der Projektnamensraum und die Datensatzschlüssel müssen Änderungen und einer Umordnung der Quelldaten standhalten. Anzeigenamen und Texte sind keine Identitätsschlüssel.

<!-- section: models-fields-and-templates -->
## Modelle, Felder und Vorlagen

`NoteType::builder(key)` sammelt Deklarationen; `.build()` validiert sie und liefert ein gemeinsam nutzbares, unveränderliches Modell. `model.note()` erzeugt eine Notiz, die das Modell hält. Die eingebauten Notizen für Basic, Cloze und Bildverdeckung bringen ihre eigenen Modelle mit.

Felder und Vorlagen haben stabile Schlüssel und optionale Anzeigenamen. Verwende in Vorlagen Feldschlüssel, zum Beispiel `{{front}}`, auch wenn der Anzeigename in Anki `正面` lautet. Der Abschluss validiert und kompiliert die Referenzen, einschließlich Abschnitten, Filtern, Cloze und Vorlagen für den Kartenbrowser. Auch Zuweisungen verwenden Schlüssel; es gibt weder erratene Schlüssel noch einen Rückgriff auf Anzeigenamen.

<!-- section: text-and-html -->
## Text und HTML

Gewöhnliche Zeichenfolgen bedeuten immer Text – bei Basic, Cloze und eigenen Notizen. Text maskiert HTML; `Content::html` bindet Markup ausdrücklich ein. Cloze-Syntax wie `{{c1::answer}}` bleibt in Text erhalten, ohne eingebettetes HTML als vertrauenswürdig einzustufen. `Content::sequence` kombiniert Text, HTML, Bilder und Audio in einem Feld.

Typisierte Bild- und Audioinhalte behalten ihre Medienabhängigkeiten bis zum Export. Wandle Medien vor dem Hinzufügen nicht in Zeichenfolgen um, es sei denn, du deklarierst die Datei zusätzlich ausdrücklich für dein selbst geschriebenes HTML, CSS oder Skript.

<!-- section: media-ownership -->
## Besitz von Mediendaten

`Media` ist ein Snapshot mit eigenem Datenbesitz und einem deterministisch aus dem Inhalt abgeleiteten Dateinamen. Ein Import ist erst erfolgreich, wenn die Eingabe innerhalb ihres Budgets gelesen und validiert wurde. Anschließend darf sich die Quelldatei ändern oder verschwinden. Klone teilen sich den Speicher und können in mehreren Projekten verwendet werden. `with_export_name` wählt einen festen, portablen Dateinamen, ohne bestehende Klone oder Inhaltsknoten zu verändern.

Typisierte Referenzen sammeln Medien automatisch. Ausdrücklich einem Modell oder Projekt zugewiesene Mediendateien werden auch dann aufgenommen, wenn die statische Analyse ihre Verwendung nicht erkennen kann. Die Bibliothek ermittelt keine Dateiinhalte aus beliebigem HTML oder Skripten.

<!-- section: build-and-update -->
## Erzeugen und aktualisieren

Ein erfolgreicher Build liefert eine Ausgabe, die garantiert ein APKG enthält. Ein Bericht enthält Beobachtungen; ein Snapshot besteht aus JSON-Daten und besitzt keine Dateien. Temporäre Artefakte bleiben nur bestehen, solange mindestens ein Artefaktbesitzer vorhanden ist.

Eine Aktualisierung verwendet die vollständigen Nachweise, die im ursprünglich verteilten APKG der Vorgängerversion eingebettet sind. Der unabhängige Vergleich trennt Befunde von der Veröffentlichungsrichtlinie. Befunde mit hohem Risiko blockieren Aktualisierungen standardmäßig; harte Fehler lassen sich nicht freigeben. Anki-Importeinstellungen, lokale Änderungen und die Lernplanung der Karten müssen weiterhin im Client geprüft werden. Siehe [Aktualisierungen](/docs/updates/) und [Build-Garantien](/docs/build-guarantees/).
