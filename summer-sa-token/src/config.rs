//! Sa-Token configuration module
//!
//! This module defines the configuration for summer-sa-token plugin.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use summer::config::Configurable;
// Re-export CoreConfig
pub use sa_token_core::config::SaTokenConfig as CoreConfig;

summer::submit_config_schema!("sa-token", SaTokenConfig);

/// Token style for summer-sa-token
///
/// This is a local wrapper around the core TokenStyle to support JsonSchema.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum TokenStyle {
    /// UUID style
    Uuid,
    /// Simple UUID (without hyphens)
    SimpleUuid,
    /// 32-character random string
    Random32,
    /// 64-character random string
    Random64,
    /// 128-character random string
    Random128,
    /// JWT style (JSON Web Token)
    Jwt,
    /// Hash style (SHA256 hash)
    Hash,
    /// Timestamp style (millisecond timestamp + random)
    Timestamp,
    /// Tik style (short 8-character token)
    Tik,
}

impl From<TokenStyle> for sa_token_core::config::TokenStyle {
    fn from(style: TokenStyle) -> Self {
        match style {
            TokenStyle::Uuid => sa_token_core::config::TokenStyle::Uuid,
            TokenStyle::SimpleUuid => sa_token_core::config::TokenStyle::SimpleUuid,
            TokenStyle::Random32 => sa_token_core::config::TokenStyle::Random32,
            TokenStyle::Random64 => sa_token_core::config::TokenStyle::Random64,
            TokenStyle::Random128 => sa_token_core::config::TokenStyle::Random128,
            TokenStyle::Jwt => sa_token_core::config::TokenStyle::Jwt,
            TokenStyle::Hash => sa_token_core::config::TokenStyle::Hash,
            TokenStyle::Timestamp => sa_token_core::config::TokenStyle::Timestamp,
            TokenStyle::Tik => sa_token_core::config::TokenStyle::Tik,
        }
    }
}

/// Logout mode when `max_login_count` is exceeded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum LogoutMode {
    Logout,
    KickOut,
    Replaced,
}

impl From<LogoutMode> for sa_token_core::config::LogoutMode {
    fn from(mode: LogoutMode) -> Self {
        match mode {
            LogoutMode::Logout => sa_token_core::config::LogoutMode::Logout,
            LogoutMode::KickOut => sa_token_core::config::LogoutMode::KickOut,
            LogoutMode::Replaced => sa_token_core::config::LogoutMode::Replaced,
        }
    }
}

/// Behavior when a non-concurrent login replaces an existing login.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum ReplacedLoginExitMode {
    OldDevice,
    NewDevice,
}

impl From<ReplacedLoginExitMode> for sa_token_core::config::ReplacedLoginExitMode {
    fn from(mode: ReplacedLoginExitMode) -> Self {
        match mode {
            ReplacedLoginExitMode::OldDevice => {
                sa_token_core::config::ReplacedLoginExitMode::OldDevice
            }
            ReplacedLoginExitMode::NewDevice => {
                sa_token_core::config::ReplacedLoginExitMode::NewDevice
            }
        }
    }
}

/// Scope affected when replacing an existing login.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum ReplacedRange {
    CurrDeviceType,
    AllDeviceType,
}

impl From<ReplacedRange> for sa_token_core::config::ReplacedRange {
    fn from(range: ReplacedRange) -> Self {
        match range {
            ReplacedRange::CurrDeviceType => sa_token_core::config::ReplacedRange::CurrDeviceType,
            ReplacedRange::AllDeviceType => sa_token_core::config::ReplacedRange::AllDeviceType,
        }
    }
}

/// Default logout scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum LogoutRange {
    Token,
    Account,
}

impl From<LogoutRange> for sa_token_core::config::LogoutRange {
    fn from(range: LogoutRange) -> Self {
        match range {
            LogoutRange::Token => sa_token_core::config::LogoutRange::Token,
            LogoutRange::Account => sa_token_core::config::LogoutRange::Account,
        }
    }
}

