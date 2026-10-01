use papeli::config::Config;
use papeli::milter::EmailProcessor;
use papeli::mime::ParsedEmail;
use papeli::templates::ALL_BUILTIN_THEMES;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, RwLock};

fn create_test_processor() -> EmailProcessor {
    let toml = r#"
    [default]
    header_title = "Lumen Pink"

    [domains."lumen.pink"]
    header_title = "Lumen Pink"

    [senders."hi@lumen.pink"]
    header_title = "Lumen Pink"
    sender_name = "Lumen Pink"

    [senders."lumenfreitas@gmail.com"]
    header_title = "Lumen Pink"
    sender_name = "Lumen Pink"
    "#;
    let cfg = Config::from_toml(toml).unwrap();
    EmailProcessor::new(Arc::new(RwLock::new(cfg)))
}

fn rich_email_content() -> &'static str {
    r#"<h2 style="color: inherit; margin-top: 0; font-size: 20px; border-bottom: 2px solid rgba(0,0,0,0.1); padding-bottom: 6px;">
        🎀 Relatório de Produção &amp; Coleção Papéis de Carta Anos 80
    </h2>

    <p>Olá, equipe querida!</p>

    <p>
        Temos o imenso prazer de anunciar a conclusão do desenvolvimento dos <strong>10 novos modelos de papel de carta</strong>, 
        inspirados no charme atemporal dos anos 80. Cada modelo foi elaborado com foco em <em>resiliência estética total</em>:
        layouts 100% fieis mesmo quando o cliente de e-mail bloqueia o carregamento de imagens remotas.
    </p>

    <blockquote style="margin: 15px 0; padding: 10px 16px; border-left: 4px solid #f472b6; background: rgba(255,255,255,0.6); font-style: italic;">
        "Escrever um e-mail deve ter a mesma delicadeza e afeto de dobrar uma folha perfumada de papel de carta."
    </blockquote>

    <h3 style="color: inherit; font-size: 16px; margin: 20px 0 10px 0;">📊 Balanço de Distribuição dos Modelos</h3>

    <table border="1" cellpadding="8" cellspacing="0" style="width: 100%; border-collapse: collapse; margin-bottom: 20px; font-size: 13px;">
        <thead style="background-color: rgba(244, 114, 182, 0.15);">
            <tr>
                <th align="left">Tema</th>
                <th align="left">Inspiração</th>
                <th align="center">Proporção</th>
                <th align="center">Status</th>
            </tr>
        </thead>
        <tbody>
            <tr>
                <td><strong>Rosinha Frufru</strong></td>
                <td>Laços &amp; Rendas Vintage</td>
                <td align="center">1 : √2 (848px)</td>
                <td align="center"><span style="color: #059669; font-weight: bold;">✔ Pronto</span></td>
            </tr>
            <tr style="background-color: rgba(0,0,0,0.02);">
                <td><strong>Moranguinho</strong></td>
                <td>Confeitos &amp; Frutinhas Doces</td>
                <td align="center">1 : √2 (848px)</td>
                <td align="center"><span style="color: #059669; font-weight: bold;">✔ Pronto</span></td>
            </tr>
            <tr>
                <td><strong>Lavanda Sonho</strong></td>
                <td>Jardim Poético &amp; Floral</td>
                <td align="center">1 : √2 (848px)</td>
                <td align="center"><span style="color: #059669; font-weight: bold;">✔ Pronto</span></td>
            </tr>
            <tr style="background-color: rgba(0,0,0,0.02);">
                <td><strong>Nuvem de Algodão</strong></td>
                <td>Céu Azul Bebê &amp; Estrelinhas</td>
                <td align="center">1 : √2 (848px)</td>
                <td align="center"><span style="color: #059669; font-weight: bold;">✔ Pronto</span></td>
            </tr>
            <tr>
                <td><strong>Menta Retrô</strong></td>
                <td>Cartão Postal Vintage 1980</td>
                <td align="center">1 : √2 (848px)</td>
                <td align="center"><span style="color: #059669; font-weight: bold;">✔ Pronto</span></td>
            </tr>
            <tr style="background-color: rgba(0,0,0,0.02);">
                <td><strong>Caderno Pautado</strong></td>
                <td>Folha Escolar com Margem Rosa</td>
                <td align="center">1 : √2 (848px)</td>
                <td align="center"><span style="color: #059669; font-weight: bold;">✔ Pronto</span></td>
            </tr>
            <tr>
                <td><strong>Carta Romântica</strong></td>
                <td>Pergaminho Baunilha &amp; Selo Aéreo</td>
                <td align="center">1 : √2 (848px)</td>
                <td align="center"><span style="color: #059669; font-weight: bold;">✔ Pronto</span></td>
            </tr>
            <tr style="background-color: rgba(0,0,0,0.02);">
                <td><strong>Constelação Pop</strong></td>
                <td>Galáxia Neon Pastel dos Anos 80</td>
                <td align="center">1 : √2 (848px)</td>
                <td align="center"><span style="color: #059669; font-weight: bold;">✔ Pronto</span></td>
            </tr>
            <tr>
                <td><strong>Jardim Encantado</strong></td>
                <td>Pêssego &amp; Botânica Romântica</td>
                <td align="center">1 : √2 (848px)</td>
                <td align="center"><span style="color: #059669; font-weight: bold;">✔ Pronto</span></td>
            </tr>
            <tr style="background-color: rgba(0,0,0,0.02);">
                <td><strong>Arco-Íris Pastel</strong></td>
                <td>Degradê Colorido Suave</td>
                <td align="center">1 : √2 (848px)</td>
                <td align="center"><span style="color: #059669; font-weight: bold;">✔ Pronto</span></td>
            </tr>
        </tbody>
    </table>

    <p>Abaixo os principais destaques técnicos implementados:</p>
    <ul>
        <li><strong>Formatação Automática de Texto Simples:</strong> E-mails enviados sem HTML são automaticamente organizados em parágrafos e citações bem diagramadas.</li>
        <li><strong>Seleção Manual por Tag:</strong> Basta incluir <code>[tema:nome]</code> ou <code>#nome</code> no assunto do e-mail. A tag é consumida e o destinatário recebe o assunto limpo!</li>
        <li><strong>Rodízio Inteligente (Round-Robin):</strong> Envio sequencial atômico entre os modelos quando nenhuma tag é especificada.</li>
    </ul>

    <p>Esperamos que cada correspondência traga um sorriso no rosto de quem a recebe! 💌</p>"#
}

