# Integre o Anki Forge a uma aplicação Rust

Use Rust 1.92 ou posterior. Na sua aplicação Cargo existente, instale o crate publicado:

```sh
cargo add ankiforge@0.3.1
```

A exportação normal de APKG não exige que o Anki esteja instalado. Para um primeiro programa completo e as instruções de `cargo run`, siga o [início rápido com Rust](/docs/quickstart/).

<!-- section: cargo-dependencies -->
## Dependências do Cargo

O crate público `ankiforge` inclui Basic, Cloze, tipos de nota personalizados, mídia com propriedade independente, oclusão de imagem, comparação e publicação preparada. Nenhuma feature é habilitada por padrão; aplicações comuns não precisam dos recursos exclusivos do repositório.

O crate é uma biblioteca nativa de Rust. Instale o vinculador e as ferramentas de compilação da sua plataforma conforme os requisitos da toolchain Rust. Consulte os [ambientes verificados](/docs/compatibility/) ao escolher onde executar a aplicação. Rust 1.92 é a versão mínima; a presença de uma plataforma na lista não garante todas as combinações de alvo e ferramentas de terceiros.

Para ler JSON, adicione explicitamente as dependências de análise na aplicação:

```sh
cargo add serde --features derive
cargo add serde_json
```

O [tutorial de JSON para APKG](/docs/json-to-anki/#rust) inclui o código completo de leitura, mídia e atualização. O SDK não oferece um conversor genérico de JSON ou CSV.

<!-- section: choose-the-next-guide -->
## Escolha o próximo guia

- [Guia de desenvolvimento com Rust](/docs/rust-guide/) para estruturar projetos e tipos de nota
- [API de Rust](/docs/rust-api/) para métodos, relatórios e erros
- [Mídia](/docs/media/) para imagens e áudio com propriedade independente
- [Atualização e distribuição](/docs/updates/) para condições de base e publicação
- [Compilação a partir do código-fonte](/docs/development/) para trabalhar no repositório
