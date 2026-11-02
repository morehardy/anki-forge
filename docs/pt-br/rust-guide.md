# Guia de criação com Rust

Um `Project` descreve uma publicação. Crie-o com um namespace estável, adicione notas sob chaves estáveis e gere um APKG. Os tipos comuns são importados diretamente de `ankiforge`; as opções avançadas ficam nos módulos públicos de domínio.

<!-- section: projects-and-diagnostics -->
## Projetos e diagnósticos

```rust
use ankiforge::{BuildOptions, Content, Note, Project};

fn main() -> anyhow::Result<()> {
    let mut project = Project::new("jp-core")?
        .default_deck("Japanese::Core");
    project.add("taberu", Note::basic("食べる", Content::html("<b>to eat</b>")))?;
    let output = project.build(BuildOptions::to("jp-core.apkg"))?;
    println!("{} notes", output.report().counts().notes);
    println!("{}", serde_json::to_string_pretty(&output.snapshot())?);
    Ok(())
}
```

Uma geração bem-sucedida retorna `BuildOutput` com um artefato garantido. `BuildReport` contém observações: contagens, diagnósticos, tempo decorrido e qualquer comparação concluída. Ele não informa o resultado da operação nem mantém a propriedade do artefato. Avisos não transformam uma saída bem-sucedida em falha.

Erros de schema são verificados na finalização do modelo. `Project::add` verifica atomicamente a chave da nota, as referências de campo, os campos obrigatórios, o baralho, as tags e os conflitos de dependência. Uma falha deixa o projeto inalterado. A geração executa as verificações restantes de cartões, pacote e atualização. Não há registro mutável separado nem etapa obrigatória de validação.

Os erros oferecem os métodos inerentes `kind()` e `code()` e implementam `std::error::Error`. Use os códigos para processamento, em vez de analisar a mensagem exibida. `BuildError::snapshot()` inclui a falha real, observações, o texto da cadeia de causas e os fatos da publicação. Wrappers de erro comuns podem preservar o erro original como sua causa.

<!-- section: models-and-assets -->
## Modelos e recursos

Um `NoteType` finalizado é imutável e pode ser compartilhado. Chame `model.note()`, atribua campos por suas chaves estáveis e adicione o resultado a um projeto. A nota mantém seu modelo; adicioná-la incorpora o modelo e a mídia automaticamente. Veja [tipos de nota personalizados](/docs/custom-notetypes/) para as declarações e regras de template.

`Media::file` e `Media::bytes` capturam snapshots que mantêm seus próprios dados. Os arquivos de origem não precisam continuar disponíveis após a importação bem-sucedida. `image.image()` e `audio.sound()` produzem conteúdo estruturado; `Content::sequence` combina esse conteúdo com texto ou HTML. Recursos de HTML, CSS ou scripts brutos devem ser incluídos explicitamente com `builder.asset` ou `project.add_asset`. Veja [mídia](/docs/media/) e [pacotes de templates](/docs/templates/).

<!-- section: output-ownership -->
## Propriedade da saída

Escolha `BuildOptions::to(path)` para uma saída persistente ou `BuildOptions::temporary()` para um arquivo temporário com vida útil controlada. Mantenha a saída ou um clone de `output.artifact()` enquanto acessa um caminho temporário. Seu último proprietário remove o arquivo. `persist_to(path)` retorna um artefato persistente; se falhar, o proprietário original continua utilizável. Snapshots JSON contêm caminhos, mas não mantêm arquivos sob sua propriedade.

<!-- section: stable-note-identity -->
## Identidade estável das notas

Use uma chave durável do registro de origem em `project.add(key, note)`. O namespace do projeto, a chave do modelo, as chaves de campo e as chaves de template independem dos nomes de exibição. Alterar o conteúdo não cria uma nova nota lógica. Chaves duplicadas causam uma falha em vez de sobrescrever a nota. Salve e reutilize as chaves escolhidas quando os registros de origem forem reordenados ou editados.

<!-- section: updating-distributed-decks -->
## Atualize baralhos distribuídos

Use o APKG original distribuído anteriormente com `update_from`. Suas evidências completas incorporadas preservam os mapeamentos de identidade e revisão. A chamada independente `project.compare(CompareOptions::against(path))` analisa o candidato e informa se a política permitiria a publicação. Veja o [fluxo com duas versões](/docs/updates/) para mudanças de risco alto, limitações de importação no cliente e autorizações explícitas.

A partir do repositório, estes exemplos executam a mesma API pública:

```sh
cargo run --locked -p ankiforge --example target_api_basic
cargo run --locked -p ankiforge --example target_api_custom_notetype
cargo run --locked -p ankiforge --example target_api_media
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

<!-- section: addition-errors-and-media-categories -->
### Erros de adição e categorias de mídia

`Project` usa um namespace estável e um baralho padrão opcional; não tem título de exibição. `NoteType`, `Field` e `Template` continuam aceitando nomes de exibição. `Project::add` é atômico. Consulte `AddError::context()` para as chaves da nota/modelo e `note::AddTarget` para o local original; `detail()` retorna `note::AddDetail` para conflitos ou uso incompatível de mídia. Em expressões `match` com esses enums não exaustivos, use `..` nas variantes com dados e inclua um braço de fallback. O `content_path` de um campo é `None` para o campo, `Some([])` para sua raiz ou os índices da sequência para um nó aninhado. Os intervalos de bytes dos separadores apontam para a folha Text/Html UTF-8 original.

`Media::image()` e `sound()` continuam sem possibilidade de falha. A adição exige `image/*` para imagens e `audio/*` ou `video/*` para referências de som. Outras categorias produzem `NOTE.MEDIA_USAGE_INVALID`. Essa verificação usa o MIME preservado, não a decodificação nem a reprodução. HTML bruto e recursos explícitos mantêm seu comportamento. Renomear a exportação não altera o MIME: um PNG renomeado para `wrong.mp3` passa na adição de imagem, mas ainda pode falhar na verificação independente de conteúdo/extensão durante a geração com `MEDIA.DECLARED_MIME_MISMATCH`.

As receitas das ferramentas agora exigem `ankiforge-project-v2`, sem `name` no nível superior; v1 é rejeitado. O pacote de contratos é 2.1.0; a identidade do pacote continua sendo `ankiforge-identity-v1`.
