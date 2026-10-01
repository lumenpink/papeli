use minijinja::{Environment, Value};
use mrml::prelude::render::RenderOptions;
use std::fmt;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

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
    <mj-style>
      .papeli-sheet {
        min-height: 848px !important;
        box-sizing: border-box !important;
        background: #ffffff !important;
        background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%) !important;
      }
    </mj-style>
  </mj-head>
  <mj-body background-color="#f1f5f9">
    <mj-section background-color="#ffffff" padding="28px 24px" border-radius="8px" css-class="papeli-sheet">
      <mj-column>
        <mj-text align="center" padding="0 0 16px 0" font-size="16px" font-weight="bold" color="#334155">
          {{ header_title | default("Lumen Pink") }}
        </mj-text>
        {% if logo_url %}
        <mj-image src="{{ logo_url }}" width="140px" align="center" padding="0 0 16px 0" />
        {% endif %}
        <mj-text font-size="14px" line-height="22px" color="#2d3748" padding="0">
          {{ content_html }}
        </mj-text>
        <mj-divider border-color="#e2e8f0" border-width="1px" padding="20px 0" />
        <mj-text padding="0">
          {{ signature_html }}
        </mj-text>
      </mj-column>
    </mj-section>
  </mj-body>
</mjml>
"##;

pub const DEFAULT_TEXT_TEMPLATE: &str = r#"=== {{ header_title | default("Lumen Pink") }} ===

{{ content_text }}

--
{{ signature_text }}
"#;

// Inclusão embutida dos 10 templates anos 80
pub const TEMPLATE_ANOS80_ROSINHA: &str = include_str!("../templates/anos80_rosinha_frufru.mjml");
pub const TEMPLATE_ANOS80_MORANGUINHO: &str = include_str!("../templates/anos80_moranguinho.mjml");
pub const TEMPLATE_ANOS80_LAVANDA: &str = include_str!("../templates/anos80_lavanda_sonho.mjml");
pub const TEMPLATE_ANOS80_NUVEM: &str = include_str!("../templates/anos80_nuvem_algodao.mjml");
pub const TEMPLATE_ANOS80_MENTA: &str = include_str!("../templates/anos80_menta_fresca.mjml");
pub const TEMPLATE_ANOS80_CADERNO: &str = include_str!("../templates/anos80_caderno_pautado.mjml");
pub const TEMPLATE_ANOS80_ROMANTICA: &str = include_str!("../templates/anos80_carta_romantica.mjml");
pub const TEMPLATE_ANOS80_CONSTELACAO: &str = include_str!("../templates/anos80_constelacao_pop.mjml");
pub const TEMPLATE_ANOS80_JARDIM: &str = include_str!("../templates/anos80_jardim_encantado.mjml");
pub const TEMPLATE_ANOS80_ARCOIRIS: &str = include_str!("../templates/anos80_arcoiris_pastel.mjml");

pub const ALL_BUILTIN_THEMES: &[&str] = &[
    "anos80_rosinha_frufru",
    "anos80_moranguinho",
    "anos80_lavanda_sonho",
    "anos80_nuvem_algodao",
    "anos80_menta_fresca",
    "anos80_caderno_pautado",
    "anos80_carta_romantica",
    "anos80_constelacao_pop",
    "anos80_jardim_encantado",
    "anos80_arcoiris_pastel",
];

pub fn canonical_theme_name(alias: &str) -> &'static str {
    let lower = alias.trim().to_lowercase();
    match lower.as_str() {
        "frufru" | "rosinha" | "anos80_rosinha_frufru" => "anos80_rosinha_frufru",
        "moranguinho" | "morango" | "anos80_moranguinho" => "anos80_moranguinho",
        "lavanda" | "sonho" | "anos80_lavanda_sonho" => "anos80_lavanda_sonho",
        "nuvem" | "algodao" | "anos80_nuvem_algodao" => "anos80_nuvem_algodao",
        "menta" | "anos80_menta_fresca" => "anos80_menta_fresca",
        "caderno" | "pautado" | "anos80_caderno_pautado" => "anos80_caderno_pautado",
        "romantica" | "pergaminho" | "anos80_carta_romantica" => "anos80_carta_romantica",
        "constelacao" | "pop" | "estrelas" | "anos80_constelacao_pop" => "anos80_constelacao_pop",
        "jardim" | "floral" | "anos80_jardim_encantado" => "anos80_jardim_encantado",
        "arcoiris" | "rainbow" | "anos80_arcoiris_pastel" => "anos80_arcoiris_pastel",
        "default" | "padrao" => "default",
        _ => "default",
    }
}

