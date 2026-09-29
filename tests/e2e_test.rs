use papeli::config::Config;
use papeli::milter::{
    handle_milter_stream, EmailProcessor, SMFIC_BODY, SMFIC_BODYEOB, SMFIC_CONNECT, SMFIC_EOH,
    SMFIC_HEADER, SMFIC_HELO, SMFIC_MAIL, SMFIC_OPTNEG, SMFIC_QUIT, SMFIC_RCPT, SMFIR_CHGHEADER,
    SMFIR_REPLBODY, SMFIS_CONTINUE,
};
use papeli::mime::ParsedEmail;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::{Arc, RwLock};

fn load_fixture(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    fs::read(&path).unwrap_or_else(|e| panic!("Falha ao ler fixture {}: {}", path.display(), e))
}

fn sha256_digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn test_config() -> Arc<RwLock<Config>> {
    let toml = r#"
    [default]
    company_name = "Papéli Test Suite"
    logo_url = "https://empresa.com/assets/logo-padrao.png"
    template_html = "default.mjml"
    template_txt = "default.txt.j2"
    disclaimer = "Aviso padrão de confidencialidade."

    [domains."acme.com"]
    company_name = "ACME Corporation"
    logo_url = "https://acme.com/logo-acme.png"
    disclaimer = "Confidencial ACME."

    [senders."carlos@acme.com"]
    sender_name = "Carlos Silva"
    sender_role = "Diretor Executivo"
    sender_phone = "+55 11 99999-9999"

    [senders."juliana@acme.com"]
    sender_name = "Juliana Costa"
    sender_role = "Controladoria"
    sender_phone = "+55 11 88888-8888"

    [senders."roberto@acme.com"]
    sender_name = "Roberto Mendes"
    sender_role = "Jurídico"
    "#;
    let cfg = Config::from_toml(toml).unwrap();
    Arc::new(RwLock::new(cfg))
}

#[test]
fn test_e2e_all_fixtures_pass_through_processor() {
    let processor = EmailProcessor::new(test_config());

    // 1. Thunderbird Mixed: Integridade bit-a-bit do PDF
    let raw_tb = load_fixture("thunderbird_mixed.eml");
    let orig_tb = ParsedEmail::parse(&raw_tb).unwrap();
    let orig_hash = sha256_digest(&orig_tb.attachments[0].data);

    let processed_tb = processor
        .process_email(&raw_tb)
        .expect("Thunderbird mixed deve ser processado");
    let result_tb = ParsedEmail::parse(&processed_tb).unwrap();
    assert_eq!(result_tb.attachments.len(), 1);
    assert_eq!(sha256_digest(&result_tb.attachments[0].data), orig_hash);
    assert!(result_tb.body_html.unwrap().contains("papeli-sig-long"));

    // 2. Inline Images CID: Preservação do Content-ID
    let raw_cid = load_fixture("inline_images_cid.eml");
    let processed_cid = processor.process_email(&raw_cid).expect("CID deve ser processado");
    let result_cid = ParsedEmail::parse(&processed_cid).unwrap();
    assert_eq!(result_cid.attachments.len(), 1);
    assert_eq!(
        result_cid.attachments[0].content_id.as_deref(),
        Some("banner_topo@papeli.local")
    );

    // 3. Apple Mail: Remoção do "Enviado do meu iPhone"
    let raw_apple = load_fixture("apple_mail_ios.eml");
    let processed_apple = processor.process_email(&raw_apple).unwrap();
    let result_apple = ParsedEmail::parse(&processed_apple).unwrap();
    assert!(!result_apple.body_text.unwrap().contains("Enviado do meu iPhone"));
    assert!(!result_apple.body_html.unwrap().contains("Enviado do meu iPhone"));

    // 4. Nested Thread: Compactação de assinaturas históricas
    let raw_thread = load_fixture("nested_thread_long.eml");
    let processed_thread = processor.process_email(&raw_thread).unwrap();
    let result_thread = ParsedEmail::parse(&processed_thread).unwrap();
    let thread_html = result_thread.body_html.unwrap();
    // Apenas 1 assinatura longa (a nova da mensagem atual)
    assert_eq!(thread_html.matches("papeli-sig-long").count(), 1);
    // As 3 anteriores foram convertidas em curtas
    assert_eq!(thread_html.matches("papeli-sig-short").count(), 3);

    // 5. Raw Plain Text: Promovido para Alternative
    let raw_text = load_fixture("raw_plain_text.eml");
    let processed_text = processor.process_email(&raw_text).unwrap();
    let result_text = ParsedEmail::parse(&processed_text).unwrap();
    assert!(result_text.body_html.is_some());
    assert!(result_text.body_text.is_some());

    // 6. Outlook MSO: Limpeza de lixo WordSection e MSO
    let raw_mso = load_fixture("outlook_desktop_mso.eml");
    let processed_mso = processor.process_email(&raw_mso).unwrap();
    let result_mso = ParsedEmail::parse(&processed_mso).unwrap();
    let mso_html = result_mso.body_html.unwrap();
    assert!(!mso_html.contains("<o:p>"));
    assert!(!mso_html.contains("WordSection1"));
    assert!(mso_html.contains("ACME Corporation"));

    // 7. Legacy Stationery: Texturas antigas expurgadas
    let raw_legacy = load_fixture("outlook_stationery_legacy.eml");
    let processed_legacy = processor.process_email(&raw_legacy).unwrap();
    let result_legacy = ParsedEmail::parse(&processed_legacy).unwrap();
    let legacy_html = result_legacy.body_html.unwrap();
    assert!(!legacy_html.contains("marble_texture.gif"));
    assert!(legacy_html.contains("COMUNICADO INTERNO"));
}

