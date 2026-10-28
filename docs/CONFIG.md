# ⚙️ Gestão de Configuração e Hot-Reload do Papéli

O módulo `src/config.rs` gerencia o carregamento, resolução hierárquica e recarga a quente (*hot-reload*) das regras de negócios do **Papéli** a partir do arquivo `papeli.toml`.

---

## 🏛️ Cascata de Resolução Hierárquica

Quando um e-mail é processado pelo filtro, o Papéli resolve as variáveis em 3 camadas de precedência:

$$\text{Remetente Específico} \quad \succ \quad \text{Regras do Domínio} \quad \succ \quad \text{Padrão (Default)}$$

```text
               [default] (Organização Geral, logo padrão, template padrão)
                  │
                  ▼
         [domains."acme.com"] (Sobrescreve nome da empresa, logo e disclaimer)
                  │
                  ▼
   [senders."diretoria@acme.com"] (Sobrescreve nome do remetente, cargo, telefone)
```

Se o remetente for de um domínio não gerenciado (ex: `@gmail.com`), `is_managed` será `false`, permitindo que o daemon ignore a mensagem sem alterá-la.

---

## 📄 Estrutura do Arquivo `papeli.toml`

```toml
[server]
# Socket de escuta (UNIX socket ou TCP inet)
listen = "unix:/run/papeli/papeli.sock"
# listen = "inet:127.0.0.1:8899"
templates_dir = "/etc/papeli/templates"
# Identificador do nó para rastreabilidade (X-Papeli-Wrapped-By)
node_id = "postfix-node-01"
# Lista opcional de nós confiáveis do cluster para prevenção de duplo encapsulamento
recognized_nodes = ["postfix-node-01", "postfix-node-02"]

[default]
company_name = "Organização Geral"
logo_url = "https://empresa.com/assets/logo-padrao.png"
template_html = "default.mjml"
template_txt = "default.txt.j2"
disclaimer = "Esta mensagem contém informações confidenciais."

[domains."acme.com"]
company_name = "ACME Corporation"
logo_url = "https://acme.com/logo.png"
disclaimer = "Mensagem protegida pela política de confidencialidade da ACME."

[senders."diretoria@acme.com"]
sender_name = "Carlos Silva"
sender_role = "Diretor Executivo"
sender_phone = "+55 11 99999-9999"

[senders."suporte@acme.com"]
sender_name = "Suporte ao Cliente"
sender_role = "Central de Atendimento 24/7"
sender_phone = "0800 123 4567"
```

---

## 🔄 Recarga a Quente (*Live Hot-Reload*)
* A estrutura interna é protegida por `Arc<RwLock<Config>>`.
* O `ConfigManager` suporta recarga dinâmica chamando `.reload()`.
* Alterações no arquivo `papeli.toml` entram em vigor instantaneamente para o próximo e-mail processado, sem necessidade de reiniciar o serviço systemd ou reiniciar o Postfix.
