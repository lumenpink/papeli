# 🌲 Motor de Manipulação MIME do Papéli

O módulo `src/mime.rs` é o coração de desmontagem e remontagem da árvore de e-mails do **Papéli**. Ele opera de forma não-destrutiva, garantindo que cabeçalhos, anexos binários e imagens inline permaneçam intactos enquanto os corpos de texto e HTML são substituídos.

---

## 🏛️ Arquitetura e Ciclo de Vida

```text
[Bytes Brutos do Postfix (SMTP)]
                 │
                 ▼
      MessageParser (mail-parser)
                 │
                 ▼
         ParsedEmail (Struct)
      ├── from, to, cc, subject, message_id, in_reply_to, references
      ├── custom_headers (X-Mailer, User-Agent, etc.)
      ├── body_text (original) & body_html (original)
      └── attachments: Vec<EmailAttachment> (PDF, PNG, CID, etc.)
                 │
                 │ (Sanitização + MJML Rendering + Assinaturas)
                 ▼
      reconstruct(new_text, new_html)
                 │
                 ▼
      MessageBuilder (mail-builder)
                 │
                 ▼
[Novos Bytes MIME Prontos para Postfix replbody]
```

---

## 🛡️ Garantias e Casos de Borda

### 1. Integridade Absoluta de Anexos (Bit-a-Bit)
Em mensagens do tipo `multipart/mixed` contendo arquivos (como PDFs, planilhas, arquivos compactados), os dados brutos são extraídos e reinseridos no novo envelope sem re-codificação destrutiva. 
* Validado via teste automatizado [`tests/mime_test.rs`](file:///home/lumen/src/papeli/tests/mime_test.rs) com comparação de hash criptográfico **SHA-256** do payload antes e depois da reconstrução.

### 2. Preservação de Imagens Inline (`cid:`)
E-mails contendo imagens incorporadas (`multipart/related`) dependem do cabeçalho `Content-ID` (ex: `<banner_topo@papeli.local>`). O `ParsedEmail` identifica quando uma parte é inline ou possui `content_id`, e a reconstrói usando `.inline()`, mantendo o vínculo funcional com tags `<img src="cid:...">`.

### 3. Promoção Transparente de Texto Puro (`text/plain`)
Quando o cliente envia um e-mail simples sem formatação (`is_plain_only == true`), o Papéli:
1. Extrai o texto limpo.
2. Promove a estrutura raiz para `multipart/alternative`.
3. Oferece a versão `text/plain` formatada e a nova versão `text/html` no papel de carta MJML.

### 4. Preservação de Threading e Headers
Os cabeçalhos de rastreamento de conversas (`Message-ID`, `In-Reply-To`, `References`) e cabeçalhos customizados de clientes (`X-Mailer`, etc.) são preservados fielmente, garantindo que clientes de e-mail continuem agrupando a mensagem na conversa correta.
