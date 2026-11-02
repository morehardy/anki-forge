Use Rust 1.92 ou posterior. Se preferir outra linguagem, escolha [Node / TypeScript ou Python](/docs/languages/).

<!-- section: install -->
## Instale a biblioteca

```sh
cargo new anki-deck
cd anki-deck
cargo add ankiforge@0.3.1
```

<!-- section: export-your-first-deck -->
## Exporte seu primeiro baralho

Salve este programa completo como `src/main.rs`:

<!-- source: anki_forge/examples/target_api_basic.rs -->
```rust
use ankiforge::{BuildOptions, Note, Project};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut project = Project::new("spanish")?.default_deck("Spanish");
    project.add("es:hola", Note::basic("hola", "hello"))?;
    let output = project.build(BuildOptions::to("spanish.apkg"))?;
    println!("{}", output.artifact().path().display());
    Ok(())
}
```
<!-- /source -->

```sh
cargo run
```

O programa grava o arquivo persistente `spanish.apkg`. Abra-o no Anki, importe-o e estude **hola → hello** no baralho **Spanish**. A exportação não exige que o Anki esteja instalado.

O namespace `spanish` e a chave estável `es:hola` identificam essa nota mesmo quando seu conteúdo muda. Você também pode [baixar o exemplo Basic do site](/generated/spanish.apkg).

<!-- section: continue -->
## Continue

Siga o [tutorial de JSON para Anki](/docs/json-to-anki/#rust) para executar uma tarefa completa com mídia e atualização.

Adicione [imagens e áudio](/docs/media/), [tipos de nota personalizados](/docs/custom-notetypes/) ou [oclusão de imagem](/docs/image-occlusion/). Guarde o APKG original distribuído antes de [comparar e publicar uma atualização](/docs/updates/). Consulte a [API de Rust](/docs/rust-api/) ou os [ambientes verificados](/docs/compatibility/) para saber mais.
