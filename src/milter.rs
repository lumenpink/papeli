use crate::config::Config;
use crate::mime::ParsedEmail;
use crate::sanitizer::{clean_plain_text, plain_text_to_html, sanitize_html};
use crate::signatures::{
    collapse_historical_signatures_html, collapse_historical_signatures_text, default_short_html,
    default_short_text,
};
use crate::templates::{
    DEFAULT_TEXT_TEMPLATE, TemplateEngine, TemplateSelector, build_long_signature_html,
    build_long_signature_text, extract_theme_tag,
};
use minijinja::context;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::sync::{Arc, RwLock};

// Protocol constants
pub const SMFIC_OPTNEG: u8 = b'O';
pub const SMFIC_MACRO: u8 = b'D';
pub const SMFIC_CONNECT: u8 = b'C';
pub const SMFIC_HELO: u8 = b'H';
pub const SMFIC_MAIL: u8 = b'M';
pub const SMFIC_RCPT: u8 = b'R';
pub const SMFIC_DATA: u8 = b'T';
pub const SMFIC_HEADER: u8 = b'L';
pub const SMFIC_EOH: u8 = b'N';
pub const SMFIC_BODY: u8 = b'B';
pub const SMFIC_BODYEOB: u8 = b'E';
pub const SMFIC_ABORT: u8 = b'A';
pub const SMFIC_QUIT: u8 = b'Q';
pub const SMFIC_QUIT_NC: u8 = b'K';
pub const SMFIC_UNKNOWN: u8 = b'U';

pub const SMFIS_CONTINUE: u8 = b'c';
pub const SMFIR_REPLBODY: u8 = b'b';
pub const SMFIR_CHGHEADER: u8 = b'm';
pub const SMFIR_ADDHEADER: u8 = b'h';

pub const SMFIF_ADDHDRS: u32 = 0x00000001;
pub const SMFIF_CHGBODY: u32 = 0x00000002;
pub const SMFIF_CHGHDRS: u32 = 0x00000010;

pub struct EmailProcessor {
    config: Arc<RwLock<Config>>,
    template_engine: Arc<TemplateEngine>,
    template_selector: Arc<TemplateSelector>,
}

impl EmailProcessor {
    pub fn new(config: Arc<RwLock<Config>>) -> Self {
        Self {
            config,
            template_engine: Arc::new(TemplateEngine::new()),
            template_selector: Arc::new(TemplateSelector::new()),
        }
    }

    pub fn process_email(&self, raw_bytes: &[u8]) -> Option<Vec<u8>> {
        let mut email = match ParsedEmail::parse(raw_bytes) {
            Ok(e) => e,
            Err(_) => return None,
        };

        let resolved = {
            let cfg = self.config.read().ok()?;
            cfg.resolve(email.sender_email.as_deref())
        };

        // Bypass: Se o e-mail não pertence a um domínio ou remetente gerenciado, não altera
        if !resolved.is_managed {
            return None;
        }

        // Bypass: Se o e-mail já foi processado anteriormente pelo Papéli, não reprocessa
        if email
            .custom_headers
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("x-papeli-processed"))
        {
            return None;
        }

        // 0. Detectar tema manual via Assunto ou Cabeçalho X-Papeli-Theme
        let (clean_subject, subject_theme) = email
            .subject
            .as_deref()
            .map(extract_theme_tag)
            .unwrap_or_else(|| (String::new(), None));

        if !clean_subject.is_empty() {
            email.subject = Some(clean_subject);
        }

