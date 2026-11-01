O Anki Forge transforma notas, modelos e arquivos de mídia em pacotes `.apkg` do Anki. Use a biblioteca quando seu material de estudo estiver em código ou dados estruturados e você precisar gerar e distribuir novas versões dos baralhos.

<!-- section: start-with-your-language -->
## Comece pela sua linguagem

Instale um pacote publicado e exporte seu primeiro baralho; [escolha por onde começar](/docs/languages/).

| Linguagem | Primeiro resultado | Próximo passo |
| --- | --- | --- |
| Rust | [Execute o exemplo do primeiro baralho](/docs/quickstart/) | [Integre a biblioteca à sua aplicação](/docs/installation/) |
| Node / TypeScript | [Instale o pacote e crie seu primeiro baralho](/docs/node-quickstart/) | [API e comportamento assíncrono](/docs/node-api/) |
| Python | [Instale o pacote e exporte um baralho](/docs/python-quickstart/) | [API e controle dos arquivos de saída](/docs/python-api/) |

Prefere ver o resultado primeiro? [Explore os exemplos de cartões](/examples/) e baixe os pacotes gerados.

<!-- section: understand-the-pieces -->
## Entenda os componentes

Uma nota armazena campos; um tipo de nota e seus modelos de cartão transformam esses campos em cartões. Os baralhos agrupam os cartões resultantes no Anki. Os [conceitos fundamentais](/docs/concepts/) explicam essas relações, como `Project` mantém a propriedade dos seus valores e como preservar a identidade das notas.

<!-- section: build-your-next-feature -->
## Adicione o próximo recurso

- [CSV para Anki](/docs/csv-to-anki/): transforme uma planilha de vocabulário em um APKG com Python.
- [JSON para Anki](/docs/json-to-anki/): gere cartões com mídia e revise uma atualização em qualquer uma das três linguagens.
- [Criação de cartões com IA](/docs/ai-anki-cards/): faça rascunhos a partir das suas anotações, confira as respostas e exporte um APKG.
- [Basic e Cloze](/docs/cards/): crie cartões de pergunta e resposta ou de preenchimento de lacunas.
- [Imagens e áudio](/docs/media/): registre arquivos reais e inclua-os nos cartões.
- [Tipos de nota personalizados](/docs/custom-notetypes/) e [pacotes de modelos](/docs/templates/): defina campos, layouts e CSS.
- [Oclusão de imagem](/docs/image-occlusion/): crie perguntas a partir de regiões de uma imagem.
- [Atualização e distribuição](/docs/updates/): preserve as evidências de identidade e revisão entre versões.

Os guias de tipos de cartão usam exemplos executáveis em Rust; o tutorial de CSV usa Python e o de JSON abrange as três linguagens. As páginas das APIs de Node e Python explicam as interfaces correspondentes e suas diferenças.

<!-- section: find-answers-and-contribute -->
## Encontre respostas e contribua

Consulte a [solução de problemas](/docs/troubleshooting/) quando uma operação falhar e os [ambientes verificados](/docs/compatibility/) para conhecer os requisitos de execução e as condições de importação nos clientes.

O Anki Forge usa a licença [MIT](https://github.com/morehardy/anki-forge/blob/main/LICENSE). Relate problemas reproduzíveis no [GitHub](https://github.com/morehardy/anki-forge/issues) ou leia o [guia de contribuição](https://github.com/morehardy/anki-forge/blob/main/docs/development.md).
