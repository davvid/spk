// Copyright (c) Sony Pictures Imageworks, et al.
// SPDX-License-Identifier: Apache-2.0
// https://github.com/imageworks/spk

use rstest::rstest;

use super::{Config, RemoteConfig};
use crate::storage::RepositoryHandle;
use crate::{get_config, load_config};

#[rstest]
fn test_config_list_remote_names_empty() {
    let config = Config::default();
    assert_eq!(config.list_remote_names().len(), 0)
}

#[rstest]
fn test_config_list_remote_names() {
    let config: Config =
        serde_json::from_str(r#"{"remote": { "origin": { "address": "http://myaddress" } } }"#)
            .unwrap();
    assert_eq!(config.list_remote_names(), vec!["origin".to_string()]);
}

#[rstest]
#[tokio::test]
async fn test_config_get_remote_unknown() {
    let config = Config::default();
    config
        .get_remote("unknown")
        .await
        .expect_err("should fail to load unknown config");
}

#[rstest]
#[tokio::test]
async fn test_config_get_remote() {
    let tmpdir = tempfile::Builder::new()
        .prefix("spfs-test")
        .tempdir()
        .unwrap();
    let remote = tmpdir.path().join("remote");
    let _ = crate::storage::fs::FsRepository::create(&remote)
        .await
        .unwrap();

    let config: Config = serde_json::from_str(&format!(
        r#"{{"remote": {{ "origin": {{ "address": "file://{}" }} }} }}"#,
        &remote.to_string_lossy()
    ))
    .unwrap();
    let repo = config.get_remote("origin").await;
    assert!(repo.is_ok());
}

#[rstest]
#[case(
    r#"
{
    "remote": {
        "addressed": {
            "address": "file:/some/path"
        },
        "configured": {
            "scheme": "fs",
            "path": "/some/path"
        }
    }
}"#
)]
fn test_remote_config_or_address(#[case] source: &str) {
    let _config: Config = serde_json::from_str(source).expect("config should have loaded properly");
}

#[rstest]
fn test_make_current_updates_config() {
    let config1 = Config::default();
    config1.make_current().unwrap();

    let changed_name = "changed";

    let mut config2 = Config::default();
    config2.user.name = changed_name.to_owned();
    config2.make_current().unwrap();

    let current_config = get_config().unwrap();
    assert_eq!(current_config.user.name, changed_name);
}

#[rstest]
#[tokio::test]
async fn test_remote_config_pinned_from_address() {
    let address = url::Url::parse("http2://test.local?lazy=true&when=~10m").expect("a valid url");
    let config = RemoteConfig::from_address(address)
        .await
        .expect("can parse address with 'when' query");
    let repo = config
        .open()
        .await
        .expect("should open pinned repo address");
    assert!(
        matches!(repo, RepositoryHandle::Pinned(_)),
        "using a when query should create a pinned repo"
    )
}

static ENV_MUTEX: once_cell::sync::Lazy<std::sync::Mutex<()>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(()));

#[rstest]
#[case::single_underscores_still_works(&["SPFS_STORAGE_ROOT"], 0, &[], |config: &Config| config.storage.root.display().to_string())]
#[case::single_underscores_has_precedence(&["SPFS_STORAGE_ROOT", "SPFS_STORAGE__ROOT"], 0, &[], |config: &Config| config.storage.root.display().to_string())]
#[case::double_underscores_will_work(&["SPFS_STORAGE__ROOT"], 0, &["SPFS_STORAGE_ROOT"], |config: &Config| config.storage.root.display().to_string())]
fn test_config_env_overrides<F: Fn(&Config) -> R, R: ToString>(
    #[case] env_vars_to_set: &[&str],
    #[case] expected_index: usize,
    #[case] env_vars_to_clear: &[&str],
    #[case] get_field: F,
) {
    // Environment manipulation is not thread safe, so run these test cases
    // serially.
    let _guard = ENV_MUTEX.lock().unwrap();
    let generated_values = env_vars_to_set
        .iter()
        .map(|&var| {
            // Set each variable name to a unique value
            let value = ulid::Ulid::new().to_string();
            let orig = std::env::var_os(var);
            std::env::set_var(var, &value);
            (value, orig)
        })
        .collect::<Vec<_>>();
    let cleared_vars = env_vars_to_clear
        .iter()
        .map(|&var| {
            let orig = std::env::var_os(var);
            if orig.is_some() {
                std::env::remove_var(var);
            }
            (var, orig)
        })
        .collect::<Vec<_>>();
    let config = load_config();
    // Restore env
    for (var, orig) in cleared_vars.iter() {
        match orig {
            Some(orig) => std::env::set_var(var, orig),
            None => {}
        }
    }
    for (var, (_, orig)) in env_vars_to_set.iter().zip(generated_values.iter()) {
        match orig {
            Some(orig) => std::env::set_var(var, orig),
            None => std::env::remove_var(var),
        }
    }
    let config = config.unwrap();
    let actual = get_field(&config).to_string();
    assert_eq!(actual, generated_values[expected_index].0);
}
