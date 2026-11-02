# Crie tipos de nota personalizados

Canonical page: https://ankiforge.dev/pt-br/docs/custom-notetypes/
Language: pt-BR
Description: Defina campos, templates de cartão e regras de geração para seu conteúdo, com chaves estáveis e modelos imutáveis reutilizáveis.

Finalize um modelo uma vez e crie notas a partir dele. O `NoteType` retornado é imutável e pode ser reutilizado em funções ou em vários projetos. Uma nota mantém seu modelo, portanto não pode apontar por engano para o nome de um modelo não registrado.

## Exemplo executável

O exemplo define chaves estáveis e nomes de exibição separados, atribui valores pela chave e usa as mesmas chaves nos templates.

```rust
use ankiforge::schema::GenerationRule;
use ankiforge::{BuildOptions, Field, NoteType, Project, Template};

fn main() -> anyhow::Result<()> {
    let vocab = NoteType::builder("jp-vocab")
        .name("Japanese Vocabulary")
        .field(Field::new("expr").name("Expression").sort().required())
        .field(Field::new("meaning").name("Meaning").required())
        .template(
            Template::new("recognition")
                .name("Recognition")
                .front("{{expr}}")
                .back("{{FrontSide}}<hr id=\"answer\">{{meaning}}")
                .generate_when(GenerationRule::all(["expr"])),
        )
        .build()?;
    let mut project = Project::new("jp-core")?.default_deck("Japanese::Core");
    project.add(
        "taberu",
        vocab
            .note()
            .field("expr", "食べる")
            .field("meaning", "to eat"),
    )?;
    project.build(BuildOptions::to("jp-core.apkg"))?;
    Ok(())
}
```

Execute a partir do repositório com `cargo run --locked -p ankiforge --example target_api_custom_notetype`. Abra `jp-core.apkg` no Anki.

## Chaves e nomes

`Field::new` e `Template::new` recebem chaves estáveis. `.name` fornece um nome de exibição; sem ele, o nome é igual à chave. Nenhum slug é derivado do nome. Chaves vazias, com espaços em branco, conflitantes ou reservadas impedem a finalização do modelo. Os nomes podem usar chinês ou outro texto Unicode sem alterar as chaves de identidade.

Os templates vinculam as chaves e são compilados para os nomes de campo do Anki. Isso abrange frente e verso, templates do navegador, seções condicionais, filtros e Cloze. O restante do HTML, os espaços em branco e o texto dos scripts são preservados. Referências desconhecidas causam uma falha com o intervalo de bytes na origem. A atribuição de campos aceita somente chaves, sem remoção silenciosa de espaços nem normalização de maiúsculas e minúsculas.

## Validação e geração

`required()` rejeita conteúdo de nota ausente ou vazio. Campos sem essa opção podem ser omitidos e são renderizados vazios. `sort()` seleciona o campo de ordenação; no máximo um campo pode ser selecionado. A geração usa a regra padrão do Anki, a menos que o template declare explicitamente `GenerationRule::all` ou `any`. Essas regras referenciam chaves de campo. `builder.cloze_field(key)` seleciona a semântica Cloze para um modelo personalizado.

`builder.asset(media)` declara uma dependência de CSS, fonte, script ou HTML bruto que mantém seus próprios dados. Conteúdo tipado de imagem e som nas notas incorpora os recursos automaticamente. Recursos explícitos permanecem na saída mesmo quando nenhuma referência estática é encontrada.

A mesma chave de modelo só pode ser reutilizada em um projeto com a mesma definição. Uma definição conflitante causa uma falha atômica ao adicionar a nota. Mudanças no nome de exibição, nos campos ou nos templates em versões posteriores exigem [análise de atualização](https://ankiforge.dev/pt-br/docs/updates/), mesmo quando as chaves estáveis permanecem iguais.

Para reutilizar arquivos e recursos, use um [pacote de templates](https://ankiforge.dev/pt-br/docs/templates/).
