use papeli::mime::ParsedEmail;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

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

#[test]
fn test_mime_reconstruction_preserves_pdf_attachment_byte_for_byte() {
    let raw = load_fixture("thunderbird_mixed.eml");
    let original = ParsedEmail::parse(&raw).expect("Falha no parse do Thunderbird");

    assert_eq!(original.attachments.len(), 1);
    let original_pdf = &original.attachments[0];
    let original_hash = sha256_digest(&original_pdf.data);

    let new_text = "Texto envelopado pelo Papéli";
    let new_html = "<html><body><h1>HTML no Papel de Carta</h1></body></html>";

    let reconstructed_raw = original
        .reconstruct(new_text, new_html)
        .expect("Falha ao reconstruir MIME");

    // Parse da mensagem reconstruída
    let reconstructed = ParsedEmail::parse(&reconstructed_raw).expect("Falha no re-parse");

    assert_eq!(reconstructed.attachments.len(), 1, "Deve manter exatamente 1 anexo");
    let reconstructed_pdf = &reconstructed.attachments[0];
    let reconstructed_hash = sha256_digest(&reconstructed_pdf.data);

    // Integridade estrita SHA-256
    assert_eq!(
        original_hash, reconstructed_hash,
        "O hash SHA-256 do PDF deve ser idêntico antes e depois da reconstrução MIME"
    );
    assert_eq!(original_pdf.name, reconstructed_pdf.name);
    assert_eq!(original_pdf.content_type, reconstructed_pdf.content_type);
    assert!(!reconstructed_pdf.is_inline);

    assert_eq!(reconstructed.body_text.as_deref(), Some(new_text));
    assert_eq!(reconstructed.body_html.as_deref(), Some(new_html));
}

#[test]
fn test_mime_reconstruction_preserves_inline_cid_images() {
    let raw = load_fixture("inline_images_cid.eml");
    let original = ParsedEmail::parse(&raw).expect("Falha no parse do inline CID");

    assert_eq!(original.attachments.len(), 1);
    let orig_inline = &original.attachments[0];
    assert!(orig_inline.is_inline);
    assert_eq!(orig_inline.content_id.as_deref(), Some("banner_topo@papeli.local"));
    let orig_hash = sha256_digest(&orig_inline.data);

    let new_text = "Novo texto do workshop";
    let new_html = "<div><img src=\"cid:banner_topo@papeli.local\" /><p>Novo layout</p></div>";

    let reconstructed_raw = original
        .reconstruct(new_text, new_html)
        .expect("Falha ao reconstruir");
    let reconstructed = ParsedEmail::parse(&reconstructed_raw).expect("Falha no re-parse");

    assert_eq!(reconstructed.attachments.len(), 1);
    let reconstructed_inline = &reconstructed.attachments[0];
    assert_eq!(sha256_digest(&reconstructed_inline.data), orig_hash);
    assert_eq!(
        reconstructed_inline.content_id.as_deref(),
        Some("banner_topo@papeli.local")
    );
}

#[test]
fn test_mime_reconstruction_promotes_plain_text_to_alternative() {
    let raw = load_fixture("raw_plain_text.eml");
    let original = ParsedEmail::parse(&raw).expect("Falha no parse do plain text");
    assert!(original.is_plain_only);
    assert!(original.body_html.is_none());

    let new_text = "Alerta formatado em texto";
    let new_html = "<html><body><h3>Alerta formatado em Papel de Carta</h3></body></html>";

    let reconstructed_raw = original
        .reconstruct(new_text, new_html)
        .expect("Falha ao reconstruir");
    let reconstructed = ParsedEmail::parse(&reconstructed_raw).expect("Falha no re-parse");

    assert_eq!(reconstructed.body_text.as_deref(), Some(new_text));
    assert_eq!(reconstructed.body_html.as_deref(), Some(new_html));
    assert!(!reconstructed.is_plain_only, "Agora deve possuir partes alternativa texto e html");
}

#[test]
fn test_mime_reconstruction_preserves_headers_and_threading() {
    let raw = load_fixture("gmail_web_thread.eml");
    let original = ParsedEmail::parse(&raw).expect("Falha no parse do Gmail thread");

    assert_eq!(original.sender_email.as_deref(), Some("anapaula@acme.com"));
    assert_eq!(original.subject.as_deref(), Some("Re: Atualização do Projeto Papéli"));
    assert!(original.in_reply_to.is_some());
    assert!(!original.references.is_empty());

    let new_text = "Corpo novo atualizado";
    let new_html = "<p>Corpo novo HTML</p>";

    let reconstructed_raw = original
        .reconstruct(new_text, new_html)
        .expect("Falha ao reconstruir");
    let reconstructed = ParsedEmail::parse(&reconstructed_raw).expect("Falha no re-parse");

    assert_eq!(reconstructed.subject, original.subject);
    assert_eq!(reconstructed.in_reply_to, original.in_reply_to);
    assert_eq!(reconstructed.references, original.references);
    assert_eq!(reconstructed.sender_email, original.sender_email);
}
