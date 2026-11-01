# Oclusão de imagem

Gere cartões Image Occlusion do Anki com código: forneça uma imagem e coordenadas de retângulos, e o Anki Forge cria uma nota com um cartão por máscara, pronta para importar. Use esse recurso para produzir perguntas sobre diagramas em lote, sem desenhar máscaras no editor do Anki. Ele não detecta rótulos na imagem; as coordenadas são fornecidas por você.

Crie uma nota estruturada de oclusão a partir de uma imagem mantida em memória ou armazenamento próprio e de retângulos. Cada máscara precisa de uma chave estável que continue identificando a mesma região nas versões seguintes. A nota entra no projeto pelo caminho comum `add(key, note)`. Estes exemplos usam Rust; Python e Node.js oferecem o mesmo construtor na [API de Python](/docs/python-api/#media-and-occlusion) e na [API de Node](/docs/node-api/#media-and-image-occlusion).

<!-- section: run-the-example -->
## Execute o exemplo

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

Abra `target/docs-examples/diagram.apkg` no Anki. O programa completo carrega a imagem de oclusão do repositório e verifica uma nota, um cartão e um recurso.

<!-- source: anki_forge/examples/docs_workflow.rs#occlusion -->
```rust
fn occlusion(output: &Path, diagram: &Path) -> anyhow::Result<()> {
    let image = Media::file(diagram)?;
    let note = Note::image_occlusion(image)
        .mask(Mask::rect("region-1", 10, 10, 40, 25))
        .build()?;
    let mut project = Project::new("docs-diagram")?.default_deck("Diagram");
    project.add("diagram", note)?;
    let built = project.build(BuildOptions::to(output.join("diagram.apkg")))?;
    verify(&built, 1, 1, 1)?;
    Ok(())
}
```
<!-- /source -->

<!-- section: multiple-masks-and-modes -->
## Várias máscaras e modos

Este exemplo independente espera `fixtures/occlusion.png`, uma imagem decodificada de pelo menos 80×80 pixels:

```rust
use ankiforge::{BuildOptions, Media, Note, Project};
use ankiforge::note::{Mask, OcclusionMode};

fn main() -> anyhow::Result<()> {
    let note = Note::image_occlusion(Media::file("fixtures/occlusion.png")?)
        .mask(Mask::rect("nucleus", 10, 10, 20, 20))
        .mask(Mask::rect("wall", 50, 50, 20, 20))
        .mode(OcclusionMode::HideOneGuessOne)
        .build()?
        .field("header", "Cell structure")
        .field("back_extra", "Review the diagram.");
    let mut project = Project::new("cell-diagram")?;
    project.add("cell", note)?;
    let output = project.build(BuildOptions::to("cell-diagram.apkg"))?;
    assert_eq!(output.report().counts().cards, 2);
    Ok(())
}
```

`HideAllGuessOne` é o padrão; `HideOneGuessOne` também é aceito. Cada máscara com chave produz seu próprio cartão Cloze. As coordenadas são valores finitos em pixels da imagem exibida, após a aplicação da orientação EXIF. A biblioteca decodifica integralmente PNG, JPEG, GIF, WebP e BMP; o limite do buffer decodificado é de 256 MiB, independente do limite de importação da mídia comprimida. Os retângulos devem ter dimensões positivas, estar dentro da imagem e usar chaves únicas e não vazias.

O construtor valida a imagem e as máscaras antes de retornar uma `Note`. Use atribuições comuns de `field` para `header`, `back_extra` e `comments`; os campos gerados `image` e `occlusion` são reservados e não podem ser sobrescritos.

Ao atualizar a partir de uma distribuição anterior, as chaves das máscaras preservam os números ordinais dos cartões mesmo quando a ordem muda. Máscaras removidas mantêm seus ordinais reservados; restaurar a mesma chave reutiliza o ordinal. Chaves novas nunca herdam a identidade de outra máscara. A faixa de alocação aceita é de 1 a 500, e seu esgotamento causa uma falha explícita. Remover ou adicionar máscaras aparece na [análise de risco da atualização](/docs/updates/); omitir um cartão do APKG não exclui o cartão já existente do estudante.