pub fn get_builtin_template(name: &str) -> Option<&'static str> {
    match canonical_theme_name(name) {
        "anos80_rosinha_frufru" => Some(TEMPLATE_ANOS80_ROSINHA),
        "anos80_moranguinho" => Some(TEMPLATE_ANOS80_MORANGUINHO),
        "anos80_lavanda_sonho" => Some(TEMPLATE_ANOS80_LAVANDA),
        "anos80_nuvem_algodao" => Some(TEMPLATE_ANOS80_NUVEM),
        "anos80_menta_fresca" => Some(TEMPLATE_ANOS80_MENTA),
        "anos80_caderno_pautado" => Some(TEMPLATE_ANOS80_CADERNO),
        "anos80_carta_romantica" => Some(TEMPLATE_ANOS80_ROMANTICA),
        "anos80_constelacao_pop" => Some(TEMPLATE_ANOS80_CONSTELACAO),
        "anos80_jardim_encantado" => Some(TEMPLATE_ANOS80_JARDIM),
        "anos80_arcoiris_pastel" => Some(TEMPLATE_ANOS80_ARCOIRIS),
        "default" => {
            if name.eq_ignore_ascii_case("default") || name.eq_ignore_ascii_case("padrao") {
                Some(DEFAULT_MJML_TEMPLATE)
            } else {
                None
            }
        }
        _ => None,
    }
}

use crate::mime::EmailAttachment;

#[derive(Clone, Copy)]
pub struct ThemeAssets {
    pub grad_png: &'static [u8],
    pub corner_tl_png: &'static [u8],
    pub corner_tr_png: &'static [u8],
    pub corner_br_png: &'static [u8],
    pub corner_bl_png: &'static [u8],
}