        let header_theme = email
            .custom_headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("x-papeli-theme"))
            .map(|(_, v)| v.trim().to_string());

        let chosen_theme = if let Some(t) = subject_theme {
            t
        } else if let Some(t) = header_theme {
            t
        } else if resolved.template_strategy.eq_ignore_ascii_case("random")
            || resolved
                .template_strategy
                .eq_ignore_ascii_case("round-robin")
        {
            self.template_selector.select_theme(
                None,
                resolved.template_pool.as_deref(),
                &resolved.template_strategy,
            )
        } else if let Some(t) = &resolved.template_html {
            t.clone()
        } else {
            self.template_selector.select_theme(
                None,
                resolved.template_pool.as_deref(),
                &resolved.template_strategy,
            )
        };

        let templates_dir = {
            let cfg = self.config.read().ok();
            cfg.map(|c| c.server.templates_dir.clone())
        };
        let mjml_template = self
            .template_engine
            .resolve_mjml_template(&chosen_theme, templates_dir.as_deref());

        // 1. Sanitizar e limpar o corpo do e-mail do usuário
        let clean_text = email
            .body_text
            .as_deref()
            .map(clean_plain_text)
            .unwrap_or_default();

        // Se o e-mail original não tinha HTML, converte o texto simples em parágrafos HTML
        let clean_html = if let Some(raw_html) = email.body_html.as_deref() {
            sanitize_html(raw_html)
        } else {
            plain_text_to_html(&clean_text, &chosen_theme)
        };

        // 2. Assinatura curta para substituir instâncias antigas no histórico
        let short_html = default_short_html(
            resolved.sender_name.as_deref().unwrap_or("Equipe"),
            &resolved.company_name,
        );
        let short_text = default_short_text(
            resolved.sender_name.as_deref().unwrap_or("Equipe"),
            &resolved.company_name,
        );

        let collapsed_html = collapse_historical_signatures_html(&clean_html, &short_html);
        let collapsed_text = collapse_historical_signatures_text(&clean_text, &short_text);

        // 3. Assinatura longa oficial ativa da mensagem atual
        let long_html_sig = build_long_signature_html(
            resolved.sender_name.as_deref(),
            resolved.sender_role.as_deref(),
            resolved.sender_phone.as_deref(),
            Some(&resolved.company_name),
            resolved.disclaimer.as_deref(),
        );

        let long_text_sig = build_long_signature_text(
            resolved.sender_name.as_deref(),
            resolved.sender_role.as_deref(),
            resolved.sender_phone.as_deref(),
            Some(&resolved.company_name),
            resolved.disclaimer.as_deref(),
        );

        // 4. Renderizar via TemplateEngine
        let ctx = context! {
            header_title => resolved.header_title,
            subject => email.subject.as_deref().unwrap_or(""),
            logo_url => resolved.logo_url.as_deref().unwrap_or(""),
            company_name => resolved.company_name,
            content_html => collapsed_html,
            content_text => collapsed_text,
            signature_html => long_html_sig,
            signature_text => long_text_sig,
        };

        let final_html = self
            .template_engine
            .render_html(&mjml_template, &ctx)
            .ok()?;
        let final_text = self
            .template_engine
            .render_text(DEFAULT_TEXT_TEMPLATE, &ctx)
            .ok()?;

        // 5. Adicionar assets inline do tema (cantoneiras pré-rotacionadas e gradiente vertical)
        for att in crate::templates::build_theme_inline_attachments(&chosen_theme) {
            email.attachments.push(att);
        }

        // 6. Reconstruir a árvore MIME intacta
        email.reconstruct(&final_text, &final_html).ok()
    }
}

