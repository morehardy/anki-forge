# Solução de problemas

Use os valores estruturados `kind` e `code` da operação que falhou, junto com a cadeia de causas e as observações. As mensagens de texto destinam-se a pessoas e podem mudar. Uma operação que falha e uma comparação concluída cuja política bloqueia a publicação são resultados diferentes.

<!-- section: installation -->
## Instalação

| Sintoma | O que verificar |
| --- | --- |
| Rust rejeita a toolchain | Execute `rustc --version`; use Rust 1.92+ e consulte a versão instalada com `cargo tree -i ankiforge` |
| Node não consegue carregar o pacote nativo | Execute `node -p "process.platform + '/' + process.arch"` e `npm ls ankiforge`; reinstale com `npm install --include=optional ankiforge@0.3.1` |
| Node informa divergência entre o código nativo e o wrapper | Use versões correspondentes dos pacotes; reinstale a mesma versão do registro ou recompile ambos a partir da mesma cópia do repositório |
| Python não consegue carregar a extensão | Execute `python -m pip show ankiforge` e `python -m pip debug --verbose`; confira o interpretador e as tags de sistema operacional e CPU do wheel |
| Python seleciona uma distribuição de código-fonte | Escolha um wheel e um interpretador compatíveis ou siga a [compilação a partir do código-fonte](/docs/development/) com um compilador |

Consulte os [ambientes verificados](/docs/compatibility/), a [instalação de Node](/docs/node-quickstart/) e a [instalação de Python](/docs/python-quickstart/).

<!-- section: models-and-notes -->
## Modelos e notas

| Sintoma | Correção |
| --- | --- |
| Chave inválida ou vazia | Forneça uma chave estável explícita; os nomes de exibição são separados |
| `NOTE.KEY_DUPLICATE` | Atribua uma chave única a cada registro lógico de origem |
| `NOTE.FIELD_UNKNOWN` | Use a chave do campo, não seu nome de exibição |
| `NOTE.FIELD_REQUIRED` | Preencha os campos declarados com `required()` antes de adicionar a nota |
| `NOTE.MODEL_CONFLICT` | Reutilize o mesmo modelo finalizado para uma mesma chave de modelo |
| Erro de referência no modelo de cartão | Use chaves declaradas nas expressões; examine o intervalo de bytes indicado no código-fonte original |
| HTML aparece como texto literal | Use `Content::html` somente para marcação intencional |
| Cartões Cloze ausentes | Use a sintaxe `{{c1::answer}}` com os números permitidos, de 1 a 500 |

Uma falha na adição é atômica. Corrija a nota ou o modelo e tente novamente; não é preciso desfazer um registro parcial de modelos ou mídia. Os construtores de modelos precisam terminar com `.build()` antes de criar notas personalizadas.

<!-- section: media-and-templates -->
## Mídia e modelos

| Sintoma | Correção |
| --- | --- |
| Falha ao importar um arquivo | Confira a origem no momento de `Media::file`; a remoção posterior do arquivo de origem é permitida |
| Tipo MIME não corresponde ao conteúdo | Informe o tipo MIME real ao construtor que recebe bytes |
| Conflito de nomes de exportação | Escolha nomes únicos e portáveis; variantes de maiúsculas/minúsculas e de normalização Unicode entram em conflito |
| Recurso de HTML, CSS ou script ausente | Declare-o com `asset` no modelo ou `add_asset` no projeto |
| Importação de mídia excede um limite | Aumente explicitamente `MediaLimits.max_bytes` antes de importar |
| Inspeção da geração excede um limite | Ajuste o contador correspondente de `InspectLimits` para a operação |
| Falha ao carregar um pacote de modelos | Use bundle-v2, chaves explícitas, modelos baseados em chaves e caminhos internos ao pacote |
| Falha em um retângulo de oclusão | Confira as dimensões de exibição decodificadas, coordenadas finitas e chaves únicas para as máscaras |

Os padrões de tratamento de texto são uniformes em todos os tipos de nota. Nós de mídia coletam suas dependências; escrever um nome de arquivo manualmente não importa seus bytes. Mesmo um pacote válido exige testes de reprodução no cliente para o codec ou a fonte que você pretende usar.

<!-- section: updates-and-outputs -->
## Atualizações e arquivos de saída

Use como base um APKG original distribuído com evidências completas de identidade. Pacotes reexportados pelo Anki não preservam essas evidências. Uma divergência de namespace ou evidências corrompidas não podem ser aceitas por uma política de atualização.

Uma comparação concluída pode retornar `allows_publication == false`. Examine os resultados originais e as evidências antes de permitir explicitamente uma categoria de risco. Não permita todos os resultados indiscriminadamente. A mesma política deve ser passada à geração da atualização se você quiser a mesma decisão. Informar uma política em uma operação que apenas cria um pacote é um erro.

Se um caminho temporário desaparecer, mantenha vivo o objeto de saída ou artefato que controla esse arquivo até que todos os leitores terminem. Relatórios e representações JSON não mantêm a propriedade do arquivo. Em falhas de persistência, examine a etapa de publicação e a durabilidade informadas pelo erro: um erro pode ser retornado depois de o destino já ter sido publicado. Consulte as [garantias dos arquivos de saída](/docs/build-guarantees/).
