# Anki-Karten mit einem KI-Assistenten erzeugen

Lass einen KI-Assistenten Frage-Antwort-Paare aus deinen Aufzeichnungen entwerfen, prüfe sie anhand der Quelle und exportiere anschließend mit Anki Forge einen Anki-Stapel im Format `.apkg`. Anki Forge ist dabei die Open-Source-Bibliothek zur Kartenerzeugung: Der Assistent entwirft Inhalte oder schreibt ein Skript, und dein lokales Python-, Node.js- oder Rust-Programm erzeugt die Karten.

Du kannst zum Einstieg mit einer Programmiersprache [eine Aufgabenbeschreibung für einen Programmierassistenten kopieren](/#ai-quickstart) oder dem CSV-Ablauf unten folgen. Der Generator auf der Website setzt den Text lokal zusammen; er ruft kein KI-Modell auf und lädt deine Aufzeichnungen nicht hoch.

<!-- section: choose-input -->
## Mit überprüfbarem Ausgangsmaterial beginnen

Verwende einen kurzen Abschnitt mit einem klaren Thema, zum Beispiel eigene Kursaufzeichnungen oder eine Vokabelliste. Extrahiere und prüfe bei PDFs, Scans oder Vorlesungsfolien zuerst den Text mit einem separaten Werkzeug. Anki Forge verarbeitet Inhalte, die du über seine APIs bereitstellst; PDF-Extraktion und OCR gehören nicht zu dieser Bibliothek.

Beschränke jede vorgeschlagene Frage auf einen Sachverhalt. Gib ausreichend Kontext an, damit die Antwort auch dann verständlich ist, wenn die Karte außerhalb des ursprünglichen Dokuments erscheint.

<!-- section: draft-cards -->
## Karten im CSV-Format anfordern

Füge dieses Beispiel in deinen Assistenten ein. Ersetze für deinen eigenen Stapel den Quellabschnitt und passe die gewünschte Notizanzahl und die IDs für deine neuen Notizen an:

[[CODE0]]

Der erwartete Inhalt entspricht Folgendem:

[[CODE1]]

Der Assistent kann die Vorderseite anders formulieren. Prüfe jede Zeile anhand der Quelle, entferne mehrdeutige oder unbelegte Antworten und speichere das geprüfte Ergebnis als `cards.csv`. Weise jeder neuen Notiz eine eindeutige ID zu und behalte sie bei späteren Änderungen bei. Lass den Assistenten für einen bestehenden Stapel keine neuen IDs erzeugen.

<!-- section: export-apkg -->
## Geprüfte Karten in ein APKG umwandeln

Unter [CSV zu Anki mit Python](/docs/csv-to-anki/) findest du das vollständige Programm, die Installationsbefehle und die Beispieleingabe. Das Skript prüft die CSV-Struktur, verlangt eindeutige IDs und schreibt ein dauerhaft gespeichertes Paket. Das Beispiel erzeugt drei Basic-Karten ohne Medien.

Wenn deine Daten bereits als JSON vorliegen oder du Bilder und Audio benötigst, verwende die [JSON-zu-Anki-Anleitung](/docs/json-to-anki/) mit vollständigen Programmen für Python, Node / TypeScript und Rust. Deine Anwendung liest CSV und JSON ein, bevor sie die Notizen an Anki Forge übergibt.

Wenn du Hilfe beim Schreiben eines eigenen Programms benötigst, verwende die [Aufgabenbeschreibung für Programmierassistenten](/#ai-quickstart) und beschreibe Eingabefelder, gewünschten Kartentyp und Ausgabedateinamen. Der [optionale Anki Forge Skill](https://github.com/morehardy/anki-forge/blob/main/docs/ai-assistants.md) stellt Assistenten, die Skills laden können, Referenzen zur öffentlichen API und aufgabenbezogene Abläufe bereit.

<!-- section: check-result -->
## Die Karten vor dem Lernen prüfen

Öffne das erzeugte APKG in Anki und prüfe Kartenanzahl, Frage, Antwort und Formatierung. Beim CSV-Beispiel werden drei Notizen und drei Karten im Stapel Spanish erwartet. Ein erfolgreicher Export bestätigt, dass ein Paket erstellt wurde; er belegt nicht, dass die Antworten einer KI richtig oder zum Lernen geeignet sind.

Verwende für Lückentextfragen [Cloze-Karten](/docs/cards/). Verwende für Diagrammbeschriftungen die [Bildverdeckung](/docs/image-occlusion/) mit dem Bild und den Maskenkoordinaten, die du selbst vorgibst. Beschriftungen im Bild werden nicht automatisch erkannt.

Bewahre das ursprünglich verteilte APKG für die nächste Veröffentlichung auf und folge dem [Aktualisierungsablauf](/docs/updates/). Behalte den Namensraum und die Notiz-IDs bei, prüfe den Vergleich und berücksichtige Ankis Importeinstellungen sowie neuere lokale Änderungen.
