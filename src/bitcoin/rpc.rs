//! JSON-RPC access to the Bitcoin node, ported from `lib/utils/rpc-client.js`
//! and `lib/bitcoin/rpc.js`.
//!
//! [`RpcBackend`] is the single entry point used by the rest of the program;
//! [`HttpRpc`] talks to a real node and [`FakeRpc`] simulates one. [`AnyRpc`]
//! picks one at runtime (`FAKERPC`).

use std::fmt;
use std::future::Future;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::config::Config;
use crate::config::auth::{self, Credentials};
use crate::tests::rpc_fake::FakeRpc;

/// Something that answers JSON-RPC calls.
pub trait RpcBackend: Send + Sync {
    /// Calls `method` with positional `params` and returns its `result`.
    fn call(
        &self,
        method: &str,
        params: Vec<Value>,
    ) -> impl Future<Output = Result<Value, RpcError>> + Send;
}

/// Calls `method` and deserializes the result into `T`.
pub async fn call_as<T: DeserializeOwned>(
    backend: &impl RpcBackend,
    method: &str,
    params: Vec<Value>,
) -> Result<T, RpcError> {
    let value = backend.call(method, params).await?;
    serde_json::from_value(value).map_err(|e| RpcError::InvalidResponse(format!("{method}: {e}")))
}

/// Why a call failed. Cloneable so that the UI can keep and show it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RpcError {
    /// The node could not be reached (refused, unreachable, reset...).
    Connection(String),
    /// No answer within `rpcclienttimeout`.
    Timeout,
    /// HTTP 401: wrong user or password, or an outdated cookie.
    Unauthorized,
    /// No usable credentials (see [`auth::AuthError`]).
    Auth(String),
    /// The node answered with a JSON-RPC error.
    Rpc { code: i64, message: String },
    /// Unexpected HTTP status without a JSON-RPC error.
    Http(u16),
    /// The answer is not the expected JSON.
    InvalidResponse(String),
}

impl RpcError {
    /// True when the node itself could not be used: every other call would
    /// fail the same way.
    pub fn is_connection_level(&self) -> bool {
        matches!(
            self,
            Self::Connection(_) | Self::Timeout | Self::Unauthorized | Self::Auth(_)
        )
    }
}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connection(message) => f.write_str(message),
            Self::Timeout => f.write_str("timeout"),
            // Message of the original.
            Self::Unauthorized => f.write_str("Incorrect rpcuser or rpcpassword"),
            Self::Auth(message) => f.write_str(message),
            Self::Rpc { code, message } => write!(f, "{message} (code {code})"),
            Self::Http(status) => write!(f, "unexpected HTTP status {status}"),
            Self::InvalidResponse(message) => write!(f, "invalid response: {message}"),
        }
    }
}

impl std::error::Error for RpcError {}

/// JSON-RPC client over HTTP for a real node.
///
/// Credentials are loaded lazily and loaded again after a 401 or a
/// connection error, so that a node restart (which writes a new cookie) does
/// not require restarting the program.
pub struct HttpRpc {
    client: reqwest::Client,
    url: String,
    config: Config,
    credentials: Mutex<Option<Credentials>>,
    next_id: AtomicU64,
}

#[derive(Deserialize)]
struct Response {
    #[serde(default)]
    result: Value,
    #[serde(default)]
    error: Option<ErrorObject>,
}

#[derive(Deserialize)]
struct ErrorObject {
    code: i64,
    message: String,
}

impl HttpRpc {
    pub fn new(config: &Config) -> Result<Self, RpcError> {
        // A node is reached directly: ignore HTTP_PROXY and friends.
        let mut builder = reqwest::Client::builder().no_proxy();
        if let Some(timeout) = config.rpc_timeout() {
            builder = builder.timeout(timeout);
        }
        let client = builder
            .build()
            .map_err(|e| RpcError::Connection(e.to_string()))?;
        Ok(Self {
            client,
            url: format!("http://{}:{}/", config.rpcconnect, config.rpcport),
            config: config.clone(),
            credentials: Mutex::new(None),
            next_id: AtomicU64::new(1),
        })
    }

