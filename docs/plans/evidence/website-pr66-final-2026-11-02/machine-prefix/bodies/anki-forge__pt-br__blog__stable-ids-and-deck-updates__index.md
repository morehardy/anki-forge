# Atualize o conteúdo da nota sem mudar seu ID

Canonical page: https://morehardy.github.io/anki-forge/pt-br/blog/stable-ids-and-deck-updates/
Language: pt-BR
Description: Como IDs estáveis e uma base de geração trabalham juntos na atualização de um baralho Anki gerado.
Published: 2026-09-22T00:00:00.000Z

Um baralho gerado raramente está pronto após a primeira exportação. Definições melhoram, exemplos ficam mais claros e erros são corrigidos. A próxima geração precisa saber quais notas continuam sendo as mesmas, agora com conteúdo melhor.

Considere esta pequena edição:

| Campo | Primeira versão | Próxima versão |
| --- | --- | --- |
| Frente | hola | hola |
| Verso | hello | hello; hi |
| ID estável | es:hola | es:hola |

O significado foi ampliado. A nota lógica continua sendo a mesma.

## Separe a identidade do conteúdo

Um ID estável explícito dá à nota uma chave durável. Mantenha esse ID ao editar a nota e escolha um novo ID ao criar outra nota lógica.

A identidade também tem um contexto de projeto. Mantenha o namespace do projeto consistente entre as versões do mesmo baralho. Um ID de nota, sozinho, é apenas uma parte de um fluxo de atualização repetível.

## Forneça uma base para a próxima geração

Um pacote anterior informa ao Anki Forge o que já foi exportado. Use-o ao gerar o projeto revisado:

```rust
let comparison = project.compare(CompareOptions::against(&previous))?;
ensure!(comparison.policy().allows_publication());
let built = project.build(BuildOptions::to(&next).update_from(&previous))?;
```

Neste trecho, `project` contém a nota revisada, `previous` aponta para a versão 1 e `next` é um caminho separado para a versão 2. O [exemplo completo](https://morehardy.github.io/anki-forge/generated/website_showcase.rs) inclui as duas definições do projeto e verifica que a identidade de uma nota foi preservada.

Mantenha o pacote de entrada e o de saída em caminhos diferentes. Sobrescrever a base remove as evidências necessárias para entender e verificar a versão.

## Guarde o pacote original distribuído

Todo pacote gerado contém evidências completas de identidade e revisão. Antes de reutilizar as identidades, a próxima geração valida essas evidências contra o banco de dados e a mídia reais. Preserve o pacote original que você distribuiu; uma reexportação do Anki não é uma base aceita.

O [guia de atualização](https://morehardy.github.io/anki-forge/pt-br/docs/updates/) descreve as atualizações estritas, as políticas de risco e as limitações de importação do cliente.

## Examine a geração e a importação

Verifique o relatório de geração e seus diagnósticos de segurança de atualização antes de distribuir um pacote. Se a geração não conseguir reconciliar a identidade de uma nota, investigue a origem e a base, em vez de tratar um arquivo de saída como prova de que a atualização está correta.

O Anki aplica então seu próprio comportamento de importação. As configurações de importação, o conteúdo das notas existentes, as edições locais e a versão do Anki podem influenciar o resultado. A identidade estável ajuda a encontrar a nota correspondente; não substitui essas escolhas nem promete que toda importação preserve todos os aspectos do estado local.

O [exemplo com duas versões](https://morehardy.github.io/anki-forge/pt-br/examples/#updates) fornece os dois pacotes para você experimentar uma atualização pequena e fácil de inspecionar antes de definir seu próprio processo de publicação.
