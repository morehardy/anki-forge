# Migre do genanki para o Anki Forge

Canonical page: https://morehardy.github.io/anki-forge/pt-br/docs/genanki-migration/
Language: pt-BR
Description: Entenda como adaptar notas, modelos e mídia do genanki às chaves estáveis e ao modelo de criação de conteúdo do Anki Forge.

Ainda está escolhendo uma biblioteca? Leia [Anki Forge ou genanki](https://morehardy.github.io/anki-forge/pt-br/docs/anki-forge-vs-genanki/).

O Anki Forge usa chaves de dados estáveis, modelos imutáveis validados, snapshots de mídia que não dependem dos arquivos de origem e operações explícitas de geração e atualização. Trata-se de um novo modelo de criação de conteúdo; ele não emula objetos genanki nem as antigas interfaces Deck e lockfile do Anki Forge.

| Conceito do genanki | Equivalente no Anki Forge |
| --- | --- |
| Model com ID numérico | `NoteType.builder(stable_key)...build()` |
| Nomes dos campos usados como identificadores | Chaves Field estáveis e nomes de exibição separados |
| Note contendo strings dos campos | `model.note().field(key, content)` com um modelo de propriedade da nota |
| Deck como contêiner de notas | Project com baralho padrão; `.deck(name)` opcional por nota |
| Caminhos dos arquivos de mídia do pacote | Valores independentes de `Media.file(path)`, coletados automaticamente pelo conteúdo tipado |
| Gravação do pacote | `project.build(BuildOptions.to(path))` |
| GUID padrão derivado dos campos ou regra personalizada de GUID | Namespace e chave estável da nota explícitos |

```python
from ankiforge import Project, NoteType, Field, Template, BuildOptions

model = (NoteType.builder('vocabulary').name('Vocabulary')
    .field(Field('word', name='Word', required=True))
    .field(Field('meaning', name='Meaning'))
    .template(Template('recognition', '{{word}}', '{{FrontSide}}<hr>{{meaning}}'))
    .build())
project = Project('vocabulary-course', default_deck='Languages::Vocabulary')
project.add('dictionary:cell', model.note().field('word', 'cell').field('meaning', '细胞'))
project.build(BuildOptions.to('vocabulary.apkg'))
```

Escolha chaves a partir de identificadores de negócio persistentes. Renomear um nome de exibição ou alterar o texto de uma definição não deve mudar essas chaves. Não use o hash do conteúdo atual dos campos como identidade, a menos que essa seja uma exigência explícita do seu importador. Para dados sem chaves, atribua e armazene as chaves fora do SDK.

As expressões dos templates de cartão usam as chaves dos campos (Field). Os nomes de exibição podem ser traduzidos de forma independente; a biblioteca converte as referências às chaves nos nomes de campo do Anki. A finalização do modelo rejeita declarações ausentes ou conflitantes antes que uma nota possa ser adicionada. A adição de notas coleta automaticamente as dependências dos modelos e rejeita definições conflitantes para uma mesma chave de modelo.

Strings representam texto simples; envolva HTML intencional em `Content.html(...)`. Use `media.image()` / `media.sound()` e `Content.sequence(...)` para compor o conteúdo. As cópias dos arquivos permanecem disponíveis mesmo se o arquivo original for removido ou alterado. Declare explicitamente as dependências referenciadas por HTML, CSS ou scripts com `.asset(media)` no construtor do modelo ou `project.add_asset(media)`.

Nas versões seguintes do baralho, crie um novo Project com o mesmo namespace e as mesmas chaves de dados e use `BuildOptions.to(next_path).update_from(previous_original_apkg)`. O APKG original fornece evidências completas de identidade. Esse fluxo não recupera a identidade de pacotes antigos arbitrários nem promete a conversão automática dos IDs numéricos ou GUIDs do genanki. Considere a primeira distribuição no novo modelo como uma mudança deliberada de identidade, a menos que uma migração de dados validada separadamente estabeleça a correspondência.

Consulte o [início rápido](https://morehardy.github.io/anki-forge/pt-br/docs/python-quickstart/), a [referência da API](https://morehardy.github.io/anki-forge/pt-br/docs/python-api/) e a [política de atualização](https://morehardy.github.io/anki-forge/pt-br/docs/python-diagnostics/).
