# 🖋️ Sistema de Assinatura Dupla do Papéli

O módulo `src/signatures.rs` implementa o mecanismo inteligente de **dupla assinatura** do Papéli, concebido para resolver o problema clássico de inchaço (*bloat*) em threads longas de e-mails corporativos.

---

## 🎯 O Problema do Inchaço de Assinaturas

Em ambientes empresariais, uma conversa com 5 a 10 respostas frequentemente acumula:
* Dezenas de logos corporativos repetidos.
* Rodapés jurídicos extensos (disclaimers de confidencialidade de 10 linhas repetidos a cada turno).
* Telefones, endereços e certificações ISO idênticas.

Isso multiplica o tamanho em bytes do e-mail por 10x e polui visualmente a leitura.

---

## 💡 A Solução Determinística do Papéli

Como o **Papéli** é o autor do papel de carta e das assinaturas, nós delimitamos as assinaturas com **marcadores semânticos de rastreamento**:

### 1. Formato no HTML
* **Assinatura Longa (Mensagem Ativa Recente):**
  ```html
  <div class="papeli-sig-long" data-papeli="signature" data-sender="carlos@acme.com">
    <strong>Carlos Silva</strong><br>
    Diretor de Tecnologia | ACME Corp<br>
    Tel: +55 11 99999-9999 | www.acme.com<br>
    <small>Aviso de confidencialidade: Esta mensagem contém dados confidenciais...</small>
  </div>
  ```

* **Assinatura Curta (Substituta no Histórico Citado):**
  ```html
  <div class="papeli-sig-short" data-papeli="signature">
    <p style="color: #666; font-size: 11px;">— Carlos Silva (ACME Corp)</p>
  </div>
  ```

### 2. Formato no Texto Puro (`text/plain`)
* **Assinatura Longa:**
  ```text
  --
  <!--papeli:sig:start:long-->
  Carlos Silva
  Diretor de Tecnologia | ACME Corp
  Tel: +55 11 99999-9999
  [Aviso legal...]
  <!--papeli:sig:end:long-->
  ```

* **Assinatura Curta:**
  ```text
  <!--papeli:sig:start:short-->
  — Carlos Silva (ACME Corp)
  <!--papeli:sig:end:short-->
  ```

---

## ⚙️ Algoritmo de Compactação
1. O filtro detecta a presença de citações (`<blockquote>`, `.gmail_quote`, etc.).
2. Ele varre o DOM localizando instâncias antigas de `.papeli-sig-long`.
3. Substitui cada bloco longo pela sua versão compacta equivalente (`.papeli-sig-short`).
4. Aplica a nova assinatura longa apenas no final da mensagem atual do remetente.

*Resultado:* Conversas limpas, economia de mais de 70% dos bytes em threads e leitura despoluída.