fn reply_email_content() -> &'static str {
    r#"<p>Oi Mari! Que alegria ler a sua mensagem! 💕</p>

    <p>
        Já revisei todas as lâminas dos 10 papéis de carta e a harmonia das cores ficou simplesmente perfeita! 
        As bordas duplas e as cantoneiras dos anos 80 trouxeram exatamente o aconchego nostálgico que queríamos. 
        Pode liberar a aprovação final para toda a equipe.
    </p>

    <p>Um beijo enorme e bom trabalho a todos!</p>

    <br>
    <div class="gmail_quote">
        <div dir="ltr" class="gmail_attr">Em qua., 30 de set. de 2026 às 16:20, Mariana Costa &lt;mariana@vintage.com&gt; escreveu:<br></div>
        <blockquote class="gmail_quote" style="margin:0px 0px 0px 0.8ex; border-left:1px solid rgb(204,204,204); padding-left:1ex">
            <div dir="ltr">
                <p>Olá Lumen! Tudo bem por aí?</p>
                <p>
                    Finalizamos os testes de renderização cruzada no Gmail, Outlook e Thunderbird. 
                    Conforme combinamos, ajustamos a altura mínima para a proporção clássica do papel (1:√2). 
                    Ficamos no aguardo do seu OK para disparar a primeira tiragem oficial!
                </p>
                <div class="papeli-sig-long" data-papeli="signature">
                    <p><strong>Mariana Costa</strong> &bull; Coordenação de Estilo</p>
                    <p style="color: #718096;">Vintage Papelaria Fina</p>
                </div>
            </div>
        </blockquote>
    </div>"#
}

