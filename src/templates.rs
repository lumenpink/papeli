use minijinja::{Environment, Value};
use mrml::prelude::render::RenderOptions;
use std::fmt;

#[derive(Debug)]
pub enum TemplateError {
    JinjaError(String),
    MjmlError(String),
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemplateError::JinjaError(e) => write!(f, "Template Jinja Error: {}", e),
            TemplateError::MjmlError(e) => write!(f, "Template MJML Error: {}", e),
        }
    }
}

impl std::error::Error for TemplateError {}

pub const DEFAULT_MJML_TEMPLATE: &str = r##"
<mjml>
  <mj-head>
    <mj-title>{{ subject | default("Mensagem") }}</mj-title>
    <mj-attributes>
      <mj-all font-family="-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif" />
    </mj-attributes>
  </mj-head>
  <mj-body background-color="#f4f6f8">
    <mj-section background-color="#ffffff" padding="24px 20px" border-radius="6px">
      <mj-column>
        {% if logo_url %}
        <mj-image src="{{ logo_url }}" width="140px" align="left" padding="0 0 16px 0" />
        {% endif %}
        <mj-text font-size="14px" line-height="22px" color="#2d3748" padding="0">
          <mj-raw>{{ content_html }}</mj-raw>
        </mj-text>
        <mj-divider border-color="#e2e8f0" border-width="1px" padding="20px 0" />
        <mj-raw>{{ signature_html }}</mj-raw>
      </mj-column>
    </mj-section>
  </mj-body>
</mjml>
"##;

pub const DEFAULT_TEXT_TEMPLATE: &str = r#"=== {{ company_name | default("Papéli") }} ===

{{ content_text }}

--
{{ signature_text }}
"#;

pub struct TemplateEngine {
    env: Environment<'static>,
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateEngine {
    pub fn new() -> Self {
        let env = Environment::new();
        Self { env }
    }

    pub fn render_html(&self, mjml_template: &str, context: &Value) -> Result<String, TemplateError> {
        // 1. Interpolar variáveis dinâmicas no template MJML via MiniJinja
        let rendered_mjml = self
            .env
            .render_str(mjml_template, context)
            .map_err(|e| TemplateError::JinjaError(e.to_string()))?;

        // 2. Compilar MJML para HTML responsivo de e-mail via MRML (Rust nativo)
        let parsed = mrml::parse(&rendered_mjml)
            .map_err(|e| TemplateError::MjmlError(format!("Parse error: {:?}", e)))?;

        let opts = RenderOptions::default();
        parsed
            .element
            .render(&opts)
            .map_err(|e| TemplateError::MjmlError(format!("Render error: {:?}", e)))
    }

    pub fn render_text(&self, text_template: &str, context: &Value) -> Result<String, TemplateError> {
        self.env
            .render_str(text_template, context)
            .map_err(|e| TemplateError::JinjaError(e.to_string()))
    }
}

pub fn build_long_signature_html(
    sender_name: Option<&str>,
    sender_role: Option<&str>,
    sender_phone: Option<&str>,
    company_name: Option<&str>,
    disclaimer: Option<&str>,
) -> String {
    let name = sender_name.unwrap_or("Equipe");
    let role = sender_role.unwrap_or("");
    let phone = sender_phone.unwrap_or("");
    let company = company_name.unwrap_or("");
    let disc = disclaimer.unwrap_or("");

    format!(
        r#"<div class="papeli-sig-long" data-papeli="signature">
  <table cellpadding="0" cellspacing="0" style="font-family: inherit; font-size: 13px; color: #4a5568;">
    <tr><td><strong>{}</strong>{}</td></tr>
    {}
    {}
    {}
  </table>
</div>"#,
        name,
        if !role.is_empty() {
            format!(" &bull; {}", role)
        } else {
            "".into()
        },
        if !company.is_empty() {
            format!("<tr><td style=\"color: #718096;\">{}</td></tr>", company)
        } else {
            "".into()
        },
        if !phone.is_empty() {
            format!(
                "<tr><td style=\"color: #718096;\">Tel: {}</td></tr>",
                phone
            )
        } else {
            "".into()
        },
        if !disc.is_empty() {
            format!(
                "<tr><td style=\"font-size: 11px; color: #a0aec0; padding-top: 6px;\">{}</td></tr>",
                disc
            )
        } else {
            "".into()
        }
    )
}

pub fn build_long_signature_text(
    sender_name: Option<&str>,
    sender_role: Option<&str>,
    sender_phone: Option<&str>,
    company_name: Option<&str>,
    disclaimer: Option<&str>,
) -> String {
    let name = sender_name.unwrap_or("Equipe");
    let role = sender_role.unwrap_or("");
    let phone = sender_phone.unwrap_or("");
    let company = company_name.unwrap_or("");
    let disc = disclaimer.unwrap_or("");

    let mut lines = Vec::new();
    lines.push("<!--papeli:sig:start:long-->".to_string());
    if !role.is_empty() {
        lines.push(format!("{} - {}", name, role));
    } else {
        lines.push(name.to_string());
    }
    if !company.is_empty() {
        lines.push(company.to_string());
    }
    if !phone.is_empty() {
        lines.push(format!("Tel: {}", phone));
    }
    if !disc.is_empty() {
        lines.push(format!("[Aviso Legal: {}]", disc));
    }
    lines.push("<!--papeli:sig:end:long-->".to_string());

    lines.join("\n")
}
