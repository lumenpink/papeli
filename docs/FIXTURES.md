# 🧪 Catálogo de Fixtures de Teste do Papéli

Este documento cataloga a suíte de fixtures de e-mails em formato bruto RFC 5322 (`.eml`), localizada em `tests/fixtures/`. 
Cada fixture foi elaborada para espelhar comportamentos reais, idiossincrasias e estruturas MIME geradas pelos principais clientes de e-mail do mercado corporativo.

---

## 📋 Matriz de Fixtures

| Arquivo | Cliente / Origem | Estrutura MIME | Desafios & Casos de Borda Testados |
| :--- | :--- | :--- | :--- |
| [`outlook_desktop_mso.eml`](file:///home/lumen/src/papeli/tests/fixtures/outlook_desktop_mso.eml) | Microsoft Outlook 16 (Desktop) | `multipart/alternative` | Tags MSO (`<o:p>`), namespaces XML (`xmlns:v`, `xmlns:o`), classes WordSection, estilos embutidos e tags condicionais `<!--[if !mso]>`. |
| [`gmail_web_thread.eml`](file:///home/lumen/src/papeli/tests/fixtures/gmail_web_thread.eml) | Gmail Web / Google Workspace | `multipart/alternative` | Cabeçalhos `In-Reply-To` e `References`, bloco `gmail_quote`, bloco `gmail_signature` e histórico aninhado. |
| [`apple_mail_ios.eml`](file:///home/lumen/src/papeli/tests/fixtures/apple_mail_ios.eml) | Apple Mail (iPhone / iOS) | `multipart/alternative` | Formatação `dir="auto"`, espaçamentos com divs e assinatura padrão *"Enviado do meu iPhone"*. |
| [`thunderbird_mixed.eml`](file:///home/lumen/src/papeli/tests/fixtures/thunderbird_mixed.eml) | Mozilla Thunderbird | `multipart/mixed` | Mensagem com anexo binário (`application/pdf`, base64). O filtro deve envelopar o corpo mantendo o anexo 100% íntegro. |
| [`inline_images_cid.eml`](file:///home/lumen/src/papeli/tests/fixtures/inline_images_cid.eml) | Web / Cliente Avançado | `multipart/related` | Imagem embutida com `Content-ID: <banner_topo@papeli.local>` e tag `<img src="cid:...">`. O MIME related e o CID não podem ser corrompidos. |
| [`raw_plain_text.eml`](file:///home/lumen/src/papeli/tests/fixtures/raw_plain_text.eml) | Mutt / Cliente Terminal | `text/plain` puro | E-mail sem partes HTML. O Papéli deve converter transparentemente para `multipart/alternative`, gerando versão HTML com papel de carta. |
| [`raw_html_only.eml`](file:///home/lumen/src/papeli/tests/fixtures/raw_html_only.eml) | Sistema de Notificação / RH | `text/html` puro | E-mail sem alternativa de texto puro. O Papéli deve envelopar o HTML e gerar o fallback em `text/plain`. |
| [`nested_thread_long.eml`](file:///home/lumen/src/papeli/tests/fixtures/nested_thread_long.eml) | Conversa Encaminhada (4 turnos) | `multipart/alternative` | Teste do **Mecanismo de Assinatura Dupla**: presença de 3 assinaturas longas antigas no histórico (`.papeli-sig-long`) que devem ser compactadas para `.papeli-sig-short`. |
| [`outlook_stationery_legacy.eml`](file:///home/lumen/src/papeli/tests/fixtures/outlook_stationery_legacy.eml) | Outlook com Papel Legado | `multipart/alternative` | Mensagem com textura de fundo (`background-image`), tabelas com bordas ridge e fontes obsoletas. Deve ser achatada e higienizada. |

---

## 🔬 Detalhamento Técnico das Fixtures

### 1. `outlook_desktop_mso.eml`
* **Origem:** Microsoft Outlook 16 (Desktop Windows).
* **Problema:** Clientes corporativos no Windows frequentemente usam o motor de renderização do MS Word para compor e-mails. Isso injeta centenas de linhas de regras de estilo como `mso-fareast-language`, `@page WordSection1`, e condicionais XML.
* **Validação:** O sanitizador deve arrancar todas as tags `<head>`, `<style>` e namespaces XML, mantendo apenas os parágrafos de texto reais no corpo.

### 2. `gmail_web_thread.eml`
* **Origem:** Google Chrome acessando o Gmail.
* **Problema:** O Gmail envolve mensagens anteriores dentro de `<div class="gmail_quote">` e assinaturas pessoais em `<div class="gmail_signature">`.
* **Validação:** Garantir que o miolo da nova mensagem seja isolado e envelopado pelo papel de carta, preservando a citação abaixo.

### 3. `apple_mail_ios.eml`
* **Origem:** iOS Mail no iPhone.
* **Problema:** Envio de texto móvel rápido que frequentemente contém o rodapé automático "Enviado do meu iPhone" em uma `<div>` própria.
* **Validação:** Capacidade de remover ou ignorar rodapés padrão de celulares antes de injetar a assinatura corporativa oficial.

### 4. `thunderbird_mixed.eml`
* **Origem:** Thunderbird no Linux/macOS enviando anexo em PDF.
* **Problema:** Em e-mails `multipart/mixed`, alterar o corpo de forma descuidada pode quebrar o boundary do MIME ou corromper o stream de bytes do PDF.
* **Validação:** O teste valida que após a passagem pelo filtro, o hash SHA-256 do arquivo `relatorio_q3_consolidado.pdf` permanece estritamente idêntico.

### 5. `inline_images_cid.eml`
* **Origem:** E-mail com imagem inline via protocolo `cid:`.
* **Problema:** `multipart/related` requer que o `Content-ID` do cabeçalho da imagem coincida exatamente com o atributo `src="cid:..."` do HTML.
* **Validação:** O parser de MIME e o compilador MJML devem manter essa vinculação intacta.

### 6. `raw_plain_text.eml`
* **Origem:** Clientes em modo texto ou scripts automatizados.
* **Problema:** Não há árvore MIME prévia; a mensagem é uma folha única `text/plain`.
* **Validação:** O filtro deve transformar o nó em `multipart/alternative`, mantendo o texto limpo na subparte de texto e gerando uma subparte HTML novinha em folha com o papel de carta MJML.

### 7. `raw_html_only.eml`
* **Origem:** Aplicações legadas geradoras de relatórios em HTML puro.
* **Problema:** Sem parte de texto, o que pode prejudicar a entregabilidade e pontuação em filtros de spam (SpamAssassin).
* **Validação:** Envelopar o HTML e gerar a contrapartida em texto plano.

### 8. `nested_thread_long.eml`
* **Origem:** Thread de discussão com 4 e-mails acumulados entre múltiplos colaboradores.
* **Problema:** Assinaturas completas antigas ocupando megabytes de dados inúteis a cada nova resposta enviada.
* **Validação:** Identificar deterministamente todas as instâncias passadas de `data-papeli="signature"` com classe `papeli-sig-long` e substituí-las por `papeli-sig-short`.

### 9. `outlook_stationery_legacy.eml`
* **Origem:** Outlook com templates legados de papel de carta dos anos 2000.
* **Problema:** Imagens de fundo tipo mármore/pergaminho e bordas extravagantes criadas com tabelas pesadas.
* **Validação:** Destruir os estilos externos e re-envelopar o conteúdo puro no design moderno e responsivo do Papéli.

---

## 🛠️ Como Executar os Testes de Fixtures

Para validar a integridade de todas as fixtures contra a suíte de testes:

```bash
cargo test --test fixtures_test
```
