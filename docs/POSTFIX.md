# 📮 Guia de Implantação e Integração com Postfix

Este documento orienta a instalação em produção do **Papéli** e sua integração nativa com o **Postfix** como um filtro Milter.

---

## 🎯 Regra de Ouro: A Ordem dos Milters e o DKIM

Ao modificar o corpo do e-mail para injetar o papel de carta, o hash criptográfico do corpo (`bh` no cabeçalho DKIM) é alterado.
Se você rodar o Papéli **depois** do OpenDKIM ou Rspamd, a assinatura DKIM será **invalidada**, fazendo com que os destinatários marquem o e-mail como SPAM ou phishing.

Por essa razão, o Postfix deve ser configurado para executar os milters **em ordem sequencial estrita**:

```text
smtpd_milters = unix:/run/papeli/papeli.sock, inet:localhost:8891
                └───────────┬─────────────┘  └───────┬────────┘
                    1º PAPÉLI                   2º OPENDKIM
         (Envelopa no papel de carta)    (Assina a mensagem final)
```

---

## 🛠️ Passo 1: Configurando o Postfix (`master.cf`)

Como não queremos envelopar e-mails que chegam de terceiros (como notificações de fornecedores ou newsletters), o Papéli deve atuar **exclusivamente nos serviços de envio de usuários autenticados** (`submission` na porta 587 e `submissions` na porta 465).

Edite `/etc/postfix/master.cf`:

```text
# Porta 587 (STARTTLS Submission)
submission inet n       -       y       -       -       smtpd
  -o syslog_name=postfix/submission
  -o smtpd_tls_security_level=encrypt
  -o smtpd_sasl_auth_enable=yes
  -o smtpd_client_restrictions=permit_sasl_authenticated,reject
  -o smtpd_milters=unix:/run/papeli/papeli.sock,inet:localhost:8891
  -o milter_default_action=accept

# Porta 465 (SMTPS Implícito)
submissions inet n       -       y       -       -       smtpd
  -o syslog_name=postfix/submissions
  -o smtpd_tls_wrappermode=yes
  -o smtpd_sasl_auth_enable=yes
  -o smtpd_client_restrictions=permit_sasl_authenticated,reject
  -o smtpd_milters=unix:/run/papeli/papeli.sock,inet:localhost:8891
  -o milter_default_action=accept
```

> **Atenção:** Mantenha a porta 25 (`smtp inet`) **sem** o `unix:/run/papeli/papeli.sock`, garantindo que e-mails externos recebidos não sejam alterados.

---

## 🔐 Passo 2: Permissões do Socket UNIX

Para que o Postfix consiga se comunicar com o socket do Papéli sem erros de permissão negada:

1. Garanta que o diretório de runtime pertença ao grupo `postfix`:
   ```bash
   sudo mkdir -p /run/papeli /etc/papeli/templates
   sudo chown -R postfix:postfix /run/papeli /etc/papeli
   sudo chmod 770 /run/papeli
   ```

2. Se estiver usando chroot no Postfix (padrão em Debian/Ubuntu), certifique-se de apontar o socket para dentro do chroot (ex: `/var/spool/postfix/papeli/papeli.sock`) ou desative o chroot no `master.cf` mudando a coluna `chroot` de `y` para `n`.

---

## 🚀 Passo 3: Instalando e Iniciando o Serviço Systemd

1. Copie o binário compilado:
   ```bash
   sudo cp target/release/papeli /usr/local/bin/papeli
   sudo chmod +x /usr/local/bin/papeli
   ```

2. Copie o arquivo de configuração de exemplo:
   ```bash
   sudo cp deploy/papeli.example.toml /etc/papeli/papeli.toml
   ```

3. Instale a unidade systemd:
   ```bash
   sudo cp deploy/papeli.service /etc/systemd/system/
   sudo systemctl daemon-reload
   sudo systemctl enable --now papeli
   ```

4. Recarregue o Postfix:
   ```bash
   sudo postfix reload
   ```

---

## 🧪 Passo 4: Validando com o `swaks`

Você pode testar o envio de um e-mail de saída autenticado usando a ferramenta `swaks`:

```bash
swaks --to destinatario@externo.com \
      --from carlos@suaempresa.com \
      --server localhost \
      --port 587 \
      --auth LOGIN \
      --auth-user carlos@suaempresa.com \
      --auth-password 'suasenha' \
      --tls \
      --body "Olá, este é um teste do Papéli com Postfix!"
```

Ao abrir o e-mail no cliente de destino:
1. O texto original aparecerá formatado e envelopado no cartão centralizado MJML.
2. O logotipo corporativo e a assinatura ativa estarão presentes.
3. A verificação DKIM (`dkim=pass`) estará íntegra e verde nos cabeçalhos da mensagem!
