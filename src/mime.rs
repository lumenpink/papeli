use mail_builder::MessageBuilder;
use mail_builder::headers::raw::Raw;
use mail_parser::{MessageParser, MimeHeaders};
use std::fmt;

#[derive(Debug)]
pub enum MimeError {
    ParseError(String),
    BuildError(String),
}

impl fmt::Display for MimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MimeError::ParseError(msg) => write!(f, "MIME Parse Error: {}", msg),
            MimeError::BuildError(msg) => write!(f, "MIME Build Error: {}", msg),
        }
    }
}

impl std::error::Error for MimeError {}

#[derive(Debug, Clone, PartialEq)]
pub struct EmailAttachment {
    pub name: String,
    pub content_type: String,
    pub data: Vec<u8>,
    pub content_id: Option<String>,
    pub is_inline: bool,
}

#[derive(Debug, Clone)]
pub struct ParsedEmail {
    pub from: Option<String>,
    pub sender_name: Option<String>,
    pub sender_email: Option<String>,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub subject: Option<String>,
    pub message_id: Option<String>,
    pub date: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
    pub custom_headers: Vec<(String, String)>,

    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub is_plain_only: bool,
    pub is_html_only: bool,

    pub attachments: Vec<EmailAttachment>,
}

impl ParsedEmail {
    pub fn parse(raw_bytes: &[u8]) -> Result<Self, MimeError> {
        let msg = MessageParser::default()
            .parse(raw_bytes)
            .ok_or_else(|| MimeError::ParseError("Falha ao parsear mensagem MIME".into()))?;

        let sender_name = msg
            .from()
            .and_then(|f| f.first())
            .and_then(|addr| addr.name())
            .map(|s| s.to_string());

        let sender_email = msg
            .from()
            .and_then(|f| f.first())
            .and_then(|addr| addr.address())
            .map(|s| s.to_string());

        let from = match (&sender_name, &sender_email) {
            (Some(name), Some(email)) => Some(format!("\"{}\" <{}>", name, email)),
            (None, Some(email)) => Some(email.clone()),
            _ => None,
        };

        let mut to = Vec::new();
        if let Some(to_addrs) = msg.to() {
            for addr in to_addrs.iter() {
                if let Some(address) = addr.address() {
                    to.push(address.to_string());
                }
            }
        }

        let mut cc = Vec::new();
        if let Some(cc_addrs) = msg.cc() {
            for addr in cc_addrs.iter() {
                if let Some(address) = addr.address() {
                    cc.push(address.to_string());
                }
            }
        }

        let subject = msg.subject().map(|s| s.to_string());
        let message_id = msg.message_id().map(|s| s.to_string());
        let date = msg.date().map(|d| d.to_rfc3339());
        let in_reply_to = msg.in_reply_to().as_text().map(|s| s.to_string());

        let mut references = Vec::new();
        if let Some(refs) = msg.references().as_text_list() {
            for r in refs {
                references.push(r.to_string());
            }
        }

        let mut custom_headers = Vec::new();
        for header in msg.headers() {
            let name = header.name();
            let lower_name = name.to_lowercase();
            // Preservar cabeçalhos sem duplicar headers gerados pelo mail-builder
            if !matches!(
                lower_name.as_str(),
                "content-type"
                    | "mime-version"
                    | "content-transfer-encoding"
                    | "from"
                    | "to"
                    | "cc"
                    | "subject"
                    | "message-id"
                    | "in-reply-to"
                    | "references"
                    | "date"
            ) && let Some(val) = header.value().as_text()
            {
                custom_headers.push((name.to_string(), val.to_string()));
            }
        }

        let has_explicit_html = msg.parts.iter().any(|p| p.is_text_html());
        let has_explicit_text = msg.parts.iter().any(|p| p.is_text() && !p.is_text_html())
            || msg.is_content_type("text", "plain")
            || !has_explicit_html;

        let is_plain_only = !has_explicit_html;
        let is_html_only = has_explicit_html && !has_explicit_text;

        let body_text = if is_html_only {
            None
        } else {
            msg.body_text(0).map(|s| s.to_string())
        };

        let body_html = if is_plain_only {
            None
        } else {
            msg.body_html(0).map(|s| s.to_string())
        };

        let mut attachments = Vec::new();
        for part in msg.attachments() {
            let name = part
                .attachment_name()
                .unwrap_or("unnamed_attachment")
                .to_string();
            let ctype = part
                .content_type()
                .map(|c| {
                    if let Some(sub) = c.subtype() {
                        format!("{}/{}", c.ctype(), sub)
                    } else {
                        c.ctype().to_string()
                    }
                })
                .unwrap_or_else(|| "application/octet-stream".to_string());

            let content_id = part.content_id().map(|cid| cid.to_string());
            let is_inline = part
                .content_disposition()
                .map(|d| d.is_inline())
                .unwrap_or(false)
                || content_id.is_some();
            let data = part.contents().to_vec();

            attachments.push(EmailAttachment {
                name,
                content_type: ctype,
                data,
                content_id,
                is_inline,
            });
        }

        Ok(Self {
            from,
            sender_name,
            sender_email,
            to,
            cc,
            subject,
            message_id,
            date,
            in_reply_to,
            references,
            custom_headers,
            body_text,
            body_html,
            is_plain_only,
            is_html_only,
            attachments,
        })
    }

