# API de Rust

Os valores comuns estão disponíveis na raiz do crate: `Project`, `Note`, `NoteType`, `Field`, `Template`, `Content`, `Media`, `BuildOptions` e `BuildOutput`. Os módulos públicos de domínio são `note`, `schema`, `media`, `build`, `update` e `diagnostics`. As ferramentas do repositório formam uma interface separada, habilitada por feature; aplicações consumidoras não precisam delas.

<!-- section: authoring -->
## Criação de conteúdo

| Entrada | Responsabilidade |
| --- | --- |
| `Project::new(namespace)` | Validar um namespace estável de publicação |
| `project.default_deck(name)` | Definir o destino padrão |
| `project.add(key, note)` | Incorporar atomicamente a nota, o modelo e a mídia |
| `project.add_asset(media)` | Incluir um recurso explícito de HTML/CSS/script bruto |
| `Note::basic(front, back)` / `Note::cloze(text)` | Criar uma nota que mantém seu modelo integrado |
| `model.note().field(key, content)` | Criar uma nota personalizada que mantém seu modelo imutável |
| `note.deck(name).tag(tag)` / `.tags(iterable)` | Configurar uma nota antes de adicioná-la |
| `NoteType::builder(key)...build()` | Validar e finalizar um modelo imutável |
| `NoteType::from_bundle(path)` | Carregar um modelo e seus recursos com dados próprios de bundle-v2 |

`Field::new(key).name(display)` e `Template::new(key).name(display)` separam identidade e apresentação. Os templates vinculam chaves de campo, incluindo filtros, seções condicionais e templates do navegador. `Field::required()` verifica conteúdo não vazio; `Field::sort()` seleciona o campo de ordenação. `schema::GenerationRule::all` e `any` recebem chaves de campo. `builder.cloze_field(key)` declara um modelo Cloze.

`schema::FieldKey` e `TemplateKey` são tipos distintos. Strings literais, strings com propriedade, strings emprestadas, `Cow<str>` e chaves emprestadas ou com propriedade podem ser combinadas. As conversões preservam o valor original; a finalização do schema valida as chaves. Uma chave de template não é aceita implicitamente como chave de campo.

<!-- section: content-and-media -->
## Conteúdo e mídia

Todas as strings comuns se tornam `Content::text`. HTML exige `Content::html`. `Content::sequence` combina valores semânticos até a renderização durante a geração. `Media::file(path)` e `Media::bytes(bytes, mime)` mantêm snapshots próprios; suas variantes `with_limits` aplicam um limite `media::MediaLimits { max_bytes }` antes da leitura. O padrão é 256 MiB por recurso. `with_export_name(name)` retorna um valor renomeado que compartilha o mesmo snapshot; clones existentes preservam seus nomes.

`media.image()` e `media.sound()` mantêm as dependências dos recursos. Nomes de arquivo fixos devem ser portáteis e livres de colisões. Recursos explícitos são incluídos mesmo sem uma referência visível estaticamente. Veja [mídia](/docs/media/).

`Note::image_occlusion(media)` retorna `note::ImageOcclusionBuilder`. `.mask(Mask::rect(key, x, y, width, height)).mode(mode).build()` valida as dimensões da imagem decodificada, os retângulos e as chaves estáveis únicas. Tanto `HideAllGuessOne` quanto `HideOneGuessOne` são aceitos. Veja [oclusão de imagem](/docs/image-occlusion/).

<!-- section: build-options -->
## Opções de geração

`project.build(BuildOptions::to(path))` publica um APKG persistente. `BuildOptions::temporary()` retorna uma saída temporária com vida útil controlada. Ambos aceitam `.update_from(previous_apkg)`, `.inspect_limits(limits)` e `.update_policy(policy)`. Uma política sem base de atualização é um erro de configuração. A ordem de chamada dos setters não altera essa regra.

`build::InspectLimits::default()` fornece limites finitos; seus campos públicos podem ser modificados. Todos os valores são `u64`; zero significa zero, não ilimitado.

| Campo | Padrão |
| --- | --- |
| `max_archive_bytes` | 2 GiB |
| `max_entries` | 100.000 |
| `max_central_directory_bytes` | 16 MiB |
| `max_zip_entry_bytes` / `max_zip_total_bytes` | 1 GiB / 4 GiB |
| `max_meta_bytes` / `max_media_map_bytes` | 64 KiB / 16 MiB |
| `max_identity_bytes` | 64 MiB |
| `max_collection_bytes` / `max_media_bytes` | 512 MiB / 256 MiB |
| `max_decoded_total_bytes` / `max_zstd_window_bytes` | 4 GiB / 64 MiB |

Os limites se aplicam de forma independente à inspeção da base e do candidato. Uma verificação que falha informa o recurso, o limite, a quantidade observada e a entrada relacionada, quando disponível. Aumentar um limite é uma ação explícita do chamador; a importação de mídia tem seu próprio limite anterior.

<!-- section: build-compare-and-output -->
## Geração, comparação e saída

`BuildOutput::artifact()` retorna uma referência a um `build::ApkgArtifact` que mantém a propriedade do arquivo. `BuildOutput::report()` retorna observações. Clones de artefatos compartilham a vida útil do arquivo temporário; `persist_to` retorna um proprietário persistente ou `build::PersistError`, com os fatos da publicação e a causa subjacente de E/S.

