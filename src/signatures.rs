use scraper::{Html, Selector};

pub struct SignatureConfig<'a> {
    pub sender_name: &'a str,
    pub company_name: &'a str,
    pub long_html: &'a str,
    pub short_html: Option<&'a str>,
    pub long_text: &'a str,
    pub short_text: Option<&'a str>,
}

pub fn default_short_html(sender_name: &str, company_name: &str) -> String {
    let label = if company_name.is_empty()
        || company_name.eq_ignore_ascii_case("lumen")
        || company_name.eq_ignore_ascii_case("baquara")
    {
        sender_name.to_string()
    } else {
        format!("{} ({})", sender_name, company_name)
    };
    format!(
        "<div class=\"papeli-sig-short\" data-papeli=\"signature\" style=\"font-family: Georgia, 'Times New Roman', Palatino, serif;\"><p style=\"color: #64748b; font-size: 12px; margin: 4px 0; font-family: Georgia, 'Times New Roman', Palatino, serif;\">— {}</p></div>",
        label
    )
}

pub fn default_short_text(sender_name: &str, company_name: &str) -> String {
    if company_name.is_empty()
        || company_name.eq_ignore_ascii_case("lumen")
        || company_name.eq_ignore_ascii_case("baquara")
    {
        format!(
            "<!--papeli:sig:start:short-->\n— {}\n<!--papeli:sig:end:short-->",
            sender_name
        )
    } else {
        format!(
            "<!--papeli:sig:start:short-->\n— {} ({})\n<!--papeli:sig:end:short-->",
            sender_name, company_name
        )
    }
}

pub fn collapse_historical_signatures_html(html: &str, fallback_short_html: &str) -> String {
    let document = Html::parse_fragment(html);
    let quote_selector = match Selector::parse("blockquote, .gmail_quote, .OutlookQuote") {
        Ok(s) => s,
        Err(_) => return html.to_string(),
    };

    let sig_long_selector = match Selector::parse(".papeli-sig-long, div[data-papeli='signature']") {
        Ok(s) => s,
        Err(_) => return html.to_string(),
    };

    // Se não há bloco de citação nem assinaturas, retorna inalterado
    if document.select(&quote_selector).next().is_none()
        || document.select(&sig_long_selector).next().is_none()
    {
        return html.to_string();
    }

    // Substituição determinística das assinaturas longas antigas
    // Localiza e substitui blocos <div class="papeli-sig-long" ...>...</div> por short_html
    let mut result = html.to_string();
    let marker = "papeli-sig-long";

    while let Some(pos) = result.find(marker) {
        // Encontra o início da tag <div
        if let Some(div_start) = result[..pos].rfind("<div") {
            // Encontra o fechamento </div> correspondente
            if let Some(div_end_offset) = find_matching_close_tag(&result[div_start..], "div") {
                let div_end = div_start + div_end_offset;
                result.replace_range(div_start..div_end, fallback_short_html);
                continue;
            }
        }
        // Fallback para evitar loop infinito se a tag for malformada
        break;
    }

    result
}

pub fn collapse_historical_signatures_text(text: &str, fallback_short_text: &str) -> String {
    let mut result = text.to_string();
    let start_marker = "<!--papeli:sig:start:long-->";
    let end_marker = "<!--papeli:sig:end:long-->";

    while let Some(start_pos) = result.find(start_marker) {
        if let Some(end_offset) = result[start_pos..].find(end_marker) {
            let end_pos = start_pos + end_offset + end_marker.len();
            result.replace_range(start_pos..end_pos, fallback_short_text);
        } else {
            break;
        }
    }

    result
}

fn find_matching_close_tag(html_fragment: &str, tag_name: &str) -> Option<usize> {
    let open_pat = format!("<{}", tag_name);
    let close_pat = format!("</{}>", tag_name);

    let mut depth = 0;
    let mut cursor = 0;

    while cursor < html_fragment.len() {
        let next_open = html_fragment[cursor..].find(&open_pat);
        let next_close = html_fragment[cursor..].find(&close_pat);

        match (next_open, next_close) {
            (Some(o), Some(c)) if o < c => {
                depth += 1;
                cursor += o + open_pat.len();
            }
            (_, Some(c)) => {
                depth -= 1;
                cursor += c + close_pat.len();
                if depth == 0 {
                    return Some(cursor);
                }
            }
            _ => break,
        }
    }

    None
}
