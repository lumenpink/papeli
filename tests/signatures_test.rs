use papeli::mime::ParsedEmail;
use papeli::signatures::{
    collapse_historical_signatures_html, collapse_historical_signatures_text, default_short_html,
    default_short_text,
};
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
fn test_collapse_signatures_html_in_nested_thread() {
    let raw = load_fixture("nested_thread_long.eml");
    let email = ParsedEmail::parse(&raw).unwrap();
    let original_html = email.body_html.expect("Deve possuir HTML");

    // No e-mail original há 3 assinaturas longas
    assert_eq!(original_html.matches("papeli-sig-long").count(), 3);
    assert_eq!(original_html.matches("papeli-sig-short").count(), 0);

    let short_sig = default_short_html("Carlos Silva", "ACME Corp");
    let collapsed_html = collapse_historical_signatures_html(&original_html, &short_sig);

    // Todas as assinaturas longas devem ter sido substituídas por assinaturas curtas
    assert_eq!(
        collapsed_html.matches("papeli-sig-long").count(),
        0,
        "Não deve sobrar nenhuma assinatura longa no histórico"
    );
    assert_eq!(
        collapsed_html.matches("papeli-sig-short").count(),
        3,
        "Deve conter 3 assinaturas curtas compactadas"
    );

    // Economia substancial de bytes
    assert!(
        collapsed_html.len() < original_html.len(),
        "O HTML compactado deve ser menor em bytes que o original"
    );
    let bytes_saved = original_html.len() - collapsed_html.len();
    assert!(bytes_saved > 500, "Economia deve ser expressiva (> 500 bytes)");
}

#[test]
fn test_collapse_signatures_text_in_nested_thread() {
    let raw = load_fixture("nested_thread_long.eml");
    let email = ParsedEmail::parse(&raw).unwrap();
    let original_text = email.body_text.expect("Deve possuir texto");

    assert_eq!(original_text.matches("<!--papeli:sig:start:long-->").count(), 3);
    assert_eq!(original_text.matches("<!--papeli:sig:start:short-->").count(), 0);

    let short_sig = default_short_text("Carlos Silva", "ACME Corp");
    let collapsed_text = collapse_historical_signatures_text(&original_text, &short_sig);

    assert_eq!(
        collapsed_text.matches("<!--papeli:sig:start:long-->").count(),
        0
    );
    assert_eq!(
        collapsed_text.matches("<!--papeli:sig:start:short-->").count(),
        3
    );

    assert!(collapsed_text.len() < original_text.len());
    let bytes_saved = original_text.len() - collapsed_text.len();
    assert!(bytes_saved > 200);
}
