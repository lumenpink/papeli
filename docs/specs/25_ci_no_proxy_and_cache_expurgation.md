# Especificação Técnica: Injeção de no_proxy e Expurgamento de actions/cache na CI

- **Ticket**: #25
- **Escopo**: Adequação do workflow de automação `.forgejo/workflows/ci.yml` para operar de forma hermética sob proxy do runner, prevenindo travamento do `sccache` em localhost e eliminando chamadas desnecessárias a `actions/cache`.

## 1. Contexto e Motivação
A esteira de integração contínua do Papéli falhou na execução #8 (Job `Test & Build (Rust)`) com `exitcode 137` durante a compilação inicial (`Compiling unicode-ident`).
A causa-raiz identificada decorre de duas falhas de configuração no ambiente:
1. O daemon do Forgejo Runner injeta globalmente a variável `http_proxy=http://172.17.0.1:3142`. As imagens GOST utilizam `RUSTC_WRAPPER=/usr/local/bin/sccache`, que executa um daemon local em `127.0.0.1`. Sem as variáveis `no_proxy` e `NO_PROXY` contendo `localhost,127.0.0.1,::1`, as requisições de compilação para o daemon são desviadas para o proxy do apt, congelando o processo até que ocorra encerramento forçado por limite de tempo/memória. Adicionalmente, testes com sockets TCP locais (ex.: `milter_test.rs`) são suscetíveis à mesma interceptação.
2. O step `Cache Cargo` (`actions/cache@v4`) tenta contactar serviços de cache remoto via proxy HTTP (`ECONNREFUSED 172.17.0.1:3142`). Como o runner já fornece cache persistente de crates e objetos compilados via bind-mount direto de `/media/hd1/forgejo-cache/cargo/registry` e `/media/hd1/forgejo-cache/sccache`, a ação de cache é supérflua, lenta e causadora de anomalias no fechamento dos containers.

## 2. Requisitos Técnicos
1. **Configuração de Variáveis `no_proxy` / `NO_PROXY`**:
   - Injetar no bloco `env:` global de `.forgejo/workflows/ci.yml` as variáveis:
     - `no_proxy: "localhost,127.0.0.1,::1"`
     - `NO_PROXY: "localhost,127.0.0.1,::1"`
   - Garantir que todos os jobs (`test`, `deploy-site`, `deploy-banca`) herdem essa configuração.

2. **Expurgamento de `actions/cache`**:
   - Remover integralmente o passo `Cache Cargo` (`uses: https://github.com/actions/cache@v4`) do job `test`.
   - Garantir que nenhum resquício de `actions/cache` permaneça no arquivo de workflow.

3. **Padronização da Imagem Base de Teste**:
   - Atualizar a imagem do container do job `test` de `gost-ci-rust:latest` para `gambi.2lp.in/lumen/gost-ci-cross:latest`, alinhando-se aos padrões canônicos dos demais serviços do ecossistema GOST.

4. **Verificação Automatizada e Testes de Mutação**:
   - Criar script de teste `tests/test_ci_no_proxy_and_cache.sh` verificando:
     - Existência de `no_proxy` e `NO_PROXY` no workflow.
     - Ausência de `actions/cache`.
     - Presença da imagem `gost-ci-cross:latest` no job de testes.
     - Testes de mutação verificando a sensibilidade a alterações nessas diretivas.
