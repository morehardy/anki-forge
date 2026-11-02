# Atualize baralhos do Anki com IDs de nota estáveis

Canonical page: https://morehardy.github.io/anki-forge/pt-br/docs/updates/
Language: pt-BR
Description: Publique uma nova versão de um baralho gerado: preserve os IDs das notas, use o APKG original anterior como base e revise a comparação antes de distribuir.

Publique correções ou novos cartões para um baralho do Anki que você já distribuiu. IDs de nota estáveis ajudam o Anki Forge e o Anki a reconhecer notas existentes em vez de tratá-las como novas; as opções de importação do Anki e as edições locais mais recentes também influenciam o que será alterado.

Mantenha estáveis o namespace do projeto e as chaves de notas, tipos de nota, campos e templates de cartão. Gere cada nova versão a partir do **APKG original distribuído** anterior, que contém evidências completas de identidade em um formato versionado. Guarde o pacote junto com a revisão do seu código-fonte.

## Execute duas versões e compare

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

O comando grava `spanish-v1.apkg` e `spanish-v2.apkg`, compara a resposta alterada e verifica as contagens do pacote atualizado. O código executável é:

```rust
fn vocabulary(answer: &str) -> anyhow::Result<Project> {
    let mut project = Project::new("docs-spanish-updates")?.default_deck("Spanish");
    project.add("hola", Note::basic("hola", answer))?;
    Ok(project)
}

fn updates(output: &Path) -> anyhow::Result<()> {
    let previous = output.join("spanish-v1.apkg");
    let next = output.join("spanish-v2.apkg");
    vocabulary("hello")?.build(BuildOptions::to(&previous))?;
    let project = vocabulary("hello; hi")?;
    let comparison = project.compare(CompareOptions::against(&previous))?;
    ensure!(comparison.policy().allows_publication());
    let built = project.build(BuildOptions::to(&next).update_from(&previous))?;
    verify(&built, 1, 1, 0)?;
    ensure!(built.report().comparison().is_some());
    Ok(())
}
```

`compare` é concluído com sucesso mesmo quando os resultados bloqueiam a publicação. Consulte `comparison.policy().allows_publication()` separadamente do nível de risco original e das evidências. `build(...update_from(...))` executa a mesma análise e retorna `BuildError` se a política bloquear o candidato, antes da publicação.

## Política e limites de inspeção

Este programa completo define um limite mais rigoroso e usa a mesma configuração na comparação e na atualização:

```rust
use ankiforge::{BuildOptions, Note, Project};
use ankiforge::build::InspectLimits;
use ankiforge::update::{CompareOptions, RiskLevel, UpdatePolicy};

fn main() -> anyhow::Result<()> {
    let mut first = Project::new("policy-example")?;
    first.add("cell", Note::basic("Cell?", "Unit of life"))?;
    first.build(BuildOptions::to("policy-v1.apkg"))?;
    let mut next = Project::new("policy-example")?;
    next.add("cell", Note::basic("Cell?", "The basic unit of life"))?;
    let mut limits = InspectLimits::default();
    limits.max_collection_bytes = 1 << 30;
    let policy = UpdatePolicy::default().fail_on(RiskLevel::Medium);
    let comparison = next.compare(CompareOptions::against("policy-v1.apkg")
        .inspect_limits(limits.clone()).update_policy(policy.clone()))?;
    assert!(comparison.policy().allows_publication());
    next.build(BuildOptions::to("policy-v2.apkg").update_from("policy-v1.apkg")
        .inspect_limits(limits).update_policy(policy))?;
    Ok(())
}
```

A política padrão bloqueia resultados High e Critical. Depois de revisar uma categoria específica, a aplicação pode optar explicitamente por `UpdatePolicy::default().allow(RiskCode::NoteRemoved)`. Essa permissão aceita **todos os resultados da categoria**, preservando seus níveis e evidências originais. Ela não representa a aprovação individual de notas selecionadas. Uma permissão sem resultado correspondente gera um aviso; códigos de risco desconhecidos são rejeitados. Não permita automaticamente todos os códigos retornados por uma comparação.

Evidências ausentes ou corrompidas, divergência de namespace, dados inválidos do candidato e a ultrapassagem dos limites de inspeção são erros que impedem a operação, não categorias de risco que podem ser permitidas. Quando a análise não consegue terminar, a comparação retorna um erro com as observações já concluídas. Informar uma política explícita em uma criação sem `update_from` é um erro de configuração, independentemente da ordem dos métodos de configuração.

