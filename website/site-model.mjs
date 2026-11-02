import { catalogs } from './src/i18n/messages.mjs';

import { configuredLocales, locales, localeIds, localePath, resolveLocalePath } from './locale-model.mjs';
export { configuredLocales, locales, localeIds, releaseLocales, stripBase } from './locale-model.mjs';

// Public topic identities are independent of translated titles and URL prefixes.
// Translated text is keyed by locale ID; adding a release locale adds keys, not fields.
const documents = [
  {
    "name": "docs",
    "source": "website/src/content/docs/docs/index.mdx",
    "route": "/docs/",
    "title": {"en": "Build Anki decks from code", "zh-CN": "用代码生成 Anki 牌组", "es": "Crea mazos de Anki con código", "pt-BR": "Crie baralhos do Anki com código", "ja": "コードから Anki デッキを生成する", "de": "Anki-Stapel aus Code erzeugen"},
    "description": {"en": "Choose a language, build your first deck, and learn how to add media, templates and repeatable updates.", "zh-CN": "选择开发语言，生成第一个牌组，再添加媒体、模板和可重复的更新。", "es": "Elige un lenguaje, crea tu primer mazo de Anki y aprende a añadir imágenes, audio, plantillas y actualizaciones reproducibles.", "pt-BR": "Escolha uma linguagem de programação, gere seu primeiro baralho e aprenda a adicionar mídia e templates e a atualizar o baralho.", "ja": "言語を選んで最初のデッキを作り、メディア、テンプレート、繰り返し実行できる更新を追加します。", "de": "Wähle eine Programmiersprache, erzeuge deinen ersten Stapel und ergänze Medien, Vorlagen und wiederholbare Aktualisierungen."}
  },
  {
    "name": "languages",
    "source": "website/src/content/docs/docs/languages.mdx",
    "route": "/docs/languages/",
    "title": {"en": "Choose your language", "zh-CN": "选择开发语言", "es": "Elige tu lenguaje de programación", "pt-BR": "Escolha sua linguagem de programação", "ja": "プログラミング言語を選ぶ", "de": "Deine Programmiersprache wählen"},
    "description": {"en": "Install the public Rust, Node or Python package and export your first Anki deck.", "zh-CN": "安装公开 Rust、Node 或 Python 包，导出第一个 Anki 牌组。", "es": "Instala el paquete publicado de Rust, Node.js o Python y exporta tu primer mazo de Anki.", "pt-BR": "Instale o pacote público de Rust, Node ou Python e exporte seu primeiro baralho do Anki.", "ja": "Rust、Node、Python の公開パッケージをインストールして、最初の Anki デッキをエクスポートします。", "de": "Installiere das veröffentlichte Paket für Rust, Node oder Python und exportiere deinen ersten Anki-Stapel."}
  },
  {
    "name": "quickstart",
    "source": "website/src/content/docs/docs/quickstart.mdx",
    "route": "/docs/quickstart/",
    "title": {"en": "Generate Anki flashcards with Rust", "zh-CN": "用 Rust 生成 Anki 卡片", "es": "Genera un APKG de Anki con Rust", "pt-BR": "Gere cartões do Anki com Rust", "ja": "Rust で Anki カードを生成する", "de": "Anki-Karten mit Rust erzeugen"},
    "description": {"en": "Install the public Rust package, write a Spanish flashcard, and export a persistent APKG file.", "zh-CN": "安装公开 Rust 包，编写西班牙语卡片并导出持久保存的 APKG。", "es": "Instala el paquete de Rust, crea una tarjeta de vocabulario y exporta un archivo APKG persistente con un programa completo.", "pt-BR": "Instale o pacote público de Rust, crie um cartão de espanhol e exporte um APKG persistente com um programa completo.", "ja": "Rust の公開パッケージをインストールし、スペイン語のカードを作成して、後から使える APKG ファイルとして保存します。", "de": "Installiere das veröffentlichte Rust-Paket, erstelle eine spanische Vokabelkarte und exportiere mit einem vollständigen Programm ein dauerhaft gespeichertes APKG."}
  },
  {
    "source": "docs/json-to-anki.md",
    "route": "/docs/json-to-anki/",
    "name": "json-to-anki",
    "title": {"en": "Generate and update Anki decks from JSON", "zh-CN": "用 JSON 数据生成并更新 Anki 牌组", "es": "Genera y actualiza mazos de Anki desde JSON", "pt-BR": "Gere e atualize baralhos do Anki a partir de JSON", "ja": "JSON から Anki デッキを生成・更新する", "de": "Anki-Stapel aus JSON erzeugen und aktualisieren"},
    "description": {"en": "Read JSON with Rust, Python or TypeScript, package images and audio, then review an answer-only APKG update.", "zh-CN": "用 Rust、Node 或 Python 读取 JSON，打包图片和音频，再检查仅修改答案的更新。", "es": "Lee JSON con Rust, Python o TypeScript, incluye imágenes y audio y revisa una actualización de APKG que solo cambia una respuesta.", "pt-BR": "Leia JSON com Rust, Python ou TypeScript, inclua imagens e áudio e revise uma atualização de APKG que altera apenas uma resposta.", "ja": "Rust、Python、TypeScript で JSON を読み取り、画像と音声を同梱し、解答だけを変更する APKG 更新を確認します。", "de": "Lies JSON mit Rust, Python oder TypeScript ein, bündle Bilder und Audio und prüfe eine APKG-Aktualisierung, die nur eine Antwort ändert."}
  },
  {
    "source": "docs/csv-to-anki.md",
    "route": "/docs/csv-to-anki/",
    "name": "csv-to-anki",
    "title": {"en": "CSV to Anki: generate flashcards with Python", "zh-CN": "CSV 转 Anki：用 Python 批量生成卡片", "es": "CSV a Anki: crea tarjetas con Python", "pt-BR": "CSV para Anki: gere cartões com Python", "ja": "Python で CSV を Anki カードに変換する", "de": "CSV zu Anki: Karten mit Python erzeugen"},
    "description": {"en": "Convert a UTF-8 vocabulary spreadsheet to an Anki APKG with a complete Python script, sample CSV, stable note IDs and import checks.", "zh-CN": "用完整 Python 脚本将 UTF-8 CSV 词汇表转换为 Anki APKG，包含示例数据、稳定笔记 ID 和导入检查步骤。", "es": "Convierte una hoja de vocabulario CSV UTF-8 en un APKG con un script completo de Python, ID de nota estables y comprobaciones de importación.", "pt-BR": "Converta uma planilha de vocabulário CSV UTF-8 em um APKG com um script Python completo, IDs de nota estáveis e verificações de importação.", "ja": "UTF-8 の語彙表を Anki APKG に変換する完全な Python スクリプト。CSV サンプル、安定したノート ID、インポート確認の手順を含みます。", "de": "Wandle eine UTF-8-Vokabeltabelle mit einem vollständigen Python-Skript in ein Anki-APKG um – mit CSV-Beispiel, stabilen Notiz-IDs und Importprüfungen."}
  },
  {
    "source": "docs/ai-anki-cards.md",
    "route": "/docs/ai-anki-cards/",
    "name": "ai-anki-cards",
    "title": {"en": "AI-assisted Anki flashcards: from notes to APKG", "zh-CN": "AI 辅助生成 Anki 卡片：从笔记到 APKG", "es": "Crea tarjetas de Anki con ayuda de IA", "pt-BR": "Crie cartões do Anki com apoio de IA", "ja": "AI を使った Anki カード作成：学習メモから APKG へ", "de": "Anki-Karten mit KI-Unterstützung: von Aufzeichnungen zum APKG"},
    "description": {"en": "Draft Anki cards with an AI assistant, review the answers, then export an APKG using Anki Forge. Follow a CSV workflow or use a coding-assistant brief.", "zh-CN": "让 AI 助手根据笔记起草卡片，核对答案后用 Anki Forge 导出 APKG。提供 CSV 制卡流程和编程助手任务说明入口。", "es": "Prepara tarjetas a partir de tus apuntes con un asistente de IA, revisa las respuestas y exporta un APKG con Anki Forge mediante CSV o código.", "pt-BR": "Crie rascunhos de cartões a partir das suas anotações, confira as respostas e exporte um APKG com Anki Forge usando CSV ou instruções para um assistente de programação.", "ja": "AI アシスタントでカードの下書きを作り、解答を確認して Anki Forge で APKG を生成。CSV の手順とコーディングアシスタント向け依頼文を用意しています。", "de": "Lass einen KI-Assistenten Karten entwerfen, prüfe die Antworten und exportiere mit Anki Forge ein APKG. Nutze CSV oder eine Aufgabenbeschreibung für Programmierassistenten."}
  },
  {
    "source": "docs/anki-forge-vs-genanki.md",
    "route": "/docs/anki-forge-vs-genanki/",
    "name": "anki-forge-vs-genanki",
    "title": {"en": "Anki Forge vs genanki: choose for your workflow", "zh-CN": "Anki Forge 与 genanki：如何选择", "es": "Anki Forge frente a genanki: cuál elegir", "pt-BR": "Anki Forge ou genanki: escolha para o seu fluxo", "ja": "Anki Forge と genanki：作業に合う選択", "de": "Anki Forge oder genanki: passend zu deinem Ablauf wählen"},
    "description": {"en": "Compare Anki APKG authoring workflows, runtime requirements, update review and migration costs.", "zh-CN": "比较 Anki APKG 编写流程、运行环境、更新检查和迁移成本。", "es": "Compara la creación de APKG, los entornos de ejecución, la revisión de actualizaciones y los costes de migración de Anki Forge y genanki.", "pt-BR": "Compare a criação de APKGs do Anki, os requisitos de execução, a revisão de atualizações e os custos de migração.", "ja": "Anki APKG の作成手順、実行環境の要件、更新の確認、移行コストを比較します。", "de": "Vergleiche die Erstellung von Anki-APKGs, Laufzeitanforderungen, Aktualisierungsprüfungen und Migrationsaufwand."}
  },
  {
    "source": "docs/source-builds.md",
    "route": "/docs/development/",
    "name": "development",
    "title": {"en": "Build SDKs from source", "zh-CN": "从源码构建 SDK", "es": "Compila los SDK desde el código fuente", "pt-BR": "Compile os SDKs a partir do código-fonte", "ja": "ソースから SDK をビルドする", "de": "SDKs aus dem Quellcode bauen"},
    "description": {"en": "Build SDKs and examples from a source checkout.", "zh-CN": "从仓库源码构建 SDK，运行示例并区分公开包与源码功能。", "es": "Compila los SDK y ejecuta los ejemplos desde el repositorio. Distingue estas compilaciones de los paquetes publicados.", "pt-BR": "Compile os SDKs e execute exemplos a partir do repositório, distinguindo os recursos do código-fonte dos pacotes publicados.", "ja": "チェックアウトしたソースから SDK とサンプルをビルドし、公開パッケージとソース専用の機能を区別します。", "de": "Baue SDKs und Beispiele aus dem Repository und unterscheide Quellcodefunktionen von veröffentlichten Paketen."}
  },
  {
    "source": "docs/rust-guide.md",
    "route": "/docs/rust-guide/",
    "name": "rust-guide",
    "title": {"en": "Rust authoring guide", "zh-CN": "Rust 编写指南", "es": "Guía de desarrollo con Rust", "pt-BR": "Guia de desenvolvimento com Rust", "ja": "Rust での作成ガイド", "de": "Inhalte mit Rust erstellen"},
    "description": {"en": "Projects, validation, media and updates through the supported Rust API.", "zh-CN": "通过受支持的 Rust API 创建 Project、校验内容、处理媒体与更新。", "es": "Crea proyectos, valida notas, gestiona recursos multimedia y actualiza mazos con la API pública de Rust.", "pt-BR": "Crie projetos, valide notas, gerencie mídia e atualize baralhos pela API pública de Rust.", "ja": "サポートされる Rust API でプロジェクトを作成し、検証、メディア処理、更新を行います。", "de": "Erstelle Projekte, validiere Notizen und verwalte Medien und Aktualisierungen über die unterstützte Rust-API."}
  },
  {
    "source": "docs/template-bundles.md",
    "route": "/docs/templates/",
    "name": "templates",
    "title": {"en": "Reusable Anki card templates, CSS and media", "zh-CN": "复用 Anki 卡片模板、CSS 与媒体", "es": "Reutiliza plantillas de Anki, CSS y recursos multimedia", "pt-BR": "Reutilize templates de cartão, CSS e mídia no Anki", "ja": "Anki のカードテンプレート、CSS、メディアを再使用する", "de": "Anki-Kartenvorlagen, CSS und Medien wiederverwenden"},
    "description": {"en": "Load a reusable Anki note type from one bundle of card templates, CSS and media, for normal or Cloze cards, with Rust, Python or Node.js.", "zh-CN": "从一个模板包加载可复用的 Anki 笔记类型，包含卡片模板、CSS 和媒体，支持普通与 Cloze 卡片，可使用 Rust、Python 或 Node.js。", "es": "Carga un tipo de nota de Anki reutilizable desde un paquete de plantillas, CSS y recursos multimedia, para tarjetas normales o Cloze, con Rust, Python o Node.js.", "pt-BR": "Carregue um tipo de nota reutilizável de um pacote com templates, CSS e mídia para cartões normais ou Cloze, usando Rust, Python ou Node.js.", "ja": "Rust、Python、Node.js でテンプレート、CSS、メディアのバンドルから、通常・Cloze カード用の再使用可能なノートタイプを読み込みます。", "de": "Lade einen wiederverwendbaren Anki-Notiztyp aus einem Paket mit Kartenvorlagen, CSS und Medien – für normale oder Cloze-Karten mit Rust, Python oder Node.js."}
  },
  {
    "source": "docs/api-overview.md",
    "route": "/docs/api/",
    "name": "api",
    "title": {"en": "Public API overview", "zh-CN": "公开 API 总览", "es": "Resumen de la API pública", "pt-BR": "Visão geral das APIs públicas", "ja": "公開 API の概要", "de": "Überblick über die öffentliche API"},
    "description": {"en": "Explore Anki Forge APIs for Rust, Python and Node.js: create notes, package images and audio, diagnose failures, and compare deck updates before distribution.", "zh-CN": "查找 Rust、Node 和 Python 的内容编写、媒体、诊断与更新 API。", "es": "Encuentra las API de Rust, Python y Node para crear notas, añadir recursos multimedia, diagnosticar errores y revisar actualizaciones.", "pt-BR": "Encontre as APIs de Rust, Python e Node para criar notas, incluir mídia, diagnosticar erros e revisar atualizações.", "ja": "作成、メディア、診断、更新に使う Rust、Python、Node の API を探せます。", "de": "Finde die Rust-, Python- und Node-APIs für Inhaltserstellung, Medien, Diagnostik und Aktualisierungen."}
  },
  {
    "source": "docs/installation.md",
    "route": "/docs/installation/",
    "name": "installation",
    "title": {"en": "Add Anki Forge to a Rust application", "zh-CN": "集成到 Rust 应用", "es": "Integra Anki Forge en tu aplicación de Rust", "pt-BR": "Integre o Anki Forge à sua aplicação Rust", "ja": "Rust アプリケーションに組み込む", "de": "Anki Forge in deine Rust-Anwendung integrieren"},
    "description": {"en": "Integrate the Rust crate, configure Cargo dependencies and choose the next guide.", "zh-CN": "安装公开 Rust 包，配置 Cargo 依赖并选择下一步指南。", "es": "Instala el crate publicado, configura las dependencias de Cargo y elige cómo añadir recursos multimedia y actualizaciones.", "pt-BR": "Instale o crate público de Rust, configure as dependências do Cargo e escolha o próximo guia.", "ja": "公開 Rust crate を組み込み、Cargo の依存関係を設定して、次のガイドを選びます。", "de": "Installiere das veröffentlichte Rust-Crate, konfiguriere Cargo-Abhängigkeiten und wähle die nächste Anleitung."}
  },
  {
    "source": "docs/concepts.md",
    "route": "/docs/concepts/",
    "name": "concepts",
    "title": {"en": "Core concepts", "zh-CN": "核心概念", "es": "Conceptos fundamentales de Anki Forge", "pt-BR": "Conceitos fundamentais", "ja": "基本概念", "de": "Grundbegriffe"},
    "description": {"en": "Projects, owned notes and media, stable keys, text and update identity.", "zh-CN": "理解 Project、笔记与媒体的所有权、稳定键，以及更新时如何识别原有笔记。", "es": "Entiende proyectos, notas, mazos, claves estables, recursos multimedia y la identidad de las actualizaciones.", "pt-BR": "Entenda Project, notas e mídia com propriedade independente, chaves estáveis, texto e identidade nas atualizações.", "ja": "Project、データを所有するノートとメディア、安定キー、テキスト、更新時の識別情報を理解します。", "de": "Verstehe Projekte, Notizen und Medien mit eigenem Datenbesitz, stabile Schlüssel, Text und Identität bei Aktualisierungen."}
  },
  {
    "source": "docs/cards.md",
    "route": "/docs/cards/",
    "name": "cards",
    "title": {"en": "Generate Anki Basic and Cloze flashcards", "zh-CN": "生成 Anki 问答卡片与 Cloze 填空卡片", "es": "Crea tarjetas Basic y Cloze de Anki", "pt-BR": "Gere cartões Basic e Cloze do Anki", "ja": "Anki の Basic と Cloze カードを生成する", "de": "Basic- und Cloze-Karten für Anki erzeugen"},
    "description": {"en": "Generate Anki Basic question-and-answer and Cloze fill-in-the-blank cards from code. A verified Rust example builds two notes and three cards.", "zh-CN": "用代码批量生成 Anki 问答（Basic）卡片与 Cloze 填空卡片。经过验证的 Rust 示例用两条笔记生成三张卡片。", "es": "Genera desde código tarjetas Basic de Anki de pregunta y respuesta y tarjetas Cloze con huecos. Un ejemplo verificado de Rust crea dos notas y tres tarjetas.", "pt-BR": "Gere cartões Basic de pergunta e resposta e cartões Cloze com lacunas usando código. Um exemplo verificado em Rust cria duas notas e três cartões.", "ja": "コードで質問・解答の Basic カードと穴埋めの Cloze カードを生成。検証済みの Rust サンプルは2件のノートから3枚のカードを作ります。", "de": "Erzeuge Basic-Karten mit Frage und Antwort sowie Cloze-Lückentextkarten aus Code. Ein geprüftes Rust-Beispiel erstellt zwei Notizen und drei Karten."}
  },
  {
    "source": "docs/custom-notetypes.md",
    "route": "/docs/custom-notetypes/",
    "name": "custom-notetypes",
    "title": {"en": "Custom note types", "zh-CN": "自定义笔记类型", "es": "Crea tipos de nota personalizados para Anki", "pt-BR": "Crie tipos de nota personalizados", "ja": "独自のノートタイプ", "de": "Eigene Notiztypen"},
    "description": {"en": "Define custom Anki note types with Rust: declare fields, card templates and generation rules, validate stable keys, and reuse immutable models across projects.", "zh-CN": "为自己的内容定义字段、卡片模板与生成规则。", "es": "Define campos, plantillas y reglas de generación con claves estables y modelos inmutables reutilizables.", "pt-BR": "Defina campos, templates de cartão e regras de geração para seu conteúdo, com chaves estáveis e modelos imutáveis reutilizáveis.", "ja": "独自コンテンツのフィールド、カードテンプレート、生成ルールを定義します。", "de": "Definiere Felder, Kartenvorlagen und Erzeugungsregeln für deine Inhalte mit stabilen Schlüsseln und wiederverwendbaren unveränderlichen Modellen."}
  },
  {
    "source": "docs/media.md",
    "route": "/docs/media/",
    "name": "media",
    "title": {"en": "Add images and audio to Anki cards", "zh-CN": "为 Anki 卡片添加图片与音频", "es": "Añade imágenes y audio a tus tarjetas Anki", "pt-BR": "Adicione imagens e áudio aos cartões do Anki", "ja": "Anki カードに画像と音声を追加する", "de": "Anki-Karten um Bilder und Audio ergänzen"},
    "description": {"en": "Add images and audio to generated Anki cards with Rust, Python or Node.js. Import files, compose card content, and manage export names, MIME types and budgets.", "zh-CN": "用 Rust、Python 或 Node.js 为生成的 Anki 卡片添加图片和音频：导入文件、组合卡片内容，并管理导出名称、MIME 类型与媒体预算。", "es": "Añade imágenes y audio a las tarjetas Anki que generes con Rust, Python o Node.js. Importa archivos, combina contenido y gestiona nombres de exportación, tipos MIME y límites de importación.", "pt-BR": "Use Rust, Python ou Node.js para importar arquivos, compor cartões com imagens e áudio e gerenciar nomes de exportação, tipos MIME e limites de mídia.", "ja": "Rust、Python、Node.js で画像・音声をインポートしてカードを組み立て、エクスポート名、MIME タイプ、メディア上限を管理します。", "de": "Importiere mit Rust, Python oder Node.js Dateien, kombiniere Karteninhalte und verwalte Exportnamen, MIME-Typen und Medienbudgets."}
  },
  {
    "source": "docs/image-occlusion.md",
    "route": "/docs/image-occlusion/",
    "name": "image-occlusion",
    "title": {"en": "Generate Anki Image Occlusion cards", "zh-CN": "生成 Anki 图片遮挡卡片", "es": "Genera tarjetas Anki con oclusión de imágenes", "pt-BR": "Gere cartões de oclusão de imagem do Anki", "ja": "Anki の画像穴埋めカードを生成する", "de": "Anki-Karten mit Bildverdeckung erzeugen"},
    "description": {"en": "Create Anki Image Occlusion cards programmatically from an image and the rectangle masks you supply, in either occlusion mode, with Rust, Python or Node.js.", "zh-CN": "用代码批量生成 Anki 图片遮挡卡片：提供图片和矩形遮罩坐标，支持两种遮挡模式，可使用 Rust、Python 或 Node.js。", "es": "Crea tarjetas de oclusión de imágenes de Anki desde código con Rust, Python o Node.js: proporciona la imagen y las máscaras rectangulares y elige uno de los dos modos de oclusión.", "pt-BR": "Crie cartões de oclusão de imagem com Rust, Python ou Node.js: forneça a imagem e as máscaras retangulares e escolha um dos dois modos de oclusão.", "ja": "Rust、Python、Node.js で画像と指定した矩形マスクから画像穴埋めカードを作ります。2つの遮蔽モードに対応しています。", "de": "Erzeuge Bildverdeckungskarten per Code mit Rust, Python oder Node.js: Gib Bild und Rechteckmasken vor und wähle einen der beiden Verdeckungsmodi."}
  },
  {
    "source": "docs/updates.md",
    "route": "/docs/updates/",
    "name": "updates",
    "title": {"en": "Update Anki decks with stable note IDs", "zh-CN": "用稳定笔记 ID 更新 Anki 牌组", "es": "Actualiza mazos de Anki con ID de nota estables", "pt-BR": "Atualize baralhos do Anki com IDs de nota estáveis", "ja": "安定したノート ID で Anki デッキを更新する", "de": "Anki-Stapel mit stabilen Notiz-IDs aktualisieren"},
    "description": {"en": "Release a new version of a generated Anki deck: keep stable note IDs, build from the previous original APKG and review the comparison before publishing.", "zh-CN": "发布生成牌组的新版本：保持稳定笔记 ID，以上一版原始 APKG 为基线构建，并在分发前检查比较结果。", "es": "Publica una nueva versión de un mazo de Anki generado: conserva los ID de nota estables, parte del APKG original anterior y revisa la comparación antes de distribuirlo.", "pt-BR": "Publique uma nova versão de um baralho gerado: preserve os IDs das notas, use o APKG original anterior como base e revise a comparação antes de distribuir.", "ja": "ノート ID を維持し、前回配布時のオリジナル APKG を基準にビルドして、比較を確認してから新バージョンを配布します。", "de": "Veröffentliche eine neue Stapelversion: Behalte stabile Notiz-IDs bei, verwende das ursprüngliche APKG der Vorgängerversion als Basis und prüfe den Vergleich vor der Verteilung."}
  },
  {
    "source": "docs/build-guarantees.md",
    "route": "/docs/build-guarantees/",
    "name": "build-guarantees",
    "title": {"en": "Build and output guarantees", "zh-CN": "构建与输出保证", "es": "Garantías de generación y de los archivos APKG", "pt-BR": "Garantias de geração e dos arquivos de saída", "ja": "ビルドと出力の保証", "de": "Build- und Ausgabegarantien"},
    "description": {"en": "Understand temporary artifacts, path protection and candidate publication.", "zh-CN": "理解临时产物、路径保护和候选版本发布。", "es": "Comprende la vida útil de los archivos, los límites de inspección y la publicación de una versión candidata revisada.", "pt-BR": "Entenda os artefatos temporários, a proteção dos caminhos, os limites de inspeção e a publicação de candidatos revisados.", "ja": "一時成果物、パスの保護、検査の上限、確認済み候補の公開を理解します。", "de": "Verstehe temporäre Artefakte, Pfadschutz, Inspektionsgrenzen und die Veröffentlichung geprüfter Kandidaten."}
  },
  {
    "source": "docs/troubleshooting.md",
    "route": "/docs/troubleshooting/",
    "name": "troubleshooting",
    "title": {"en": "Troubleshooting", "zh-CN": "排查问题", "es": "Soluciona errores de Anki Forge", "pt-BR": "Solução de problemas", "ja": "トラブルシューティング", "de": "Fehlersuche"},
    "description": {"en": "Resolve installation, media, template, identity and output errors.", "zh-CN": "解决安装、媒体、模板、身份与输出错误。", "es": "Diagnostica errores de instalación, notas, recursos multimedia, plantillas, identidad y publicación de archivos.", "pt-BR": "Resolva erros de instalação, notas, mídia, templates, identidade e publicação dos arquivos de saída.", "ja": "インストール、メディア、テンプレート、識別情報、出力のエラーを解決します。", "de": "Behebe Fehler bei Installation, Notizen, Medien, Vorlagen, Identität und der Veröffentlichung von Ausgabedateien."}
  },
  {
    "source": "docs/compatibility.md",
    "route": "/docs/compatibility/",
    "name": "compatibility",
    "title": {"en": "Compatibility and releases", "zh-CN": "兼容性与发布", "es": "Compatibilidad de Anki Forge y entornos verificados", "pt-BR": "Compatibilidade e ambientes verificados", "ja": "互換性と検証済みの環境", "de": "Kompatibilität und geprüfte Umgebungen"},
    "description": {"en": "Published packages, verified environments and Anki client conditions.", "zh-CN": "查看公开包、已验证运行环境和 Anki 客户端条件。", "es": "Consulta los paquetes públicos, los entornos probados y las condiciones de importación y reproducción en Anki.", "pt-BR": "Consulte os pacotes públicos, os ambientes testados e as condições de importação e reprodução nos clientes Anki.", "ja": "公開パッケージ、検証した実行環境、Anki クライアントでのインポートと再生の条件を確認します。", "de": "Prüfe veröffentlichte Pakete, verifizierte Laufzeitumgebungen und Bedingungen für Import und Wiedergabe in Anki-Clients."}
  },
  {
    "source": "docs/rust-api.md",
    "route": "/docs/rust-api/",
    "name": "rust-api",
    "title": {"en": "Rust API guide", "zh-CN": "Rust API 指南", "es": "Referencia de la API de Rust", "pt-BR": "Referência da API de Rust", "ja": "Rust API ガイド", "de": "Rust-API-Referenz"},
    "description": {"en": "Find public authoring methods, build defaults, reports and diagnostic behavior.", "zh-CN": "查找内容编写方法、构建默认值、报告和诊断行为。", "es": "Consulta los métodos de creación de contenido, las opciones de generación, los informes y los errores públicos de Rust.", "pt-BR": "Consulte os métodos públicos de criação de conteúdo, os padrões de geração, os relatórios e os diagnósticos de Rust.", "ja": "公開される作成メソッド、ビルドの既定値、レポート、診断の動作を確認できます。", "de": "Finde öffentliche Methoden zur Inhaltserstellung, Build-Vorgaben, Berichte und Diagnoseverhalten in Rust."}
  },
  {
    "source": "docs/node/quick-start.md",
    "route": "/docs/node-quickstart/",
    "name": "node-quickstart",
    "title": {"en": "Generate Anki flashcards with Node.js", "zh-CN": "用 Node.js 生成 Anki 卡片", "es": "Genera un APKG de Anki con Node.js", "pt-BR": "Gere cartões do Anki com Node.js", "ja": "Node.js で Anki カードを生成する", "de": "Anki-Karten mit Node.js erzeugen"},
    "description": {"en": "Install ankiforge from npm, generate Anki flashcards with Node.js or TypeScript, and export an APKG deck you can import into Anki.", "zh-CN": "从 npm 安装 ankiforge，用 Node.js 或 TypeScript 生成 Anki 卡片，并导出可导入 Anki 的 APKG 牌组。", "es": "Instala el paquete publicado de Node.js y exporta tu primer mazo con un programa completo. Continúa con TypeScript, imágenes y actualizaciones.", "pt-BR": "Instale ankiforge pelo npm, gere cartões com Node.js ou TypeScript e exporte um baralho APKG para importar no Anki.", "ja": "公開 Node パッケージをインストールし、完全な JavaScript プログラムで最初の永続 APKG をエクスポートします。", "de": "Installiere ankiforge über npm, erzeuge Karten mit Node.js oder TypeScript und exportiere einen APKG-Stapel zum Import in Anki."}
  },
  {
    "source": "docs/node/api.md",
    "route": "/docs/node-api/",
    "name": "node-api",
    "title": {"en": "Node and TypeScript API", "zh-CN": "Node 与 TypeScript API", "es": "API de Node y TypeScript", "pt-BR": "API de Node e TypeScript", "ja": "Node と TypeScript API", "de": "Node- und TypeScript-API"},
    "description": {"en": "Public methods, async behavior, build options and reports.", "zh-CN": "公开方法、异步行为、构建选项和报告。", "es": "Métodos públicos de Node, operaciones asíncronas, opciones de generación, gestión de archivos de salida e informes.", "pt-BR": "Métodos públicos, comportamento assíncrono, opções de geração, arquivos de saída e relatórios do SDK de Node.", "ja": "Node SDK の作成メソッド、非同期処理、メディア、所有権、型付きレポート、TypeScript の設定を確認できます。", "de": "Öffentliche Methoden, asynchrones Verhalten, Build-Optionen, Ausgabedateien und Berichte des Node-SDK."}
  },
  {
    "source": "docs/python/quick-start.md",
    "route": "/docs/python-quickstart/",
    "name": "python-quickstart",
    "title": {"en": "Generate Anki flashcards with Python", "zh-CN": "用 Python 生成 Anki 卡片", "es": "Genera un APKG de Anki con Python", "pt-BR": "Gere cartões do Anki com Python", "ja": "Python で Anki カードを生成する", "de": "Anki-Karten mit Python erzeugen"},
    "description": {"en": "Install ankiforge from PyPI, generate Anki flashcards with a short Python script, and export an APKG deck you can import into Anki.", "zh-CN": "从 PyPI 安装 ankiforge，用简短的 Python 脚本生成 Anki 问答卡片，并导出可导入 Anki 的 APKG 牌组。", "es": "Instala el wheel publicado de Python y crea tu primer archivo APKG de Anki con un programa completo y claves de nota estables.", "pt-BR": "Instale ankiforge pelo PyPI, crie cartões de pergunta e resposta com um script Python curto e exporte um baralho APKG para importar no Anki.", "ja": "仮想環境に公開 Python パッケージをインストールし、短い完全なプログラムで永続 APKG を作成します。", "de": "Installiere ankiforge über PyPI, erzeuge Karten mit einem kurzen Python-Skript und exportiere einen APKG-Stapel zum Import in Anki."}
  },
  {
    "source": "docs/python/api.md",
    "route": "/docs/python-api/",
    "name": "python-api",
    "title": {"en": "Python API", "zh-CN": "Python API", "es": "API de Python", "pt-BR": "API de Python", "ja": "Python API リファレンス", "de": "Python-API"},
    "description": {"en": "Project, media, build options, reports and temporary artifact ownership.", "zh-CN": "Project、媒体、构建选项、报告和临时产物的所有权。", "es": "Project, recursos multimedia, opciones de generación, informes y gestión de la vida útil de los archivos temporales.", "pt-BR": "Project, mídia, opções de geração, relatórios e controle da vida útil dos artefatos temporários em Python.", "ja": "Python のノートとモデル、メディア、ビルドと比較、エラー、並行処理、候補の事前生成と保存に使う API を確認できます。", "de": "Project, Medien, Build-Optionen, Berichte und die Kontrolle über die Lebensdauer temporärer Artefakte in Python."}
  },
  {
    "source": "docs/python/diagnostics.md",
    "route": "/docs/python-diagnostics/",
    "name": "python-diagnostics",
    "title": {"en": "Python diagnostics", "zh-CN": "Python 诊断", "es": "Diagnósticos de Python", "pt-BR": "Diagnósticos de Python", "ja": "Python の診断と更新ポリシー", "de": "Python-Diagnostik"},
    "description": {"en": "Understand native authoring errors, build reports and update evidence.", "zh-CN": "理解原生内容编写错误、构建报告和更新证据。", "es": "Interpreta los errores nativos, los informes de generación y las pruebas y políticas de actualización de Anki Forge.", "pt-BR": "Entenda os erros nativos de criação de conteúdo, os relatórios de geração, as evidências e as políticas de atualização em Python.", "ja": "構造化されたエラー、比較結果、更新ポリシー、検査上限、成果物の所有権を扱います。", "de": "Verstehe native Fehler bei der Inhaltserstellung, Build-Berichte, Aktualisierungsnachweise und Risikorichtlinien in Python."}
  },
  {
    "source": "bindings/python/MIGRATION.md",
    "route": "/docs/python-migration/",
    "name": "python-migration",
    "title": {"en": "Python API transition", "zh-CN": "Python API 迁移", "es": "Transición a la nueva API de Python", "pt-BR": "Transição para a nova API de Python", "ja": "Python API の移行", "de": "Umstieg auf die neue Python-API"},
    "description": {"en": "Use the current owned-value SDK and understand the clean-slate boundary.", "zh-CN": "使用当前拥有值所有权的 SDK，理解新版 API 的迁移边界。", "es": "Adapta tu código al SDK actual de Python y conoce los límites de compatibilidad con la API anterior.", "pt-BR": "Adapte seu código ao SDK atual de Python, que mantém a propriedade dos seus valores, e entenda a ruptura com a API anterior.", "ja": "旧 Deck・レジストリ・ランタイムの API を、安定キーとデータを所有する新しい Python API に置き換えます。", "de": "Passe deinen Code an das aktuelle Python-SDK mit eigenem Datenbesitz an und verstehe den Bruch mit der früheren API."}
  },
  {
    "source": "docs/python/genanki-migration.md",
    "route": "/docs/genanki-migration/",
    "name": "genanki-migration",
    "title": {"en": "Move from genanki", "zh-CN": "从 genanki 迁移", "es": "Pasar de genanki a Anki Forge", "pt-BR": "Migre do genanki para o Anki Forge", "ja": "genanki からの移行", "de": "Von genanki zu Anki Forge wechseln"},
    "description": {"en": "Understand content, template and media behavior when moving to Anki Forge.", "zh-CN": "了解迁入 Anki Forge 时内容、模板与媒体行为的变化。", "es": "Adapta notas, modelos, plantillas y recursos multimedia de genanki al modelo de Anki Forge y sus claves estables.", "pt-BR": "Entenda como adaptar notas, modelos e mídia do genanki às chaves estáveis e ao modelo de criação de conteúdo do Anki Forge.", "ja": "genanki のモデル、フィールド、デッキ、メディアを Anki Forge の概念に対応付け、識別情報を引き継ぐ条件を確認します。", "de": "Verstehe, wie du Notizen, Modelle, Vorlagen und Medien aus genanki an die stabilen Schlüssel und das Inhaltsmodell von Anki Forge anpasst."}
  }
];

