# Fehlersuche

Nutze die strukturierten Angaben `kind` und `code` des fehlgeschlagenen Vorgangs sowie dessen Ursachenverkettung und Beobachtungen. Anzeigetext ist für Menschen gedacht und kann sich ändern. Ein fehlgeschlagener Vorgang und ein abgeschlossener Vergleich, dessen Richtlinie die Veröffentlichung blockiert, sind unterschiedliche Ergebnisse.

<!-- section: installation -->
## Installation

| Symptom | Prüfung |
| --- | --- |
| Rust weist die Toolchain zurück | Führe `rustc --version` aus; verwende Rust ab Version 1.92 und prüfe die installierte Version mit `cargo tree -i ankiforge` |
| Node kann sein natives Paket nicht laden | Führe `node -p "process.platform + '/' + process.arch"` und `npm ls ankiforge` aus; installiere mit `npm install --include=optional ankiforge@0.3.1` neu |
| Node meldet unterschiedliche Versionen von nativer Bibliothek und Wrapper | Verwende passende Paketversionen; installiere beide aus derselben Registry-Version neu oder baue beide aus demselben Repository-Stand |
| Python kann seine Erweiterung nicht laden | Führe `python -m pip show ankiforge` und `python -m pip debug --verbose` aus; prüfe Interpreter sowie Betriebssystem-/CPU-Tags des Wheels |
| Python wählt eine Quelldistribution | Wähle ein passendes Wheel bzw. einen passenden Interpreter oder folge [Aus dem Quellcode bauen](/docs/development/) mit einem Compiler |

Siehe [geprüfte Umgebungen](/docs/compatibility/), [Node-Einrichtung](/docs/node-quickstart/) und [Python-Einrichtung](/docs/python-quickstart/).

<!-- section: models-and-notes -->
## Modelle und Notizen

| Symptom | Korrektur |
| --- | --- |
| Ungültiger oder leerer Schlüssel | Gib einen ausdrücklichen stabilen Schlüssel an; Anzeigenamen sind davon getrennt |
| `NOTE.KEY_DUPLICATE` | Gib jedem logischen Quelldatensatz einen eindeutigen Schlüssel |
| `NOTE.FIELD_UNKNOWN` | Weise dem Feld über seinen Schlüssel einen Wert zu, nicht über seinen Anzeigenamen |
| `NOTE.FIELD_REQUIRED` | Fülle Felder, die mit `required()` deklariert sind, vor dem Hinzufügen |
| `NOTE.MODEL_CONFLICT` | Verwende für denselben Modellschlüssel dasselbe abgeschlossene Modell |
| Fehler bei Vorlagenreferenzen | Verwende deklarierte Schlüssel in Vorlagen; prüfe den ursprünglichen Bytebereich der Quelle |
| HTML wird wörtlich angezeigt | Verwende `Content::html` nur für bewusst eingebundenes Markup |
| Cloze-Karten fehlen | Gib Syntax wie `{{c1::answer}}` mit unterstützten Nummern von 1–500 an |

Ein Fehler beim Hinzufügen ist atomar. Korrigiere die Notiz oder das Modell und versuche es erneut; du musst keine teilweise erfolgte Modell- oder Medienregistrierung rückgängig machen. Modell-Builder müssen mit `.build()` abgeschlossen werden, bevor sie eigene Notizen erzeugen können.

<!-- section: media-and-templates -->
## Medien und Vorlagen

| Symptom | Korrektur |
| --- | --- |
| Dateiimport schlägt fehl | Prüfe die Quelle zum Zeitpunkt von `Media::file`; spätere Löschung der Quelldatei wird unterstützt |
| MIME-Typ und Inhalt passen nicht zusammen | Übergib dem Bytes-Konstruktor den tatsächlichen MIME-Typ |
| Konflikt beim Exportnamen | Wähle eindeutige portable Namen; Varianten in Groß-/Kleinschreibung und Unicode-Normalisierung verursachen Konflikte |
| Ressource aus direktem HTML, CSS oder Skript fehlt | Deklariere sie mit `asset` am Modell oder `add_asset` am Projekt |
| Medienimport überschreitet ein Limit | Erhöhe `MediaLimits.max_bytes` vor dem Import ausdrücklich |
| Build-Inspektion überschreitet ein Limit | Passe den betroffenen Zähler in `InspectLimits` für diesen Vorgang an |
| Vorlagenpaket kann nicht geladen werden | Verwende bundle-v2, ausdrückliche Schlüssel, schlüsselbasierte Vorlagen und Pfade innerhalb des Pakets |
| Rechteck für Bildverdeckung ist ungültig | Prüfe die dekodierten Anzeigeabmessungen, endliche Koordinaten und eindeutige Maskenschlüssel |

Die Textvorgaben sind für alle Notizarten gleich. Medienknoten sammeln Abhängigkeiten; einen Dateinamen von Hand einzutragen importiert dessen Bytes nicht. Auch ein gültiges Paket benötigt Wiedergabetests im Client für die vorgesehenen Codecs und Schriftarten.

<!-- section: updates-and-outputs -->
## Aktualisierungen und Ausgaben

Verwende als Basis das ursprünglich verteilte APKG mit vollständigen Identitätsnachweisen. Aus Anki erneut exportierte Pakete erhalten diese Nachweise nicht. Ein abweichender Namensraum oder beschädigte Nachweise lassen sich nicht durch eine Aktualisierungsrichtlinie freigeben.

Ein abgeschlossener Vergleich kann `allows_publication == false` ergeben. Prüfe die ursprünglichen Befunde und Nachweise, bevor du eine Risikokategorie ausdrücklich freigibst. Gib nicht pauschal alle Ergebnisse frei. Für dieselbe Entscheidung musst du dieselbe Richtlinie auch an den tatsächlichen Aktualisierungs-Build übergeben. Eine Richtlinie bei einer reinen Erstellungsanfrage ist ein Fehler.

Wenn ein temporärer Pfad verschwunden ist, halte seinen Ausgabe- oder Artefaktbesitzer, bis alle Leser fertig sind. Berichte und JSON-Snapshots sind keine Besitzer. Prüfe bei Fehlern beim dauerhaften Speichern die Veröffentlichungsphase und Dauerhaftigkeit im Fehler: Ein Fehler kann zurückgegeben werden, nachdem das Ziel bereits veröffentlicht wurde. Siehe [Ausgabegarantien](/docs/build-guarantees/).
