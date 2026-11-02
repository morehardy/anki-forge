# 德语文案审阅

多数句子语法成立，du 的语气一致，Notiz、Karte、Stapel 区分基本清楚。主要问题是名词堆叠和直译式口号；batch 译成 Stapel 时又与牌组术语冲突，应改成明确的批量媒体操作。

保留 Notiz／Karte／Stapel、Bildverdeckung 和常见开发词 Build、API、Snapshot。不要为了追求“全德语”制造更长的新名词。

P2 表示应优先处理的事实范围或理解歧义；P3 表示可选的自然度、清晰度和术语优化。编号按位置统计，同一原因可能影响多处文字。本页建议已应用到产品源文件，并补齐相关术语的一致性。表中原文和行号保留审阅时快照，便于追溯；不作为正式发布的独立审阅凭据。

## 界面文案

| 编号与位置 | 原文 | 建议替换 | 问题 |
| --- | --- | --- | --- |
| de-01 P3<br>[home.evolve2](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/de.json:19) | Erhalte ihre Identität. | Behalte ihre IDs bei. | Erhalte ihre Identität 文法可通但抽象；首页直接说保留 ID。 |
| de-02 P3<br>[home.evolveDescription](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/de.json:20) | Korrigiere eine Definition oder ergänze ein Beispiel. Stabile IDs und ein vorheriger Build helfen Anki Forge, vorhandene Notizen beim erneuten Export wiederzuerkennen. | Korrigiere eine Definition oder ergänze ein Beispiel. Behalte die Notiz-IDs bei und verwende das ursprünglich verteilte APKG der Vorgängerversion, damit Anki Forge vorhandene Notizen beim erneuten Export zuordnen kann. | vorheriger Build 不明确指哪个文件；点明原始 APKG。 |
| de-03 P3<br>[update.comparison](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/de.json:82) | Tatsächlicher Vergleich | Vergleichsergebnis | Tatsächlicher Vergleich 生硬，没有对照对象。 |
| de-04 P3<br>[update.none](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/de.json:83) | Keine Befunde | Keine Änderungen erkannt | Befunde 在此有检查报告或医疗语感；简单状态用“未发现变化”。 |
| de-05 P3<br>[error.heading](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/de.json:114) | Diese Seite fehlt. | Seite nicht gefunden. | Diese Seite fehlt 像网站漏做了页面；常见 404 表达更准确自然。 |
| de-06 P3<br>[examples.heading2](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/de.json:121) | Etwas zum Merken. | Karten zum Lernen. | Etwas zum Merken 机械对应 Something to remember；直接点出代码生成学习卡片。 |
| de-07 P3<br>[examples.updateDescription](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/de.json:129) | Der zweite Build ändert die Antwort von „hello“ zu „hello; hi“. Er vergleicht mit dem ersten Paket und prüft, dass die Notizidentität erhalten bleibt. | Der zweite Build ändert die Antwort von „hello“ zu „hello; hi“. Das Programm vergleicht das Ergebnis mit dem ersten Paket und prüft, ob die Notiz-ID gleich geblieben ist. | Er vergleicht mit 缺少比较宾语，且 Build 被拟人化；写出程序和结果。 |
| de-08 P3<br>[examples.limitations](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/de.json:131) | Ankis Importeinstellungen und lokale Änderungen beeinflussen, wie Aktualisierungen angewendet werden. Die Vorschauen zeigen Inhalt und Identität; Kartengestaltung und Wiederholungsverhalten stammen von Anki. | Ankis Importeinstellungen und lokale Änderungen beeinflussen, wie Aktualisierungen angewendet werden. Die Vorschau zeigt Inhalte und IDs; wie die Karten aussehen und beim Lernen angezeigt werden, bestimmt Anki. | Wiederholungsverhalten stammen von Anki 生硬；明确由 Anki 决定显示与学习行为。 |
| de-09 P3<br>[performance.title](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/de.json:165) | Der Export, gemessen. | Gemessene Exportzeiten | Der Export, gemessen 照搬英语悬置式标题。 |
| de-10 P3<br>[performance.workload](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/de.json:166) | 1.000 Basic-Notizen pro Lastfall | 1.000 Basic-Notizen pro Testszenario | Lastfall 在此过于工程化；Testszenario 更适合软件基准测试。 |
| de-11 P3<br>[performance.range](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/de.json:168) | Weniger Exportzeit in fünf gemessenen Lastfällen | Zeitersparnis in fünf Testszenarien | 与 performance.workload 统一术语，并与显示的百分比对应。 |
| de-12 P3<br>[performance.description](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/de.json:172) | Jeder Lastfall enthält {notes} Basic-Notizen und deckt reinen Text, einzelne Medien oder gemeinsam verwendete gemischte Medien ab. Rust benötigte {minimum}–{maximum} % weniger Exportzeit als genanki. | Jeder Test verwendet {notes} Basic-Notizen. Die fünf Tests decken reine Textnotizen, separate Mediendateien und gemeinsam genutzte Medien ab. Die Rust-API von Anki Forge benötigte dabei {minimum}–{maximum} % weniger Exportzeit als genanki. | 原句让“每个测试”同时覆盖多种场景；重写为五组测试整体覆盖，并明确 API 主语。 |
| de-13 P3<br>[performance.limitations](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/de.json:189) | Diese Ergebnisse messen die Stapelerzeugung, nicht die Geschwindigkeit von Anki-Import oder Wiederholungen. Andere Hardware und Lastfälle können abweichen. | Gemessen wurde die Erzeugung der Stapel, nicht der Import oder das Lernen in Anki. Auf anderer Hardware oder mit anderen Daten können die Ergebnisse abweichen. | 原句 Andere Hardware und Lastfälle können abweichen 主语是硬件而非结果；简化表达。 |
| de-14 P3<br>[machine.tasks](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/de.json:199) | Aufgabenanleitungen | Anleitungen nach Aufgabe | Aufgabenanleitungen 是生硬名词组合；用介词结构自然。 |

