# 🧪 Suíte de Testes Automatizados e E2E do Papéli

O **Papéli** adota uma filosofia rigorosa de testes automatizados com cobertura integral de todos os subsistemas, cobrindo parsing de MIME, sanitização de HTML tóxico, substituição determinística de assinaturas, compilação de MJML e transações do protocolo Milter sobre sockets de rede.

---

## 📊 Matriz de Cobertura de Testes (30 Testes Automatizados)

| Arquivo de Teste | Quantidade | Escopo & Verificações |
| :--- | :---: | :--- |
| [`tests/fixtures_test.rs`](file:///home/lumen/src/papeli/tests/fixtures_test.rs) | **9 testes** | Validação de integridade do catálogo de e-mails de múltiplos clientes (Outlook MSO, Gmail Web, Apple Mail iOS, Thunderbird, CIDs, texto puro, HTML puro, threads aninhadas, papel legado). |
| [`tests/mime_test.rs`](file:///home/lumen/src/papeli/tests/mime_test.rs) | **4 testes** | Reconstrução da árvore MIME sem corrupção; integridade bit-a-bit de anexos em PDF via hash **SHA-256**; preservação de `Content-ID` em imagens inline; promoção transparente de `text/plain` para `multipart/alternative`; preservação de cabeçalhos de threading (`Message-ID`, `In-Reply-To`, `References`). |
| [`tests/sanitizer_test.rs`](file:///home/lumen/src/papeli/tests/sanitizer_test.rs) | **4 testes** | Whitelist semântica Ammonia; remoção de cascas `<html>`, `<head>`, `<style>` e scripts maliciosos; eliminação de rodapés padrão de celulares (*"Enviado do meu iPhone"*); achatamento e remoção de estilos de papéis de carta legados. |
| [`tests/signatures_test.rs`](file:///home/lumen/src/papeli/tests/signatures_test.rs) | **2 testes** | Validação do mecanismo de dupla assinatura: detecção determinística de marcadores semânticos (`papeli-sig-long`), substituição por assinaturas curtas (`papeli-sig-short`) no histórico citado e cálculo da redução substancial de bytes. |
| [`tests/templates_test.rs`](file:///home/lumen/src/papeli/tests/templates_test.rs) | **3 testes** | Compilação de templates MJML responsivos via **MRML** nativo em Rust; interpolação dinâmica com **MiniJinja**; renderização de templates estruturados em texto plano (`.txt.j2`). |
| [`tests/config_test.rs`](file:///home/lumen/src/papeli/tests/config_test.rs) | **2 testes** | Resolução hierárquica em cascata (`senders` $\succ$ `domains` $\succ$ `default`); simulação de recarga a quente (*live hot-reload*) em tempo de execução sem derrubar o serviço. |
| [`tests/milter_test.rs`](file:///home/lumen/src/papeli/tests/milter_test.rs) | **3 testes** | Regras de bypass para remetentes externos não gerenciados; envelopamento de remetentes gerenciados; teste do protocolo de rede Milter via TCP (`SMFIC_OPTNEG`, `SMFIC_BODY`, `SMFIC_BODYEOB`, `SMFIR_REPLBODY`, `SMFIC_QUIT`). |
| [`tests/e2e_test.rs`](file:///home/lumen/src/papeli/tests/e2e_test.rs) | **3 testes** | **1.** Passagem de todas as 9 fixtures completas pelo pipeline ponta a ponta do `EmailProcessor`;<br>**2.** Sessão Milter real sobre **UNIX Domain Socket** (`UnixListener` + `UnixStream`);<br>**3.** Teste de estresse com 10 threads concorrentes processando e-mails simultaneamente sem corridas ou falhas. |

---

## 🏃 Como Executar a Bateria Completa

Para rodar todos os 30 testes unitários e de integração:

```bash
cargo test
```

Para rodar com medição detalhada e logs desativando a captura:

```bash
cargo test -- --nocapture
```