`project.compare(update::CompareOptions::against(path))` retorna um `ComparisonReport` concluído mesmo se resultados de risco alto bloquearem sua política. Ele oferece `findings()`, `highest_risk()`, `policy()`, `diagnostics()` e `snapshot()`. `UpdatePolicy::default()` bloqueia High/Critical; `.fail_on(level)` altera o limiar e `.allow(RiskCode)` autoriza uma categoria inteira de risco registrada. Ele não pode autorizar evidências de identidade ausentes/corrompidas nem outros erros impeditivos.

<!-- section: reports-and-errors -->
## Relatórios e erros

`BuildReport::snapshot()` retorna `build::json::ReportSnapshot`: observações sem estado de sucesso/falha nem propriedade de arquivos. `BuildOutput::snapshot()` e `BuildError::snapshot()` retornam `build::json::BuildSnapshot`, incluindo o resultado real e o relatório. `update::json::ComparisonSnapshot` separa os resultados originais da análise da avaliação da política. Todos os DTOs de snapshot implementam `serde::Serialize` e não mantêm arquivos. Nenhum trait de extensão é necessário.

Os campos de caminho dos snapshots preservam seus valores nativos `PathBuf` em Rust. Sua forma JSON é uma string para Unicode válido ou uma codificação nativa sem perda: `{"encoding":"unix_bytes","bytes":[...]}` para nomes Unix que não são UTF-8, e `{"encoding":"windows_wide","units":[...]}` para nomes Windows com substitutos UTF-16 sem par. Isso se aplica tanto aos caminhos de artefatos bem-sucedidos quanto aos caminhos de publicação de falhas. Para restaurar um caminho JSON na plataforma original, desserialize-o como `build::json::PathSnapshot` e chame `into_path_buf()`; caminhos codificados de outra plataforma são rejeitados. `PathSnapshot::new(path)` fornece a mesma representação quando você serializa por conta própria o caminho nativo de um erro.

Os erros ficam junto de suas operações: erros de schema e pacote em `schema`, erros de adição/oclusão de imagem em `note`, erros de mídia em `media`, erros de geração/persistência em `build` e erros de comparação/política em `update`. Cada um oferece `kind()` e `code()`, implementa `std::error::Error + Send + Sync + 'static` e preserva os erros reais de origem.

<!-- section: addition-context -->
## Contexto da adição

Adições que falham deixam o projeto inalterado. O contexto estruturado registra as chaves originais da nota/modelo e o alvo, com detalhes opcionais de conflito ou uso de mídia. Rust oferece `note::{AddContext, AddTarget, AddDetail}` por meio de `AddError::context()` / `detail()`; os SDKs oferecem `AddError.details` com contexto/detalhes tipados. Os campos de Node usam camelCase e são profundamente congelados; Python usa TypedDicts em snake_case. Tags preservam seu índice de inserção; localizações de campo preservam os índices originais da sequência (null/None para o campo, [] para sua raiz) e intervalos opcionais de bytes UTF-8. Erros de baralho padrão na geração não têm chaves de nota/modelo e continuam disponíveis na cadeia de causas.

Imagens tipadas exigem MIME image/* na adição; referências de som aceitam audio/* ou video/*. Outras categorias geram `NOTE.MEDIA_USAGE_INVALID`. Os construtores continuam sem possibilidade de falha; recursos explícitos e HTML bruto não são afetados. A verificação usa o MIME preservado na importação e não certifica a reprodução. A validação independente de MIME/extensão durante a geração continua valendo. Veja o [documento de projeto](https://github.com/morehardy/anki-forge/blob/main/docs/plans/2026-09-28-rust-api-validation-and-errors-design.md) para a tabela completa de alvos/detalhes.

<!-- section: review-once-publish-once -->
## Revise uma vez, publique uma vez

A API de publicação preparada está incluída no pacote público `0.3.1`.

```rust,no_run
use ankiforge::{BuildOptions, Project};
# fn publish(project: &Project) -> Result<(), Box<dyn std::error::Error>> {
let prepared = project.prepare_publication(
    BuildOptions::to("next.apkg").update_from("previous.apkg"),
)?;
let report = prepared.report().clone(); // observations; owns no file
if report.comparison().is_some_and(|c| !c.policy().allows_publication()) {
    drop(prepared); // deletes the unpublished candidate; reprepare with a new policy
    return Ok(());
}
match prepared.publish() { // consumes the owner on every outcome
    Ok(output) => println!("Published {}", output.artifact().path().display()),
    Err(error) => {
        // Replacement may have succeeded even if durability confirmation failed.
        eprintln!("{:?}", error.publications());
        return Err(error.into());
    }
}
# Ok(())
# }
```

A preparação executa a inspeção completa com limites e a comparação. Nenhum destino é publicado até `publish`, que usa o mesmo candidato privado e verifica novamente os aliases da base. Caminhos relativos são fixados na chamada de preparação. Edições no projeto não alteram o candidato. A duração do relatório não inclui o tempo de revisão. Liberar um proprietário ainda não publicado remove o candidato; a saída temporária publicada segue a regra comum de remoção pelo último proprietário do artefato. Não é possível restaurar um proprietário a partir de JSON nem tentar novamente com um proprietário já consumido. Gerações nativas omitem o manifesto intermediário não utilizado, mas preservam as evidências de identidade incorporadas e a validação de mídia. As ferramentas internas de preparação ainda recebem o manifesto completo e sua impressão digital.