    /// Cached credentials, loading them if needed.
    fn credentials(&self) -> Result<Credentials, RpcError> {
        let mut cached = self.credentials.lock().expect("credentials lock");
        if let Some(credentials) = cached.as_ref() {
            return Ok(credentials.clone());
        }
        let (credentials, _) =
            auth::load_auth(&self.config).map_err(|e| RpcError::Auth(e.to_string()))?;
        *cached = Some(credentials.clone());
        Ok(credentials)
    }

    fn forget_credentials(&self) {
        *self.credentials.lock().expect("credentials lock") = None;
    }

    async fn send(
        &self,
        credentials: &Credentials,
        method: &str,
        params: &[Value],
    ) -> Result<Value, RpcError> {
        // JSON-RPC 1.0, like bitcoin-cli: errors come with HTTP 500/404.
        let body = json!({
            "jsonrpc": "1.0",
            "id": self.next_id.fetch_add(1, Ordering::Relaxed),
            "method": method,
            "params": params,
        });
        let response = self
            .client
            .post(&self.url)
            .basic_auth(&credentials.user, Some(&credentials.password))
            .json(&body)
            .send()
            .await
            .map_err(transport_error)?;
        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(RpcError::Unauthorized);
        }
        let bytes = response.bytes().await.map_err(transport_error)?;
        match serde_json::from_slice::<Response>(&bytes) {
            Ok(Response {
                error: Some(error), ..
            }) => Err(RpcError::Rpc {
                code: error.code,
                message: error.message,
            }),
            Ok(Response { result, .. }) if status.is_success() => Ok(result),
            Ok(_) => Err(RpcError::Http(status.as_u16())),
            Err(_) if !status.is_success() => Err(RpcError::Http(status.as_u16())),
            Err(e) => Err(RpcError::InvalidResponse(e.to_string())),
        }
    }
}

impl RpcBackend for HttpRpc {
    async fn call(&self, method: &str, params: Vec<Value>) -> Result<Value, RpcError> {
        let credentials = self.credentials()?;
        match self.send(&credentials, method, &params).await {
            Err(RpcError::Unauthorized) => {
                // Maybe the node restarted with a new cookie: retry once if
                // the credentials changed.
                self.forget_credentials();
                let fresh = self.credentials()?;
                if fresh == credentials {
                    return Err(RpcError::Unauthorized);
                }
                self.send(&fresh, method, &params).await
            }
            Err(e @ RpcError::Connection(_)) => {
                self.forget_credentials();
                Err(e)
            }
            result => result,
        }
    }
}

/// Turns a reqwest error into a short message, preferring the underlying I/O
/// error ("Connection refused (os error 111)") to reqwest's wrapper text.
fn transport_error(error: reqwest::Error) -> RpcError {
    if error.is_timeout() {
        return RpcError::Timeout;
    }
    let mut source: Option<&dyn std::error::Error> = Some(&error);
    while let Some(e) = source {
        if let Some(io) = e.downcast_ref::<std::io::Error>() {
            return RpcError::Connection(io.to_string());
        }
        source = e.source();
    }
    let innermost = std::iter::successors(Some(&error as &dyn std::error::Error), |e| e.source())
        .last()
        .map(ToString::to_string)
        .unwrap_or_default();
    RpcError::Connection(innermost)
}

/// The backend chosen at runtime.
pub enum AnyRpc {
    /// Boxed: the HTTP client is much larger than the fake.
    Http(Box<HttpRpc>),
    Fake(FakeRpc),
}

impl AnyRpc {
    /// [`FakeRpc`] when `FAKERPC` is set (see [`crate::config::env_flag`]),
    /// otherwise [`HttpRpc`] for the configured node.
    pub fn from_env(config: &Config) -> Result<Self, RpcError> {
        if crate::config::env_flag_var("FAKERPC") {
            Ok(Self::Fake(FakeRpc::new(crate::tests::seed_from_env())))
        } else {
            HttpRpc::new(config).map(|rpc| Self::Http(Box::new(rpc)))
        }
    }
}