pub fn get_theme_assets(theme_name: &str) -> Option<ThemeAssets> {
    match canonical_theme_name(theme_name) {
        "anos80_rosinha_frufru" => Some(ThemeAssets {
            grad_png: include_bytes!("../assets/grad_anos80_rosinha_frufru.png"),
            corner_tl_png: include_bytes!("../assets/corner_anos80_rosinha_frufru_tl.png"),
            corner_tr_png: include_bytes!("../assets/corner_anos80_rosinha_frufru_tr.png"),
            corner_br_png: include_bytes!("../assets/corner_anos80_rosinha_frufru_br.png"),
            corner_bl_png: include_bytes!("../assets/corner_anos80_rosinha_frufru_bl.png"),
        }),
        "anos80_moranguinho" => Some(ThemeAssets {
            grad_png: include_bytes!("../assets/grad_anos80_moranguinho.png"),
            corner_tl_png: include_bytes!("../assets/corner_anos80_moranguinho_tl.png"),
            corner_tr_png: include_bytes!("../assets/corner_anos80_moranguinho_tr.png"),
            corner_br_png: include_bytes!("../assets/corner_anos80_moranguinho_br.png"),
            corner_bl_png: include_bytes!("../assets/corner_anos80_moranguinho_bl.png"),
        }),
        "anos80_lavanda_sonho" => Some(ThemeAssets {
            grad_png: include_bytes!("../assets/grad_anos80_lavanda_sonho.png"),
            corner_tl_png: include_bytes!("../assets/corner_anos80_lavanda_sonho_tl.png"),
            corner_tr_png: include_bytes!("../assets/corner_anos80_lavanda_sonho_tr.png"),
            corner_br_png: include_bytes!("../assets/corner_anos80_lavanda_sonho_br.png"),
            corner_bl_png: include_bytes!("../assets/corner_anos80_lavanda_sonho_bl.png"),
        }),
        "anos80_nuvem_algodao" => Some(ThemeAssets {
            grad_png: include_bytes!("../assets/grad_anos80_nuvem_algodao.png"),
            corner_tl_png: include_bytes!("../assets/corner_anos80_nuvem_algodao_tl.png"),
            corner_tr_png: include_bytes!("../assets/corner_anos80_nuvem_algodao_tr.png"),
            corner_br_png: include_bytes!("../assets/corner_anos80_nuvem_algodao_br.png"),
            corner_bl_png: include_bytes!("../assets/corner_anos80_nuvem_algodao_bl.png"),
        }),
        "anos80_menta_fresca" => Some(ThemeAssets {
            grad_png: include_bytes!("../assets/grad_anos80_menta_fresca.png"),
            corner_tl_png: include_bytes!("../assets/corner_anos80_menta_fresca_tl.png"),
            corner_tr_png: include_bytes!("../assets/corner_anos80_menta_fresca_tr.png"),
            corner_br_png: include_bytes!("../assets/corner_anos80_menta_fresca_br.png"),
            corner_bl_png: include_bytes!("../assets/corner_anos80_menta_fresca_bl.png"),
        }),
        "anos80_caderno_pautado" => Some(ThemeAssets {
            grad_png: include_bytes!("../assets/grad_anos80_caderno_pautado.png"),
            corner_tl_png: include_bytes!("../assets/corner_anos80_caderno_pautado_tl.png"),
            corner_tr_png: include_bytes!("../assets/corner_anos80_caderno_pautado_tr.png"),
            corner_br_png: include_bytes!("../assets/corner_anos80_caderno_pautado_br.png"),
            corner_bl_png: include_bytes!("../assets/corner_anos80_caderno_pautado_bl.png"),
        }),
        "anos80_carta_romantica" => Some(ThemeAssets {
            grad_png: include_bytes!("../assets/grad_anos80_carta_romantica.png"),
            corner_tl_png: include_bytes!("../assets/corner_anos80_carta_romantica_tl.png"),
            corner_tr_png: include_bytes!("../assets/corner_anos80_carta_romantica_tr.png"),
            corner_br_png: include_bytes!("../assets/corner_anos80_carta_romantica_br.png"),
            corner_bl_png: include_bytes!("../assets/corner_anos80_carta_romantica_bl.png"),
        }),
        "anos80_constelacao_pop" => Some(ThemeAssets {
            grad_png: include_bytes!("../assets/grad_anos80_constelacao_pop.png"),
            corner_tl_png: include_bytes!("../assets/corner_anos80_constelacao_pop_tl.png"),
            corner_tr_png: include_bytes!("../assets/corner_anos80_constelacao_pop_tr.png"),
            corner_br_png: include_bytes!("../assets/corner_anos80_constelacao_pop_br.png"),
            corner_bl_png: include_bytes!("../assets/corner_anos80_constelacao_pop_bl.png"),
        }),
        "anos80_jardim_encantado" => Some(ThemeAssets {
            grad_png: include_bytes!("../assets/grad_anos80_jardim_encantado.png"),
            corner_tl_png: include_bytes!("../assets/corner_anos80_jardim_encantado_tl.png"),
            corner_tr_png: include_bytes!("../assets/corner_anos80_jardim_encantado_tr.png"),
            corner_br_png: include_bytes!("../assets/corner_anos80_jardim_encantado_br.png"),
            corner_bl_png: include_bytes!("../assets/corner_anos80_jardim_encantado_bl.png"),
        }),
        "anos80_arcoiris_pastel" => Some(ThemeAssets {
            grad_png: include_bytes!("../assets/grad_anos80_arcoiris_pastel.png"),
            corner_tl_png: include_bytes!("../assets/corner_anos80_arcoiris_pastel_tl.png"),
            corner_tr_png: include_bytes!("../assets/corner_anos80_arcoiris_pastel_tr.png"),
            corner_br_png: include_bytes!("../assets/corner_anos80_arcoiris_pastel_br.png"),
            corner_bl_png: include_bytes!("../assets/corner_anos80_arcoiris_pastel_bl.png"),
        }),
        _ => None,
    }
}

pub fn build_theme_inline_attachments(_theme_name: &str) -> Vec<EmailAttachment> {
    // Papéis de carta agora usam design 100% vetorial/CSS limpo sem dependência de imagens anexadas
    Vec::new()
}

pub fn extract_theme_tag(subject: &str) -> (String, Option<String>) {
    let lower = subject.to_lowercase();
    for prefix in &["[tema:", "[theme:"] {
        if let Some(start) = lower.find(prefix) {
            if let Some(rel_end) = lower[start..].find(']') {
                let end = start + rel_end + 1;
                let tag_content = &subject[start + prefix.len()..start + rel_end];
                let theme = canonical_theme_name(tag_content.trim());
                let mut clean = subject.to_string();
                clean.replace_range(start..end, "");
                let clean = clean.split_whitespace().collect::<Vec<_>>().join(" ");
                let chosen = if theme != "default" {
                    theme.to_string()
                } else {
                    tag_content.trim().to_string()
                };
                return (clean, Some(chosen));
            }
        }
    }

    for word in subject.split_whitespace() {
        if word.starts_with('#') && word.len() > 1 {
            let potential_tag = word.trim_start_matches('#');
            let canonical = canonical_theme_name(potential_tag);
            if canonical != "default" {
                let clean = subject.replace(word, "").split_whitespace().collect::<Vec<_>>().join(" ");
                return (clean, Some(canonical.to_string()));
            }
        }
    }

    (subject.to_string(), None)
}