    pub fn reconstruct(&self, new_text: &str, new_html: &str) -> Result<Vec<u8>, MimeError> {
        self.reconstruct_with_node(new_text, new_html, None)
    }

    pub fn reconstruct_with_node(
        &self,
        new_text: &str,
        new_html: &str,
        node_id: Option<&str>,
    ) -> Result<Vec<u8>, MimeError> {
        let mut builder = MessageBuilder::new();

        match (&self.sender_name, &self.sender_email) {
            (Some(name), Some(email)) => {
                builder = builder.from((name.as_str(), email.as_str()));
            }
            (None, Some(email)) => {
                builder = builder.from(email.as_str());
            }
            _ => {}
        }

        for to in &self.to {
            builder = builder.to(to.as_str());
        }

        for cc in &self.cc {
            builder = builder.cc(cc.as_str());
        }

        if let Some(subj) = &self.subject {
            builder = builder.subject(subj.as_str());
        }

        if let Some(msg_id) = &self.message_id {
            builder = builder.message_id(msg_id.as_str());
        }

        if let Some(in_reply) = &self.in_reply_to {
            builder = builder.in_reply_to(in_reply.as_str());
        }

        if !self.references.is_empty() {
            let refs_slice: Vec<&str> = self.references.iter().map(|s| s.as_str()).collect();
            builder = builder.references(refs_slice);
        }

        // Custom headers (ex: X-Mailer, User-Agent, etc.)
        for (name, value) in &self.custom_headers {
            if !name.eq_ignore_ascii_case("x-papeli-processed")
                && !name.eq_ignore_ascii_case("x-papeli-wrapped-by")
            {
                builder = builder.header(name.as_str(), Raw::new(value.as_str()));
            }
        }
        builder = builder.header("X-Papeli-Processed", Raw::new("true"));
        if let Some(node) = node_id {
            builder = builder.header("X-Papeli-Wrapped-By", Raw::new(node));
        }

        use mail_builder::mime::MimePart;

        // Separa anexos comuns (arquivos) de imagens inline referenciadas por CID
        let (inlines, regular_attachments): (Vec<_>, Vec<_>) =
            self.attachments.iter().partition(|a| a.is_inline);

        let text_part = MimePart::new("text/plain", new_text);
        let html_part = MimePart::new("text/html", new_html).inline();
        let alt_part = MimePart::new("multipart/alternative", vec![text_part, html_part]);

        let related_or_alt = if inlines.is_empty() {
            alt_part
        } else {
            let mut related_parts = Vec::with_capacity(inlines.len() + 1);
            related_parts.push(alt_part);
            for att in inlines {
                let cid = att.content_id.as_deref().unwrap_or(att.name.as_str());
                let part = MimePart::new(att.content_type.as_str(), att.data.clone())
                    .inline()
                    .cid(cid);
                related_parts.push(part);
            }
            MimePart::new("multipart/related", related_parts)
        };

        let body_part = if regular_attachments.is_empty() {
            related_or_alt
        } else {
            let mut mixed_parts = Vec::with_capacity(regular_attachments.len() + 1);
            mixed_parts.push(related_or_alt);
            for att in regular_attachments {
                let part = MimePart::new(att.content_type.as_str(), att.data.clone())
                    .attachment(att.name.as_str());
                mixed_parts.push(part);
            }
            MimePart::new("multipart/mixed", mixed_parts)
        };

        builder = builder.body(body_part);

        builder
            .write_to_vec()
            .map_err(|e| MimeError::BuildError(e.to_string()))
    }
}
