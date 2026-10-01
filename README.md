# ✉️ Papéli

> **Filtro Milter Postfix em Rust para envelopar e-mails corporativos em papel de carta MJML responsivo com gestão inteligente de assinaturas.**

[![Codeberg](https://img.shields.io/badge/Codeberg-lumenpink%2Fpapeli-blue?logo=codeberg)](https://codeberg.org/lumenpink/papeli)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-green.svg)](LICENSE)

---

## 🎯 Visão Geral e Propósito

Nas empresas, manter a identidade visual e assinaturas padronizadas em e-mails sempre foi um pesadelo: cada funcionário usa um cliente diferente (Outlook, Apple Mail no iPhone, Thunderbird, webmail), configurando assinaturas desatualizadas, desalinhadas ou com imagens quebradas.

O **Papéli** resolve esse problema centralizando o papel de carta e as assinaturas diretamente no servidor de e-mail (**Postfix**):
1. O usuário escreve seu e-mail normalmente em qualquer cliente de sua preferência.
2. Ao enviar via `submission` (portas 587/465), o **Papéli** intercepta a mensagem como um **Milter** (Mail Filter).
3. O e-mail é cirurgicamente higienizado (removendo tags proprietárias e lixo HTML de clientes).
4. O conteúdo limpo é envelopado em um **papel de carta responsivo pré-configurado** em **MJML** compilado nativamente em Rust.
5. Se houver histórico de respostas, assinaturas corporativas antigas são **automaticamente compactadas** para economizar preciosos bytes.
6. A mensagem reformulada é entregue ao Postfix — que em seguida aplica a assinatura **DKIM** (OpenDKIM/Rspamd) sem quebrar autenticação — e segue para o destinatário!

---

## 🏗️ Arquitetura do Sistema

```text
[Cliente de E-mail] (Thunderbird, iPhone, Outlook, Webmail)
        │
        │ SMTP Submission (Porta 587 / 465)
        ▼
   ┌─────────┐
   │ Postfix │
   └────┬────┘
        │
        │ Protocolo Milter (UNIX Socket ou TCP inet)
        ▼
 ┌─────────────────────────────────────────────────────────────┐
 │                      PAPÉLI MILTER                          │
 │                                                             │
 │  1. MAIL FROM / From:                                       │
 │     └─ Valida se remetente pertence a domínio gerenciado    │
 │  2. MIME Parser (mail-parser)                               │
 │     └─ Isola nós de texto/html sem tocar nos anexos/CIDs    │
 │  3. Sanitizer (ammonia / scraper)                           │
 │     └─ Elimina cascas <html>, estilos MSO e lixo de cliente │
 │  4. Signature Engine (DOM & ASCII markers)                  │
 │     ├─ Injeta Assinatura Longa na mensagem atual            │
 │     └─ Compacta assinaturas antigas no histórico de resposta │
 │  5. Template Engine (mrml + MiniJinja)                      │
 │     ├─ HTML: Renderiza MJML -> HTML responsivo final        │
 │     └─ TXT: Renderiza versão estruturada em texto (.txt.j2) │
 │  6. Milter Replbody & Chgheader                             │
 │     └─ Substitui corpo e atualiza Content-Type              │
 └──────────────────────────────┬──────────────────────────────┘
                                │ Mensagem envelopada & formatada
                                ▼
                       ┌─────────────────┐
                       │ OpenDKIM/Rspamd │ (Assina o e-mail pós-modificação)
                       └────────┬────────┘
                                │
                                ▼
                       [Internet / Destino]
```

---

## ✨ Principais Diferenciais

### 1. MJML Nativo em Rust (`mrml`)
Ao contrário de soluções baseadas em Node.js que consomem centenas de megabytes de RAM e adicionam latência no envio, o **Papéli** utiliza a crate `mrml` — uma re-implementação 100% nativa em Rust do compilador MJML. O consumo de memória do daemon permanece tipicamente abaixo de 20 MB, com renderização em microssegundos.

### 2. Sistema de Dupla Assinatura Inteligente
Threads de e-mail corporativo frequentemente se tornam ilegíveis pelo acúmulo de logos, rodapés jurídicos e disclaimers repetidos a cada resposta.
* **Assinatura Longa (Atual):** Inserida na mensagem nova ativa com marcador semântico (`data-papeli="signature"`).
* **Assinatura Curta (Histórico):** Detecção determinística de assinaturas longas anteriores dentro de citações/respostas (`<blockquote>`) e substituição por uma linha compacta (ex: `— Carlos Silva (ACME Corp)`).
* Disponível tanto para HTML quanto para texto puro (`text/plain`).

### 3. Preservação Cirúrgica de Anexos
A árvore MIME (`multipart/mixed`, `multipart/related`, `multipart/alternative`) é manipulada com precisão. Anexos como PDFs, arquivos compactados e imagens inline referenciadas via `cid:` são mantidos estritamente inalterados, preservando seus hashes e integridade.

### 4. Conexão Flexível (UNIX Socket & TCP)
Suporta escuta simultânea ou configurável por:
* **Socket UNIX:** `unix:/run/papeli/papeli.sock` (ideal para bare-metal / mesmo host, com controle de permissões de arquivo).
* **Socket TCP:** `inet:127.0.0.1:8899` ou `inet:[::1]:8899` (ideal para ambientes em contêineres Docker/Kubernetes).

### 5. Configuração Hierárquica com Hot-Reload (`papeli.toml`)
Resolução de templates e metadados em cascata:
$$\text{Remetente Específico} \longrightarrow \text{Domínio} \longrightarrow \text{Padrão (Default)}$$

Alterações nos arquivos `.mjml`, templates de texto ou no próprio `papeli.toml` são detectadas automaticamente em tempo de execução via *filesystem notifications* (`notify`), sem necessidade de reiniciar o serviço ou interromper o Postfix.

---

## ⚙️ Exemplo de Configuração (`papeli.toml`)

```toml
[server]
listen = "unix:/run/papeli/papeli.sock"
# listen = "inet:127.0.0.1:8899"
templates_dir = "/etc/papeli/templates"

# Padrão para domínios gerenciados sem regra específica
[default]
template_html = "default.mjml"
template_txt = "default.txt.j2"
company_name = "Organização Geral"
logo_url = "https://empresa.com/assets/logo-padrao.png"

# Regra específica para o domínio acme.com
[domains."acme.com"]
template_html = "acme_corporate.mjml"
template_txt = "acme_corporate.txt.j2"
company_name = "ACME Corporation"
logo_url = "https://acme.com/logo.png"
disclaimer = "Esta mensagem contém informações confidenciais."

# Regra para remetente específico (sobrescreve o domínio)
[senders."diretoria@acme.com"]
sender_name = "Carlos Silva"
sender_role = "Diretor Executivo"
sender_phone = "+55 11 99999-9999"
```

---

## 🚀 Integração com o Postfix

No arquivo `/etc/postfix/master.cf`, associe o Papéli **apenas aos serviços de envio de usuários autenticados** (`submission` e `submissions`), garantindo que ele rode **antes** do OpenDKIM:

```text
# /etc/postfix/master.cf

submission inet n       -       y       -       -       smtpd
  -o syslog_name=postfix/submission
  -o smtpd_tls_security_level=encrypt
  -o smtpd_sasl_auth_enable=yes
  -o smtpd_milters=unix:/run/papeli/papeli.sock,inet:localhost:8891
  -o milter_default_action=accept

submissions inet n       -       y       -       -       smtpd
  -o syslog_name=postfix/submissions
  -o smtpd_tls_wrappermode=yes
  -o smtpd_sasl_auth_enable=yes
  -o smtpd_milters=unix:/run/papeli/papeli.sock,inet:localhost:8891
  -o milter_default_action=accept
```

> **Nota sobre DKIM:** Na linha `smtpd_milters`, o Papéli (`unix:/run/papeli/papeli.sock`) é listado **antes** do OpenDKIM (`inet:localhost:8891`). O Postfix executa os milters na ordem informada, permitindo que a mensagem seja modificada pelo Papéli e logo em seguida assinada criptograficamente pelo DKIM.

---

## 💌 Coleção Oficial de Papéis de Carta Anos 80

O Papéli vem de fábrica com **10 modelos de papel de carta inspirados nos anos 80**, projetados com estética clássica, proporção de folha física de papel ($1:\sqrt{2}$, `min-height: 848px`) e imunidade absoluta a bloqueios de imagens:

| # | Tema (`canonical_name`) | Inspiração Visual | Emoji | Paleta / Destaque |
| :-: | :--- | :--- | :-: | :--- |
| **01** | `anos80_rosinha_frufru` | Laços, fitas & rendas vintage | 🎀 | Rosa Frufru (`#be185d` / `#f472b6`) |
| **02** | `anos80_moranguinho` | Confeitos & frutinhas doces | 🍓 | Vermelho Morango (`#e11d48` / `#fb7185`) |
| **03** | `anos80_lavanda_sonho` | Jardim floral poético de 1985 | 🪻 | Lavanda Violeta (`#6d28d9` / `#a78bfa`) |
| **04** | `anos80_nuvem_algodao` | Céu azul bebê & estrelinhas | ☁️ | Azul Celeste (`#0284c7` / `#38bdf8`) |
| **05** | `anos80_menta_fresca` | Cartão postal vintage 1980 | 🍃 | Verde Menta Retrô (`#16a34a` / `#4ade80`) |
| **06** | `anos80_caderno_pautado` | Folha escolar com margens | ✏️ | Grafite & Pauta (`#334155` / `#94a3b8`) |
| **07** | `anos80_carta_romantica` | Pergaminho baunilha & selo | 💌 | Âmbar Dourado (`#b45309` / `#f59e0b`) |
| **08** | `anos80_constelacao_pop` | Galáxia pop neon dos anos 80 | ✨ | Roxo Estelar (`#7e22ce` / `#c084fc`) |
| **09** | `anos80_jardim_encantado` | Pêssego & botânica romântica | 🌸 | Pêssego Floral (`#ea580c` / `#fb923c`) |
| **10** | `anos80_arcoiris_pastel` | Arco-íris em degradê suave | 🌈 | Magenta Suave (`#a21caf` / `#e879f9`) |

---

## 🎲 Estratégias de Seleção de Temas

O Papéli oferece seleção dinâmica configurável por domínio ou remetente no `papeli.toml`:

### 1. Seleção Aleatória (`random`)
Sorteia dinamicamente um dos modelos a cada novo e-mail enviado:
```toml
[domains."lumen.pink"]
header_title = "Lumen Pink"
template_strategy = "random"
```

### 2. Rodízio Sequencial (`round-robin`)
Alterna de forma determinística e atômica (`AtomicUsize`) entre os modelos da coleção:
```toml
[domains."lumen.pink"]
template_strategy = "round-robin"
```

### 3. Seleção Explícita por Assunto ou Cabeçalho
O remetente pode escolher o modelo diretamente ao redigir:
* **No Assunto:** Adicione `[tema:frufru]`, `[tema:moranguinho]` ou `#lavanda`. O Milter consome a tag e entrega o assunto limpo ao destinatário!
* **No Cabeçalho MIME:** `X-Papeli-Theme: menta`

---

## 💻 Suporte a Markdown & Clientes de Terminal (`aerc`)

Ideal para usuários de terminais e clientes TUI como o `aerc`:
* Quando um e-mail é enviado em **texto simples puro** (`text/plain`), o Papéli compila automaticamente a mensagem via `pulldown-cmark` (CommonMark + extensões GFM).
* **Emojis Temáticos nos Títulos:** Títulos em Markdown (`#`, `##`, `###`) recebem automaticamente o emoji e o sublinhado elegante da paleta do tema ativo.
* **Tabelas Estilizadas:** Tabelas em Markdown (`| Coluna | Status |`) ganham cabeçalhos temáticos em tom pastel, células com bordas suaves e espaçamento confortável.
* **Citações e Código:** Blocos de citação (`>`) e trechos de código (`code`/`pre`) herdam a cor de destaque do tema.
* **Sem Poluição WYSIWYG:** Garante uma folha de carta 100% fiel, sem conflitos de estilos herdados de editores ricos.

---

## 🗺️ Roadmap de Implementação

- [x] **#2 `infra(setup)`**: Setup inicial do workspace Rust, CI, hooks CCCP e documentação base.
- [x] **#5 `test(fixtures)`**: Suíte massiva de fixtures de e-mails reais (Outlook, Gmail, Apple Mail, Thunderbird).
- [x] **#6 `feat(mime)`**: Motor de manipulação e remontagem da árvore MIME sem quebrar anexos (RFC 2387).
- [x] **#7 `feat(sanitizer)`**: Sanitizador e extrator de conteúdo semântico com whitelist estrita.
- [x] **#8 `feat(signatures)`**: Sistema de dupla assinatura (longa ativa vs curta no histórico).
- [x] **#9 `feat(templates)`**: Engine de templates com MJML nativo (`mrml`), MiniJinja e os 10 modelos Anos 80.
- [x] **#10 `feat(config)`**: Gestor de configurações hierárquico e Hot-Reload (`papeli.toml`).
- [x] **#11 `feat(milter)`**: Daemon Milter com suporte duplo (UNIX Socket e TCP inet).
- [x] **#12 `test(e2e)`**: Bateria completa de testes automatizados ponta a ponta e estresse (35/35 testes).
- [x] **#13 `docs(postfix)`**: Guia de implantação em produção e integração com Postfix/DKIM.
- [x] **#14 `feat(markdown)`**: Compilador Markdown nativo para clientes de texto/terminal (`aerc`).
- [x] **#15 `feat(themes)`**: Estratégias `random`, `round-robin` e seleção via tags de assunto.

---

## 📄 Licença

Distribuído sob licença dual **MIT** ou **Apache-2.0**.