pub struct TemplateSelector {
    counter: AtomicUsize,
}

impl Default for TemplateSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateSelector {
    pub fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
        }
    }

    pub fn select_theme(&self, requested: Option<&str>, pool: Option<&[String]>, strategy: &str) -> String {
        if let Some(req) = requested {
            let canon = canonical_theme_name(req);
            if canon != "default" {
                return canon.to_string();
            }
            return req.to_string();
        }

        let default_pool: Vec<String> = ALL_BUILTIN_THEMES.iter().map(|s| s.to_string()).collect();
        let active_pool = pool.filter(|p| !p.is_empty()).unwrap_or(&default_pool);

        if strategy.eq_ignore_ascii_case("random") {
            use std::time::SystemTime;
            let nanos = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.subsec_nanos() as usize)
                .unwrap_or(0);
            let idx = (nanos ^ self.counter.fetch_add(1, Ordering::Relaxed)) % active_pool.len();
            active_pool[idx].clone()
        } else {
            let idx = self.counter.fetch_add(1, Ordering::Relaxed) % active_pool.len();
            active_pool[idx].clone()
        }
    }
}

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

    pub fn resolve_mjml_template(&self, theme_name: &str, templates_dir: Option<&Path>) -> String {
        // 1. Tenta carregar do diretório customizado no disco se existir
        if let Some(dir) = templates_dir {
            let candidate = if theme_name.ends_with(".mjml") {
                dir.join(theme_name)
            } else {
                dir.join(format!("{}.mjml", theme_name))
            };
            if let Ok(content) = std::fs::read_to_string(&candidate) {
                return content;
            }
        }

        // 2. Tenta carregar do diretório local ./templates se existir
        let local_candidate = if theme_name.ends_with(".mjml") {
            Path::new("templates").join(theme_name)
        } else {
            Path::new("templates").join(format!("{}.mjml", theme_name))
        };
        if let Ok(content) = std::fs::read_to_string(&local_candidate) {
            return content;
        }

        // 3. Tenta carregar dos templates embutidos
        if let Some(builtin) = get_builtin_template(theme_name) {
            return builtin.to_string();
        }

        // 4. Fallback padrão
        DEFAULT_MJML_TEMPLATE.to_string()
    }

    pub fn render_html(&self, mjml_template: &str, context: &Value) -> Result<String, TemplateError> {
        let rendered_mjml = self
            .env
            .render_str(mjml_template, context)
            .map_err(|e| TemplateError::JinjaError(e.to_string()))?;

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
    let name = sender_name.unwrap_or("Lumen Pink");
    let role = sender_role.unwrap_or("");
    let phone = sender_phone.unwrap_or("");
    let company = company_name.unwrap_or("");
    let disc = disclaimer.unwrap_or("");

    format!(
        r#"<div class="papeli-sig-long" data-papeli="signature" style="font-family: Georgia, 'Times New Roman', Palatino, serif;">
  <table cellpadding="0" cellspacing="0" style="font-family: Georgia, 'Times New Roman', Palatino, serif; font-size: 14px; line-height: 22px; color: #4b5563;">
    <tr><td style="font-family: Georgia, 'Times New Roman', Palatino, serif;"><strong>{}</strong>{}</td></tr>
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
            format!("<tr><td style=\"color: #718096; font-family: Georgia, 'Times New Roman', Palatino, serif;\">{}</td></tr>", company)
        } else {
            "".into()
        },
        if !phone.is_empty() {
            format!(
                "<tr><td style=\"color: #718096; font-family: Georgia, 'Times New Roman', Palatino, serif;\">Tel: {}</td></tr>",
                phone
            )
        } else {
            "".into()
        },
        if !disc.is_empty() {
            format!(
                "<tr><td style=\"font-size: 11px; color: #a0aec0; padding-top: 6px; font-family: Georgia, 'Times New Roman', Palatino, serif;\">{}</td></tr>",
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