/// Sa-Token configuration for summer-rs
///
/// Sa-Token configuration for summer-rs.
///
/// # Example
///
/// ```toml
/// [sa-token]
/// token_name = "sa-token"
/// timeout = 86400
/// auto_renew = true
/// ```
#[derive(Debug, Configurable, Clone, Deserialize, JsonSchema)]
#[config_prefix = "sa-token"]
pub struct SaTokenConfig {
    /// Token name (key in header or cookie)
    #[serde(default = "default_token_name")]
    pub token_name: String,

    /// Token timeout in seconds, -1 means permanent
    #[serde(default = "default_timeout")]
    pub timeout: i64,

    /// Token active timeout in seconds, -1 means no limit
    #[serde(default = "default_active_timeout")]
    pub active_timeout: i64,

    /// Enable per-token dynamic active timeout.
    #[serde(default)]
    pub dynamic_active_timeout: bool,

    /// Enable auto renew
    #[serde(default = "default_true")]
    pub auto_renew: bool,

    /// Allow concurrent login for same account
    #[serde(default = "default_true")]
    pub is_concurrent: bool,

    /// Share token when multiple logins
    #[serde(default)]
    pub is_share: bool,

    /// Token style
    #[serde(default = "default_token_style")]
    pub token_style: TokenStyle,

    /// Enable logging
    #[serde(default)]
    pub is_log: bool,

    /// Read token from cookie
    #[serde(default = "default_true")]
    pub is_read_cookie: bool,

    /// Read token from header
    #[serde(default = "default_true")]
    pub is_read_header: bool,

    /// Read token from body
    #[serde(default = "default_true")]
    pub is_read_body: bool,

    /// JWT secret key
    #[serde(default)]
    pub jwt_secret_key: Option<String>,

    /// JWT algorithm
    #[serde(default = "default_jwt_algorithm")]
    pub jwt_algorithm: Option<String>,

    /// JWT issuer
    #[serde(default)]
    pub jwt_issuer: Option<String>,

    /// JWT audience
    #[serde(default)]
    pub jwt_audience: Option<String>,

    /// Fall back to UUID token when JWT generation fails.
    #[serde(default = "default_true")]
    pub jwt_fallback_on_error: bool,

    /// Enable nonce for replay attack prevention
    #[serde(default)]
    pub enable_nonce: bool,

    /// Nonce timeout in seconds, -1 means use token timeout
    #[serde(default = "default_nonce_timeout")]
    pub nonce_timeout: i64,

    /// Enable refresh token
    #[serde(default)]
    pub enable_refresh_token: bool,

    /// Refresh token timeout in seconds
    #[serde(default = "default_refresh_token_timeout")]
    pub refresh_token_timeout: i64,

    /// Storage key prefix for Redis/database backends.
    #[serde(default = "default_storage_key_prefix")]
    pub storage_key_prefix: String,

    /// Maximum login count for the same account. -1 means unlimited.
    #[serde(default = "default_max_login_count")]
    pub max_login_count: i64,

    /// Logout mode when `max_login_count` is exceeded.
    #[serde(default = "default_overflow_logout_mode")]
    pub overflow_logout_mode: LogoutMode,

    /// Replacement behavior for non-concurrent login.
    #[serde(default = "default_replaced_login_exit_mode")]
    pub replaced_login_exit_mode: ReplacedLoginExitMode,

    /// Replacement scope for non-concurrent login.
    #[serde(default = "default_replaced_range")]
    pub replaced_range: ReplacedRange,

    /// Create token session immediately on login.
    #[serde(default)]
    pub right_now_create_token_session: bool,

    /// Check login status when fetching token session.
    #[serde(default = "default_true")]
    pub token_session_check_login: bool,

    /// Default logout scope.
    #[serde(default = "default_logout_range")]
    pub logout_range: LogoutRange,