## Evidências e identidade

O pacote inclui `ankiforge-identity.json` no formato `ankiforge-identity-v1`. Ele vincula IDs de modelos e configurações, GUIDs de notas, posições históricas de campos e templates de cartão, ordinais de cartões e máscaras, revisões e mídia à coleção real e aos dados do pacote. Uma reexportação pelo Anki não preserva essas evidências e não serve como base de atualização. Evidências ausentes nunca são reconstruídas silenciosamente.

Alterar nomes de exibição ou conteúdo preserva as chaves de identidade. Conteúdo inalterado mantém a revisão da base; conteúdo alterado recebe uma revisão posterior. Reordenar campos ou templates de cartão preserva as correspondências históricas. Identidades desativadas permanecem reservadas para que a restauração da mesma chave possa reutilizá-las. Uma chave de nota não pode ser transferida para outro modelo, e uma chave de modelo não pode alternar entre os tipos normal e Cloze.

## Importação nos clientes e revisão antes da publicação

Um APKG é um pacote de distribuição, não um mecanismo de sincronização de exclusões. Omitir notas, campos, templates de cartão ou máscaras não exclui os dados existentes de quem estuda. Mudanças no esquema de campos e templates de cartão ou no campo de ordenação apresentam alto risco, pois seu comportamento depende das opções de mesclagem e importação do Anki. Mudanças de esquema podem copiar modelos ou atualizar os carimbos de data das notas de destino antes da comparação do conteúdo das notas.

Teste a primeira importação e as atualizações em uma coleção existente nos clientes que você atende. Confira campos, GUIDs, conjuntos de cartões, edições locais e agendamento existente com as opções pretendidas de mesclagem e atualização de notas. Identidade estável, sozinha, não promete substituir edições locais mais recentes nem restaurar histórico excluído.

Grave os candidatos em um caminho separado da base arquivada. Após a revisão, distribua o pacote aprovado e guarde-o como evidência para a próxima versão. Consulte as [garantias de geração](https://morehardy.github.io/anki-forge/pt-br/docs/build-guarantees/) para entender a propriedade dos arquivos e os detalhes da publicação.

## Gere uma vez, revise e publique

Esta API está incluída no pacote público `0.3.1`.

Gere uma vez, examine o relatório e publique o mesmo arquivo candidato. Revisar e depois executar uma segunda geração comum cria outro candidato; a publicação preparada mantém o APKG efetivo vinculado ao relatório revisado.

```rust
use ankiforge::{BuildOptions, Note, Project};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut project = Project::new("spanish")?.default_deck("Spanish");
    project.add("es:hola", Note::basic("hola", "hello"))?;
    project.build(BuildOptions::to("spanish-original.apkg"))?;

    let mut next = Project::new("spanish")?.default_deck("Spanish");
    next.add("es:hola", Note::basic("hola", "hello; hi"))?;
    let prepared = next.prepare_publication(
        BuildOptions::to("spanish-reviewed.apkg").update_from("spanish-original.apkg"),
    )?;
    println!("{:?}", prepared.report().comparison());
    if prepared
        .report()
        .comparison()
        .is_some_and(|report| report.policy().allows_publication())
    {
        let output = prepared.publish()?;
        println!("{}", output.artifact().path().display());
    }
    Ok(())
}
```

Na cópia do código-fonte fixada em uma revisão, salve o programa completo acima como `anki_forge/examples/docs_publication.rs`. Este guia adiciona o exemplo; ele não está incluído em `1199196`. Em seguida, execute:

```sh
cargo run --locked -p ankiforge --example docs_publication
```

O exemplo grava `spanish-original.apkg` e, quando a política permite a publicação, `spanish-reviewed.apkg` na raiz do repositório. A preparação não publica o arquivo de destino. Revise os resultados e teste a importação no cliente antes de aprovar a distribuição; o exemplo publica automaticamente apenas essa alteração simples de conteúdo quando sua política permite. O benchmark não mede esse fluxo.

Consulte o [ciclo de geração](https://morehardy.github.io/anki-forge/pt-br/docs/build-guarantees/#prepared-publication-from-source), a [API de Rust](https://morehardy.github.io/anki-forge/pt-br/docs/rust-api/#review-once-publish-once), a [API de Node](https://morehardy.github.io/anki-forge/pt-br/docs/node-api/#prepare-review-and-publish) e a [API de Python](https://morehardy.github.io/anki-forge/pt-br/docs/python-api/#prepare-review-and-publish).
