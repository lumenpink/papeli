use minijinja::context;
use papeli::templates::{
    DEFAULT_MJML_TEMPLATE, DEFAULT_TEXT_TEMPLATE, TemplateEngine, build_long_signature_html,
    build_long_signature_text,
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

    assert!(
        html.contains("<!doctype html>"),
        "Deve ser um documento HTML completo"
    );
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
        header_title => "ACME Corporation",
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
          <mj-body background-color="#fff0f5">
            <mj-section background-url="cid:grad.png" background-repeat="repeat-x" background-color="#fff5f7">
              <mj-group>
                <mj-column width="12%">
                  <mj-image src="cid:corner_tl.png" width="36px" align="left" alt="୨୧" />
                </mj-column>
                <mj-column width="76%">
                  <mj-text align="center">Lumen Pink</mj-text>
                </mj-column>
                <mj-column width="12%">
                  <mj-image src="cid:corner_tr.png" width="36px" align="right" alt="୨୧" />
                </mj-column>
              </mj-group>
              <mj-column width="100%">
                <mj-text color="#701a75">{{ greeting }}, {{ user_name }}!</mj-text>
                <mj-raw>{{ content_html }}</mj-raw>
              </mj-column>
              <mj-group>
                <mj-column width="12%">
                  <mj-image src="cid:corner_bl.png" width="36px" align="left" alt="୨୧" />
                </mj-column>
                <mj-column width="76%">
                  <mj-text align="center">Papeli Stationery</mj-text>
                </mj-column>
                <mj-column width="12%">
                  <mj-image src="cid:corner_br.png" width="36px" align="right" alt="୨୧" />
                </mj-column>
              </mj-group>
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

#[test]
fn test_extract_theme_tags_from_subject() {
    use papeli::templates::extract_theme_tag;

    let (clean, theme) = extract_theme_tag("Reunião Importante [tema:frufru]");
    assert_eq!(clean, "Reunião Importante");
    assert_eq!(theme.as_deref(), Some("anos80_rosinha_frufru"));

    let (clean, theme) = extract_theme_tag("[theme:moranguinho] Relatório Mensal");
    assert_eq!(clean, "Relatório Mensal");
    assert_eq!(theme.as_deref(), Some("anos80_moranguinho"));

    let (clean, theme) = extract_theme_tag("Novidades da semana #nuvem");
    assert_eq!(clean, "Novidades da semana");
    assert_eq!(theme.as_deref(), Some("anos80_nuvem_algodao"));

    let (clean, theme) = extract_theme_tag("Assunto Comum Sem Tag");
    assert_eq!(clean, "Assunto Comum Sem Tag");
    assert_eq!(theme, None);
}

#[test]
fn test_all_10_builtin_themes_render_rich_email() {
    use papeli::templates::{ALL_BUILTIN_THEMES, TemplateEngine, build_long_signature_html};

    let engine = TemplateEngine::new();
    let sig_html = build_long_signature_html(
        Some("Lumen Pink"),
        Some("Designer & Dev"),
        Some("+55 11 98888-7777"),
        Some("Papéli Corp"),
        Some("Papel de carta anos 80."),
    );

    let rich_content = r#"
        <h2 style="color: inherit; margin-top: 0;">Planejamento Estratégico Q4</h2>
        <p>Olá equipe, segue o detalhamento com a tabela comparativa de resultados:</p>
        <table border="1" cellpadding="6" cellspacing="0" style="width: 100%; border-collapse: collapse; margin: 15px 0;">
            <tr style="background-color: rgba(0,0,0,0.03);">
                <th>Mês</th>
                <th>Envios</th>
                <th>Taxa de Abertura</th>
            </tr>
            <tr>
                <td>Julho</td>
                <td>12.500</td>
                <td>48,2%</td>
            </tr>
            <tr>
                <td>Agosto</td>
                <td>14.200</td>
                <td>52,7%</td>
            </tr>
        </table>
        <blockquote style="margin: 10px 0; padding-left: 10px; border-left: 3px solid #ccc;">
            "A beleza dos detalhes transforma uma simples mensagem em memória afetiva."
        </blockquote>
        <p>Agradecemos a dedicação de todos!</p>
    "#;

    let ctx = context! {
        header_title => "Lumen Pink",
        subject => "Demonstração Rica",
        content_html => rich_content,
        signature_html => sig_html,
    };

    assert_eq!(
        ALL_BUILTIN_THEMES.len(),
        10,
        "Devem existir exatamente 10 temas embutidos"
    );

    for &theme_name in ALL_BUILTIN_THEMES {
        let mjml = engine.resolve_mjml_template(theme_name, None);
        let html = engine
            .render_html(&mjml, &ctx)
            .unwrap_or_else(|e| panic!("Falha ao renderizar tema {}: {:?}", theme_name, e));

        assert!(
            html.contains("<!doctype html>"),
            "Tema {} deve gerar HTML válido",
            theme_name
        );
        assert!(
            html.contains("Lumen Pink"),
            "Tema {} deve conter Lumen Pink no cabeçalho",
            theme_name
        );
        assert!(
            html.contains("Planejamento Estratégico Q4"),
            "Tema {} deve conter título",
            theme_name
        );
        assert!(
            html.contains("Taxa de Abertura"),
            "Tema {} deve conter conteúdo da tabela",
            theme_name
        );
        assert!(
            html.contains("min-height: 848px") || html.contains("min-height:848px"),
            "Tema {} deve ter proporção de papel 848px",
            theme_name
        );
        assert!(
            !html.contains("<mj-raw>"),
            "Tema {} NUNCA deve vazar tag <mj-raw> no HTML final",
            theme_name
        );

        let attachments = papeli::templates::build_theme_inline_attachments(theme_name);
        assert_eq!(
            attachments.len(),
            0,
            "Tema {} não deve injetar attachments para manter compatibilidade máxima",
            theme_name
        );
    }
}

#[test]
fn test_template_selector_round_robin() {
    use papeli::templates::{ALL_BUILTIN_THEMES, TemplateSelector};

    let selector = TemplateSelector::new();
    let mut selected = Vec::new();

    for _ in 0..10 {
        let theme = selector.select_theme(None, None, "round-robin");
        selected.push(theme);
    }

    assert_eq!(selected.len(), 10);
    // Deve cobrir os 10 temas em sequência
    for &expected in ALL_BUILTIN_THEMES {
        assert!(
            selected.iter().any(|s| s == expected),
            "Round-robin deve selecionar {}",
            expected
        );
    }
}

#[test]
fn test_lumenlohn_mjml_compilation() {
    let engine = TemplateEngine::new();
    let template_path = std::path::Path::new("templates/lumenlohn.mjml");
    if template_path.exists() {
        let content = std::fs::read_to_string(template_path).expect("Ler lumenlohn.mjml");
        let ctx = minijinja::context! {
            subject => "Notificação Técnica Pericial",
            content_html => "<p>Manifestação técnica pericial referente aos quesitos apresentados.</p>",
        };
        let html = engine
            .render_html(&content, &ctx)
            .expect("Renderização do template lumenlohn.mjml via MRML deve ter sucesso");

        assert!(html.contains("<!doctype html>"));
        assert!(html.contains("Lumen Müller Lohn Freitas"));
        assert!(html.contains("Bacharela em Direito"));
        assert!(
            html.contains("Manifestação técnica pericial referente aos quesitos apresentados.")
        );
        assert!(html.contains("lumenlohn.com.br"));
    }
}
