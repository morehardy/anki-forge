# Crie baralhos do Anki com código

Canonical page: https://ankiforge.dev/pt-br/docs/
Language: pt-BR
Description: Escolha uma linguagem de programação, gere seu primeiro baralho e aprenda a adicionar mídia e templates e a atualizar o baralho.

O Anki Forge transforma notas, templates e arquivos de mídia em pacotes `.apkg` do Anki. Use a biblioteca quando seu material de estudo estiver em código ou dados estruturados e você precisar gerar e distribuir novas versões dos baralhos.

## Comece pela sua linguagem

Instale um pacote publicado e exporte seu primeiro baralho; [escolha por onde começar](https://ankiforge.dev/pt-br/docs/languages/).

| Linguagem | Primeiro resultado | Próximo passo |
| --- | --- | --- |
| Rust | [Execute o exemplo do primeiro baralho](https://ankiforge.dev/pt-br/docs/quickstart/) | [Integre a biblioteca à sua aplicação](https://ankiforge.dev/pt-br/docs/installation/) |
| Node / TypeScript | [Instale o pacote e crie seu primeiro baralho](https://ankiforge.dev/pt-br/docs/node-quickstart/) | [API e comportamento assíncrono](https://ankiforge.dev/pt-br/docs/node-api/) |
| Python | [Instale o pacote e exporte um baralho](https://ankiforge.dev/pt-br/docs/python-quickstart/) | [API e controle dos arquivos de saída](https://ankiforge.dev/pt-br/docs/python-api/) |

Prefere ver o resultado primeiro? [Explore os exemplos de cartões](https://ankiforge.dev/pt-br/examples/) e baixe os pacotes gerados.

## Entenda os componentes

Uma nota armazena campos; um tipo de nota e seus templates de cartão transformam esses campos em cartões. Os baralhos agrupam os cartões resultantes no Anki. Os [conceitos fundamentais](https://ankiforge.dev/pt-br/docs/concepts/) explicam essas relações, como `Project` mantém a propriedade dos seus valores e como preservar a identidade das notas.

## Adicione o próximo recurso

-   [CSV para Anki](https://ankiforge.dev/pt-br/docs/csv-to-anki/): transforme uma planilha de vocabulário em um APKG com Python.
-   [JSON para Anki](https://ankiforge.dev/pt-br/docs/json-to-anki/): gere cartões com mídia e revise uma atualização em qualquer uma das três linguagens.
-   [Criação de cartões com IA](https://ankiforge.dev/pt-br/docs/ai-anki-cards/): faça rascunhos a partir das suas anotações, confira as respostas e exporte um APKG.
-   [Basic e Cloze](https://ankiforge.dev/pt-br/docs/cards/): crie cartões de pergunta e resposta ou de preenchimento de lacunas.
-   [Imagens e áudio](https://ankiforge.dev/pt-br/docs/media/): registre arquivos reais e inclua-os nos cartões.
-   [Tipos de nota personalizados](https://ankiforge.dev/pt-br/docs/custom-notetypes/) e [pacotes de templates](https://ankiforge.dev/pt-br/docs/templates/): defina campos, layouts e CSS.
-   [Oclusão de imagem](https://ankiforge.dev/pt-br/docs/image-occlusion/): crie perguntas a partir de regiões de uma imagem.
-   [Atualização e distribuição](https://ankiforge.dev/pt-br/docs/updates/): preserve as evidências de identidade e revisão entre versões.

Os guias de tipos de cartão usam exemplos executáveis em Rust; o tutorial de CSV usa Python e o de JSON abrange as três linguagens. As páginas das APIs de Node e Python explicam as interfaces correspondentes e suas diferenças.

## Encontre respostas e contribua

Consulte a [solução de problemas](https://ankiforge.dev/pt-br/docs/troubleshooting/) quando uma operação falhar e os [ambientes verificados](https://ankiforge.dev/pt-br/docs/compatibility/) para conhecer os requisitos de execução e as condições de importação nos clientes.

O Anki Forge usa a licença [MIT](https://github.com/morehardy/anki-forge/blob/main/LICENSE). Relate problemas reproduzíveis no [GitHub](https://github.com/morehardy/anki-forge/issues) ou leia o [guia de contribuição](https://github.com/morehardy/anki-forge/blob/main/docs/development.md).