fn forward_email_content() -> &'static str {
    r#"<p>Pessoal, estou encaminhando abaixo o comunicado da gráfica com os detalhes dos papéis e acabamentos:</p>

    <br>
    <div style="padding: 12px; background-color: rgba(0,0,0,0.02); border-left: 3px solid #94a3b8; margin-bottom: 16px;">
        <p style="margin: 0 0 4px 0; font-size: 13px;"><strong>---------- Mensagem encaminhada ----------</strong></p>
        <p style="margin: 0 0 2px 0; font-size: 13px;"><strong>De:</strong> Ateliê Gráfico Anos 80 &lt;grafica@atelie80.com.br&gt;</p>
        <p style="margin: 0 0 2px 0; font-size: 13px;"><strong>Data:</strong> 29 de setembro de 2026 às 11:15</p>
        <p style="margin: 0 0 2px 0; font-size: 13px;"><strong>Assunto:</strong> Catálogo de Acabamentos e Lotes Especiais</p>
        <p style="margin: 0; font-size: 13px;"><strong>Para:</strong> Lumen Pink &lt;hi@lumen.pink&gt;</p>
    </div>

    <h3>📋 Especificações de Gramatura e Acabamento:</h3>
    <p>Informamos que os lotes dos papéis especiais com textura linho e pó de confeito já foram separados:</p>
    <ul>
        <li><strong>Lote 1984-A:</strong> Papel vergê 120g/m² com picote floral nos cantos.</li>
        <li><strong>Lote 1985-B:</strong> Papel acetinado rosa morango com aroma delicado de frutas.</li>
        <li><strong>Lote 1988-C:</strong> Papel pautado celeste com clipe decorativo prateado.</li>
    </ul>

    <p>Permanecemos à disposição para o início dos envios!</p>"#
}

fn markdown_aerc_content() -> &'static str {
    r#"## 💌 Mensagem enviada via aerc (Terminal Girl)

Olá, Lumen!

Esta mensagem foi redigida inteiramente em **Markdown puro**, direto do terminal com o cliente `aerc`.

> "A delicadeza do papel de carta anos 80 encontra a praticidade do terminal."

### 📊 Relatório de Compatibilidade:

| Componente | Formato | Status |
| :--- | :---: | ---: |
| Editor de E-mail | aerc (TUI / Plain text) | ✔ 100% Suportado |
| Processador | pulldown-cmark + MRML | ✔ Markdown Nativo |
| Design & Paleta | Anos 80 Rosinha Frufru | ✔ Harmonioso |
| Rodapé Oficial | https://lumen.pink | ✔ Configurado |

Abaixo os destaques implementados:
- **Zero interferência de WYSIWYG:** cores puras da folha de papel sem estilos conflitantes.
- **Tipografia serifada:** fontes clássicas Georgia / Palatino em todo o texto e na assinatura.
- **Imunidade a bloqueios:** 100% livre de imagens externas e anexos CID.

Com carinho,"#
}

fn build_raw_email(from: &str, to: &str, subject: &str, html_body: &str) -> Vec<u8> {
    format!(
        "From: {from}\r\nTo: {to}\r\nSubject: {subject}\r\nMIME-Version: 1.0\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n{html_body}"
    )
    .into_bytes()
}