export const docGroups = [
  { labels: { en: 'Start here', 'zh-CN': '从这里开始', es: "Empieza aquí", 'pt-BR': "Comece aqui", "ja": "はじめに", "de": "Hier beginnen" }, ids: ['docs', 'languages', 'quickstart', 'installation', 'concepts', 'anki-forge-vs-genanki'] },
  { labels: { en: 'Guides', 'zh-CN': '使用指南', es: "Guías", 'pt-BR': "Guias", "ja": "作業別ガイド", "de": "Anleitungen" }, ids: ['csv-to-anki', 'json-to-anki', 'ai-anki-cards', 'cards', 'media', 'custom-notetypes', 'templates', 'image-occlusion', 'updates'] },
  { labels: { en: 'Language guides', 'zh-CN': '开发语言指南', es: "Guías por lenguaje", 'pt-BR': "Guias por linguagem", "ja": "言語別ガイド", "de": "Anleitungen nach Programmiersprache" }, ids: ['rust-guide', 'node-quickstart', 'python-quickstart', 'python-migration', 'genanki-migration'] },
  { labels: { en: 'Reference', 'zh-CN': '参考', es: "Referencia", 'pt-BR': "Referência", "ja": "リファレンス", "de": "Referenz" }, collapsed: true, ids: ['api', 'rust-api', 'node-api', 'python-api', 'build-guarantees', 'compatibility'] },
  { labels: { en: 'Help', 'zh-CN': '帮助', es: "Ayuda", 'pt-BR': "Ajuda", "ja": "ヘルプ", "de": "Hilfe" }, ids: ['troubleshooting', 'python-diagnostics', 'development'] },
];
/** @type {{ id: string, label: Record<string, string> }[]} */
export const navigation = [
  { id: 'docs', label: { en: 'Guide', 'zh-CN': '指南', es: "Guía", 'pt-BR': "Guia", "ja": "ガイド", "de": "Anleitung" } },
  { id: 'examples', label: { en: 'Examples', 'zh-CN': '示例', es: "Ejemplos", 'pt-BR': "Exemplos", "ja": "サンプル", "de": "Beispiele" } },
  { id: 'blog', label: { en: 'Blog', 'zh-CN': '博客', es: "Blog", 'pt-BR': "Blog", "ja": "ブログ", "de": "Blog" } },
];
export const footerNavigation = [
  { id: 'articles', pageId: 'blog', resource: '', label: 'footer.articles' },
  { id: 'feed', pageId: '', resource: 'feed', label: 'footer.rss' },
  { id: 'license', pageId: '', resource: 'license', label: 'footer.license' },
  { id: 'repository', pageId: '', resource: 'repository', label: 'nav.repository' },
];
export const nextTasks = [
  { pageId: 'json-to-anki', label: 'home.json', primary: true },
  { pageId: 'csv-to-anki', label: 'home.csv', primary: false },
  { pageId: 'ai-anki-cards', label: 'home.aiCards', primary: false },
  { pageId: 'anki-forge-vs-genanki', label: 'home.compare', primary: false },
];
export const homepageModules = ['hero', 'install', 'ai-quickstart', 'performance', 'examples', 'updates', 'features', 'next'];
const common = [
  { id: 'home', route: '/', type: 'home', section: 'home', source: 'website/src/components/HomePage.astro', capabilities: homepageModules },
  { id: 'examples', route: '/examples/', type: 'examples', section: 'examples', source: 'website/src/components/ExamplesPage.astro', capabilities: ['gallery', 'updates'] },
  { id: 'blog', route: '/blog/', type: 'blog-list', section: 'blog', source: 'website/src/components/BlogPage.astro', capabilities: ['articles', 'feed'] },
];
const articles = ['your-first-anki-deck', 'stable-ids-and-deck-updates'].map(id => ({
  id: `blog:${id}`, route: `/blog/${id}/`, type: 'article', section: 'blog',
  sources: Object.fromEntries(Object.entries(configuredLocales).map(([locale, config]) => [locale, `website/src/content/blog/${config.prefix ? config.prefix.slice(1) + '/' : ''}${id}.mdx`])),
  capabilities: ['article', 'date'], metadata: { source: 'frontmatter', title: 'title', description: 'description' },
}));
export const pages = [
  ...common,
  ...documents.map(doc => ({
    id: doc.name, route: doc.route, type: 'docs', section: 'docs', sources: Object.fromEntries(Object.entries(configuredLocales).map(([locale, config]) => [locale, locale === 'en' ? doc.source : `docs${config.prefix}/${doc.name === 'docs' ? 'index' : doc.name}.md`])),
    title: doc.title, description: doc.description,
    capabilities: ['sidebar', 'search', 'toc', 'pagination', 'edit', 'code-copy'],
  })),
  ...articles,
].map((page, order) => {
  const title = ['home', 'examples', 'blog'].includes(page.id)
    ? Object.fromEntries(localeIds.map(locale => [locale, catalogs[locale]?.[`${page.id}.title`]])) : page.title;
  const description = ['home', 'examples', 'blog'].includes(page.id)
    ? Object.fromEntries(localeIds.map(locale => [locale, catalogs[locale]?.[`${page.id}.description`]])) : page.description;
  const sources = page.sources ?? Object.fromEntries(localeIds.map(locale => [locale, page.source]));
  return {
    ...page, title, description, sources, order,
    metadata: Object.fromEntries(localeIds.map(locale => [locale, { title: title?.[locale], description: description?.[locale] }])),
  };
});
export const docPages = pages.filter(page => page.type === 'docs');

