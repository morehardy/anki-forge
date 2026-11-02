# 巴西葡萄牙语文案审阅

整体可读，也能保留主要技术限制。需要优先明确 mídia exclusiva 与 cópias independentes 的含义，并统一 modelo、tipo de nota 和 template 的层级。若只改语法而不统一这些术语，全文仍会显得拼接。

保留 baralho、nota、cartão，以及用 anotações 指原始学习笔记的写法。namespace、snapshot、template 在开发者文档中可以保留英文，关键是首次说明并始终区分含义。

P2 表示应优先处理的事实范围或理解歧义；P3 表示可选的自然度、清晰度和术语优化。编号按位置统计，同一原因可能影响多处文字。本页建议已应用到产品源文件，并补齐相关术语的一致性。表中原文和行号保留审阅时快照，便于追溯；不作为正式发布的独立审阅凭据。

## 界面文案

| 编号与位置 | 原文 | 建议替换 | 问题 |
| --- | --- | --- | --- |
| pt-BR-01 P3<br>[home.choose](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/pt-BR.json:9) | Escolha sua linguagem. | Escolha sua linguagem de programação. | 明确编程语言，避免与内容语言混淆。 |
| pt-BR-02 P3<br>[ai.language](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/pt-BR.json:144) | Linguagem da aplicação | Linguagem de programação | 无障碍标签直接说明 Rust、Python、Node.js 的选择。 |
| pt-BR-03 P3<br>[ai.description](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/pt-BR.json:137) | Escolha uma linguagem, descreva sua tarefa e cole as instruções no seu assistente de programação com IA. | Escolha uma linguagem de programação, descreva sua tarefa e cole as instruções no seu assistente de programação com IA. | 语言指代具体化。 |
| pt-BR-04 P3<br>[ai.request](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/pt-BR.json:138) | O que o baralho deve fazer? | Que tipo de baralho você quer criar? | 询问想生成的牌组，比“牌组应该做什么”自然。 |
| pt-BR-05 P3<br>[home.evolve2](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/pt-BR.json:19) | Preserve a identidade delas. | Mantenha os mesmos IDs. | Preserve a identidade delas 抽象且像直译，读者真正需要保持的是 ID。 |
| pt-BR-06 P3<br>[home.evolveDescription](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/pt-BR.json:20) | Corrija uma definição ou acrescente um exemplo. IDs estáveis e uma geração anterior ajudam o Anki Forge a reconhecer as notas existentes na próxima exportação. | Corrija uma definição ou acrescente um exemplo. Mantenha os IDs das notas e use o APKG original da versão anterior para que o Anki Forge reconheça as notas existentes na próxima exportação. | uma geração anterior 指动作，不够明确地指向作为基准的原始 APKG。 |
| pt-BR-07 P3<br>[card.concert](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/pt-BR.json:66) | Um segundo de lá de referência. | Um tom de lá de referência com duração de um segundo. | 原文能理解，但“Um segundo de lá”较生硬，先说明声音再说明时长。 |
| pt-BR-08 P3<br>[update.none](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/pt-BR.json:83) | Nenhuma ocorrência | Nenhuma alteração detectada | Nenhuma ocorrência 缺乏对象，不清楚是什么没发生。 |
| pt-BR-09 P3<br>[examples.updateDescription](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/pt-BR.json:129) | A segunda geração altera a resposta de “hello” para “hello; hi”. Ela compara o resultado com o primeiro pacote e verifica se a identidade da nota foi preservada. | A segunda versão altera a resposta de “hello” para “hello; hi”. O programa compara o resultado com o primeiro pacote e verifica se o ID da nota foi mantido. | Ela 的指代是“生成”而不是程序；改成明确主语。 |
| pt-BR-10 P3<br>[performance.title](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/pt-BR.json:165) | O tempo de exportação, medido. | Tempo de exportação | 原文照搬英文插入语结构，葡语标题直接用名词更自然。 |
| pt-BR-11 P2<br>[performance.description](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/pt-BR.json:172) | Cada carga de trabalho contém {notes} notas Basic, com texto, mídia exclusiva ou mídia mista compartilhada. Rust levou de {minimum}% a {maximum}% menos tempo para exportar do que genanki. | Cada teste usa {notes} notas Basic. Os cinco testes abrangem notas só com texto, arquivos de mídia diferentes e mídia compartilhada entre notas. Nesses testes, a API Rust do Anki Forge levou de {minimum}% a {maximum}% menos tempo para exportar do que o genanki. | mídia exclusiva 没说独占于每条笔记，容易联想到独家资源；并明确性能比较对象。 |
| pt-BR-12 P2<br>[performance.basic-image-unique-v2](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/pt-BR.json:177) | Imagens exclusivas | Uma imagem diferente por nota | exclusivas 可能被理解为独家或专属内容；改为每条笔记的文件不同。 |
| pt-BR-13 P2<br>[performance.basic-audio-unique-v2](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/pt-BR.json:178) | Áudios exclusivos | Um áudio diferente por nota | 与图片标签统一，明确 unique 的比较范围。 |
| pt-BR-14 P3<br>[performance.basic-mixed-unique-v2](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/pt-BR.json:179) | Mídia mista exclusiva | Imagens e áudio sem compartilhamento | 避免 mídia mista exclusiva 的三层修饰，直接说明图片音频不共享。 |
| pt-BR-15 P3<br>[performance.basic-mixed-shared-v2](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/pt-BR.json:180) | Mídia mista compartilhada | Imagens e áudio compartilhados | 与非共享场景用同一套命名。 |
| pt-BR-24 P3<br>[home.templates](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/pt-BR.json:15) | Modelos personalizados | Templates personalizados | 与模板页统一；避免把 template 与 NoteType model 都叫 modelo。 |
| pt-BR-25 P3<br>[home.templateTitle](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/pt-BR.json:26) | Seus cartões. Seus modelos. | Seus cartões. Seus templates. | 模板模块使用 templates，与页面标题一致。 |
| pt-BR-26 P3<br>[home.templateDescription](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/pt-BR.json:27) | Comece com Basic e Cloze ou defina seus próprios tipos de nota. Reúna HTML, CSS e mídia em pacotes de modelos reutilizáveis. | Comece com Basic e Cloze ou defina seus próprios tipos de nota. Reúna HTML, CSS e mídia em pacotes de templates reutilizáveis. | 区分 tipos de nota 与 templates de cartão。 |
| pt-BR-27 P3<br>[home.templateLink](/Users/hp/Desktop/2026/anki-forge/website/src/i18n/pt-BR.json:28) | Explore os modelos | Explore os templates | 同步模板链接用词。 |

