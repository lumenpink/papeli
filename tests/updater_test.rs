use papeli::update::{
    get_config_path, get_distribution_endpoints, get_download_url, get_state_path,
    is_version_pinned, resolve_channel, save_update_config, semver_is_newer, UpdateConfig,
};
use std::env;
use tempfile::tempdir;

#[test]
fn test_xdg_path_resolution_with_custom_env() {
    let tmp = tempdir().unwrap();
    let config_dir = tmp.path().join("my_config");
    let state_dir = tmp.path().join("my_state");

    unsafe {
        env::set_var("XDG_CONFIG_HOME", &config_dir);
        env::set_var("XDG_STATE_HOME", &state_dir);
    }

    let config_path = get_config_path();
    let state_path = get_state_path();

    assert_eq!(config_path, config_dir.join("papeli").join("config.toml"));
    assert_eq!(
        state_path,
        state_dir.join("papeli").join("update_check.json")
    );

    unsafe {
        env::remove_var("XDG_CONFIG_HOME");
        env::remove_var("XDG_STATE_HOME");
    }
}

#[test]
fn test_channel_resolution_precedence() {
    let tmp = tempdir().unwrap();
    let cfg_path = tmp.path().join("config.toml");

    // 1. Padrão sem nada: "stable"
    assert_eq!(resolve_channel(None, None, Some(&cfg_path)), "stable");

    // 2. Definido no config.toml
    let cfg = UpdateConfig {
        channel: "nightly".to_string(),
        ..Default::default()
    };
    save_update_config(&cfg, Some(&cfg_path)).unwrap();
    assert_eq!(resolve_channel(None, None, Some(&cfg_path)), "nightly");

    // 3. Variável de ambiente sobrescreve config
    unsafe {
        env::set_var("PAPELI_CHANNEL", "stable");
    }
    assert_eq!(resolve_channel(None, None, Some(&cfg_path)), "stable");

    // 4. Flag de linha de comando tem maior precedência
    assert_eq!(
        resolve_channel(Some("nightly"), None, Some(&cfg_path)),
        "nightly"
    );

    unsafe {
        env::remove_var("PAPELI_CHANNEL");
    }
}

#[test]
fn test_pinning_logic_and_safeguards() {
    let cfg = UpdateConfig::default();
    assert!(!is_version_pinned(&cfg, "0.1.0"));

    let cfg_pinned = UpdateConfig {
        pinned: true,
        ..Default::default()
    };
    assert!(is_version_pinned(&cfg_pinned, "0.1.0"));

    let cfg_pin_ver = UpdateConfig {
        pinned: true,
        pin: "0.0.9".to_string(),
        ..Default::default()
    };
    assert!(is_version_pinned(&cfg_pin_ver, "0.1.0"));

    let cfg_unpinned_with_pin = UpdateConfig {
        pinned: false,
        pin: "0.0.9".to_string(),
        ..Default::default()
    };
    assert!(is_version_pinned(&cfg_unpinned_with_pin, "0.1.0"));
}

#[test]
fn test_semver_comparisons() {
    assert!(semver_is_newer("0.2.0", "0.1.0"));
    assert!(semver_is_newer("v1.0.0", "0.9.9"));
    assert!(semver_is_newer(
        "0.1.0-dev.60+20261028",
        "0.1.0-dev.50+20261027"
    ));
    assert!(!semver_is_newer("0.1.0", "0.1.0"));
    assert!(!semver_is_newer("0.1.0", "0.2.0"));
}

#[test]
fn test_banca_distribution_endpoints() {
    let (stable_bin, nightly_bin, version_url) = get_distribution_endpoints("stable");
    assert_eq!(
        stable_bin,
        "https://banca.2lp.in/downloads/papeli/stable-latest"
    );
    assert_eq!(
        nightly_bin,
        "https://banca.2lp.in/downloads/papeli/nightly-latest"
    );
    assert_eq!(
        version_url,
        "https://banca.2lp.in/downloads/papeli/stable-latest.version"
    );

    let (_, _, nightly_ver_url) = get_distribution_endpoints("nightly");
    assert_eq!(
        nightly_ver_url,
        "https://banca.2lp.in/downloads/papeli/nightly-latest.version"
    );

    let download_url = get_download_url("stable", Some("0.1.0"));
    assert_eq!(
        download_url,
        "https://banca.2lp.in/downloads/papeli/papeli-0.1.0-linux-x86_64"
    );
}

#[test]
fn test_perform_atomic_update_and_replacement() {
    let tmp = tempdir().unwrap();
    let target = tmp.path().join("bin").join("papeli");

    let initial_bytes = b"#!/bin/sh\necho v1\n";
    papeli::update::perform_atomic_update(&target, initial_bytes).unwrap();
    assert_eq!(std::fs::read(&target).unwrap(), initial_bytes);

    let updated_bytes = b"#!/bin/sh\necho v2\n";
    papeli::update::perform_atomic_update(&target, updated_bytes).unwrap();
    assert_eq!(std::fs::read(&target).unwrap(), updated_bytes);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&target).unwrap().permissions().mode();
        assert_eq!(mode & 0o111, 0o111);
    }
}

#[test]
fn test_state_cache_roundtrip() {
    let tmp = tempdir().unwrap();
    let state_file = tmp.path().join("state.json");

    let state = papeli::update::UpdateState {
        last_check_timestamp: 1729411200,
        latest_remote_version: "0.2.0".to_string(),
        channel: "nightly".to_string(),
    };

    papeli::update::save_update_state(&state, Some(&state_file)).unwrap();
    let loaded = papeli::update::load_update_state(Some(&state_file));

    assert_eq!(loaded.last_check_timestamp, 1729411200);
    assert_eq!(loaded.latest_remote_version, "0.2.0");
    assert_eq!(loaded.channel, "nightly");
}
