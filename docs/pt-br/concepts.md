# Conceitos fundamentais

<!-- section: project-and-note -->
## Projeto e nota

Um `Project` representa uma publicação com um namespace estável. Ele pode distribuir notas entre vários baralhos do Anki: `default_deck` define o destino padrão e `Note::deck` permite escolher outro para uma nota. Um baralho é um destino, não um segundo contêiner para criar conteúdo, gerenciar mídia ou gerar pacotes.

Uma `Note` contém seu modelo e seu conteúdo. `project.add(key, note)` atribui uma chave estável de origem obrigatória e incorpora as dependências de forma atômica. Chaves duplicadas, campos desconhecidos e modelos ou mídias em conflito causam uma falha sem adicionar estado parcial. O namespace do projeto e as chaves dos registros devem permanecer iguais após edições e reordenações da fonte. Nomes de exibição e textos não são chaves de identidade.

<!-- section: models-fields-and-templates -->
## Modelos, campos e templates

`NoteType::builder(key)` reúne as declarações; `.build()` valida e retorna um modelo imutável que pode ser compartilhado. `model.note()` cria uma nota que mantém esse modelo. As notas integradas Basic, Cloze e Image Occlusion incluem seus próprios modelos.

Campos e templates têm chaves estáveis e nomes de exibição opcionais. Escreva os templates usando as chaves dos campos, como `{{front}}`, mesmo quando o nome exibido no Anki for `正面`. A finalização valida e compila as referências, incluindo seções, filtros, Cloze e templates do navegador. As atribuições também usam chaves; não há tentativa de adivinhar a chave nem substituição pelo nome de exibição.

<!-- section: text-and-html -->
## Texto e HTML

Strings comuns sempre representam Text, tanto em notas Basic e Cloze quanto em notas personalizadas. Text escapa HTML; `Content::html` inclui marcação de forma intencional. A sintaxe Cloze, como `{{c1::answer}}`, é preservada em Text sem tornar confiável o HTML nela contido. `Content::sequence` combina texto, HTML, imagens e sons em um único campo.

O conteúdo tipado de imagem e som mantém as dependências de mídia até a exportação. Não converta a mídia em strings antes de adicioná-la, a menos que também declare o recurso explicitamente para seu HTML, CSS ou script escrito à mão.

<!-- section: media-ownership -->
## Propriedade da mídia

`Media` mantém um snapshot dos dados sob sua própria responsabilidade, com um nome de arquivo determinístico derivado do conteúdo. A importação só termina com sucesso depois de ler e validar a entrada dentro do limite estabelecido. Depois disso, o arquivo de origem pode mudar ou desaparecer. Clones compartilham o armazenamento e podem ser usados em vários projetos. `with_export_name` escolhe um nome de arquivo fixo e portátil sem alterar clones ou nós de conteúdo já existentes.

Referências tipadas incorporam a mídia automaticamente. Recursos declarados explicitamente em um modelo ou projeto são incluídos mesmo quando a análise estática não identifica seu uso. A biblioteca não descobre o conteúdo dos arquivos a partir de HTML ou scripts arbitrários.

<!-- section: build-and-update -->
## Geração e atualização

Uma geração bem-sucedida retorna uma saída com um APKG garantido. Um relatório contém observações; um snapshot é um conjunto de dados JSON e não mantém arquivos sob sua propriedade. Artefatos temporários permanecem disponíveis somente enquanto existir pelo menos um objeto responsável por mantê-los vivos.

Uma atualização usa as evidências completas incorporadas no APKG original distribuído anteriormente. A comparação independente separa os resultados da análise da política de publicação. Resultados de risco alto bloqueiam atualizações por padrão; erros impeditivos não podem ser autorizados. Configurações de importação do Anki, edições locais e agendamento de cartões ainda exigem validação no cliente. Veja [atualizações](/docs/updates/) e [garantias de geração](/docs/build-guarantees/).
