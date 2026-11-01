# Garantias de geração e saída

`Project::build` retorna `BuildOutput` somente após uma operação bem-sucedida, com um artefato APKG garantido. Uma operação que falha retorna `BuildError` com tipo/código estruturados, observações, causas subjacentes e os fatos reais da publicação.

<!-- section: temporary-and-persistent-artifacts -->
## Artefatos temporários e persistentes

Escolha explicitamente quem mantém o arquivo disponível:

| Solicitação | Propriedade do arquivo |
| --- | --- |
| `BuildOptions::to(path)` | Destino persistente; permanece após a liberação do handle |
| `BuildOptions::temporary()` | Arquivo temporário; é excluído com seu último proprietário |
| `artifact.persist_to(path)` | Novo proprietário persistente; o proprietário original continua utilizável |

Mantenha o `BuildOutput` ou um clone do artefato vivo enquanto lê um caminho temporário. Um relatório ou snapshot JSON não prolonga a vida do arquivo. Rust libera os proprietários normalmente; Node oferece `artifact.close()` para uma liberação determinística. Copiar o caminho para o estado da aplicação não cria um proprietário.

Handles de artefato em Rust preservam caminhos absolutos, inclusive quando a geração ou `persist_to` usa um destino relativo. Mudanças posteriores no diretório de trabalho do processo não redirecionam leituras nem cópias de um handle existente. Cada novo destino de `persist_to` é resolvido em relação ao diretório daquela chamada.

`BuildReport::snapshot()` contém somente observações. `BuildOutput::snapshot()` e `BuildError::snapshot()` também informam o resultado real da operação. Um aviso pode aparecer em um snapshot de sucesso. Não deduza o resultado pela gravidade dos diagnósticos, pela existência de um caminho ou pela presença de resultados de comparação no relatório.

<!-- section: baselines-and-publication -->
## Bases de atualização e publicação

Para atualizações, mantenha o APKG original distribuído anteriormente como uma base imutável e escolha outro destino para a saída. As evidências da base e o conteúdo candidato são verificados antes da publicação. Uma rejeição pela política não substitui a saída. Evidências ausentes ou corrompidas e limites de inspeção esgotados são falhas impeditivas.

A publicação usa substituição atômica do APKG. Essa garantia vale por arquivo; gravar um relatório JSON separado da aplicação é outra operação. A geração comum não grava um arquivo adicional de identidade: as evidências completas e versionadas ficam incorporadas no pacote distribuído.

Um erro tardio pode ocorrer após a publicação, por exemplo ao confirmar a persistência. Por isso, snapshots de erro registram os fatos de `PublicationSnapshot`: `path`, `stage` (`not_published` ou `published`), `temporary` e durabilidade (`confirmed` ou `unconfirmed`). Verifique esses fatos antes de tentar novamente ou anunciar que nenhum arquivo foi gravado. `PersistError` também preserva o estado da publicação e a causa real de E/S. A substituição atômica, por si só, não comprova durabilidade após uma falha do sistema.

A serialização dos snapshots preserva caminhos nativos sem acessar o sistema de arquivos. Caminhos Unicode permanecem strings JSON. Caminhos Unix que não são UTF-8 usam `{"encoding":"unix_bytes","bytes":[...]}`; caminhos Windows com substitutos UTF-16 sem par usam `{"encoding":"windows_wide","units":[...]}`. Os arrays contêm exatamente os bytes ou unidades de código nativos. Essa representação é usada nos caminhos de artefatos bem-sucedidos, nos caminhos de publicação de falhas e nos detalhes de caminhos dos erros dos bindings. Esses valores descrevem caminhos e não mantêm a propriedade de artefatos. Um sistema de arquivos pode rejeitar determinado caminho nativo, mas os fatos de sua falha de publicação continuam serializáveis.

<!-- section: inspection-limits -->
## Limites de inspeção

Limites finitos de arquivo compactado, quantidade de entradas, expansão, coleção/mídia decodificadas e janela zstd se aplicam de forma independente à base e ao candidato. Comece com `InspectLimits::default()` e aumente deliberadamente o limite relevante quando necessário. Um limite zero não significa ilimitado. Esses contadores não são um sandbox de memória ou CPU para todo o processo; importações de mídia têm um limite anterior e separado por recurso.

<!-- section: concurrent-work -->
## Trabalho concorrente

Gerações em Rust tomam emprestado o estado imutável do projeto. Modelos e mídias que mantêm seus próprios dados podem ser compartilhados entre projetos. O binding de Node captura um snapshot do projeto quando build ou compare é chamado; adições posteriores não alteram essa solicitação. Mantenha o proprietário do artefato vivo até que todos os leitores terminem; fechar o último proprietário temporário enquanto outro consumidor ainda precisa do caminho é um erro de gerenciamento de vida útil da aplicação.

A inspeção do pacote não comprova todos os comportamentos do cliente. Teste a renderização, a reprodução e as importações de atualização nas versões e configurações de Anki pretendidas. Veja [atualizações](/docs/updates/) para os limites relacionados ao agendamento e às mudanças de schema.

<!-- section: prepared-publication-from-source -->
## Publicação preparada

A publicação preparada está incluída nos pacotes públicos `0.3.1`.

`prepare_publication` gera, inspeciona e compara um candidato privado; `publish` usa esse mesmo arquivo. As opções e os destinos relativos são fixados na preparação. Edições no projeto não alteram o candidato. A preparação, sozinha, não substitui o destino; liberar ou fechar um proprietário ainda não publicado remove o candidato.

Toda tentativa de publicação consome o proprietário, inclusive falhas de política ou E/S. Prepare novamente para tentar outra vez ou mudar a política/destino. Relatórios permanecem legíveis e não mantêm arquivos. Node/Python oferecem `close()` idempotente e um erro de indisponibilidade com o motivo `closed` ou `consumed`; fechar após o início da publicação não a cancela. A duração não inclui o tempo de espera pela revisão. Uma falha tardia de durabilidade pode ocorrer após uma substituição bem-sucedida; confira os fatos reais da publicação.

Siga o [exemplo completo](/docs/updates/#build-once-review-then-publish-from-source) e as referências de API de cada linguagem para os erros tipados e o comportamento dos workers.