impl RpcBackend for AnyRpc {
    async fn call(&self, method: &str, params: Vec<Value>) -> Result<Value, RpcError> {
        match self {
            Self::Http(rpc) => rpc.call(method, params).await,
            Self::Fake(rpc) => rpc.call(method, params).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;
    use std::sync::Arc;
    use std::time::Duration;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    use super::*;
    use crate::config::Cli;
    use crate::test_util::TempDir;

    /// A request seen by the test server.
    #[derive(Debug, Clone)]
    struct Request {
        authorization: String,
        body: Value,
    }

    type Handler = dyn Fn(&Request) -> (u16, String) + Send + Sync;

    /// Serves HTTP/1.1 on a local port, one request per connection, answering
    /// with `handler`. Returns the address and the requests received.
    async fn serve(handler: Arc<Handler>) -> (SocketAddr, Arc<Mutex<Vec<Request>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let log = seen.clone();
        tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                let request = read_request(&mut stream).await;
                log.lock().unwrap().push(request.clone());
                let (status, body) = handler(&request);
                let response = format!(
                    "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes()).await;
                let _ = stream.shutdown().await;
            }
        });
        (addr, seen)
    }

    async fn read_request(stream: &mut tokio::net::TcpStream) -> Request {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            let n = stream.read(&mut chunk).await.unwrap();
            buf.extend_from_slice(&chunk[..n]);
            let text = String::from_utf8_lossy(&buf);
            if let Some(end) = text.find("\r\n\r\n") {
                let head = &text[..end];
                let header = |name: &str| {
                    head.lines()
                        .find_map(|l| {
                            let (k, v) = l.split_once(':')?;
                            k.eq_ignore_ascii_case(name).then(|| v.trim().to_owned())
                        })
                        .unwrap_or_default()
                };
                let length: usize = header("content-length").parse().unwrap_or(0);
                if buf.len() >= end + 4 + length {
                    let body = serde_json::from_slice(&buf[end + 4..end + 4 + length]).unwrap();
                    return Request {
                        authorization: header("authorization"),
                        body,
                    };
                }
            }
            if n == 0 {
                panic!("connection closed before the request was complete");
            }
        }
    }

    fn config(addr: SocketAddr) -> Config {
        let mut config = Config::merge(&Cli::default(), None, "/nonexistent/.satoprc".into());
        config.rpcconnect = addr.ip().to_string();
        config.rpcport = addr.port();
        config.rpcuser = Some("u".into());
        config.rpcpassword = Some("p".into());
        config
    }

    fn ok(result: Value) -> (u16, String) {
        (
            200,
            json!({"result": result, "error": null, "id": 1}).to_string(),
        )
    }

    #[tokio::test]
    async fn a_successful_call_returns_the_result() {
        let (addr, seen) = serve(Arc::new(|_: &Request| ok(json!(840004)))).await;
        let rpc = HttpRpc::new(&config(addr)).unwrap();

        let result = rpc.call("getblockcount", vec![]).await.unwrap();
        assert_eq!(result, json!(840004));

        let requests = seen.lock().unwrap();
        assert_eq!(requests[0].authorization, "Basic dTpw"); // "u:p"
        assert_eq!(requests[0].body["jsonrpc"], "1.0");
        assert_eq!(requests[0].body["method"], "getblockcount");
        assert_eq!(requests[0].body["params"], json!([]));
    }

    #[tokio::test]
    async fn params_and_typed_results() {
        let (addr, seen) = serve(Arc::new(|_: &Request| {
            ok(json!({"feerate": 0.0002, "blocks": 6}))
        }))
        .await;
        let rpc = HttpRpc::new(&config(addr)).unwrap();
        let fee: crate::bitcoin::types::SmartFee =
            call_as(&rpc, "estimatesmartfee", vec![json!(6)])
                .await
                .unwrap();
        assert_eq!(fee.feerate, Some(0.0002));
        assert_eq!(seen.lock().unwrap()[0].body["params"], json!([6]));
    }

    #[tokio::test]
    async fn rpc_errors_come_with_http_500() {
        let (addr, _) = serve(Arc::new(|_: &Request| {
            (
                500,
                json!({"result": null, "error": {"code": -8, "message": "Block height out of range"}, "id": 1})
                    .to_string(),
            )
        }))
        .await;
        let rpc = HttpRpc::new(&config(addr)).unwrap();
        let err = rpc.call("getblockhash", vec![json!(-1)]).await.unwrap_err();
        assert_eq!(
            err,
            RpcError::Rpc {
                code: -8,
                message: "Block height out of range".into()
            }
        );
        assert!(!err.is_connection_level());
    }

    #[tokio::test]
    async fn http_errors_without_json() {
        let (addr, _) = serve(Arc::new(|_: &Request| (503, "busy".to_owned()))).await;
        let rpc = HttpRpc::new(&config(addr)).unwrap();
        assert_eq!(rpc.call("uptime", vec![]).await, Err(RpcError::Http(503)));
    }

    #[tokio::test]
    async fn unauthorized_with_fixed_credentials() {
        let (addr, seen) = serve(Arc::new(|_: &Request| (401, String::new()))).await;
        let rpc = HttpRpc::new(&config(addr)).unwrap();
        let err = rpc.call("uptime", vec![]).await.unwrap_err();
        assert_eq!(err, RpcError::Unauthorized);
        assert_eq!(err.to_string(), "Incorrect rpcuser or rpcpassword");
        // Same credentials after reloading: no pointless retry.
        assert_eq!(seen.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn unauthorized_reloads_a_changed_cookie_and_retries() {
        let dir = TempDir::new("rpc-cookie");
        let cookie = dir.file(".cookie", "u:old");
        let cookie_path = cookie.clone();
        // The node "restarts" while answering the first request.
        let (addr, seen) = serve(Arc::new(move |request: &Request| {
            if request.authorization == "Basic dTpuZXc=" {
                ok(json!(17)) // "u:new"
            } else {
                std::fs::write(&cookie_path, "u:new").unwrap();
                (401, String::new())
            }
        }))
        .await;
        let mut config = config(addr);
        config.rpccookiefile = Some(cookie);
        let rpc = HttpRpc::new(&config).unwrap();

        assert_eq!(rpc.call("uptime", vec![]).await, Ok(json!(17)));
        let requests = seen.lock().unwrap();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].authorization, "Basic dTpvbGQ="); // "u:old"
    }

    #[tokio::test]
    async fn timeout() {
        // Accepts the connection but never answers.
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (_stream, _) = listener.accept().await.unwrap();
            tokio::time::sleep(Duration::from_secs(10)).await;
        });
        let mut config = config(addr);
        config.rpcclienttimeout = 200;
        let rpc = HttpRpc::new(&config).unwrap();
        let err = rpc.call("uptime", vec![]).await.unwrap_err();
        assert_eq!(err, RpcError::Timeout);
        assert!(err.is_connection_level());
    }

    #[tokio::test]
    async fn connection_refused() {
        let addr = {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            listener.local_addr().unwrap()
        };
        let rpc = HttpRpc::new(&config(addr)).unwrap();
        let err = rpc.call("uptime", vec![]).await.unwrap_err();
        assert!(
            matches!(&err, RpcError::Connection(m) if m.contains("refused")),
            "{err:?}"
        );
        assert!(err.is_connection_level());
    }

    #[tokio::test]
    async fn missing_credentials_fail_before_connecting() {
        let addr: SocketAddr = "127.0.0.1:1".parse().unwrap();
        let mut config = config(addr);
        config.rpcpassword = None;
        let rpc = HttpRpc::new(&config).unwrap();
        let err = rpc.call("uptime", vec![]).await.unwrap_err();
        assert!(matches!(err, RpcError::Auth(_)), "{err:?}");
        assert!(err.is_connection_level());
    }

    #[tokio::test]
    async fn invalid_json_result() {
        let (addr, _) = serve(Arc::new(|_: &Request| (200, "not json".to_owned()))).await;
        let rpc = HttpRpc::new(&config(addr)).unwrap();
        let err = rpc.call("uptime", vec![]).await.unwrap_err();
        assert!(matches!(err, RpcError::InvalidResponse(_)), "{err:?}");
    }
}
