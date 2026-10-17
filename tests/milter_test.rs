use papeli::config::Config;
use papeli::milter::{
    EmailProcessor, SMFIC_BODY, SMFIC_BODYEOB, SMFIC_OPTNEG, SMFIC_QUIT, SMFIR_REPLBODY,
    SMFIS_CONTINUE, handle_milter_stream,
};
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, RwLock};

fn load_fixture(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    fs::read(&path).unwrap_or_else(|e| panic!("Falha ao ler fixture {}: {}", path.display(), e))
}

fn sample_config() -> Arc<RwLock<Config>> {
    let toml = r#"
    [default]
    company_name = "Papéli Test"
    logo_url = "https://empresa.com/logo.png"
    template_html = "default"

    [domains."acme.com"]
    company_name = "ACME Corporation"
    logo_url = "https://acme.com/acme-logo.png"
    disclaimer = "Mensagem sigilosa ACME."

    [senders."carlos@acme.com"]
    sender_name = "Carlos Silva"
    sender_role = "Diretor"
    sender_phone = "+55 11 99999-9999"
    "#;
    let cfg = Config::from_toml(toml).unwrap();
    Arc::new(RwLock::new(cfg))
}

#[test]
fn test_email_processor_bypasses_unmanaged_senders() {
    let cfg = sample_config();
    let processor = EmailProcessor::new(cfg);

    let raw = b"From: stranger@external.com\r\nTo: user@acme.com\r\nSubject: Hi\r\n\r\nHello";
    let res = processor.process_email(raw);
    assert!(
        res.is_none(),
        "E-mails externos não gerenciados devem sofrer bypass"
    );
}

#[test]
fn test_email_processor_wraps_managed_sender() {
    let cfg = sample_config();
    let processor = EmailProcessor::new(cfg);

    let raw = load_fixture("gmail_web_thread.eml");
    let res = processor
        .process_email(&raw)
        .expect("Deve processar remetente gerenciado");

    let parsed_res =
        papeli::mime::ParsedEmail::parse(&res).expect("Falha ao parsear e-mail processado");
    let html = parsed_res.body_html.expect("Deve conter corpo HTML");
    let text = parsed_res.body_text.expect("Deve conter corpo de texto");

    assert!(html.contains("ACME Corporation"));
    assert!(html.contains("https://acme.com/acme-logo.png"));
    assert!(html.contains("Carlos, perfeito!"));
    assert!(html.contains("papeli-sig-long"));
    assert!(text.contains("ACME Corporation"));
}

#[test]
fn test_milter_stream_wire_protocol() {
    let cfg = sample_config();
    let processor = EmailProcessor::new(cfg);

    // Cria listener TCP local em porta efêmera (0)
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    let proc_thread = Arc::new(processor);
    let handle = std::thread::spawn(move || {
        if let Ok((stream, _)) = listener.accept() {
            handle_milter_stream(stream, &proc_thread);
        }
    });

    let mut client = TcpStream::connect(addr).unwrap();

    // 1. Enviar SMFIC_OPTNEG
    send_frame(
        &mut client,
        &[SMFIC_OPTNEG, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0],
    );
    let optneg_reply = read_frame(&mut client);
    assert_eq!(optneg_reply[0], SMFIC_OPTNEG);

    // 2. Enviar corpo de mensagem
    let raw_email = load_fixture("gmail_web_thread.eml");
    let mut body_packet = Vec::new();
    body_packet.push(SMFIC_BODY);
    body_packet.extend_from_slice(&raw_email);
    send_frame(&mut client, &body_packet);
    let body_reply = read_frame(&mut client);
    assert_eq!(body_reply[0], SMFIS_CONTINUE);

    // 3. Enviar SMFIC_BODYEOB (End of Message)
    send_frame(&mut client, &[SMFIC_BODYEOB]);

    // Esperar pelo menos um SMFIR_REPLBODY e depois SMFIS_CONTINUE
    let mut received_replbody = false;
    let mut received_continue = false;

    for _ in 0..10 {
        let frame = read_frame(&mut client);
        if frame[0] == SMFIR_REPLBODY {
            received_replbody = true;
        } else if frame[0] == SMFIS_CONTINUE {
            received_continue = true;
            break;
        }
    }

    assert!(
        received_replbody,
        "Milter deve enviar resposta SMFIR_REPLBODY com novo corpo"
    );
    assert!(
        received_continue,
        "Milter deve finalizar com SMFIS_CONTINUE"
    );

    // 4. Enviar SMFIC_QUIT
    send_frame(&mut client, &[SMFIC_QUIT]);
    let _ = handle.join();
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