    /// Keep token session after logout.
    #[serde(default)]
    pub is_logout_keep_token_session: bool,
}

impl Default for SaTokenConfig {
    fn default() -> Self {
        Self {
            token_name: default_token_name(),
            timeout: default_timeout(),
            active_timeout: default_active_timeout(),
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
            jwt_algorithm: default_jwt_algorithm(),
            jwt_issuer: None,
            jwt_audience: None,
            jwt_fallback_on_error: true,
            enable_nonce: false,
            nonce_timeout: default_nonce_timeout(),
            enable_refresh_token: false,
            refresh_token_timeout: default_refresh_token_timeout(),
            storage_key_prefix: default_storage_key_prefix(),
            max_login_count: default_max_login_count(),
            overflow_logout_mode: LogoutMode::Logout,
            replaced_login_exit_mode: ReplacedLoginExitMode::OldDevice,
            replaced_range: ReplacedRange::CurrDeviceType,
            right_now_create_token_session: false,
            token_session_check_login: true,
            logout_range: LogoutRange::Token,
            is_logout_keep_token_session: false,
        }
    }
}

impl From<SaTokenConfig> for CoreConfig {
    fn from(config: SaTokenConfig) -> Self {
        CoreConfig {
            token_name: config.token_name,
            timeout: config.timeout,
            active_timeout: config.active_timeout,
            dynamic_active_timeout: config.dynamic_active_timeout,
            auto_renew: config.auto_renew,
            is_concurrent: config.is_concurrent,
            is_share: config.is_share,
            token_style: config.token_style.into(),
            is_log: config.is_log,
            is_read_cookie: config.is_read_cookie,
            is_read_header: config.is_read_header,
            is_read_body: config.is_read_body,
            jwt_secret_key: config.jwt_secret_key,
            jwt_algorithm: config.jwt_algorithm,
            jwt_issuer: config.jwt_issuer,
            jwt_audience: config.jwt_audience,
            jwt_fallback_on_error: config.jwt_fallback_on_error,
            enable_nonce: config.enable_nonce,
            nonce_timeout: config.nonce_timeout,
            enable_refresh_token: config.enable_refresh_token,
            refresh_token_timeout: config.refresh_token_timeout,
            storage_key_prefix: config.storage_key_prefix,
            max_login_count: config.max_login_count,
            overflow_logout_mode: config.overflow_logout_mode.into(),
            replaced_login_exit_mode: config.replaced_login_exit_mode.into(),
            replaced_range: config.replaced_range.into(),
            right_now_create_token_session: config.right_now_create_token_session,
            token_session_check_login: config.token_session_check_login,
            logout_range: config.logout_range.into(),
            is_logout_keep_token_session: config.is_logout_keep_token_session,
        }
    }
}

// Default value functions
fn default_token_name() -> String {
    "sa-token".to_string()
}

fn default_timeout() -> i64 {
    2592000 // 30 days
}

fn default_active_timeout() -> i64 {
    -1
}

fn default_true() -> bool {
    true
}

fn default_jwt_algorithm() -> Option<String> {
    Some("HS256".to_string())
}

fn default_nonce_timeout() -> i64 {
    -1
}

fn default_refresh_token_timeout() -> i64 {
    604800 // 7 days
}

fn default_token_style() -> TokenStyle {
    TokenStyle::Uuid
}

fn default_storage_key_prefix() -> String {
    "sa:".to_string()
}

fn default_max_login_count() -> i64 {
    -1
}

fn default_overflow_logout_mode() -> LogoutMode {
    LogoutMode::Logout
}

fn default_replaced_login_exit_mode() -> ReplacedLoginExitMode {
    ReplacedLoginExitMode::OldDevice
}

fn default_replaced_range() -> ReplacedRange {
    ReplacedRange::CurrDeviceType
}

fn default_logout_range() -> LogoutRange {
    LogoutRange::Token
}
