use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub enum ConfigError {
    IoError(String),
    TomlError(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "Config IO Error: {}", e),
            ConfigError::TomlError(e) => write!(f, "Config TOML Error: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_listen")]
    pub listen: String,
    #[serde(default = "default_templates_dir")]
    pub templates_dir: PathBuf,
    #[serde(default = "default_node_id")]
    pub node_id: String,
    #[serde(default)]
    pub recognized_nodes: Vec<String>,
}

fn default_listen() -> String {
    "unix:/run/papeli/papeli.sock".to_string()
}

fn default_templates_dir() -> PathBuf {
    PathBuf::from("/etc/papeli/templates")
}

fn default_node_id() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("HOST"))
        .unwrap_or_else(|_| "papeli-node".to_string())
}

impl ServerConfig {
    pub fn is_node_recognized(&self, node: &str) -> bool {
        if self.recognized_nodes.is_empty() {
            true
        } else {
            self.recognized_nodes
                .iter()
                .any(|n| n.eq_ignore_ascii_case(node))
                || self.node_id.eq_ignore_ascii_case(node)
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            listen: default_listen(),
            templates_dir: default_templates_dir(),
            node_id: default_node_id(),
            recognized_nodes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub header_title: Option<String>,
    pub template_html: Option<String>,
    pub template_txt: Option<String>,
    pub template_pool: Option<Vec<String>>,
    pub template_strategy: Option<String>,
    pub company_name: Option<String>,
    pub logo_url: Option<String>,
    pub disclaimer: Option<String>,
    pub sender_name: Option<String>,
    pub sender_role: Option<String>,
    pub sender_phone: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub default: ProfileConfig,
    #[serde(default)]
    pub domains: HashMap<String, ProfileConfig>,
    #[serde(default)]
    pub senders: HashMap<String, ProfileConfig>,
}

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub is_managed: bool,
    pub header_title: String,
    pub company_name: String,
    pub logo_url: Option<String>,
    pub template_html: Option<String>,
    pub template_txt: Option<String>,
    pub template_pool: Option<Vec<String>>,
    pub template_strategy: String,
    pub disclaimer: Option<String>,
    pub sender_name: Option<String>,
    pub sender_role: Option<String>,
    pub sender_phone: Option<String>,
    pub extra: HashMap<String, toml::Value>,
}

impl Config {
    pub fn from_toml(content: &str) -> Result<Self, ConfigError> {
        toml::from_str(content).map_err(|e| ConfigError::TomlError(e.to_string()))
    }

    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(format!("{}: {}", path.display(), e)))?;
        Self::from_toml(&content)
    }

    pub fn is_domain_managed(&self, domain: &str) -> bool {
        self.domains.contains_key(domain)
    }

    pub fn resolve(&self, sender_email: Option<&str>) -> ResolvedConfig {
        let mut header_title = self.default.header_title.clone().unwrap_or_else(|| {
            self.default
                .sender_name
                .clone()
                .unwrap_or_else(|| "Lumen Pink".to_string())
        });
        let mut company_name = self.default.company_name.clone().unwrap_or_default();
        let mut logo_url = self.default.logo_url.clone();
        let mut template_html = self.default.template_html.clone();
        let mut template_txt = self.default.template_txt.clone();
        let mut template_pool = self.default.template_pool.clone();
        let mut template_strategy = self.default.template_strategy.clone().unwrap_or_else(|| {
            if template_html.is_some() {
                "static".to_string()
            } else {
                "round-robin".to_string()
            }
        });
        if (self.default.template_strategy.as_deref() == Some("random")
            || self.default.template_strategy.as_deref() == Some("round-robin"))
            && self.default.template_html.is_none()
        {
            template_html = None;
        }
        let mut disclaimer = self.default.disclaimer.clone();
        let mut sender_name = self.default.sender_name.clone();
        let mut sender_role = self.default.sender_role.clone();
        let mut sender_phone = self.default.sender_phone.clone();
        let mut extra = self.default.extra.clone();

        let mut is_managed = false;

        if let Some(email) = sender_email {
            let email_clean = email.trim().to_lowercase();
            let domain = email_clean.split('@').nth(1).unwrap_or("");

            // 1. Cascata: Aplicar regras do Domínio
            if let Some(dom_cfg) = self.domains.get(domain) {
                is_managed = true;
                if let Some(v) = &dom_cfg.header_title {
                    header_title = v.clone();
                } else if let Some(sn) = &dom_cfg.sender_name {
                    header_title = sn.clone();
                }
                if let Some(v) = &dom_cfg.company_name {
                    company_name = v.clone();
                }
                if dom_cfg.logo_url.is_some() {
                    logo_url = dom_cfg.logo_url.clone();
                }
                if dom_cfg.template_html.is_some() {
                    template_html = dom_cfg.template_html.clone();
                }
                if dom_cfg.template_txt.is_some() {
                    template_txt = dom_cfg.template_txt.clone();
                }
                if dom_cfg.template_pool.is_some() {
                    template_pool = dom_cfg.template_pool.clone();
                }
                if let Some(strat) = &dom_cfg.template_strategy {
                    template_strategy = strat.clone();
                    if dom_cfg.template_html.is_none()
                        && (strat.eq_ignore_ascii_case("random")
                            || strat.eq_ignore_ascii_case("round-robin"))
                    {
                        template_html = None;
                    }
                }
                if dom_cfg.disclaimer.is_some() {
                    disclaimer = dom_cfg.disclaimer.clone();
                }
                if dom_cfg.sender_name.is_some() {
                    sender_name = dom_cfg.sender_name.clone();
                }
                if dom_cfg.sender_role.is_some() {
                    sender_role = dom_cfg.sender_role.clone();
                }
                if dom_cfg.sender_phone.is_some() {
                    sender_phone = dom_cfg.sender_phone.clone();
                }
                for (k, v) in &dom_cfg.extra {
                    extra.insert(k.clone(), v.clone());
                }
            }

            // 2. Cascata: Aplicar regras específicas do Remetente (sobrescreve domínio)
            if let Some(snd_cfg) = self.senders.get(&email_clean) {
                is_managed = true;
                if let Some(v) = &snd_cfg.header_title {
                    header_title = v.clone();
                } else if let Some(sn) = &snd_cfg.sender_name {
                    header_title = sn.clone();
                }
                if let Some(v) = &snd_cfg.company_name {
                    company_name = v.clone();
                }
                if snd_cfg.logo_url.is_some() {
                    logo_url = snd_cfg.logo_url.clone();
                }
                if snd_cfg.template_html.is_some() {
                    template_html = snd_cfg.template_html.clone();
                }
                if snd_cfg.template_txt.is_some() {
                    template_txt = snd_cfg.template_txt.clone();
                }
                if snd_cfg.template_pool.is_some() {
                    template_pool = snd_cfg.template_pool.clone();
                }
                if let Some(strat) = &snd_cfg.template_strategy {
                    template_strategy = strat.clone();
                    if snd_cfg.template_html.is_none()
                        && (strat.eq_ignore_ascii_case("random")
                            || strat.eq_ignore_ascii_case("round-robin"))
                    {
                        template_html = None;
                    }
                }
                if snd_cfg.disclaimer.is_some() {
                    disclaimer = snd_cfg.disclaimer.clone();
                }
                if snd_cfg.sender_name.is_some() {
                    sender_name = snd_cfg.sender_name.clone();
                }
                if snd_cfg.sender_role.is_some() {
                    sender_role = snd_cfg.sender_role.clone();
                }
                if snd_cfg.sender_phone.is_some() {
                    sender_phone = snd_cfg.sender_phone.clone();
                }
                for (k, v) in &snd_cfg.extra {
                    extra.insert(k.clone(), v.clone());
                }
            }
        }

        ResolvedConfig {
            is_managed,
            header_title,
            company_name,
            logo_url,
            template_html,
            template_txt,
            template_pool,
            template_strategy,
            disclaimer,
            sender_name,
            sender_role,
            sender_phone,
            extra,
        }
    }
}

pub struct ConfigManager {
    config: Arc<RwLock<Config>>,
    path: PathBuf,
}

impl ConfigManager {
    pub fn new(path: PathBuf) -> Result<Self, ConfigError> {
        let cfg = Config::from_file(&path)?;
        Ok(Self {
            config: Arc::new(RwLock::new(cfg)),
            path,
        })
    }

    pub fn from_config(config: Config, path: PathBuf) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            path,
        }
    }

    pub fn get_config(&self) -> Arc<RwLock<Config>> {
        Arc::clone(&self.config)
    }

    pub fn reload(&self) -> Result<(), ConfigError> {
        let new_cfg = Config::from_file(&self.path)?;
        let mut lock = self
            .config
            .write()
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        *lock = new_cfg;
        Ok(())
    }
}
