# Especificação Técnica: Prevenção de Duplo Encapsulamento e Rastreabilidade de Nó

- **Ticket**: #23
- **Escopo**: Prevenção de duplo envelopamento de papel de carta e rastreamento de nós em topologias distribuídas via cabeçalho `X-Papeli-Wrapped-By`.

## 1. Contexto e Motivação
Quando o `papeli` atua como filtro Milter no Postfix, e-mails reencaminhados internamente ou que trafegam em loops de reescrita local podem sofrer duplo encapsulamento (papel de carta dentro de papel de carta). Além disso, em clusters de múltiplos nós Postfix/Milter, é necessário saber qual instância realizou o envelopamento original e prevenir que nós subsequentes reinjetem um segundo template.

## 2. Requisitos Técnicos

1. **Configuração do Nó (`src/config.rs`)**:
   - `ServerConfig.node_id`: Identificador único da instância (ex.: `"postfix-node-01"`). Caso omitido, deve adotar como fallback o hostname local da máquina ou `"papeli-node"`.
   - `ServerConfig.recognized_nodes`: Lista de identificadores de nós confiáveis do cluster (`Vec<String>`). Se vazia ou não informada, qualquer nó com cabeçalho `X-Papeli-Wrapped-By` presente é reconhecido por padrão.

2. **Injeção de Cabeçalho MIME (`src/mime.rs`)**:
   - Durante a reconstrução do MIME em `reconstruct()`, injetar:
     - `X-Papeli-Processed: true`
     - `X-Papeli-Wrapped-By: <node-id>`

3. **Prevenção de Loop e Duplo Encapsulamento (`src/milter.rs`)**:
   - Antes de iniciar o processamento/envelopamento, inspecionar os cabeçalhos recebidos.
   - Interromper imediatamente o processamento (retornando `None` para bypass transparente) se:
     1. O cabeçalho `X-Papeli-Processed` já estiver presente; OU
     2. Qualquer cabeçalho `X-Papeli-Wrapped-By` for encontrado e corresponder a um nó reconhecido do cluster (ou, se a lista estiver vazia, qualquer nó emissor).