fn build_raw_plain_email(from: &str, to: &str, subject: &str, plain_body: &str) -> Vec<u8> {
    format!(
        "From: {from}\r\nTo: {to}\r\nSubject: {subject}\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=UTF-8\r\n\r\n{plain_body}"
    )
    .into_bytes()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut recipient = "hi@lumen.pink".to_string();
    let mut do_send = false;
    let mut send_both = false;
    let mut send_obizero = false;
    let mut use_markdown = false;
    let mut target_theme: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        if args[i] == "--send" {
            do_send = true;
            if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                recipient = args[i + 1].clone();
                i += 1;
            }
        } else if args[i] == "--to" {
            if i + 1 < args.len() {
                recipient = args[i + 1].clone();
                i += 1;
            }
        } else if args[i] == "--both" {
            send_both = true;
            do_send = true;
        } else if args[i] == "--obizero" {
            send_obizero = true;
            do_send = true;
        } else if args[i] == "--markdown" || args[i] == "--aerc" {
            use_markdown = true;
        } else if args[i] == "--theme" {
            if i + 1 < args.len() {
                target_theme = Some(args[i + 1].clone());
                i += 1;
            }
        }
        i += 1;
    }

    let out_dir = Path::new("previews");
    fs::create_dir_all(out_dir).unwrap();

    let processor = create_test_processor();

    println!("========================================================");
    println!(" ★ Papéli - Gerador de E-mails de Teste dos 10 Temas ★");
    println!("========================================================");
    println!("Diretório de saída: {}", out_dir.display());
    if send_both {
        println!("Destinatários: Gmail (lumenfreitas@gmail.com) E FairEmail (hi@lumen.pink via obizero SMTP)");
    } else if send_obizero {
        println!("Destinatário: FairEmail (hi@lumen.pink via obizero SMTP 127.0.0.1:25)");
    } else {
        println!("Destinatário configurado: {}", recipient);
    }
    if do_send {
        println!("Modo de envio ATIVADO!");
    } else {
        println!("Dica: use '--send {}' ou '--both' para disparar", recipient);
    }
    println!("--------------------------------------------------------");

    let mut index_cards = Vec::new();

    let sender = if recipient.contains("gmail.com") {
        "lumenfreitas@gmail.com"
    } else {
        "hi@lumen.pink"
    };

    let themes_to_run: Vec<&str> = if let Some(ref t) = target_theme {
        let canon = papeli::templates::canonical_theme_name(t);
        if canon != "default" {
            vec![canon]
        } else {
            ALL_BUILTIN_THEMES.to_vec()
        }
    } else {
        ALL_BUILTIN_THEMES.to_vec()
    };

    // 1. Gerar um e-mail completo para os temas selecionados
    for (idx, &theme_name) in themes_to_run.iter().enumerate() {
        let subject_with_tag = format!("Coleção Anos 80: [{}] [tema:{}]", theme_name, theme_name);

        let raw = if use_markdown {
            build_raw_plain_email(
                sender,
                &recipient,
                &subject_with_tag,
                markdown_aerc_content(),
            )
        } else {
            build_raw_email(
                sender,
                &recipient,
                &subject_with_tag,
                rich_email_content(),
            )
        };

        let processed = processor
            .process_email(&raw)
            .unwrap_or_else(|| panic!("Falha ao processar tema {}", theme_name));

        let parsed = ParsedEmail::parse(&processed).unwrap();
        let html_content = parsed.body_html.unwrap();

        let eml_file = out_dir.join(format!("{}.eml", theme_name));
        let html_file = out_dir.join(format!("{}.html", theme_name));

        fs::write(&eml_file, &processed).unwrap();
        fs::write(&html_file, &html_content).unwrap();

        println!("✔ [{:02}/10] Tema: {:<25} -> previews/{}.html", idx + 1, theme_name, theme_name);

        if send_both {
            // 1. Enviar para o Gmail via msmtp
            let gmail_raw = if use_markdown {
                build_raw_plain_email(
                    "lumenfreitas@gmail.com",
                    "lumenfreitas@gmail.com",
                    &subject_with_tag,
                    markdown_aerc_content(),
                )
            } else {
                build_raw_email(
                    "lumenfreitas@gmail.com",
                    "lumenfreitas@gmail.com",
                    &subject_with_tag,
                    rich_email_content(),
                )
            };
            if let Some(processed_gmail) = processor.process_email(&gmail_raw) {
                send_via_msmtp("lumenfreitas@gmail.com", &processed_gmail);
                println!("  ↳ [Gmail] E-mail enviado com sucesso para lumenfreitas@gmail.com!");
            }

            // 2. Enviar para o FairEmail via SMTP obizero local (127.0.0.1:25)
            let obizero_raw = if use_markdown {
                build_raw_plain_email(
                    "hi@lumen.pink",
                    "hi@lumen.pink",
                    &subject_with_tag,
                    markdown_aerc_content(),
                )
            } else {
                build_raw_email(
                    "hi@lumen.pink",
                    "hi@lumen.pink",
                    &subject_with_tag,
                    rich_email_content(),
                )
            };
            if let Some(processed_obizero) = processor.process_email(&obizero_raw) {
                send_via_obizero_smtp("hi@lumen.pink", &processed_obizero);
            }
        } else if send_obizero {
            send_via_obizero_smtp(&recipient, &processed);
        } else if do_send {
            send_via_msmtp(&recipient, &processed);
            println!("  ↳ E-mail enviado com sucesso para {}!", recipient);
        }

        index_cards.push(format!(
            r#"<div style="border: 2px solid #f472b6; border-radius: 12px; padding: 16px; margin: 12px 0; background: #fffafb;">
                <h3 style="margin-top: 0; color: #be185d;">#{:02} - {}</h3>
                <p style="color: #666; font-size: 14px;">Arquivo: <code>{}.html</code> | Tag de Assunto: <code>[tema:{}]</code></p>
                <a href="{}.html" target="_blank" style="display: inline-block; padding: 8px 16px; background-color: #f472b6; color: white; text-decoration: none; border-radius: 6px; font-weight: bold;">Visualizar Carta &rarr;</a>
            </div>"#,
            idx + 1, theme_name, theme_name, theme_name, theme_name
        ));
    }

    // 2. Gerar caso de Resposta (Reply com citação e assinatura colapsada)
    if target_theme.is_none() {
        let reply_subj = "Re: Planejamento dos Papéis [tema:frufru]";
        let raw = build_raw_email(
            sender,
            &recipient,
            reply_subj,
            reply_email_content(),
        );
        let processed = processor.process_email(&raw).unwrap();
        let parsed = ParsedEmail::parse(&processed).unwrap();
        let html_content = parsed.body_html.unwrap();

        fs::write(out_dir.join("caso_resposta_thread.eml"), &processed).unwrap();
        fs::write(out_dir.join("caso_resposta_thread.html"), &html_content).unwrap();
        println!("✔ Caso Especial: Resposta de E-mail (Thread) -> previews/caso_resposta_thread.html");

        if do_send {
            send_via_msmtp(&recipient, &processed);
            println!("  ↳ E-mail de resposta enviado para {}!", recipient);
        }

        // 3. Gerar caso de Encaminhamento (Forward)
        let fwd_subj = "Fwd: Catálogo de Acabamentos e Lotes [tema:moranguinho]";
        let raw = build_raw_email(
            sender,
            &recipient,
            fwd_subj,
            forward_email_content(),
        );
        let processed = processor.process_email(&raw).unwrap();
        let parsed = ParsedEmail::parse(&processed).unwrap();
        let html_content = parsed.body_html.unwrap();

        fs::write(out_dir.join("caso_encaminhamento.eml"), &processed).unwrap();
        fs::write(out_dir.join("caso_encaminhamento.html"), &html_content).unwrap();
        println!("✔ Caso Especial: Encaminhamento (Forward) -> previews/caso_encaminhamento.html");

        if do_send {
            send_via_msmtp(&recipient, &processed);
            println!("  ↳ E-mail de encaminhamento enviado para {}!", recipient);
        }
    }

    // 4. Gerar página índice de demonstração
    let index_html = format!(
        r#"<!doctype html>
<html lang="pt-BR">
<head>
    <meta charset="utf-8">
    <title>Galeria de Papéis de Carta Anos 80 - Papéli</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            background-color: #fdf2f8;
            color: #831843;
            max-width: 800px;
            margin: 40px auto;
            padding: 20px;
        }}
        h1 {{ text-align: center; color: #be185d; }}
        .header-box {{ text-align: center; margin-bottom: 30px; background: white; padding: 20px; border-radius: 16px; border: 2px dashed #f472b6; }}
    </style>
</head>
<body>
    <div class="header-box">
        <h1>💌 Galeria Oficial de Papéis de Carta Anos 80</h1>
        <p>Demonstração completa dos 10 modelos renderizados pelo <strong>Papéli</strong> com proporção de folha (1:√2), sem dependência de imagens externas e com imunidade ao bloqueio de clientes de e-mail.</p>
    </div>

    <h2>🌸 Os 10 Modelos da Coleção:</h2>
    {}

    <h2>📬 Casos Especiais de Conversa:</h2>
    <div style="border: 2px solid #a855f7; border-radius: 12px; padding: 16px; margin: 12px 0; background: #faf5ff;">
        <h3 style="margin-top: 0; color: #7e22ce;">Thread de Resposta (Reply com Citação e Assinatura Compactada)</h3>
        <p style="color: #666; font-size: 14px;">Arquivo: <code>caso_resposta_thread.html</code></p>
        <a href="caso_resposta_thread.html" target="_blank" style="display: inline-block; padding: 8px 16px; background-color: #a855f7; color: white; text-decoration: none; border-radius: 6px; font-weight: bold;">Visualizar Resposta &rarr;</a>
    </div>

    <div style="border: 2px solid #f43f5e; border-radius: 12px; padding: 16px; margin: 12px 0; background: #fff1f2;">
        <h3 style="margin-top: 0; color: #be123c;">Encaminhamento de E-mail (Forward de Mensagem Rica)</h3>
        <p style="color: #666; font-size: 14px;">Arquivo: <code>caso_encaminhamento.html</code></p>
        <a href="caso_encaminhamento.html" target="_blank" style="display: inline-block; padding: 8px 16px; background-color: #f43f5e; color: white; text-decoration: none; border-radius: 6px; font-weight: bold;">Visualizar Encaminhamento &rarr;</a>
    </div>
</body>
</html>"#,
        index_cards.join("\n")
    );

    fs::write(out_dir.join("index.html"), index_html).unwrap();
    println!("--------------------------------------------------------");
    println!("✨ Galeria de demonstração gerada em: previews/index.html");
    println!("========================================================");
}

fn send_via_msmtp(recipient: &str, raw_eml: &[u8]) {
    let mut child = Command::new("msmtp")
        .arg("--")
        .arg(recipient)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Falha ao invocar comando msmtp");

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(raw_eml).expect("Falha ao escrever no stdin do msmtp");
    }

    let output = child.wait_with_output().expect("Falha ao aguardar msmtp");
    if !output.status.success() {
        eprintln!(
            "Erro no msmtp: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn smtp_write_and_expect(stream: &mut std::net::TcpStream, cmd: &str) -> bool {
    use std::io::{Read, Write};
    if stream.write_all(cmd.as_bytes()).is_err() {
        return false;
    }
    let mut resp = [0u8; 1024];
    if let Ok(n) = stream.read(&mut resp) {
        let s = String::from_utf8_lossy(&resp[..n]);
        s.starts_with("2") || s.starts_with("3")
    } else {
        false
    }
}

fn send_via_obizero_smtp(recipient: &str, raw_eml: &[u8]) {
    use std::io::{Read, Write};
    use std::net::TcpStream;

    let mut stream = match TcpStream::connect("127.0.0.1:25") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Falha ao conectar no SMTP local obizero (127.0.0.1:25): {}", e);
            return;
        }
    };

    let mut buf = [0u8; 1024];
    let _ = stream.read(&mut buf);

    if !smtp_write_and_expect(&mut stream, "EHLO obizero\r\n") {
        eprintln!("Erro no EHLO");
        return;
    }
    if !smtp_write_and_expect(&mut stream, "MAIL FROM:<hi@lumen.pink>\r\n") {
        eprintln!("Erro no MAIL FROM");
        return;
    }
    if !smtp_write_and_expect(&mut stream, &format!("RCPT TO:<{}>\r\n", recipient)) {
        eprintln!("Erro no RCPT TO");
        return;
    }
    if !smtp_write_and_expect(&mut stream, "DATA\r\n") {
        eprintln!("Erro no DATA");
        return;
    }
    if stream.write_all(raw_eml).is_err() {
        eprintln!("Erro ao enviar corpo do e-mail");
        return;
    }
    if !smtp_write_and_expect(&mut stream, "\r\n.\r\n") {
        eprintln!("Erro ao finalizar DATA");
        return;
    }
    let _ = smtp_write_and_expect(&mut stream, "QUIT\r\n");
    println!("  ↳ [Obizero SMTP] E-mail entregue com sucesso via 127.0.0.1:25 para {}!", recipient);
}
