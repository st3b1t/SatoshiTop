//! JSON-RPC credentials, ported from `lib/utils/load-auth.js`.
//!
//! The auth cookie written by the node takes precedence over `rpcuser` and
//! `rpcpassword`. The node rewrites the cookie at every start, so callers are
//! expected to call [`load_auth`] again before reconnecting: it only reads one
//! small file.

use std::fmt;
use std::path::{Path, PathBuf};

use super::Config;

/// User and password for HTTP Basic authentication.
#[derive(Clone, PartialEq, Eq)]
pub struct Credentials {
    pub user: String,
    pub password: String,
}

impl fmt::Debug for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Credentials")
            .field("user", &self.user)
            .field("password", &"***")
            .finish()
    }
}

/// Where the credentials come from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthSource {
    /// Read from the cookie file at this path.
    Cookie(PathBuf),
    /// `rpcuser` and `rpcpassword`. `missing_cookie` is the configured cookie
    /// file that did not exist, if any: worth a warning in verbose mode.
    UserPassword { missing_cookie: Option<PathBuf> },
}

/// Why no credentials could be loaded.
#[derive(Debug)]
pub enum AuthError {
    /// The cookie file is configured but does not exist, and there is no
    /// user and password to fall back to. The node may not be running yet.
    CookieNotFound(PathBuf),
    /// The cookie file exists but cannot be read.
    CookieUnreadable(PathBuf, std::io::Error),
    /// The cookie file is not `user:password`.
    CookieMalformed(PathBuf),
    /// Neither a cookie file nor a user and password are configured.
    NoCredentials,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CookieNotFound(path) => write!(f, "Cookie file {} not found", path.display()),
            Self::CookieUnreadable(path, e) => {
                write!(f, "Cannot read cookie file {}: {e}", path.display())
            }
            // Message of the original.
            Self::CookieMalformed(path) => {
                write!(f, "Cookie file {} in unexpected format", path.display())
            }
            Self::NoCredentials => {
                f.write_str("no RPC credentials: set rpccookiefile or rpcuser/rpcpassword")
            }
        }
    }
}

impl std::error::Error for AuthError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CookieUnreadable(_, e) => Some(e),
            _ => None,
        }
    }
}

/// Loads the credentials for `config`: the cookie file if it exists,
/// otherwise `rpcuser` and `rpcpassword`.
pub fn load_auth(config: &Config) -> Result<(Credentials, AuthSource), AuthError> {
    load(
        config.rpccookiefile.as_deref(),
        config.rpcuser.as_deref(),
        config.rpcpassword.as_deref(),
    )
}

fn load(
    cookiefile: Option<&Path>,
    user: Option<&str>,
    password: Option<&str>,
) -> Result<(Credentials, AuthSource), AuthError> {
    let mut missing_cookie = None;
    if let Some(path) = cookiefile {
        match std::fs::read_to_string(path) {
            Ok(text) => {
                let credentials =
                    parse_cookie(&text).ok_or_else(|| AuthError::CookieMalformed(path.into()))?;
                return Ok((credentials, AuthSource::Cookie(path.into())));
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                missing_cookie = Some(path.to_path_buf());
            }
            Err(e) => return Err(AuthError::CookieUnreadable(path.into(), e)),
        }
    }

    // Empty values (e.g. `RPCUSER=`) count as unset.
    fn non_empty(s: Option<&str>) -> Option<&str> {
        s.map(str::trim).filter(|s| !s.is_empty())
    }
    match (non_empty(user), non_empty(password)) {
        (Some(user), Some(password)) => Ok((
            Credentials {
                user: user.to_owned(),
                password: password.to_owned(),
            },
            AuthSource::UserPassword { missing_cookie },
        )),
        _ => Err(match missing_cookie {
            Some(path) => AuthError::CookieNotFound(path),
            None => AuthError::NoCredentials,
        }),
    }
}

