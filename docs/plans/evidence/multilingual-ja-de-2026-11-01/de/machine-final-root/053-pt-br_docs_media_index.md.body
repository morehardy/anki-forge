# Adicione imagens e áudio aos cartões do Anki

Canonical page: https://ankiforge.dev/pt-br/docs/media/
Language: pt-BR
Description: Use Rust, Python ou Node.js para importar arquivos, compor cartões com imagens e áudio e gerenciar nomes de exportação, tipos MIME e limites de mídia.

Inclua imagens e áudio nos cartões Anki gerados, como cartões de vocabulário com pronúncia ou perguntas sobre diagramas. O Anki Forge copia os arquivos que você fornece para o APKG; criar o áudio ou as imagens, por exemplo com síntese de voz, exige uma ferramenta separada. Os exemplos usam Rust; a seção de importação em lote também mostra as chamadas de Python e Node.js.

Um valor `Media` mantém um snapshot imutável sob sua própria responsabilidade. Importe arquivos com `Media::file` ou bytes nativos com um tipo MIME explícito por meio de `Media::bytes`. Após uma importação bem-sucedida, modificar ou excluir o arquivo de origem não altera o recurso. Clones compartilham o armazenamento e podem ser usados em diferentes projetos.

## Execute o fluxo de mídia

Na raiz do repositório:

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
cargo run --locked -p ankiforge --example target_api_media
```

O fluxo usa arquivos de teste PNG/WAV do repositório e grava `media.apkg` com uma nota, um cartão e três recursos. O segundo exemplo gera sua própria imagem e seu próprio áudio pequenos, inclui mídia referenciada por CSS e mídia explícita, e grava `spanish-media.apkg`.

```rust
fn media(output: &Path, image: &Path, sound: &Path) -> anyhow::Result<()> {
    let image = Media::file(image)?.with_export_name("cell.png")?;
    let sound = Media::file(sound)?;
    let mut project = Project::new("docs-media")?;
    project.add("cell", Note::basic(image.image(), sound.sound()))?;
    let css = Media::bytes(b".card { color: navy; }".to_vec(), "text/css")?
        .with_export_name("course.css")?;
    project.add_asset(css)?;
    let built = project.build(BuildOptions::to(output.join("media.apkg")))?;
    verify(&built, 1, 1, 3)?;
    Ok(())
}
```

## Combine conteúdo e recursos explícitos

Este programa completo usa `fixtures/pixel.png`, `fixtures/silence.wav` e `fixtures/labels.woff`. Na sua aplicação, forneça os caminhos dos seus próprios arquivos de origem. A fonte usada no teste é a fonte redistribuível do repositório.

```rust
use ankiforge::{BuildOptions, Content, Field, Media, NoteType, Project, Template};

