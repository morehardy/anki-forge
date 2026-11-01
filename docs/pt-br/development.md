# Compile os SDKs a partir do código-fonte

Use este guia para desenvolver no repositório e trabalhar com alterações ainda não publicadas. Para os pacotes publicados `0.3.1`, use o guia de instalação de [Rust](/docs/installation/), [Node](/docs/node-quickstart/) ou [Python](/docs/python-quickstart/).

Requisitos: Git, Rust 1.92.0 com Cargo e Node.js 22.13+ com npm ou CPython 3.11/3.12 para o SDK correspondente. O repositório fixa a toolchain de Rust; instale-a com `rustup toolchain install 1.92.0` se necessário.

<!-- section: source-builds -->
## Compilação do código-fonte

Para compilar a versão atual do repositório:

```sh
git clone https://github.com/morehardy/anki-forge.git
cd anki-forge
```

Aplicações Rust podem usar `ankiforge = { path = "/absolute/path/to/anki-forge/anki_forge" }` no manifesto. Execute exemplos com `cargo run --locked -p ankiforge --example target_api_basic`.

Compile o addon de Node a partir deste repositório:

```sh
cd bindings/node
npm run setup
npm run build
npm run pack:local
```

Instale em uma aplicação separada os dois tarballs reais, da fachada e do binário nativo da plataforma, cujos caminhos são exibidos por `pack:local`, usando `npm install --offline --ignore-scripts --omit=optional /absolute/path/facade.tgz /absolute/path/native.tgz`. Essas são compilações do código-fonte, não instalações a partir do registro. A configuração de TypeScript está na [API de Node](/docs/node-api/#typescript).

Para Python, ative um ambiente virtual CPython 3.11/3.12, instale o Maturin e execute na raiz do repositório:

```sh
python -m pip install maturin==1.15.0
maturin develop --manifest-path bindings/python/native/Cargo.toml --locked
```

Wheels instalados normalmente não precisam de Cargo em tempo de execução. Não misture wrappers do código-fonte com binários nativos mais antigos do registro.