/// Parses `user:password`, splitting on the first `:` so that a password may
/// contain colons (the original drops everything after a second one).
fn parse_cookie(text: &str) -> Option<Credentials> {
    let (user, password) = text.trim().split_once(':')?;
    let (user, password) = (user.trim(), password.trim());
    (!user.is_empty() && !password.is_empty()).then(|| Credentials {
        user: user.to_owned(),
        password: password.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::TempDir;

    const COOKIE: &str =
        "__cookie__:5c2f9e0a3d8b4f1e6a7c9d0b2e4f6a8c1d3e5f7a9b0c2d4e6f8a0b1c3d5e7f9a\n";

    fn creds(user: &str, password: &str) -> Credentials {
        Credentials {
            user: user.into(),
            password: password.into(),
        }
    }

    #[test]
    fn a_valid_cookie_wins_over_user_and_password() {
        let dir = TempDir::new("cookie");
        let path = dir.file(".cookie", COOKIE);
        let (credentials, source) = load(Some(&path), Some("u"), Some("p")).unwrap();
        assert_eq!(credentials.user, "__cookie__");
        assert_eq!(
            credentials.password,
            "5c2f9e0a3d8b4f1e6a7c9d0b2e4f6a8c1d3e5f7a9b0c2d4e6f8a0b1c3d5e7f9a"
        );
        assert_eq!(source, AuthSource::Cookie(path));
    }

    #[test]
    fn the_cookie_password_may_contain_colons() {
        let dir = TempDir::new("colons");
        let path = dir.file(".cookie", " user:pa:ss \n");
        let (credentials, _) = load(Some(&path), None, None).unwrap();
        assert_eq!(credentials, creds("user", "pa:ss"));
    }

    #[test]
    fn malformed_cookies_are_rejected() {
        let dir = TempDir::new("malformed");
        for (i, text) in ["", "\n", "nocolon", ":password", "user:", " : "]
            .iter()
            .enumerate()
        {
            let path = dir.file(&format!("cookie{i}"), text);
            let err = load(Some(&path), Some("u"), Some("p")).unwrap_err();
            assert!(
                matches!(err, AuthError::CookieMalformed(_)),
                "{text:?}: {err}"
            );
            assert_eq!(
                err.to_string(),
                format!("Cookie file {} in unexpected format", path.display())
            );
        }
    }

    #[test]
    fn a_missing_cookie_falls_back_to_user_and_password() {
        let dir = TempDir::new("fallback");
        let path = dir.path().join(".cookie");
        let (credentials, source) = load(Some(&path), Some("u"), Some("p")).unwrap();
        assert_eq!(credentials, creds("u", "p"));
        assert_eq!(
            source,
            AuthSource::UserPassword {
                missing_cookie: Some(path)
            }
        );
    }

    #[test]
    fn a_missing_cookie_without_fallback_is_an_error() {
        let dir = TempDir::new("notfound");
        let path = dir.path().join(".cookie");
        let err = load(Some(&path), Some("u"), None).unwrap_err();
        assert!(matches!(err, AuthError::CookieNotFound(_)));
        assert_eq!(
            err.to_string(),
            format!("Cookie file {} not found", path.display())
        );
    }

    #[test]
    fn an_unreadable_cookie_is_an_error() {
        let dir = TempDir::new("unreadable");
        // A directory exists but cannot be read as a file.
        let err = load(Some(dir.path()), Some("u"), Some("p")).unwrap_err();
        assert!(matches!(err, AuthError::CookieUnreadable(..)), "{err}");
    }

    #[test]
    fn user_and_password_without_cookie() {
        let (credentials, source) = load(None, Some(" u "), Some("p")).unwrap();
        assert_eq!(credentials, creds("u", "p"));
        assert_eq!(
            source,
            AuthSource::UserPassword {
                missing_cookie: None
            }
        );
    }

    #[test]
    fn missing_or_empty_credentials_are_an_error() {
        for (user, password) in [
            (None, None),
            (Some("u"), None),
            (None, Some("p")),
            (Some(""), Some("p")),
        ] {
            let err = load(None, user, password).unwrap_err();
            assert!(
                matches!(err, AuthError::NoCredentials),
                "{user:?} {password:?}"
            );
        }
        assert_eq!(
            AuthError::NoCredentials.to_string(),
            "no RPC credentials: set rpccookiefile or rpcuser/rpcpassword"
        );
    }

    #[test]
    fn load_auth_reads_the_config() {
        let mut config = Config::merge(&crate::config::Cli::default(), None, "/x".into());
        config.rpcuser = Some("u".into());
        config.rpcpassword = Some("p".into());
        assert_eq!(load_auth(&config).unwrap().0, creds("u", "p"));
    }

    #[test]
    fn debug_hides_the_password() {
        let debug = format!("{:?}", creds("u", "secret"));
        assert!(!debug.contains("secret"), "{debug}");
        assert!(debug.contains("***"), "{debug}");
    }
}
