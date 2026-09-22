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
        "p", "br", "b", "strong", "i", "em", "u", "s", "strike", "a", "ul", "ol", "li",
        "blockquote", "span", "div", "table", "thead", "tbody", "tr", "th", "td", "img", "h1",
        "h2", "h3", "h4", "h5", "h6", "hr", "pre", "code", "sub", "sup",
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
    for mso_class in &["WordSection1", "WordSection2", "MsoNormal", "EmailStyle17", "MsoChpDefault"] {
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
            if let Some(open_start) = before.rfind('<') {
                if let Some(open_rel_end) = before[open_start..].find('>') {
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
            }

            if !removed {
                result.replace_range(pos..pos + footer.len(), "");
            }
        }
    }
    result.trim().to_string()
}