pub fn handle_milter_stream<S: Read + Write>(mut stream: S, processor: &EmailProcessor) {
    let mut header_buffer = Vec::new();
    let mut body_buffer = Vec::new();

    loop {
        // Leitura do frame Milter: 4 bytes len (big endian)
        let mut len_buf = [0u8; 4];
        if stream.read_exact(&mut len_buf).is_err() {
            break;
        }
        let packet_len = u32::from_be_bytes(len_buf) as usize;
        if packet_len == 0 {
            continue;
        }

        let mut packet = vec![0u8; packet_len];
        if stream.read_exact(&mut packet).is_err() {
            break;
        }

        let cmd = packet[0];
        let payload = &packet[1..];

        match cmd {
            SMFIC_OPTNEG => {
                // Negociação de protocolo
                let mut response = Vec::new();
                response.push(SMFIC_OPTNEG);
                response.extend_from_slice(&6u32.to_be_bytes()); // Protocol v6
                let actions = SMFIF_CHGBODY | SMFIF_ADDHDRS | SMFIF_CHGHDRS;
                response.extend_from_slice(&actions.to_be_bytes());
                response.extend_from_slice(&0u32.to_be_bytes()); // No step flags skipped

                send_frame(&mut stream, &response);
            }
            SMFIC_MACRO => {
                // Notificação unidirecional do MTA. O Milter NUNCA deve responder a macros!
            }
            SMFIC_CONNECT | SMFIC_HELO | SMFIC_MAIL | SMFIC_RCPT | SMFIC_DATA | SMFIC_UNKNOWN => {
                send_simple_response(&mut stream, SMFIS_CONTINUE);
            }
            SMFIC_HEADER => {
                if let Some(pos) = payload.iter().position(|&b| b == 0) {
                    let name = &payload[..pos];
                    let mut val = &payload[pos + 1..];
                    if let Some(val_pos) = val.iter().position(|&b| b == 0) {
                        val = &val[..val_pos];
                    }
                    header_buffer.extend_from_slice(name);
                    header_buffer.extend_from_slice(b": ");
                    header_buffer.extend_from_slice(val);
                    header_buffer.extend_from_slice(b"\r\n");
                } else {
                    header_buffer.extend_from_slice(payload);
                    header_buffer.extend_from_slice(b"\r\n");
                }
                send_simple_response(&mut stream, SMFIS_CONTINUE);
            }
            SMFIC_EOH => {
                send_simple_response(&mut stream, SMFIS_CONTINUE);
            }
            SMFIC_BODY => {
                // Acumula chunks do corpo
                body_buffer.extend_from_slice(payload);
                send_simple_response(&mut stream, SMFIS_CONTINUE);
            }
            SMFIC_BODYEOB => {
                // Fim da mensagem: processar e substituir corpo se necessário
                let full_message = if header_buffer.is_empty()
                    || body_buffer.starts_with(b"From:")
                    || body_buffer.starts_with(b"From: ")
                    || body_buffer.starts_with(b"Received:")
                    || body_buffer.starts_with(b"Return-Path:")
                {
                    body_buffer.clone()
                } else {
                    let mut full = Vec::with_capacity(header_buffer.len() + 2 + body_buffer.len());
                    full.extend_from_slice(&header_buffer);
                    full.extend_from_slice(b"\r\n");
                    full.extend_from_slice(&body_buffer);
                    full
                };

                eprintln!(
                    "[papeli] EOB received. headers_len={}, body_len={}, full_len={}",
                    header_buffer.len(),
                    body_buffer.len(),
                    full_message.len()
                );

                if let Some(new_email_bytes) = processor.process_email(&full_message) {
                    eprintln!(
                        "[papeli] Email processed and modified! New len={}",
                        new_email_bytes.len()
                    );
                    let (header_part, body_part) = split_headers_body(&new_email_bytes);

                    // 1. Atualiza cabeçalho Content-Type para o novo boundary gerado
                    if let Some(ctype) = extract_header_value(header_part, "Content-Type") {
                        send_change_header(&mut stream, 1, "Content-Type", &ctype);
                    }
                    if let Some(mime_ver) = extract_header_value(header_part, "MIME-Version") {
                        send_change_header(&mut stream, 1, "MIME-Version", &mime_ver);
                    }
                    if let Some(subj) = extract_header_value(header_part, "Subject") {
                        send_change_header(&mut stream, 1, "Subject", &subj);
                    }
                    // Remove Content-Transfer-Encoding no nível superior, pois multipart/* não pode ter CTE base64/qp raiz
                    send_change_header(&mut stream, 1, "Content-Transfer-Encoding", "");

                    // 2. Envia replbody exclusivamente com o corpo da mensagem (após CRLF CRLF)
                    for chunk in body_part.chunks(65000) {
                        let mut repl_packet = Vec::with_capacity(chunk.len() + 1);
                        repl_packet.push(SMFIR_REPLBODY);
                        repl_packet.extend_from_slice(chunk);
                        send_frame(&mut stream, &repl_packet);
                    }
                } else {
                    eprintln!("[papeli] process_email returned None (bypass or parse error)");
                }

                send_simple_response(&mut stream, SMFIS_CONTINUE);
                header_buffer.clear();
                body_buffer.clear();
            }
            SMFIC_ABORT => {
                header_buffer.clear();
                body_buffer.clear();
            }
            SMFIC_QUIT | SMFIC_QUIT_NC => {
                break;
            }
            _ => {
                send_simple_response(&mut stream, SMFIS_CONTINUE);
            }
        }
    }
}

