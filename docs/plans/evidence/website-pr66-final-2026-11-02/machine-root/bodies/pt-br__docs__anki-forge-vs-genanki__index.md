# Anki Forge ou genanki: escolha para o seu fluxo

Canonical page: https://ankiforge.dev/pt-br/docs/anki-forge-vs-genanki/
Language: pt-BR
Description: Compare a criação de APKGs do Anki, os requisitos de execução, a revisão de atualizações e os custos de migração.

Considere o Anki Forge ao criar um fluxo repetível de publicação que precise de mídia com propriedade independente, diagnósticos estruturados, comparação e revisão de políticas antes de lançar atualizações. Mantenha um fluxo comprovado com genanki quando os scripts Python, modelos e recursos de mídia existentes já atenderem às suas necessidades e a migração trouxer mais custo do que benefício.

| Critério | Anki Forge | genanki |
| --- | --- | --- |
| Linguagem de desenvolvimento | Núcleo em Rust; interfaces Rust, Python e Node / TypeScript | Python |
| Recursos básicos em comum | Basic, Cloze, modelos personalizados, mídia e geração de APKG sem conexão | Basic, Cloze, modelos personalizados, mídia e geração de APKG sem conexão |
| Identidade | Namespace e chaves de nota explícitos; preservação do APKG original para atualizações | GUIDs configuráveis e IDs numéricos de modelos e baralhos; suporte a atualizações por reimportação |
| Antes da publicação | Resultados de comparação, política de risco e candidato preparado inspecionado | O script e os testes definem a revisão antes da publicação |
| Propriedade da mídia | Os valores tipados de imagem e áudio mantêm snapshots independentes dos arquivos de origem; clones compartilham o armazenamento | O script fornece os arquivos de mídia e suas referências ao pacote |
| Oclusão de imagem por código | Máscaras retangulares com chaves pela API pública | Avalie um modelo ou extensão adequado ao seu fluxo |
| Custo de execução | Dependências nativas; exige plataforma e interpretador compatíveis | Pode ser mais econômico manter a implantação Python existente |
| Custo de migração | Validação explícita da correspondência de identidades anteriores, dos modelos e das importações | Nenhuma migração para um fluxo genanki que já funciona |

## A mesma tarefa simples

Para transformar três perguntas em JSON em um baralho com uma imagem e um som, ambas as bibliotecas permitem que Python leia o JSON e crie as notas. Com genanki, escolha os IDs do modelo e do baralho, atribua os valores dos campos e as regras de GUID, adicione os arquivos de mídia ao pacote e grave o APKG. Com Anki Forge, escolha um namespace e chaves de negócio duráveis, componha as notas com mídia de propriedade independente, gere a versão 1 e, antes da publicação, compare e prepare a versão 2 usando o APKG original da versão 1 como base. O [tutorial completo de JSON](https://ankiforge.dev/pt-br/docs/json-to-anki/) inclui implementações em Rust, Python e TypeScript e seus comandos de execução.

## Baralhos genanki existentes

O [genanki permite GUIDs personalizados](https://github.com/kerrickstaley/genanki#note-guids), mídia e atualizações por reimportação. A identidade estável não é uma capacidade exclusiva do Anki Forge. A regra padrão de GUID pode depender dos campos; defina uma regra intencional quando esses campos puderem mudar.

O Anki Forge não recupera automaticamente IDs numéricos ou GUIDs arbitrários do genanki. Sem uma correspondência validada separadamente, considere a primeira distribuição com Anki Forge como o início de uma nova identidade. Nem esta comparação nem a geração bem-sucedida de um APKG prometem uma migração sem perdas ou a preservação de todo o histórico de revisão. Leia o [guia de migração](https://ankiforge.dev/pt-br/docs/genanki-migration/) antes de migrar um baralho existente.

## Medições históricas de desempenho

A comparação de 2026-10-08 mediu a API Project de Rust em relação ao genanki 0.13.1 em um Apple M1 Pro. Com 1.000 notas Basic, as cinco cargas de trabalho medidas em Rust levaram de 45,7% a 53,9% menos tempo para exportar; algumas cargas com mídia apresentaram picos maiores de uso de memória. Consulte o [método, os resultados brutos e as limitações](https://github.com/morehardy/anki-forge/blob/bef4aeb653fc875f216614e73d8a617be039b9cf/benchmarks/results/20261008-latest-commit-genanki/report.md). A medição não avaliou o desempenho atual dos ambientes Python ou Node e não oferece a mesma garantia de velocidade para esses SDKs ou outras cargas de trabalho.

Mantenedores do Anki Forge · conteúdo revisado em 2026-10-25 com base nas APIs públicas do Anki Forge 0.3.1, na [documentação do genanki](https://github.com/kerrickstaley/genanki) e na medição arquivada. Consulte a [compatibilidade do ambiente de execução](https://ankiforge.dev/pt-br/docs/compatibility/) e [escolha uma linguagem](https://ankiforge.dev/pt-br/docs/languages/) para começar uma nova tarefa.
