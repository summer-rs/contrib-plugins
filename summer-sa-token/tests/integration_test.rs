#[cfg(feature = "with-summer-redis")]
use sa_token_adapter::storage::SaStorage;
use sa_token_core::token::TokenValue;
use std::sync::OnceLock;
#[cfg(feature = "with-summer-redis")]
use std::time::{SystemTime, UNIX_EPOCH};
use summer_sa_token::sa_token_plugin_axum::{
    OptionalSaTokenExtractor, SaTokenLayer, SaTokenState, StpUtil,
};
use summer_web::axum::{
    body::Body,
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use tower::ServiceExt;

#[cfg(feature = "with-summer-redis")]
use summer_sa_token::storage::SummerRedisStorage;

#[cfg(feature = "with-summer-redis")]
use summer_redis::redis::AsyncCommands;

fn test_state() -> SaTokenState {
    static STATE: OnceLock<SaTokenState> = OnceLock::new();

    STATE
        .get_or_init(|| {
            SaTokenState::builder()
                .storage(std::sync::Arc::new(
                    summer_sa_token::sa_token_plugin_axum::MemoryStorage::new(),
                ))
                .token_name("Authorization".to_string())
                .timeout(3600)
                .build()
        })
        .clone()
}

async fn token_echo(optional_token: OptionalSaTokenExtractor) -> impl IntoResponse {
    match optional_token.0 {
        Some(token) => format!("Token: {}", token.as_str()),
        None => "No token".to_string(),
    }
}

async fn token_presence(optional_token: OptionalSaTokenExtractor) -> impl IntoResponse {
    match optional_token.0 {
        Some(_) => "Has token",
        None => "No token",
    }
}

#[tokio::test]
async fn state_can_create_auth_layer() {
    let state = test_state();

    let _layer = SaTokenLayer::new(state);
}

#[tokio::test]
async fn login_returns_a_valid_token_with_user_info() {
    let state = test_state();

    let token = state.manager.login("test_user").await.unwrap();

    assert!(!token.as_str().is_empty());
    assert!(state.manager.is_valid(&token).await);

    let token_info = state.manager.get_token_info(&token).await.unwrap();
    assert_eq!(token_info.login_id, "test_user");
}

#[tokio::test]
async fn logout_by_login_id_invalidates_existing_token() {
    let state = test_state();
    let token = state.manager.login("logout_user").await.unwrap();

    assert!(state.manager.is_valid(&token).await);

    state
        .manager
        .logout_by_login_id("logout_user")
        .await
        .unwrap();

    assert!(!state.manager.is_valid(&token).await);
}

#[tokio::test]
async fn concurrent_logins_keep_each_token_valid() {
    let state = test_state();

    let token1 = state.manager.login("multi_user").await.unwrap();
    let token2 = state.manager.login("multi_user").await.unwrap();

    assert!(state.manager.is_valid(&token1).await);
    assert!(state.manager.is_valid(&token2).await);
}

#[tokio::test]
async fn invalid_token_is_rejected() {
    let state = test_state();
    let fake_token = TokenValue::new("fake-token-12345");

    assert!(!state.manager.is_valid(&fake_token).await);
}

#[tokio::test]
async fn roles_and_permissions_are_stored_for_login_id() {
    let state = test_state();

    state.manager.login("role_user").await.unwrap();
    StpUtil::set_roles("role_user", vec!["admin".to_string(), "user".to_string()])
        .await
        .unwrap();
    StpUtil::set_permissions(
        "role_user",
        vec!["user:read".to_string(), "user:write".to_string()],
    )
    .await
    .unwrap();

    assert!(StpUtil::has_role("role_user", "admin").await);
    assert!(StpUtil::has_role("role_user", "user").await);
    assert!(!StpUtil::has_role("role_user", "superadmin").await);

    assert!(StpUtil::has_permission("role_user", "user:read").await);
    assert!(StpUtil::has_permission("role_user", "user:write").await);
    assert!(!StpUtil::has_permission("role_user", "user:delete").await);

    let roles = StpUtil::get_roles("role_user").await;
    assert_eq!(roles.len(), 2);
    assert!(roles.contains(&"admin".to_string()));

    let permissions = StpUtil::get_permissions("role_user").await;
    assert_eq!(permissions.len(), 2);
    assert!(permissions.contains(&"user:read".to_string()));
}

#[tokio::test]
async fn optional_extractor_reads_token_from_authorization_header() {
    let state = test_state();
    let token = state.manager.login("middleware_user").await.unwrap();
    let app = Router::new()
        .route("/test", get(token_echo))
        .layer(SaTokenLayer::new(state));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("Authorization", token.as_str())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn optional_extractor_allows_missing_token() {
    let state = test_state();
    let app = Router::new()
        .route("/test", get(token_presence))
        .layer(SaTokenLayer::new(state));

    let response = app
        .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[cfg(feature = "with-summer-redis")]
async fn create_redis_connection() -> Option<summer_redis::Redis> {
    let url = std::env::var("REDIS_URL").ok()?;
    let client = summer_redis::redis::Client::open(url).expect("redis client should open");
    Some(
        client
            .get_connection_manager()
            .await
            .expect("redis connection manager should connect"),
    )
}

#[cfg(feature = "with-summer-redis")]
fn unique_storage_key_prefix(tag: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should be after epoch")
        .as_nanos();
    format!("demo-{tag}-{nanos}")
}

#[cfg(feature = "with-summer-redis")]
#[tokio::test]
async fn redis_storage_uses_keys_passed_by_core() {
    let Some(mut redis) = create_redis_connection().await else {
        eprintln!("skipping: REDIS_URL is not set");
        return;
    };
    let prefix = unique_storage_key_prefix("generated");
    let storage = SummerRedisStorage::new(redis.clone());

    storage
        .set(&format!("{prefix}:login:token:user1"), "token-123", None)
        .await
        .expect("set should succeed");

    let stored_key = format!("{prefix}:login:token:user1");
    let raw: Option<String> = redis
        .get(&stored_key)
        .await
        .expect("prefixed key should be readable");
    assert_eq!(raw.as_deref(), Some("token-123"));

    redis
        .del::<_, ()>(&stored_key)
        .await
        .expect("test key should be removed");
}

#[cfg(feature = "with-summer-redis")]
#[tokio::test]
async fn redis_storage_uses_patterns_passed_by_core() {
    let Some(mut redis) = create_redis_connection().await else {
        eprintln!("skipping: REDIS_URL is not set");
        return;
    };
    let prefix = unique_storage_key_prefix("keys");
    let storage = SummerRedisStorage::new(redis.clone());

    redis
        .set::<_, _, ()>(format!("{prefix}:token:one"), "a")
        .await
        .expect("set should succeed");
    redis
        .set::<_, _, ()>(format!("{prefix}:session:user1"), "b")
        .await
        .expect("set should succeed");
    redis
        .set::<_, _, ()>(format!("{prefix}:refresh:token-1"), "c")
        .await
        .expect("set should succeed");

    let mut keys = storage
        .keys(&format!("{prefix}:*"))
        .await
        .expect("keys should succeed");
    keys.sort();

    assert_eq!(
        keys,
        vec![
            format!("{prefix}:refresh:token-1"),
            format!("{prefix}:session:user1"),
            format!("{prefix}:token:one")
        ]
    );

    redis
        .del::<_, ()>(&keys)
        .await
        .expect("test keys should be removed");

    let remaining: Vec<String> = redis
        .keys(format!("{prefix}:*"))
        .await
        .expect("keys should succeed");
    assert!(remaining.is_empty());
}

mod config_conversion {
    use summer_sa_token::{
        CoreConfig, LogoutMode, LogoutRange, ReplacedLoginExitMode, ReplacedRange, SaTokenConfig,
    };

    #[test]
    fn local_config_converts_to_core_config() {
        let config = SaTokenConfig {
            token_name: "TestToken".to_string(),
            timeout: 7200,
            dynamic_active_timeout: true,
            auto_renew: true,
            storage_key_prefix: "demo:".to_string(),
            max_login_count: 2,
            overflow_logout_mode: LogoutMode::KickOut,
            replaced_login_exit_mode: ReplacedLoginExitMode::NewDevice,
            replaced_range: ReplacedRange::AllDeviceType,
            right_now_create_token_session: true,
            token_session_check_login: false,
            logout_range: LogoutRange::Account,
            is_logout_keep_token_session: true,
            ..Default::default()
        };

        let core_config: CoreConfig = config.into();
        assert_eq!(core_config.token_name, "TestToken");
        assert_eq!(core_config.timeout, 7200);
        assert!(core_config.dynamic_active_timeout);
        assert!(core_config.auto_renew);
        assert_eq!(core_config.storage_key_prefix, "demo:");
        assert_eq!(core_config.max_login_count, 2);
        assert_eq!(
            core_config.overflow_logout_mode,
            sa_token_core::config::LogoutMode::KickOut
        );
        assert_eq!(
            core_config.replaced_login_exit_mode,
            sa_token_core::config::ReplacedLoginExitMode::NewDevice
        );
        assert_eq!(
            core_config.replaced_range,
            sa_token_core::config::ReplacedRange::AllDeviceType
        );
        assert!(core_config.right_now_create_token_session);
        assert!(!core_config.token_session_check_login);
        assert_eq!(
            core_config.logout_range,
            sa_token_core::config::LogoutRange::Account
        );
        assert!(core_config.is_logout_keep_token_session);
    }
}

mod path_auth_builder {
    use summer_sa_token::PathAuthBuilder;

    #[test]
    fn new_builder_is_not_configured() {
        let builder = PathAuthBuilder::new();
        assert!(!builder.is_configured());
    }

    #[test]
    fn include_marks_builder_configured() {
        let builder = PathAuthBuilder::new()
            .include("/api/**")
            .include("/admin/**");

        assert!(builder.is_configured());
    }

    #[test]
    fn exclude_marks_builder_configured() {
        let builder = PathAuthBuilder::new()
            .include("/api/**")
            .exclude("/api/public/**")
            .exclude("/login");

        assert!(builder.is_configured());
    }

    #[test]
    fn include_all_marks_builder_configured() {
        let builder = PathAuthBuilder::new().include_all(["/api/**", "/admin/**", "/user/**"]);

        assert!(builder.is_configured());
    }

    #[test]
    fn exclude_all_marks_builder_configured() {
        let builder = PathAuthBuilder::new()
            .include("/api/**")
            .exclude_all(["/api/public/**", "/api/health"]);

        assert!(builder.is_configured());
    }

    #[test]
    fn authenticated_and_permit_all_are_aliases() {
        let builder = PathAuthBuilder::new()
            .authenticated("/api/**")
            .permit_all("/api/public/**");

        assert!(builder.is_configured());
    }

    #[test]
    fn merge_combines_configured_builders() {
        let builder1 = PathAuthBuilder::new().include("/api/**");
        let builder2 = PathAuthBuilder::new().include("/admin/**");

        let merged = builder1.merge(builder2);
        assert!(merged.is_configured());
    }

    #[test]
    fn configured_builder_can_be_built() {
        let builder = PathAuthBuilder::new()
            .include("/api/**")
            .exclude("/api/public/**");

        let _config = builder.build();
    }
}
