use mail_parser::{MessageParser, MimeHeaders};
use std::fs;
use std::path::Path;

fn load_fixture(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    fs::read(&path).unwrap_or_else(|e| panic!("Falha ao ler fixture {}: {}", path.display(), e))
}

#[test]
fn test_fixture_outlook_desktop_mso() {
    let raw = load_fixture("outlook_desktop_mso.eml");
    let msg = MessageParser::default().parse(&raw).expect("Erro no parse do Outlook MSO");

    assert_eq!(msg.subject().unwrap(), "Re: Cotação de Servidores Dell PowerEdge");
    assert!(msg.body_text(0).unwrap().contains("Prezados"));
    let html = msg.body_html(0).unwrap();
    assert!(html.contains("WordSection1"), "HTML deve conter seção Word MSO");
    assert!(html.contains("urn:schemas-microsoft-com:vml"), "HTML deve conter namespace VML");
}

#[test]
fn test_fixture_gmail_web_thread() {
    let raw = load_fixture("gmail_web_thread.eml");
    let msg = MessageParser::default().parse(&raw).expect("Erro no parse do Gmail Thread");

    assert_eq!(msg.subject().unwrap(), "Re: Atualização do Projeto Papéli");
    assert!(msg.in_reply_to().as_text().is_some());
    let html = msg.body_html(0).unwrap();
    assert!(html.contains("gmail_quote"), "HTML deve conter bloco de citação do Gmail");
    assert!(html.contains("gmail_signature"), "HTML deve conter assinatura do Gmail");
}

#[test]
fn test_fixture_apple_mail_ios() {
    let raw = load_fixture("apple_mail_ios.eml");
    let msg = MessageParser::default().parse(&raw).expect("Erro no parse do Apple Mail");

    assert_eq!(msg.subject().unwrap(), "Contrato assinado");
    assert!(msg.body_text(0).unwrap().contains("Enviado do meu iPhone"));
    assert!(msg.body_html(0).unwrap().contains("Enviado do meu iPhone"));
}

#[test]
fn test_fixture_thunderbird_mixed_with_attachment() {
    let raw = load_fixture("thunderbird_mixed.eml");
    let msg = MessageParser::default().parse(&raw).expect("Erro no parse do Thunderbird Mixed");

    assert_eq!(msg.subject().unwrap(), "Relatório Financeiro Q3 - ACME Corp");
    assert!(msg.body_text(0).is_some());
    assert!(msg.body_html(0).is_some());

    assert_eq!(msg.attachment_count(), 1, "Deve possuir exatamente 1 anexo");
    let attachment = msg.attachment(0).expect("Anexo não encontrado");
    assert_eq!(attachment.attachment_name().unwrap(), "relatorio_q3_consolidado.pdf");
    assert_eq!(attachment.content_type().unwrap().ctype(), "application");
    assert_eq!(attachment.content_type().unwrap().subtype().unwrap(), "pdf");
}

#[test]
fn test_fixture_inline_images_cid() {
    let raw = load_fixture("inline_images_cid.eml");
    let msg = MessageParser::default().parse(&raw).expect("Erro no parse do Inline CID");

    assert_eq!(msg.subject().unwrap(), "Convite: Workshop de Inovação 2026");
    let html = msg.body_html(0).unwrap();
    assert!(html.contains("cid:banner_topo@papeli.local"));

    assert_eq!(msg.attachment_count(), 1);
    let inline_part = msg.attachment(0).unwrap();
    assert_eq!(inline_part.content_id().unwrap(), "banner_topo@papeli.local");
    assert_eq!(inline_part.content_type().unwrap().subtype().unwrap(), "png");
}

#[test]
fn test_fixture_raw_plain_text() {
    let raw = load_fixture("raw_plain_text.eml");
    let msg = MessageParser::default().parse(&raw).expect("Erro no parse do Raw Plain Text");

    assert_eq!(msg.subject().unwrap(), "[ALERTA] Manutenção programada do gateway");
    assert!(msg.body_text(0).unwrap().contains("O gateway de pagamento passará por atualização"));
    // Verificar Content-Type da mensagem raiz
    assert!(msg.is_content_type("text", "plain"), "A mensagem original é puramente text/plain");
}

#[test]
fn test_fixture_raw_html_only() {
    let raw = load_fixture("raw_html_only.eml");
    let msg = MessageParser::default().parse(&raw).expect("Erro no parse do Raw HTML Only");

    assert_eq!(msg.subject().unwrap(), "Seu holerite de Setembro/2026 está disponível");
    assert!(msg.is_content_type("text", "html"), "A mensagem original é puramente text/html");
    assert!(msg.body_html(0).unwrap().contains("demonstrativo de pagamento"));
}

#[test]
fn test_fixture_nested_thread_long() {
    let raw = load_fixture("nested_thread_long.eml");
    let msg = MessageParser::default().parse(&raw).expect("Erro no parse de Thread Aninhada");

    assert!(msg.subject().unwrap().contains("Proposta Comercial de Consultoria"));
    let html = msg.body_html(0).unwrap();
    let text = msg.body_text(0).unwrap();

    // No HTML, deve haver 3 assinaturas longas na conversa completa
    let count_html_long = html.matches("papeli-sig-long").count();
    assert_eq!(count_html_long, 3, "Deve haver 3 assinaturas longas na conversa aninhada");

    // No Texto, deve haver 3 blocos delimitados de assinatura
    let count_text_long = text.matches("<!--papeli:sig:start:long-->").count();
    assert_eq!(count_text_long, 3, "Deve haver 3 marcadores de assinatura no texto plano");
}

#[test]
fn test_fixture_outlook_stationery_legacy() {
    let raw = load_fixture("outlook_stationery_legacy.eml");
    let msg = MessageParser::default().parse(&raw).expect("Erro no parse do Papel Legado");

    assert_eq!(msg.subject().unwrap(), "Comunicado: Reforma da Recepção Principal");
    let html = msg.body_html(0).unwrap();
    assert!(html.contains("marble_texture.gif"));
    assert!(html.contains("stationery-border"));
}
