# Reutilize templates de cartão, CSS e mídia no Anki

Canonical page: https://morehardy.github.io/anki-forge/pt-br/docs/templates/
Language: pt-BR
Description: Carregue um tipo de nota reutilizável de um pacote com templates, CSS e mídia para cartões normais ou Cloze, usando Rust, Python ou Node.js.

Reutilize o mesmo design de cartões Anki em vários baralhos: mantenha os templates, o CSS e a mídia de um tipo de nota em um diretório e carregue-o para cartões normais ou Cloze. Os exemplos usam Rust; Python e Node.js carregam pacotes com `NoteType.from_bundle` e `NoteType.fromBundle` (veja a [API de Python](https://morehardy.github.io/anki-forge/pt-br/docs/python-api/#models-and-templates) e a [API de Node](https://morehardy.github.io/anki-forge/pt-br/docs/node-api/#immutable-models)).

Um pacote reúne uma declaração de modelo reutilizável, o texto dos templates, CSS e todas as dependências de mídia explícitas. `NoteType::from_bundle(directory)` valida o pacote completo e captura snapshots dos dados antes de retornar o mesmo `NoteType` imutável usado na criação direta por código. Ele não modifica um projeto parcialmente.

## Execute o exemplo com pacote de templates

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

O fluxo carrega o [pacote de teste de Cloze personalizado](https://github.com/morehardy/anki-forge/blob/main/contracts/fixtures/template-bundle/custom-cloze/anki-template.yaml) e grava `target/docs-examples/template.apkg`. O template usa **Languages::Cloze** como destino. Uma nota gera dois cartões.

## Manifesto e referências dos templates

O nome do arquivo de manifesto é `anki-template.yaml`. Somente o novo formato é aceito:

```yaml
format_version: template-bundle-v2
note_type:
  key: language-cloze
  name: Language Cloze
  cloze_field: text
  fields:
    - key: text
      name: Sentence
      sort: true
      required: true
    - key: extra
      name: Extra
  templates:
    - key: cloze
      name: Cloze
      front_file: front.html
      back_file: back.html
      browser_front_file: browser-front.html
      target_deck: Languages::Cloze
css_file: style.css
```

`note_type.key`, as chaves de campo e as chaves de template são identificadores estáveis obrigatórios. `name` é opcional e usa a chave correspondente por padrão. `cloze_field` seleciona um modelo Cloze; não é necessário declarar um tipo separado. Um campo sem `required: true` pode ser omitido. Propriedades desconhecidas no manifesto são rejeitadas.

Todos os templates usam **chaves de campo**, incluindo as versões de frente, verso e navegador: `{{cloze:text}}`, `{{extra}}` e seções condicionais como `{{#extra}}`. Nomes de exibição como Sentence e Extra são gerados para o Anki após a vinculação. O carregador preserva os caminhos originais dos arquivos e os intervalos de bytes dos templates nos erros.

## Carregue e reutilize um modelo

```rust
fn bundle(output: &Path, template_directory: &Path) -> anyhow::Result<()> {
    let model = NoteType::from_bundle(template_directory)?;
    let mut project = Project::new("docs-languages")?;
    project.add(
        "capital",
        model
            .note()
            .field("text", "{{c1::Madrid}} is in {{c2::Spain}}.")
            .field("extra", "A city and its country."),
    )?;
    let built = project.build(BuildOptions::to(output.join("template.apkg")))?;
    verify(&built, 1, 2, 0)?;
    Ok(())
}
```

Para uma aplicação independente, este exemplo completo espera o mesmo pacote copiado para `fixtures/template-bundle`:

```rust
use ankiforge::{BuildOptions, NoteType, Project};

fn main() -> anyhow::Result<()> {
    let model = NoteType::from_bundle("fixtures/template-bundle")?;
    let mut project = Project::new("bundle-example")?;
    project.add("capital", model.note()
        .field("text", "{{c1::Madrid}} is in {{c2::Spain}}.")
        .field("extra", "A city and its country."))?;
    let output = project.build(BuildOptions::to("bundle-example.apkg"))?;
    assert_eq!(output.report().counts().cards, 2);
    Ok(())
}
```

O modelo retornado independe do diretório de origem. Ele pode ser compartilhado com outro projeto ou usado depois que esse diretório for removido. Uma falha no carregamento não retorna um modelo parcial e remove os snapshots que não foram compartilhados.

## Recursos e limites

Declare explicitamente todos os recursos usados em templates, CSS ou scripts brutos. Cada entrada de `assets` usa um caminho relativo ao pacote e um nome de exportação fixo:

```yaml
assets:
  - path: assets/badge.png
    export_as: badge.png
  - path: assets/labels.woff
    export_as: labels.woff
```

Os templates podem então usar `<img src="badge.png">`, e o CSS pode usar `url('labels.woff')`. A propriedade `css_file` fornece o arquivo-fonte de CSS; recursos não são deduzidos de HTML, CSS ou JavaScript arbitrários. Recursos declarados permanecem no APKG mesmo quando nenhuma referência estática é detectada. Os nomes de exportação seguem as mesmas regras de portabilidade e de colisão por Unicode/maiúsculas e minúsculas de [Media](https://morehardy.github.io/anki-forge/pt-br/docs/media/).

O nome completo de exportação, incluindo a extensão, é limitado a 255 bytes UTF-8. O JSON Schema verifica a sintaxe e a quantidade de caracteres; `NoteType::from_bundle` também verifica o limite em bytes, o conteúdo dos arquivos e as referências antes de retornar um modelo validado. Passar na validação do schema, por si só, não garante o carregamento.

Caminhos absolutos, caminhos que escapam do diretório e links simbólicos para fora do pacote são rejeitados. O manifesto é limitado a 256 KiB; cada arquivo de texto de template/CSS é limitado a 2 MiB e deve ser UTF-8. Snapshots de recursos usam o limite padrão de mídia ou um `MediaLimits` fornecido pelo chamador por meio de `NoteType::from_bundle_with_limits`. São verificados tanto os tamanhos antes da leitura quanto os bytes efetivamente lidos.

Erros de pacote preservam o tipo/código, o caminho de origem, o deslocamento em bytes quando aplicável e o erro subjacente de schema, E/S, YAML, UTF-8 ou mídia. A finalização do modelo valida o schema; a adição de notas verifica separadamente os conflitos no projeto. Veja [tipos de nota personalizados](https://morehardy.github.io/anki-forge/pt-br/docs/custom-notetypes/) e [solução de problemas](https://morehardy.github.io/anki-forge/pt-br/docs/troubleshooting/).
