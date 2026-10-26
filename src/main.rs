use papeli::config::{Config, ConfigManager};
use papeli::milter::{EmailProcessor, run_milter_listener};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut config_path = PathBuf::from("papeli.toml");

    let mut i = 1;
    while i < args.len() {
        if (args[i] == "--config" || args[i] == "-c") && i + 1 < args.len() {
            config_path = PathBuf::from(&args[i + 1]);
            i += 1;
        }
        i += 1;
    }

    println!("====================================================");
    println!(" ★ Papéli - Filtro Milter de Papel de Carta em Rust ★");
    println!("====================================================");

    let config = if config_path.exists() {
        println!("Carregando configuração de: {}", config_path.display());
        Config::from_file(&config_path)?
    } else {
        println!(
            "Aviso: {} não encontrado. Usando configuração padrão.",
            config_path.display()
        );
        let mut cfg = Config::default();
        cfg.default.company_name = Some("Papéli".into());
        cfg
    };

    let listen = config.server.listen.clone();
    let manager = ConfigManager::from_config(config, config_path.clone());
    let processor = Arc::new(EmailProcessor::new(manager.get_config()));

    println!("Servidor Milter iniciando na escuta: {}", listen);
    println!("Pronto para interceptar e-mails de saída do Postfix.");
    println!("Pressione Ctrl+C para encerrar.");

    run_milter_listener(&listen, processor)?;

    Ok(())
}
