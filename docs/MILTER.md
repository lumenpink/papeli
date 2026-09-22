# 🔌 Daemon e Protocolo Milter do Papéli

O módulo `src/milter.rs` implementa o servidor do protocolo **Sendmail Milter** nativo do Postfix em Rust puro, sem dependências de bibliotecas C (`libmilter`).

---

## 🏛️ Diagrama da Sessão Milter

```text
[Postfix MTA]                                        [Papéli Daemon]
     │                                                      │
     │── SMFIC_OPTNEG (Protocolo v6, ações solicitadas) ───>│
     │<── SMFIC_OPTNEG (SMFIF_CHGBODY, SMFIF_CHGHDRS) ──────│
     │                                                      │
     │── SMFIC_CONNECT / HELO / MAIL / RCPT ───────────────>│
     │<── SMFIS_CONTINUE ───────────────────────────────────│
     │                                                      │
     │── SMFIC_HEADER / SMFIC_EOH ─────────────────────────>│
     │<── SMFIS_CONTINUE ───────────────────────────────────│
     │                                                      │
     │── SMFIC_BODY (Chunks do e-mail) ────────────────────>│
     │<── SMFIS_CONTINUE ───────────────────────────────────│
     │                                                      │
     │── SMFIC_BODYEOB (Fim da mensagem) ──────────────────>│
     │                                                      │ (Executa EmailProcessor)
     │                                                      │  - Bypass se domínio não for gerenciado
     │                                                      │  - Sanitização + MJML + Assinaturas
     │                                                      │
     │<── SMFIR_REPLBODY (Chunks do novo corpo envelopado) ─│
     │<── SMFIS_CONTINUE (Libera entrega para o Postfix) ───│
     │                                                      │
     │── SMFIC_QUIT ───────────────────────────────────────>│
```

---

## 🚀 Suporte Duplo a Sockets (UNIX & TCP)

O Papéli suporta ambos os modos de escuta configuráveis via `papeli.toml`:

### 1. UNIX Domain Socket (`unix:/run/papeli/papeli.sock`)
* Máxima performance através de IPC local de baixa latência (sem overhead de pilha TCP/IP do kernel).
* Controle de acesso através de permissões POSIX (`chmod 660` e grupo `postfix`).

### 2. TCP Socket (`inet:127.0.0.1:8899` ou `inet:[::]:8899`)
* Ideal para arquiteturas distribuídas e orquestradas (Kubernetes / Docker Compose).
* Permite que o container do Postfix se comunique com o container do Papéli através de rede virtual interna.

---

## 🛡️ Política de Bypass e Segurança
* **E-mails de fora:** Quando um e-mail é recebido de um remetente não listado nos domínios do `papeli.toml`, o `EmailProcessor` retorna imediatamente `None`. O Milter responde com `SMFIS_CONTINUE` sem injetar papel de carta nem alterar um único byte.
* **Tamanho de Chunks:** Em conformidade com a especificação RFC Milter, chunks de substituição de corpo (`SMFIR_REPLBODY`) são divididos em blocos seguros de até 65.000 bytes.
