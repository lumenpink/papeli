use papeli::config::{Config, ConfigManager};
use std::fs;

const SAMPLE_TOML: &str = r#"
[server]
listen = "unix:/run/papeli/papeli.sock"
templates_dir = "/etc/papeli/templates"

[default]
company_name = "Organização Geral"
logo_url = "https://empresa.com/logo-padrao.png"
template_html = "default.mjml"
template_txt = "default.txt.j2"
disclaimer = "Mensagem confidencial geral."

[domains."acme.com"]
company_name = "ACME Corporation"
logo_url = "https://acme.com/logo.png"
disclaimer = "Esta mensagem pertence exclusivamente à ACME."

[senders."diretoria@acme.com"]
sender_name = "Carlos Silva"
sender_role = "Diretor Executivo"
sender_phone = "+55 11 99999-9999"
"#;

#[test]
fn test_config_hierarchical_resolution() {
    let cfg = Config::from_toml(SAMPLE_TOML).expect("Falha no parse do TOML");

    // 1. Remetente externo / não gerenciado
    let unmanaged = cfg.resolve(Some("contato@gmail.com"));
    assert!(!unmanaged.is_managed);
    assert_eq!(unmanaged.company_name, "Organização Geral");
    assert_eq!(
        unmanaged.logo_url.as_deref(),
        Some("https://empresa.com/logo-padrao.png")
    );
    assert_eq!(unmanaged.sender_name, None);

    // 2. Colaborador comum do domínio acme.com (herda regras do domínio)
    let domain_user = cfg.resolve(Some("colaborador@acme.com"));
    assert!(domain_user.is_managed);
    assert_eq!(domain_user.company_name, "ACME Corporation");
    assert_eq!(
        domain_user.logo_url.as_deref(),
        Some("https://acme.com/logo.png")
    );
    assert_eq!(
        domain_user.disclaimer.as_deref(),
        Some("Esta mensagem pertence exclusivamente à ACME.")
    );
    assert_eq!(domain_user.sender_name, None); // não tem override de remetente

    // 3. Remetente específico (diretoria@acme.com) sobrescreve domínio
    let director = cfg.resolve(Some("diretoria@acme.com"));
    assert!(director.is_managed);
    assert_eq!(director.company_name, "ACME Corporation"); // herdou do domínio
    assert_eq!(director.sender_name.as_deref(), Some("Carlos Silva"));
    assert_eq!(director.sender_role.as_deref(), Some("Diretor Executivo"));
    assert_eq!(director.sender_phone.as_deref(), Some("+55 11 99999-9999"));
}

#[test]
fn test_config_manager_live_reload() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("papeli.toml");

    fs::write(&config_path, SAMPLE_TOML).unwrap();

    let manager = ConfigManager::new(config_path.clone()).expect("Falha ao criar ConfigManager");

    // Valor inicial
    {
        let lock = manager.get_config();
        let cfg = lock.read().unwrap();
        let res = cfg.resolve(Some("user@acme.com"));
        assert_eq!(res.company_name, "ACME Corporation");
    }

    // Atualiza arquivo em disco
    let updated_toml = SAMPLE_TOML.replace("ACME Corporation", "ACME Global Holdings");
    fs::write(&config_path, updated_toml).unwrap();

    // Executa reload
    manager.reload().expect("Falha ao recarregar config");

    // Verifica valor atualizado imediatamente
    {
        let lock = manager.get_config();
        let cfg = lock.read().unwrap();
        let res = cfg.resolve(Some("user@acme.com"));
        assert_eq!(res.company_name, "ACME Global Holdings");
    }
}

#[test]
fn test_server_config_node_recognition() {
    let toml_empty = r#"
    [server]
    node_id = "master-node"
    "#;
    let cfg_empty = Config::from_toml(toml_empty).unwrap();
    // Quando lista de nós reconhecidos estiver vazia, qualquer nó é aceito por padrão
    assert!(cfg_empty.server.is_node_recognized("any-node"));
    assert!(cfg_empty.server.is_node_recognized("master-node"));

    let toml_cluster = r#"
    [server]
    node_id = "node-1"
    recognized_nodes = ["node-1", "node-2", "node-3"]
    "#;
    let cfg_cluster = Config::from_toml(toml_cluster).unwrap();
    assert!(cfg_cluster.server.is_node_recognized("node-1"));
    assert!(cfg_cluster.server.is_node_recognized("NODE-2")); // case insensitive
    assert!(cfg_cluster.server.is_node_recognized("node-3"));
    assert!(!cfg_cluster.server.is_node_recognized("rogue-node"));
}
