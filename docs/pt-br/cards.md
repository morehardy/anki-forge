# Cartões Basic e Cloze

Gere cartões Basic (pergunta e resposta) e Cloze (preenchimento de lacunas) do Anki com código, em vez de adicioná-los um por um no Anki. Uma nota Basic tem frente e verso. Uma nota Cloze gera um cartão para cada número de omissão aceito. Este exemplo em Rust cria duas notas e três cartões. Python e Node.js oferecem os mesmos construtores `Note.basic` e `Note.cloze`; veja a [API de Python](/docs/python-api/#project-and-note-values) e a [API de Node](/docs/node-api/#project-and-notes).

Na raiz do repositório:

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

Abra `target/docs-examples/cards.apkg` no Anki. A pergunta Basic é **hola**; os cartões Cloze ocultam **Madrid** e **Spain** separadamente.

<!-- section: author-the-notes -->
## Crie as notas

Esta função foi extraída do [programa executável completo](https://github.com/morehardy/anki-forge/blob/main/anki_forge/examples/docs_workflow.rs). A função auxiliar `verify` confere as quantidades de notas, cartões e mídias da saída gerada com sucesso.

<!-- source: anki_forge/examples/docs_workflow.rs#cards -->
```rust
fn cards(output: &Path) -> anyhow::Result<()> {
    let mut project = Project::new("docs-spanish")?.default_deck("Spanish");
    project.add("hola", Note::basic("hola", "hello"))?;
    project.add(
        "capital",
        Note::cloze("{{c1::Madrid}} is in {{c2::Spain}}.")
            .field("back_extra", "A city and its country."),
    )?;
    let built = project.build(BuildOptions::to(output.join("cards.apkg")))?;
    verify(&built, 2, 3, 0)?;
    Ok(())
}
```
<!-- /source -->

O primeiro argumento de `Project::new` é um namespace estável. Cada chamada a `add` precisa de uma chave estável do registro de origem. Reutilizar a mesma chave para outra nota causa uma falha; nenhuma chave baseada no conteúdo é atribuída automaticamente. Uma resposta alterada sob a mesma chave continua pertencendo à mesma nota lógica quando a atualização é gerada a partir da distribuição anterior.

<!-- section: text-fields-and-card-identity -->
## Texto, campos e identidade dos cartões

As strings de Basic e Cloze são Text. Use `Content::html` para incluir HTML de forma intencional; a sintaxe de omissão Cloze continua sendo reconhecida dentro de Text. `back_extra` é a chave do campo adicional do Cloze integrado. Use o mesmo método `Note::field` para texto, HTML, imagens e sons.

Use `{{c1::answer}}` ou `{{c1::answer::hint}}`. Uma frase comum não fornece uma pergunta Cloze. Os números de omissão aceitos vão de 1 a 500; alterá-los pode mudar a identidade dos cartões. Revise essas mudanças com a [comparação e a política de atualização](/docs/updates/).

Depois, adicione [mídia](/docs/media/), um [modelo personalizado](/docs/custom-notetypes/) ou [oclusão de imagem](/docs/image-occlusion/).
