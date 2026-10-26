use ammonia::Builder;
use scraper::{Html, Selector};
use std::collections::{HashMap, HashSet};

const DEVICE_FOOTERS: &[&str] = &[
    "Enviado do meu iPhone",
    "Sent from my iPhone",
    "Enviado do meu iPad",
    "Sent from my iPad",
    "Enviado do meu smartphone Samsung Galaxy",
    "Enviado pelo meu smartphone Samsung",
    "Sent from my Galaxy",
    "Obter o Outlook para iOS",
    "Obter o Outlook para Android",
    "Enviado do Outlook para iOS",
    "Enviado do Outlook para Android",
    "Get Outlook for iOS",
    "Get Outlook for Android",
];

pub fn clean_plain_text(text: &str) -> String {
    let mut lines: Vec<&str> = text.lines().collect();

    // Remove rodapés automáticos de celulares no final da mensagem
    while let Some(last_line) = lines.last() {
        let trimmed = last_line.trim();
        if trimmed.is_empty() {
            lines.pop();
            continue;
        }

        let is_device_footer = DEVICE_FOOTERS
            .iter()
            .any(|footer| trimmed.eq_ignore_ascii_case(footer));

        if is_device_footer {
            lines.pop();
        } else {
            break;
        }
    }

    lines.join("\n").trim().to_string()
}

use pulldown_cmark::{Options, Parser, html};

#[derive(Debug, Clone)]
pub struct ThemePalette {
    pub emoji: &'static str,
    pub primary: &'static str,
    pub accent: &'static str,
    pub header_bg: &'static str,
    pub border_soft: &'static str,
}

pub fn get_theme_palette(theme_name: &str) -> ThemePalette {
    let canon = crate::templates::canonical_theme_name(theme_name);
    match canon {
        "anos80_rosinha_frufru" => ThemePalette {
            emoji: "🎀",
            primary: "#be185d",
            accent: "#f472b6",
            header_bg: "rgba(244, 114, 182, 0.18)",
            border_soft: "#fbcfe8",
        },
        "anos80_moranguinho" => ThemePalette {
            emoji: "🍓",
            primary: "#e11d48",
            accent: "#fb7185",
            header_bg: "rgba(251, 113, 133, 0.18)",
            border_soft: "#fecdd3",
        },
        "anos80_lavanda_sonho" => ThemePalette {
            emoji: "🪻",
            primary: "#6d28d9",
            accent: "#a78bfa",
            header_bg: "rgba(167, 139, 250, 0.18)",
            border_soft: "#ddd6fe",
        },
        "anos80_nuvem_algodao" => ThemePalette {
            emoji: "☁️",
            primary: "#0284c7",
            accent: "#38bdf8",
            header_bg: "rgba(56, 189, 248, 0.18)",
            border_soft: "#bae6fd",
        },
        "anos80_menta_fresca" => ThemePalette {
            emoji: "🍃",
            primary: "#16a34a",
            accent: "#4ade80",
            header_bg: "rgba(74, 222, 128, 0.18)",
            border_soft: "#bbf7d0",
        },
        "anos80_caderno_pautado" => ThemePalette {
            emoji: "✏️",
            primary: "#334155",
            accent: "#94a3b8",
            header_bg: "rgba(148, 163, 184, 0.18)",
            border_soft: "#e2e8f0",
        },
        "anos80_carta_romantica" => ThemePalette {
            emoji: "💌",
            primary: "#b45309",
            accent: "#f59e0b",
            header_bg: "rgba(245, 158, 11, 0.18)",
            border_soft: "#fde68a",
        },
        "anos80_constelacao_pop" => ThemePalette {
            emoji: "✨",
            primary: "#7e22ce",
            accent: "#c084fc",
            header_bg: "rgba(192, 132, 252, 0.18)",
            border_soft: "#f3e8ff",
        },
        "anos80_jardim_encantado" => ThemePalette {
            emoji: "🌸",
            primary: "#ea580c",
            accent: "#fb923c",
            header_bg: "rgba(251, 146, 60, 0.18)",
            border_soft: "#fed7aa",
        },
        "anos80_arcoiris_pastel" => ThemePalette {
            emoji: "🌈",
            primary: "#a21caf",
            accent: "#e879f9",
            header_bg: "rgba(232, 121, 249, 0.18)",
            border_soft: "#fae8ff",
        },
        _ => ThemePalette {
            emoji: "💌",
            primary: "#be185d",
            accent: "#f472b6",
            header_bg: "rgba(244, 114, 182, 0.15)",
            border_soft: "#fbcfe8",
        },
    }
}