pub fn split_headers_body(raw: &[u8]) -> (&[u8], &[u8]) {
    if let Some(pos) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
        (&raw[..pos], &raw[pos + 4..])
    } else if let Some(pos) = raw.windows(2).position(|w| w == b"\n\n") {
        (&raw[..pos], &raw[pos + 2..])
    } else {
        (b"", raw)
    }
}

pub fn extract_header_value(headers: &[u8], target_name: &str) -> Option<String> {
    let headers_str = std::str::from_utf8(headers).ok()?;
    let target = target_name.to_lowercase();
    let mut current_header: Option<(String, String)> = None;
    let mut result = None;

    for line in headers_str.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            if let Some((_, ref mut val)) = current_header {
                val.push(' ');
                val.push_str(line.trim());
            }
        } else {
            if let Some((name, val)) = current_header.take()
                && name == target
            {
                result = Some(val);
                break;
            }
            if let Some(colon_pos) = line.find(':') {
                let name = line[..colon_pos].trim().to_lowercase();
                let val = line[colon_pos + 1..].trim().to_string();
                current_header = Some((name, val));
            }
        }
    }
    if result.is_none()
        && let Some((name, val)) = current_header
        && name == target
    {
        result = Some(val);
    }
    result
}

pub fn send_change_header<W: Write>(writer: &mut W, index: u32, name: &str, value: &str) {
    let mut pkt = Vec::new();
    pkt.push(SMFIR_CHGHEADER);
    pkt.extend_from_slice(&index.to_be_bytes());
    pkt.extend_from_slice(name.as_bytes());
    pkt.push(0);
    pkt.extend_from_slice(value.as_bytes());
    pkt.push(0);
    send_frame(writer, &pkt);
}

fn send_frame<W: Write>(writer: &mut W, data: &[u8]) {
    let len = (data.len() as u32).to_be_bytes();
    let _ = writer.write_all(&len);
    let _ = writer.write_all(data);
    let _ = writer.flush();
}

fn send_simple_response<W: Write>(writer: &mut W, code: u8) {
    send_frame(writer, &[code]);
}

pub fn run_milter_listener(listen: &str, processor: Arc<EmailProcessor>) -> std::io::Result<()> {
    if let Some(sock_path) = listen.strip_prefix("unix:") {
        let path = Path::new(sock_path);
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }
        let listener = UnixListener::bind(path)?;
        for s in listener.incoming().flatten() {
            let proc = Arc::clone(&processor);
            std::thread::spawn(move || {
                handle_milter_stream(s, &proc);
            });
        }
    } else if let Some(addr) = listen.strip_prefix("inet:") {
        let listener = TcpListener::bind(addr)?;
        for s in listener.incoming().flatten() {
            let proc = Arc::clone(&processor);
            std::thread::spawn(move || {
                handle_milter_stream(s, &proc);
            });
        }
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "Formato de escuta inválido: {}. Use 'unix:/caminho' ou 'inet:ip:porta'",
                listen
            ),
        ));
    }

    Ok(())
}