#[test]
fn test_e2e_unix_socket_milter_session() {
    let processor = EmailProcessor::new(test_config());
    let temp_dir = tempfile::tempdir().unwrap();
    let socket_path = temp_dir.path().join("papeli.sock");

    let listener = UnixListener::bind(&socket_path).unwrap();
    let proc_thread = Arc::new(processor);

    let handle = std::thread::spawn(move || {
        if let Ok((stream, _)) = listener.accept() {
            handle_milter_stream(stream, &proc_thread);
        }
    });

    let mut client = UnixStream::connect(&socket_path).unwrap();

    // 1. OPTNEG
    send_frame(&mut client, &[SMFIC_OPTNEG, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0]);
    let optneg_reply = read_frame(&mut client);
    assert_eq!(optneg_reply[0], SMFIC_OPTNEG);

    // 2. CONNECT / HELO / MAIL / RCPT / HEADER / EOH
    send_frame(&mut client, &[SMFIC_CONNECT, b'l', b'o', b'c', b'a', b'l', 0]);
    assert_eq!(read_frame(&mut client)[0], SMFIS_CONTINUE);

    send_frame(&mut client, &[SMFIC_HELO, b'm', b'a', b'i', b'l', 0]);
    assert_eq!(read_frame(&mut client)[0], SMFIS_CONTINUE);

    send_frame(
        &mut client,
        &[SMFIC_MAIL, b'<', b'c', b'a', b'r', b'l', b'o', b's', b'@', b'a', b'c', b'm', b'e', b'.', b'c', b'o', b'm', b'>', 0],
    );
    assert_eq!(read_frame(&mut client)[0], SMFIS_CONTINUE);

    send_frame(&mut client, &[SMFIC_RCPT, b'<', b'd', b'e', b's', b't', b'@', b'e', b'x', b'.', b'c', b'o', b'm', b'>', 0]);
    assert_eq!(read_frame(&mut client)[0], SMFIS_CONTINUE);

    send_frame(&mut client, &[SMFIC_HEADER, b'F', b'r', b'o', b'm', 0, b'c', b'a', b'r', b'l', b'o', b's', 0]);
    assert_eq!(read_frame(&mut client)[0], SMFIS_CONTINUE);

    send_frame(&mut client, &[SMFIC_EOH]);
    assert_eq!(read_frame(&mut client)[0], SMFIS_CONTINUE);

    // 3. BODY (Envia mensagem do Thunderbird com PDF)
    let raw_email = load_fixture("thunderbird_mixed.eml");
    let mut body_packet = Vec::new();
    body_packet.push(SMFIC_BODY);
    body_packet.extend_from_slice(&raw_email);
    send_frame(&mut client, &body_packet);
    assert_eq!(read_frame(&mut client)[0], SMFIS_CONTINUE);

    // 4. BODYEOB (Fim de mensagem)
    send_frame(&mut client, &[SMFIC_BODYEOB]);

    // Lê resposta
    let mut got_replbody = false;
    let mut got_chgheader = false;
    let mut content_type_header = String::new();
    let mut new_body_bytes = Vec::new();

    loop {
        let frame = read_frame(&mut client);
        if frame[0] == SMFIR_REPLBODY {
            got_replbody = true;
            new_body_bytes.extend_from_slice(&frame[1..]);
        } else if frame[0] == SMFIR_CHGHEADER {
            got_chgheader = true;
            if frame.len() > 5 {
                let payload = &frame[5..];
                if let Some(pos) = payload.iter().position(|&b| b == 0) {
                    let name = std::str::from_utf8(&payload[..pos]).unwrap_or("");
                    let val = std::str::from_utf8(&payload[pos + 1..])
                        .unwrap_or("")
                        .trim_matches('\0');
                    if name.eq_ignore_ascii_case("content-type") {
                        content_type_header = val.to_string();
                    }
                }
            }
        } else if frame[0] == SMFIS_CONTINUE {
            break;
        }
    }

    assert!(got_replbody, "Deve receber frame SMFIR_REPLBODY via UNIX Socket");
    assert!(got_chgheader, "Deve receber frame SMFIR_CHGHEADER via UNIX Socket");

    // Valida o e-mail resultante como o MTA faz (combinando cabeçalhos editados com novo corpo)
    let mut full_bytes = format!("Content-Type: {}\r\n\r\n", content_type_header).into_bytes();
    full_bytes.extend_from_slice(&new_body_bytes);
    let parsed_result = ParsedEmail::parse(&full_bytes).unwrap();
    assert_eq!(parsed_result.attachments.len(), 1);
    assert!(parsed_result.body_html.unwrap().contains("papeli-sig-long"));

    // 5. QUIT
    send_frame(&mut client, &[SMFIC_QUIT]);
    let _ = handle.join();
}

#[test]
fn test_e2e_concurrent_processing_throughput() {
    let processor = Arc::new(EmailProcessor::new(test_config()));
    let raw_email = Arc::new(load_fixture("gmail_web_thread.eml"));

    let mut handles = Vec::new();

    for _ in 0..10 {
        let proc = Arc::clone(&processor);
        let raw = Arc::clone(&raw_email);

        handles.push(std::thread::spawn(move || {
            let res = proc.process_email(&raw).expect("Thread deve processar com sucesso");
            let parsed = ParsedEmail::parse(&res).expect("Parse de saída deve ser válido");
            assert!(parsed.body_html.unwrap().contains("ACME Corporation"));
        }));
    }

    for h in handles {
        h.join().expect("Todas as threads devem concluir com sucesso");
    }
}

fn send_frame<W: Write>(writer: &mut W, data: &[u8]) {
    let len = (data.len() as u32).to_be_bytes();
    writer.write_all(&len).unwrap();
    writer.write_all(data).unwrap();
    writer.flush().unwrap();
}

fn read_frame<R: Read>(reader: &mut R) -> Vec<u8> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).unwrap();
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).unwrap();
    buf
}