pub fn plain_text_to_html(text: &str, theme_name: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let palette = get_theme_palette(theme_name);

    // Injeta emoji do tema nos títulos Markdown se ainda não houver emoji no título
    let mut processed_lines = Vec::new();
    for line in trimmed.lines() {
        let line_trimmed = line.trim();
        if line_trimmed.starts_with('#') {
            let heading_level = line_trimmed.chars().take_while(|&c| c == '#').count();
            let title_text = line_trimmed[heading_level..].trim();
            let starts_with_emoji = title_text
                .chars()
                .next()
                .is_some_and(|c| !c.is_alphanumeric() && !c.is_ascii());
            if !starts_with_emoji && !title_text.is_empty() {
                let hashes = &line_trimmed[..heading_level];
                processed_lines.push(format!("{} {} {}", hashes, palette.emoji, title_text));
                continue;
            }
        }
        processed_lines.push(line.to_string());
    }
    let text_with_emojis = processed_lines.join("\n");

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(&text_with_emojis, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    style_markdown_html(&html_output, &palette)
}

fn style_markdown_html(raw_html: &str, palette: &ThemePalette) -> String {
    let table_tag = format!(
        "<table border=\"1\" cellpadding=\"8\" cellspacing=\"0\" style=\"width: 100%; border-collapse: collapse; margin: 18px 0; font-size: 13px; border: 1px solid {};\">",
        palette.border_soft
    );
    let th_style = format!(
        "background-color: {}; color: {}; font-weight: bold; padding: 10px 8px; border: 1px solid {};",
        palette.header_bg, palette.primary, palette.border_soft
    );
    let td_style = format!("padding: 8px; border: 1px solid {};", palette.border_soft);
    let bq_tag = format!(
        "<blockquote style=\"margin: 14px 0; padding: 10px 16px; border-left: 3px solid {}; background: {}; color: #4b5563; font-style: italic;\">",
        palette.accent, palette.header_bg
    );
    let h1_tag = format!(
        "<h1 style=\"color: {}; font-size: 22px; font-weight: bold; margin: 22px 0 10px 0; line-height: 1.3; border-bottom: 2px solid {}; padding-bottom: 6px;\">",
        palette.primary, palette.border_soft
    );
    let h2_tag = format!(
        "<h2 style=\"color: {}; font-size: 18px; font-weight: bold; margin: 20px 0 8px 0; line-height: 1.3; border-bottom: 2px solid {}; padding-bottom: 5px;\">",
        palette.primary, palette.border_soft
    );
    let h3_tag = format!(
        "<h3 style=\"color: {}; font-size: 16px; font-weight: bold; margin: 16px 0 6px 0; line-height: 1.3;\">",
        palette.primary
    );
    let hr_tag = format!(
        "<hr style=\"border: none; border-top: 1px dashed {}; margin: 24px 0;\" />",
        palette.accent
    );
    let code_tag = format!(
        "<code style=\"background-color: {}; color: {}; padding: 2px 5px; border-radius: 4px; font-family: monospace; font-size: 13px;\">",
        palette.header_bg, palette.primary
    );

    raw_html
        .replace("<table>", &table_tag)
        .replace("<th style=\"", &format!("<th style=\"{} ", th_style))
        .replace("<th>", &format!("<th style=\"{}\">", th_style))
        .replace("<td style=\"", &format!("<td style=\"{} ", td_style))
        .replace("<td>", &format!("<td style=\"{}\">", td_style))
        .replace("<blockquote>", &bq_tag)
        .replace("<p>", "<p style=\"margin: 0 0 1em 0; line-height: 1.6;\">")
        .replace("<h1>", &h1_tag)
        .replace("<h2>", &h2_tag)
        .replace("<h3>", &h3_tag)
        .replace("<code>", &code_tag)
        .replace(
            "<pre>",
            "<pre style=\"background-color: #f8fafc; border: 1px solid #e2e8f0; border-radius: 6px; padding: 12px; overflow-x: auto; font-family: monospace; font-size: 13px; line-height: 1.4;\">",
        )
        .replace("<hr />", &hr_tag)
        .replace("<hr>", &hr_tag)
}

pub fn sanitize_html(raw_html: &str) -> String {
    // 1. Se contiver tag <body>, extrair apenas o conteúdo interno de <body>
    let document = Html::parse_document(raw_html);
    let body_selector = Selector::parse("body").unwrap();

    let content_to_clean = if let Some(body) = document.select(&body_selector).next() {
        body.inner_html()
    } else {
        raw_html.to_string()
    };

    // 2. Configurar o sanitizador estrito com Ammonia
    let mut allowed_tags = HashSet::new();
    for tag in &[
        "p",
        "br",
        "b",
        "strong",
        "i",
        "em",
        "u",
        "s",
        "strike",
        "a",
        "ul",
        "ol",
        "li",
        "blockquote",
        "span",
        "div",
        "table",
        "thead",
        "tbody",
        "tr",
        "th",
        "td",
        "img",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "hr",
        "pre",
        "code",
        "sub",
        "sup",
    ] {
        allowed_tags.insert(*tag);
    }

    let mut tag_attributes = HashMap::new();
    let mut a_attrs = HashSet::new();
    a_attrs.insert("href");
    a_attrs.insert("title");
    a_attrs.insert("target");
    tag_attributes.insert("a", a_attrs);

    let mut img_attrs = HashSet::new();
    img_attrs.insert("src");
    img_attrs.insert("alt");
    img_attrs.insert("title");
    img_attrs.insert("width");
    img_attrs.insert("height");
    tag_attributes.insert("img", img_attrs);

    let mut table_attrs = HashSet::new();
    table_attrs.insert("width");
    table_attrs.insert("cellpadding");
    table_attrs.insert("cellspacing");
    table_attrs.insert("border");
    table_attrs.insert("align");
    tag_attributes.insert("table", table_attrs);

    let mut td_attrs = HashSet::new();
    td_attrs.insert("align");
    td_attrs.insert("valign");
    td_attrs.insert("width");
    td_attrs.insert("colspan");
    td_attrs.insert("rowspan");
    tag_attributes.insert("td", td_attrs);

    let mut generic_attrs = HashSet::new();
    generic_attrs.insert("class");
    generic_attrs.insert("style");
    generic_attrs.insert("dir");
    generic_attrs.insert("data-papeli");
    generic_attrs.insert("data-sender");

    let mut builder = Builder::new();
    builder
        .tags(allowed_tags)
        .tag_attributes(tag_attributes)
        .generic_attributes(generic_attrs)
        .url_schemes(HashSet::from(["http", "https", "mailto", "cid", "tel"]))
        .strip_comments(true);

    let cleaned = builder.clean(&content_to_clean).to_string();

    // 3. Remover classes MSO e WordSection legadas do Outlook
    let mut cleaned = cleaned;
    for mso_class in &[
        "WordSection1",
        "WordSection2",
        "MsoNormal",
        "EmailStyle17",
        "MsoChpDefault",
    ] {
        cleaned = cleaned.replace(mso_class, "");
    }

    // 4. Remover nós de rodapé móvel no HTML
    remove_device_footers_html(&cleaned)
}

fn remove_device_footers_html(html: &str) -> String {
    let mut result = html.to_string();
    for footer in DEVICE_FOOTERS {
        while let Some(pos) = result.find(footer) {
            let before = &result[..pos];
            let after = &result[pos + footer.len()..];

            let mut removed = false;
            if let Some(open_start) = before.rfind('<')
                && let Some(open_rel_end) = before[open_start..].find('>')
            {
                let open_end = open_start + open_rel_end + 1;
                let tag_open = &before[open_start..open_end];
                let tag_name = tag_open
                    .trim_start_matches('<')
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_end_matches('>');

                let close_tag = format!("</{}>", tag_name);
                if let Some(close_rel) = after.find(&close_tag) {
                    // Verifica se não há tags intermediárias
                    let between = &after[..close_rel];
                    if !between.contains('<') {
                        let close_end = pos + footer.len() + close_rel + close_tag.len();
                        result.replace_range(open_start..close_end, "");
                        removed = true;
                    }
                }
            }

            if !removed {
                result.replace_range(pos..pos + footer.len(), "");
            }
        }
    }
    result.trim().to_string()
}