export function pagePath(id, locale = 'en', section = '') {
  const page = pages.find(page => page.id === id);
  if (!page || !Object.hasOwn(locales, locale)) throw new Error(`Unknown page or release locale: ${id}/${locale}`);
  return `${localePath(page.route, locale)}${section ? `#${section}` : ''}`;
}
export function resolvePage(pathname, base = '/') {
  const resolved = resolveLocalePath(pathname, base);
  if (!resolved) return null;
  const page = pages.find(page => page.route === resolved.route);
  return page ? { page, locale: resolved.locale, path: localePath(page.route, resolved.locale) } : null;
}
export function pagePair(pathname, base = '/') {
  return resolvePage(pathname, base)?.page;
}
export function localeFromPath(pathname, base = '/') {
  return resolveLocalePath(pathname, base)?.locale ?? 'en';
}
export function documentationEntries(active = localeIds) {
  return docPages.flatMap(page => active.filter(locale => !page.sources[locale]?.startsWith('website/')).map(locale => ({
    id: page.id, name: page.id === 'docs' ? 'index' : page.id, locale,
    source: page.sources[locale], route: pagePath(page.id, locale),
    title: page.title[locale], description: page.description[locale],
    output: `src/content/docs${locales[locale].prefix}/docs/${page.id === 'docs' ? 'index' : page.id}.md`,
  })));
}
export const importedDocs = documentationEntries().filter(entry => entry.locale === 'en');
// This compatibility name now covers every non-English release locale.
export const translatedDocs = documentationEntries().filter(entry => entry.locale !== 'en');
