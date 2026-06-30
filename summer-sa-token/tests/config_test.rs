use summer_sa_token::{
    LogoutMode, LogoutRange, ReplacedLoginExitMode, ReplacedRange, SaTokenConfig, TokenStyle,
};

fn parse_config(toml: &str) -> SaTokenConfig {
    toml::from_str(toml).expect("config should parse")
}

#[test]
fn config_can_be_built_directly() {
    let config = SaTokenConfig {
        token_name: "Authorization".to_string(),
        timeout: 86400,
        active_timeout: -1,
        dynamic_active_timeout: false,
        auto_renew: true,
        is_concurrent: true,
        is_share: false,
        token_style: TokenStyle::Uuid,
        is_log: false,
        is_read_cookie: true,
        is_read_header: true,
        is_read_body: true,
        jwt_secret_key: None,
        jwt_algorithm: Some("HS256".to_string()),
        jwt_issuer: None,
        jwt_audience: None,
        jwt_fallback_on_error: true,
        enable_nonce: false,
        nonce_timeout: -1,
        enable_refresh_token: false,
        refresh_token_timeout: 604800,
        storage_key_prefix: "sa:".to_string(),
        max_login_count: -1,
        overflow_logout_mode: LogoutMode::Logout,
        replaced_login_exit_mode: ReplacedLoginExitMode::OldDevice,
        replaced_range: ReplacedRange::CurrDeviceType,
        right_now_create_token_session: false,
        token_session_check_login: true,
        logout_range: LogoutRange::Token,
        is_logout_keep_token_session: false,
    };

    assert_eq!(config.token_name, "Authorization");
    assert_eq!(config.timeout, 86400);
    assert!(config.auto_renew);
    assert!(config.is_concurrent);
}

#[test]
fn default_config_matches_core_defaults() {
    let config = SaTokenConfig::default();

    assert_eq!(config.token_name, "sa-token");
    assert_eq!(config.timeout, 2592000);
    assert_eq!(config.active_timeout, -1);
    assert!(!config.dynamic_active_timeout);
    assert!(config.auto_renew);
    assert!(config.is_concurrent);
    assert!(!config.is_share);
    assert!(matches!(config.token_style, TokenStyle::Uuid));
    assert!(config.is_read_cookie);
    assert!(config.is_read_header);
    assert!(config.is_read_body);
    assert_eq!(config.storage_key_prefix, "sa:");
    assert_eq!(config.max_login_count, -1);
    assert!(matches!(config.overflow_logout_mode, LogoutMode::Logout));
    assert!(matches!(
        config.replaced_login_exit_mode,
        ReplacedLoginExitMode::OldDevice
    ));
    assert!(matches!(
        config.replaced_range,
        ReplacedRange::CurrDeviceType
    ));
    assert!(!config.right_now_create_token_session);
    assert!(config.token_session_check_login);
    assert!(matches!(config.logout_range, LogoutRange::Token));
    assert!(!config.is_logout_keep_token_session);
}

#[test]
fn toml_config_overrides_defaults() {
    let config = parse_config(
        r#"
        token_name = "Authorization"
        timeout = 3600
        dynamic_active_timeout = true
        auto_renew = true
        is_concurrent = false
        token_style = "Uuid"
        storage_key_prefix = "demo:"
        max_login_count = 3
        overflow_logout_mode = "KickOut"
        replaced_login_exit_mode = "NewDevice"
        replaced_range = "AllDeviceType"
        right_now_create_token_session = true
        token_session_check_login = false
        logout_range = "Account"
        is_logout_keep_token_session = true
    "#,
    );

    assert_eq!(config.token_name, "Authorization");
    assert_eq!(config.timeout, 3600);
    assert!(config.dynamic_active_timeout);
    assert!(config.auto_renew);
    assert!(!config.is_concurrent);
    assert_eq!(config.storage_key_prefix, "demo:");
    assert_eq!(config.max_login_count, 3);
    assert!(matches!(config.overflow_logout_mode, LogoutMode::KickOut));
    assert!(matches!(
        config.replaced_login_exit_mode,
        ReplacedLoginExitMode::NewDevice
    ));
    assert!(matches!(
        config.replaced_range,
        ReplacedRange::AllDeviceType
    ));
    assert!(config.right_now_create_token_session);
    assert!(!config.token_session_check_login);
    assert!(matches!(config.logout_range, LogoutRange::Account));
    assert!(config.is_logout_keep_token_session);
}

#[test]
fn minimal_toml_uses_defaults_for_missing_fields() {
    let config = parse_config(
        r#"
        token_name = "X-Token"
    "#,
    );

    assert_eq!(config.token_name, "X-Token");
    assert_eq!(config.timeout, 2592000);
}

#[test]
fn jwt_options_deserialize_from_toml() {
    let config = parse_config(
        r#"
        token_name = "Authorization"
        token_style = "Jwt"
        jwt_secret_key = "my-secret-key"
        jwt_algorithm = "HS512"
        jwt_issuer = "my-app"
    "#,
    );

    assert!(matches!(config.token_style, TokenStyle::Jwt));
    assert_eq!(config.jwt_secret_key, Some("my-secret-key".to_string()));
    assert_eq!(config.jwt_algorithm, Some("HS512".to_string()));
    assert_eq!(config.jwt_issuer, Some("my-app".to_string()));
}

#[test]
fn storage_key_prefix_deserializes_from_toml() {
    let config = parse_config(
        r#"
        token_name = "Authorization"
        storage_key_prefix = "demo:"
    "#,
    );

    assert_eq!(config.storage_key_prefix, "demo:");
}

#[test]
fn config_is_cloneable() {
    let config = SaTokenConfig {
        token_name: "TestToken".to_string(),
        timeout: 7200,
        ..Default::default()
    };

    let cloned = config.clone();
    assert_eq!(config.token_name, cloned.token_name);
    assert_eq!(config.timeout, cloned.timeout);
}

#[test]
fn token_style_variants_deserialize() {
    #[derive(serde::Deserialize)]
    struct StyleOnly {
        token_style: TokenStyle,
    }

    let cases = [
        (r#"token_style = "Uuid""#, TokenStyle::Uuid),
        (r#"token_style = "Random32""#, TokenStyle::Random32),
        (r#"token_style = "Random64""#, TokenStyle::Random64),
        (r#"token_style = "Random128""#, TokenStyle::Random128),
        (r#"token_style = "Jwt""#, TokenStyle::Jwt),
        (r#"token_style = "Hash""#, TokenStyle::Hash),
        (r#"token_style = "Timestamp""#, TokenStyle::Timestamp),
        (r#"token_style = "Tik""#, TokenStyle::Tik),
    ];

    for (toml, expected) in cases {
        let style = toml::from_str::<StyleOnly>(toml)
            .expect("token style should parse")
            .token_style;
        let parsed_variant = std::mem::discriminant(&style);
        let expected_variant = std::mem::discriminant(&expected);

        assert_eq!(parsed_variant, expected_variant);
    }
}

#[test]
fn refresh_token_options_deserialize_from_toml() {
    let config = parse_config(
        r#"
        token_name = "Authorization"
        enable_refresh_token = true
        refresh_token_timeout = 1209600
    "#,
    );

    assert!(config.enable_refresh_token);
    assert_eq!(config.refresh_token_timeout, 1209600);
}

#[test]
fn nonce_options_deserialize_from_toml() {
    let config = parse_config(
        r#"
        token_name = "Authorization"
        enable_nonce = true
        nonce_timeout = 300
    "#,
    );

    assert!(config.enable_nonce);
    assert_eq!(config.nonce_timeout, 300);
}
