use papeli::mime::ParsedEmail;
use papeli::sanitizer::{clean_plain_text, sanitize_html};
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
fn test_sanitizer_outlook_mso_strips_styles_and_meta() {
    let raw = load_fixture("outlook_desktop_mso.eml");
    let email = ParsedEmail::parse(&raw).unwrap();
    let original_html = email.body_html.expect("Deve possuir HTML");

    let cleaned = sanitize_html(&original_html);

    assert!(!cleaned.contains("<head>"), "Não deve conter <head>");
    assert!(!cleaned.contains("<style>"), "Não deve conter <style>");
    assert!(
        !cleaned.contains("xmlns:v"),
        "Não deve conter namespaces VML"
    );
    assert!(
        !cleaned.contains("<!--[if"),
        "Não deve conter condicionais MSO"
    );
    assert!(!cleaned.contains("<o:p>"), "Não deve conter tags <o:p>");

    // Conteúdo original preservado
    assert!(cleaned.contains("Prezados,"));
    assert!(cleaned.contains("Segue a cotação solicitada dos novos servidores."));
    assert!(cleaned.contains("Atenciosamente,"));
    assert!(cleaned.contains("Marcos Oliveira"));
}

#[test]
fn test_sanitizer_apple_mail_strips_iphone_signature() {
    let raw = load_fixture("apple_mail_ios.eml");
    let email = ParsedEmail::parse(&raw).unwrap();

    let cleaned_text = clean_plain_text(&email.body_text.unwrap());
    assert!(!cleaned_text.contains("Enviado do meu iPhone"));
    assert_eq!(
        cleaned_text,
        "Aprovado por aqui. Podem seguir com a assinatura eletrônica."
    );

    let cleaned_html = sanitize_html(&email.body_html.unwrap());
    assert!(!cleaned_html.contains("Enviado do meu iPhone"));
    assert!(cleaned_html.contains("Aprovado por aqui. Podem seguir com a assinatura eletrônica."));
}

#[test]
fn test_sanitizer_outlook_legacy_stationery_flattened() {
    let raw = load_fixture("outlook_stationery_legacy.eml");
    let email = ParsedEmail::parse(&raw).unwrap();

    let cleaned = sanitize_html(&email.body_html.unwrap());

    assert!(
        !cleaned.contains("marble_texture.gif"),
        "Texturas antigas devem ser expurgadas"
    );
    assert!(
        !cleaned.contains("Comic Sans"),
        "CSS intrusivo deve ser removido"
    );
    assert!(!cleaned.contains("<body"), "Não deve conter tag body");

    assert!(cleaned.contains("COMUNICADO INTERNO"));
    assert!(cleaned.contains("A recepção principal do edifício sede"));
    assert!(cleaned.contains("Secretaria Geral"));
}

#[test]
fn test_sanitizer_security_strips_scripts_and_events() {
    let dirty_html = r#"
        <html>
        <head><script>alert('pwned');</script></head>
        <body onload="malicious()">
            <p>Olá amigo,</p>
            <a href="javascript:stealTokens()" onclick="evil()">Clique aqui</a>
            <iframe src="https://phishing.com"></iframe>
            <form action="/login"><input type="password"></form>
        </body>
        </html>
    "#;

    let cleaned = sanitize_html(dirty_html);

    assert!(!cleaned.contains("<script"));
    assert!(!cleaned.contains("alert"));
    assert!(!cleaned.contains("onload"));
    assert!(!cleaned.contains("onclick"));
    assert!(!cleaned.contains("javascript:"));
    assert!(!cleaned.contains("<iframe"));
    assert!(!cleaned.contains("<form"));
    assert!(!cleaned.contains("<input"));

    assert!(cleaned.contains("<p>Olá amigo,</p>"));
    assert!(cleaned.contains("Clique aqui"));
}

#[test]
fn test_plain_text_to_html_markdown_parsing() {
    let md = r#"
## Relatório de Teste

Olá, este é um e-mail escrito em **Markdown** via `aerc`.

> "A simplicidade é o último grau de sofisticação."

### Metas Atingidas:

| Módulo | Status | Prioridade |
| :--- | :---: | ---: |
| Milter Postfix | Concluído | Alta |
| Markdown Parser | Concluído | Alta |
| Design Anos 80 | Pronto | Máxima |

- [x] Teste de compatibilidade
- [x] Suporte ao FairEmail e Gmail
"#;

    let html = papeli::sanitizer::plain_text_to_html(md, "anos80_rosinha_frufru");

    assert!(html.contains("<h2"), "Deve converter ## em <h2>");
    assert!(
        html.contains("🎀"),
        "Deve injetar emoji do tema nos títulos"
    );
    assert!(
        html.contains("<strong>Markdown</strong>"),
        "Deve converter ** em <strong>"
    );
    assert!(
        html.contains("<blockquote"),
        "Deve converter > em <blockquote>"
    );
    assert!(
        html.contains("<table"),
        "Deve converter tabela Markdown em <table>"
    );
    assert!(
        html.contains("<th style=\"background-color: rgba(244, 114, 182, 0.18)"),
        "Cabeçalho da tabela deve ter a cor do tema"
    );
    assert!(
        html.contains("border: 1px solid #fbcfe8"),
        "Bordas da tabela devem ser na cor suave do tema"
    );
    assert!(
        html.contains("Milter Postfix"),
        "Deve conter conteúdo da tabela"
    );
}