fn main() -> anyhow::Result<()> {
    let image = Media::file("fixtures/pixel.png")?.with_export_name("badge.png")?;
    let sound = Media::file("fixtures/silence.wav")?;
    let font = Media::file("fixtures/labels.woff")?.with_export_name("labels.woff")?;
    let model = NoteType::builder("media-card")
        .field(Field::new("front"))
        .field(Field::new("back"))
        .template(Template::new("card")
            .front(r#"{{front}}<img src="badge.png">"#)
            .back("{{FrontSide}}<hr>{{back}}"))
        .css("@font-face { font-family: labels; src: url('labels.woff'); } .card { font-family: labels; }")
        .asset(font)
        .asset(image.clone())
        .build()?;
    let mut project = Project::new("media-guide")?;
    project.add("cell", model.note()
        .field("front", Content::sequence([Content::text("Cell "), image.image()]))
        .field("back", sound.sound()))?;
    let output = project.build(BuildOptions::to("media-guide.apkg"))?;
    assert_eq!(output.report().counts().media, 3);
    Ok(())
}
```

Os valores tipados `image()` e `sound()` mantêm as dependências até a exportação. Referências manuais em HTML, CSS ou scripts precisam de `builder.asset(media)` ou `project.add_asset(media)`; a biblioteca não deduz o conteúdo de arquivos a partir de strings arbitrárias. Um recurso explícito é incluído mesmo sem uma referência detectável estaticamente.

Passe o nome literal do arquivo a `with_export_name`. Referências tipadas de som escapam entidades HTML ao serem renderizadas, portanto um nome como `tone&copy;.mp3` continua apontando exatamente para esse recurso. Não escape o nome previamente. Sinais de porcentagem e caracteres de fragmento nos nomes de arquivos de som permanecem literais; não são caminhos de URL.

## Nomes, MIME e limites

Os nomes de exportação padrão são derivados do conteúdo e independem do caminho de origem. Use `with_export_name` antes de criar as referências quando precisar de um nome fixo. Renomear um clone não renomeia clones anteriores nem nós de conteúdo já criados.

Os nomes não podem conter caminhos, caracteres de controle ou nomes reservados não portáteis. As verificações de conflito usam Unicode NFC e normalização de maiúsculas e minúsculas: variantes que diferem apenas pela caixa ou pela normalização também entram em conflito. Pares de nome e conteúdo exatamente iguais são idempotentes; conteúdos diferentes sob o mesmo nome causam uma falha sem registro parcial.

O nome completo de exportação, incluindo a extensão, é limitado a 255 bytes UTF-8. Os JSON Schemas do projeto e do pacote de templates verificam a sintaxe do nome e a quantidade de caracteres; os carregadores também verificam esse limite em bytes. Por exemplo, 128 repetições de `é` passam pelo limite de caracteres do schema, mas ocupam 256 bytes e retornam `MEDIA.EXPORT_NAME_INVALID` no carregamento. Use `tools::load_project` ou `NoteType::from_bundle` para validar a entrada completa antes de gerar o pacote.

A sintaxe MIME deve ser compatível com o conteúdo identificável. Bytes nativos não têm a restrição de 64 KiB para dados inline. O limite de importação padrão é de 256 MiB por recurso; `file_with_limits` e `bytes_with_limits` recebem `MediaLimits { max_bytes }` antes da leitura. Snapshots grandes podem usar armazenamento temporário, removido quando o último proprietário deixa de existir. A propriedade do snapshot não garante uma visão atômica do sistema de arquivos quando a origem é modificada simultaneamente durante a importação.

O JSON do projeto aplica o mesmo limite padrão por recurso ao decodificar um array `bytes` inline. Ele para no primeiro byte excedente, sem coletar o restante do array, e preserva um `MediaError` com o limite e a quantidade de bytes observada. A contagem no momento da rejeição é um limite inferior; não é o tamanho total do array ainda não lido.

Um pacote de mídia válido não comprova a reprodução de codecs em todos os clientes Anki. Teste imagens, sons e fontes nos clientes que pretende usar. Veja [solução de problemas](https://ankiforge.dev/pt-br/docs/troubleshooting/#media-and-templates).

### Uso de imagem e som tipados

Os construtores continuam sem possibilidade de falha. `Project::add` verifica o MIME preservado na importação: imagens exigem `image/*`; referências de som aceitam `audio/*` e `video/*`. `application/octet-stream`, texto, fontes e PDF falham no uso tipado com `NOTE.MEDIA_USAGE_INVALID`; registre-os como recursos explícitos para HTML/CSS bruto. Declarações explícitas válidas de MIME de imagem/vídeo desconhecido passam na validação de categoria, sem certificar que um decodificador ou reprodutor aceite os bytes. Renomear um arquivo nunca altera o MIME preservado. A geração ainda inspeciona os bytes preparados e valida a extensão de exportação de forma independente: um PNG chamado `wrong.mp3` pode passar na adição como imagem e falhar na geração com `MEDIA.DECLARED_MIME_MISMATCH`.

## Importação de mídia em lote

Estes métodos estão incluídos no pacote público `0.3.1`.

Importe uma coleção com concorrência limitada, preserve a ordem de entrada e mantenha os snapshots de mídia para exportá-los depois:

```rust
use ankiforge::Media;
let images = Media::files(["first.png", "second.png"])?;
```

Em Node, use `await Media.files(['first.png', 'second.png'])`; em Python, use `Media.files(['first.png', 'second.png'])`.

### Comportamento e limites dos lotes

Use `Media::files(paths)` em Rust, `await Media.files(paths)` em Node ou `Media.files(paths)` em Python para importar arquivos na ordem de entrada, com no máximo quatro workers por chamada. Lotes com menos de 16 arquivos são processados sequencialmente. Rust também oferece `files_with_limits`; Node e Python aceitam os mesmos limites de `file`. O limite se aplica a cada recurso. Se uma importação falhar, a chamada espera o trabalho já iniciado, libera os snapshots do lote e retorna o primeiro erro na ordem de entrada. Proprietários de mídia já existentes continuam válidos. Python libera o GIL durante a importação; Node executa o trabalho fora do event loop. Importações individuais continuam criando snapshots imediatamente; é preciso escolher o método em lote para obter concorrência na importação.

Snapshots ativos compartilham um limite de memória de **64 MiB** por processo. Recursos individuais maiores que 1 MiB continuam sendo transferidos para arquivos temporários. A mídia codificada compartilha um pool de buffers de **16 MiB** por preparação, alocado sob demanda, e passa a usar arquivos temporários quando ele se esgota. Esses limites não incluem outras áreas de memória de trabalho e não são um teto de RSS; gerações simultâneas podem ter, cada uma, seu próprio pool de codificação.
