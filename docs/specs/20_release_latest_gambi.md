# Especificação Técnica — Publicação Automática de Release linux-x86_64 com Tag latest na Gambi (#20)

- **ID do Ticket:** #20
- **Status:** Aprovado
- **Autora:** Lumen
- **Alinhamento:** GOST 1.5.0

## 1. Contexto & Motivação
O daemon milter `papeli` envelopa mensagens em papel de carta responsivo e assina e-mails corporativos. Ele roda no ecossistema central de correio e é referenciado em esteiras irmãs como o Lauda.
O objetivo é automatizar a publicação de binários pré-compilados para a arquitetura `linux-x86_64` na release da Gambi (`gambi.2lp.in`) sob a tag canônica `latest`, saneando dependências e jobs órfãos.

## 2. Requisitos Técnicos
1. **Gatilho:**
   - Push na branch `main` e tags `v*`.
2. **Dependência:**
   - O job `release` depende da aprovação prévia do job `test` (`needs: [test]`).
3. **Compilação do Binário:**
   - `cargo build --release --bin papeli`.
4. **Publicação da Release:**
   - Utilização de `softprops/action-gh-release@v2`.
   - `tag_name: "latest"`.
   - Anexo único do artefato `target/release/papeli`.
5. **Saneamento:**
   - Remoção de jobs legados/órfãos (`trigger-gost-ci`).

## 3. Critérios de Aceitação
- Teste automatizado `tests/test_ci_release_workflow.sh` validando a estrutura do workflow.
- Validação estrita via `cccp check --strict`.
