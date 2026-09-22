# 🧼 Higienização e Isolamento de Conteúdo do Papéli

O módulo `src/sanitizer.rs` é responsável por extrair, limpar e isolar o conteúdo de e-mails recebidos antes que eles sejam envelopados no papel de carta MJML.

---

## 🎯 Por que a Higienização é Crítica?

Os clientes de e-mail injetam uma quantidade massiva de metadados proprietários, estilos conflitantes e cascas de layout que, se fossem inseridos diretamente em um template MJML:
1. Quebrariam a responsividade em dispositivos móveis.
2. Herdariam fontes indesejadas (como *Comic Sans* de papéis de carta legados ou fontes do Word).
3. Vazariam tags `<head>`, `<style>` e scripts no corpo da mensagem.
4. Conteriam rodapés repetitivos de dispositivos móveis (*"Enviado do meu iPhone"*).

---

## 🔬 Pipeline de Higienização

```text
[HTML Original do E-mail]
          │
          ▼
   1. Extrator de Corpo (scraper)
      └─ Se contiver <body>, extrai apenas o inner_html de <body>
          │
          ▼
   2. Whitelist Estrita Semântica (ammonia)
      ├─ Tags permitidas: p, br, b, strong, i, em, u, a, ul, ol, li,
      │                   blockquote, span, div, table, img, h1-h6, hr, pre, code.
      ├─ Atributos: href, src, alt, title, width, height, class, style, dir, data-papeli.
      ├─ Protocolos aceitos: http, https, mailto, cid, tel.
      └─ Remoção estrita: <script>, <style>, <iframe>, <form>, <head>, <meta>, <o:p>, VML.
          │
          ▼
   3. Extrator de Rodapés de Dispositivos Móveis
      └─ Remove tags/linhas contendo "Enviado do meu iPhone", "Sent from my Galaxy", etc.
          │
          ▼
[Conteúdo Semântico Puro Pronto para o MJML <mj-text> ou <mj-raw>]
```

---

## 📱 Rodapés Móveis Suportados

Tanto no HTML quanto no texto plano (`text/plain`), o sanitizador limpa automaticamente rodapés padrões como:
* `Enviado do meu iPhone` / `Sent from my iPhone`
* `Enviado do meu iPad` / `Sent from my iPad`
* `Enviado do meu smartphone Samsung Galaxy` / `Sent from my Galaxy`
* `Enviado do Outlook para iOS` / `Get Outlook for iOS`
* `Enviado do Outlook para Android` / `Get Outlook for Android`

---

## 🔒 Garantias de Segurança
* **Proteção contra XSS:** Scripts e atributos com `javascript:` são completamente eliminados pelo parser do Ammonia.
* **Integridade CID:** O esquema de URL `cid:` é explicitamente preservado para garantir que imagens inline de clientes continuem visíveis.
