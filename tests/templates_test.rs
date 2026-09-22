use minijinja::context;
use papeli::templates::{
    build_long_signature_html, build_long_signature_text, TemplateEngine, DEFAULT_MJML_TEMPLATE,
    DEFAULT_TEXT_TEMPLATE,
};

#[test]
fn test_template_engine_renders_default_html_stationery() {
    let engine = TemplateEngine::new();

    let sig_html = build_long_signature_html(
        Some("Carlos Silva"),
        Some("CTO"),
        Some("+55 11 99999-9999"),
        Some("ACME Corp"),
        Some("Confidencial."),
    );

    let ctx = context! {
        subject => "Proposta Comercial",
        logo_url => "https://acme.com/logo.png",
        content_html => "<p>Prezado cliente, segue a proposta solicitada.</p>",
        signature_html => sig_html,
    };

    let html = engine
        .render_html(DEFAULT_MJML_TEMPLATE, &ctx)
        .expect("Falha ao renderizar template MJML");

    assert!(html.contains("<!doctype html>"), "Deve ser um documento HTML completo");
    assert!(html.contains("Prezado cliente, segue a proposta solicitada."));
    assert!(html.contains("https://acme.com/logo.png"));
    assert!(html.contains("Carlos Silva"));
    assert!(html.contains("CTO"));
    assert!(html.contains("papeli-sig-long"));
}

#[test]
fn test_template_engine_renders_default_text_stationery() {
    let engine = TemplateEngine::new();

    let sig_text = build_long_signature_text(
        Some("Carlos Silva"),
        Some("CTO"),
        Some("+55 11 99999-9999"),
        Some("ACME Corp"),
        Some("Confidencial."),
    );

    let ctx = context! {
        company_name => "ACME Corporation",
        content_text => "Prezado cliente, segue a proposta solicitada.",
        signature_text => sig_text,
    };

    let text = engine
        .render_text(DEFAULT_TEXT_TEMPLATE, &ctx)
        .expect("Falha ao renderizar template de texto");

    assert!(text.contains("=== ACME Corporation ==="));
    assert!(text.contains("Prezado cliente, segue a proposta solicitada."));
    assert!(text.contains("<!--papeli:sig:start:long-->"));
    assert!(text.contains("Carlos Silva - CTO"));
    assert!(text.contains("ACME Corp"));
    assert!(text.contains("Tel: +55 11 99999-9999"));
}

#[test]
fn test_template_engine_custom_mjml() {
    let engine = TemplateEngine::new();

    let custom_mjml = r##"
        <mjml>
          <mj-body background-color="#003366">
            <mj-section>
              <mj-column>
                <mj-text color="#ffffff">{{ greeting }}, {{ user_name }}!</mj-text>
                <mj-raw>{{ content_html }}</mj-raw>
              </mj-column>
            </mj-section>
          </mj-body>
        </mjml>
    "##;

    let ctx = context! {
        greeting => "Olá",
        user_name => "Mariana",
        content_html => "<p>Bem-vinda à nossa plataforma.</p>",
    };

    let html = engine.render_html(custom_mjml, &ctx).unwrap();

    assert!(html.contains("Olá, Mariana!"));
    assert!(html.contains("Bem-vinda à nossa plataforma."));
}
