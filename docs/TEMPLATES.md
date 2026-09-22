# 🎨 Motor de Templates MJML e Texto do Papéli

O módulo `src/templates.rs` implementa o motor de renderização do **Papéli**, combinando a expressividade da engine **MiniJinja** com a compilação nativa de alta performance do **MRML** (Rust MJML).

---

## 🏛️ Pipeline de Renderização

```text
[Contexto de Variáveis] (logo, sender, role, company, content_html, signature_html)
                     │
                     ▼
           1. MiniJinja (Rust)
              └─ Interpola variáveis dinâmicas no template MJML
                     │
                     ▼
           2. MRML Compiler (Rust)
              └─ Compila MJML para HTML responsivo (cross-client tables)
                     │
                     ▼
[HTML Final Responsivo e Compatível com Outlook, Gmail, Apple Mail, Webmail]
```

---

## 📑 Variáveis Disponíveis no Contexto

| Variável | Tipo | Descrição |
| :--- | :--- | :--- |
| `content_html` | `String` | O miolo limpo e sanitizado do e-mail do usuário. |
| `content_text` | `String` | Versão em texto limpo do e-mail. |
| `signature_html` | `String` | Assinatura corporativa ativa formatada com marcadores semânticos. |
| `signature_text` | `String` | Assinatura em texto delimitada por marcadores ASCII. |
| `logo_url` | `Option<String>` | URL pública do logotipo corporativo. |
| `company_name` | `Option<String>` | Razão social ou nome fantasia da organização. |
| `subject` | `Option<String>` | Assunto da mensagem. |
| `sender_name` | `Option<String>` | Nome de exibição do remetente. |
| `sender_role` | `Option<String>` | Cargo do remetente. |
| `sender_phone` | `Option<String>` | Telefone / ramal. |
| `disclaimer` | `Option<String>` | Aviso legal de confidencialidade. |

---

## 📐 Template MJML Padrão (`DEFAULT_MJML_TEMPLATE`)

O template padrão foi projetado para máxima legibilidade e elegância corporativa:
* Container centralizado com fundo suave (`#f4f6f8`).
* Cartão branco com cantos arredondados (`#ffffff`, border-radius 6px).
* Espaço de cabeçalho para logotipo da empresa.
* Divisor sutil (`#e2e8f0`).
* Área dedicada para o rodapé e assinatura.

---

## ⚡ Performance
* **MRML vs Node.js:** Enquanto o compilador MJML padrão em Node.js consome mais de 100 MB de RAM e dezenas de milissegundos por e-mail, o motor do Papéli em Rust compila em **menos de 1 milissegundo** com consumo virtualmente imperceptível de memória (< 5 MB).