## 正文与元数据

### de-15 P2

[docs/de/media.md](/Users/hp/Desktop/2026/anki-forge/docs/de/media.md:3)

Stapel 在 Anki 已指牌组；这里 batch import 导入的是媒体文件，会混淆功能。

原文片段：

> der unten beschriebene Stapelimport zeigt auch die Python- und Node.js-Aufrufe.

建议替换：

> der unten beschriebene Import mehrerer Mediendateien zeigt auch die Python- und Node.js-Aufrufe.

### de-16 P2

[docs/de/media.md](/Users/hp/Desktop/2026/anki-forge/docs/de/media.md:107)

同一术语冲突；明确对象是多个媒体文件。

原文片段：

> ### Verhalten und Budgets des Stapelimports

建议替换：

> ### Verhalten und Limits beim Import mehrerer Mediendateien

### de-17 P3

[docs/de/node-api.md](/Users/hp/Desktop/2026/anki-forge/docs/de/node-api.md:110)

im Stapel 也会被理解为“往牌组里导入”；用动作说明批量处理。

原文片段：

> ## Medienimport im Stapel und vorbereitete Veröffentlichung

建议替换：

> ## Mehrere Mediendateien importieren und die Veröffentlichung vorbereiten

### de-18 P3

[docs/de/anki-forge-vs-genanki.md](/Users/hp/Desktop/2026/anki-forge/docs/de/anki-forge-vs-genanki.md:11)

tragen Snapshots mit eigenem Datenbesitz 名词堆叠，明确对象维护数据快照即可。

原文片段：

> Typisierte Bilder und Audio tragen Snapshots mit eigenem Datenbesitz

建议替换：

> Typisierte Bild- und Audioobjekte verwalten unveränderliche Snapshots der Mediendaten

### de-19 P3

[docs/de/concepts.md](/Users/hp/Desktop/2026/anki-forge/docs/de/concepts.md:8)

standhalten 像承受负荷；这里是要求开发者不改变键。

原文片段：

> Der Projektnamensraum und die Datensatzschlüssel müssen Änderungen und einer Umordnung der Quelldaten standhalten.

建议替换：

> Behalte den Projektnamensraum und die Datensatzschlüssel bei, wenn du Quelldaten bearbeitest oder neu sortierst.

### de-20 P3

[docs/de/build-guarantees.md](/Users/hp/Desktop/2026/anki-forge/docs/de/build-guarantees.md:3)

Art 泛指种类，且修饰结构不自然；补齐错误对象。

原文片段：

> mit strukturierter Art und Code

建议替换：

> mit strukturierten Angaben zu Fehlerart und Fehlercode

### de-21 P3

[docs/de/build-guarantees.md](/Users/hp/Desktop/2026/anki-forge/docs/de/build-guarantees.md:39)

concurrent 不必等于 parallel；本节包含快照和生命周期，Nebenläufigkeit 更贴合语义。

原文片段：

> ## Parallele Arbeit

建议替换：

> ## Nebenläufige Vorgänge

### de-22 P3

[docs/de/csv-to-anki.md](/Users/hp/Desktop/2026/anki-forge/docs/de/csv-to-anki.md:3)

teilbar 也表示可分割；“可分享”用 weitergeben 更清楚。

原文片段：

> exportiert einen teilbaren Stapel.

建议替换：

> exportiert einen Stapel, den du weitergeben kannst.

### de-23 P3

[website/site-model.mjs](/Users/hp/Desktop/2026/anki-forge/website/site-model.mjs:90) · `page:installation.title.de`

标题明确与正文相同的 Rust 范围。

原文片段：

> Anki Forge in deine Anwendung integrieren

建议替换：

> Anki Forge in deine Rust-Anwendung integrieren