## 正文与元数据

### pt-BR-16 P2

[docs/pt-br/anki-forge-vs-genanki.md](/Users/hp/Desktop/2026/anki-forge/docs/pt-br/anki-forge-vs-genanki.md:11)

“独立副本”没说明独立于源文件，易误解为每个 clone 都复制数据；补清共享存储。

原文片段：

> Imagens e sons tipados contêm cópias independentes dos dados

建议替换：

> Os valores tipados de imagem e áudio mantêm snapshots independentes dos arquivos de origem; clones compartilham o armazenamento

### pt-BR-17 P3

[docs/pt-br/genanki-migration.md](/Users/hp/Desktop/2026/anki-forge/docs/pt-br/genanki-migration.md:5)

同一所有权问题；明确独立于源文件，不是彼此独立的深拷贝。

原文片段：

> cópias independentes de mídia

建议替换：

> snapshots de mídia que não dependem dos arquivos de origem

### pt-BR-18 P2

[docs/pt-br/updates.md](/Users/hp/Desktop/2026/anki-forge/docs/pt-br/updates.md:71)

达到/超过上限的条件被省略；上限本身不是错误。

原文片段：

> dados inválidos do candidato e limites de inspeção são erros

建议替换：

> dados inválidos do candidato e a ultrapassagem dos limites de inspeção são erros

### pt-BR-19 P3

[docs/pt-br/updates.md](/Users/hp/Desktop/2026/anki-forge/docs/pt-br/updates.md:5)

modelos 同时指模型与模板，且别处使用 templates；建议类型叫 tipos de nota，卡片模板叫 templates de cartão。

原文片段：

> as chaves de notas, modelos, campos e modelos de cartão

建议替换：

> as chaves de notas, tipos de nota, campos e templates de cartão

### pt-BR-20 P3

[docs/pt-br/genanki-migration.md](/Users/hp/Desktop/2026/anki-forge/docs/pt-br/genanki-migration.md:32)

表达式位于卡片模板，不是整个 NoteType；与模板页术语一致。

原文片段：

> As expressões dos modelos usam chaves Field.

建议替换：

> As expressões dos templates de cartão usam as chaves dos campos (Field).

### pt-BR-21 P3

[website/src/content/blog/pt-br/stable-ids-and-deck-updates.mdx](/Users/hp/Desktop/2026/anki-forge/website/src/content/blog/pt-br/stable-ids-and-deck-updates.mdx:4)

英文 outlive 隐喻逐词移植显得刻意；直接说明文章解决的问题。

原文片段：

> title: A identidade da nota deve durar mais que seu texto

建议替换：

> title: Atualize o conteúdo da nota sem mudar seu ID

### pt-BR-22 P3

[website/site-model.mjs](/Users/hp/Desktop/2026/anki-forge/website/site-model.mjs:90) · `page:installation.title.pt-BR`

本页只有 Rust 集成步骤，应在标题点明。

原文片段：

> Integre o Anki Forge à sua aplicação

建议替换：

> Integre o Anki Forge à sua aplicação Rust

### pt-BR-23 P3

[website/site-model.mjs](/Users/hp/Desktop/2026/anki-forge/website/site-model.mjs:76) · `page:templates.title.pt-BR`

与正文 templates 统一；modelo 留给模型对象，减少两个层级同名。

原文片段：

> Reutilize modelos de cartão, CSS e mídia no Anki

建议替换：

> Reutilize templates de cartão, CSS e mídia no Anki
