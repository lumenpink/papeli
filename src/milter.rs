use crate::config::Config;
use crate::mime::ParsedEmail;
use crate::sanitizer::{clean_plain_text, sanitize_html};
use crate::signatures::{
    collapse_historical_signatures_html, collapse_historical_signatures_text, default_short_html,
    default_short_text,
};
use crate::templates::{
    build_long_signature_html, build_long_signature_text, TemplateEngine, DEFAULT_MJML_TEMPLATE,
    DEFAULT_TEXT_TEMPLATE,
};
use minijinja::context;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::sync::{Arc, RwLock};

// Protocol constants
pub const SMFIC_OPTNEG: u8 = b'O';
pub const SMFIC_CONNECT: u8 = b'C';
pub const SMFIC_HELO: u8 = b'H';
pub const SMFIC_MAIL: u8 = b'M';
pub const SMFIC_RCPT: u8 = b'R';
pub const SMFIC_HEADER: u8 = b'L';
pub const SMFIC_EOH: u8 = b'N';
pub const SMFIC_BODY: u8 = b'B';
pub const SMFIC_BODYEOB: u8 = b'E';
pub const SMFIC_ABORT: u8 = b'A';
pub const SMFIC_QUIT: u8 = b'Q';

pub const SMFIS_CONTINUE: u8 = b'c';
pub const SMFIR_REPLBODY: u8 = b'b';

pub const SMFIF_ADDHDRS: u32 = 0x00000001;
pub const SMFIF_CHGBODY: u32 = 0x00000002;
pub const SMFIF_CHGHDRS: u32 = 0x00000010;

pub struct EmailProcessor {
    config: Arc<RwLock<Config>>,
    template_engine: Arc<TemplateEngine>,
}

impl EmailProcessor {
    pub fn new(config: Arc<RwLock<Config>>) -> Self {
        Self {
            config,
            template_engine: Arc::new(TemplateEngine::new()),
        }
    }

    pub fn process_email(&self, raw_bytes: &[u8]) -> Option<Vec<u8>> {
        let email = match ParsedEmail::parse(raw_bytes) {
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

        // 1. Sanitizar e limpar o corpo do e-mail do usuário
        let clean_text = email
            .body_text
            .as_deref()
            .map(clean_plain_text)
            .unwrap_or_default();
        let clean_html = email
            .body_html
            .as_deref()
            .map(sanitize_html)
            .unwrap_or_default();

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
            .render_html(DEFAULT_MJML_TEMPLATE, &ctx)
            .ok()?;
        let final_text = self
            .template_engine
            .render_text(DEFAULT_TEXT_TEMPLATE, &ctx)
            .ok()?;

        // 5. Reconstruir a árvore MIME intacta
        email.reconstruct(&final_text, &final_html).ok()
    }
}

pub fn handle_milter_stream<S: Read + Write>(mut stream: S, processor: &EmailProcessor) {
    let mut raw_message_buffer = Vec::new();

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
            SMFIC_CONNECT | SMFIC_HELO | SMFIC_MAIL | SMFIC_RCPT => {
                send_simple_response(&mut stream, SMFIS_CONTINUE);
            }
            SMFIC_HEADER => {
                // Armazena cabeçalho na mensagem bruta se estiver compondo
                send_simple_response(&mut stream, SMFIS_CONTINUE);
            }
            SMFIC_EOH => {
                send_simple_response(&mut stream, SMFIS_CONTINUE);
            }
            SMFIC_BODY => {
                // Acumula chunks do corpo
                raw_message_buffer.extend_from_slice(payload);
                send_simple_response(&mut stream, SMFIS_CONTINUE);
            }
            SMFIC_BODYEOB => {
                // Fim da mensagem: processar e substituir corpo se necessário
                if let Some(new_email_bytes) = processor.process_email(&raw_message_buffer) {
                    // Envia replbody em pedaços de no máximo 65535 bytes
                    for chunk in new_email_bytes.chunks(65000) {
                        let mut repl_packet = Vec::with_capacity(chunk.len() + 1);
                        repl_packet.push(SMFIR_REPLBODY);
                        repl_packet.extend_from_slice(chunk);
                        send_frame(&mut stream, &repl_packet);
                    }
                }

                send_simple_response(&mut stream, SMFIS_CONTINUE);
                raw_message_buffer.clear();
            }
            SMFIC_ABORT => {
                raw_message_buffer.clear();
            }
            SMFIC_QUIT => {
                break;
            }
            _ => {
                send_simple_response(&mut stream, SMFIS_CONTINUE);
            }
        }
    }
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
        for stream in listener.incoming() {
            if let Ok(s) = stream {
                let proc = Arc::clone(&processor);
                std::thread::spawn(move || {
                    handle_milter_stream(s, &proc);
                });
            }
        }
    } else if let Some(addr) = listen.strip_prefix("inet:") {
        let listener = TcpListener::bind(addr)?;
        for stream in listener.incoming() {
            if let Ok(s) = stream {
                let proc = Arc::clone(&processor);
                std::thread::spawn(move || {
                    handle_milter_stream(s, &proc);
                });
            }
        }
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Formato de escuta inválido: {}. Use 'unix:/caminho' ou 'inet:ip:porta'", listen),
        ));
    }

    Ok(())
}
