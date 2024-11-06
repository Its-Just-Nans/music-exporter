use anyhow::Result;
use hyper::service::Service;
use hyper::{server::conn::http1, Request, Response};
use hyper_util::rt::tokio::TokioIo;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use url::Url;

#[derive(Debug, Clone)]
pub struct ReceivedCode {
    pub code: String,
}

struct OAuthService {
    tx: Arc<Mutex<Option<oneshot::Sender<ReceivedCode>>>>,
}

async fn handle_callback(
    req: Request<hyper::body::Incoming>,
    tx: Arc<Mutex<Option<oneshot::Sender<ReceivedCode>>>>,
) -> Response<String> {
    let uri = req.uri();

    match Url::parse(&format!("http://localhost{}", uri)) {
        Ok(url) => {
            let params: std::collections::HashMap<_, _> = url.query_pairs().collect();

            if let Some(code) = params.get("code") {
                let received_code = ReceivedCode {
                    code: code.to_string(),
                };

                // Send the code through the channel
                if let Some(tx) = tx.lock().unwrap().take() {
                    let _ = tx.send(received_code);
                }

                Response::builder()
                    .status(200)
                    .body(String::from(
                        "Authorization successful! You can close this window.",
                    ))
                    .unwrap()
            } else {
                Response::builder()
                    .status(400)
                    .body(String::from("Missing authorization code"))
                    .unwrap()
            }
        }
        Err(_) => Response::builder()
            .status(400)
            .body(String::from("Invalid callback URL"))
            .unwrap(),
    }
}

impl Service<Request<hyper::body::Incoming>> for OAuthService {
    type Response = Response<String>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<hyper::body::Incoming>) -> Self::Future {
        let tx = self.tx.clone();

        Box::pin(async move {
            let response = handle_callback(req, tx).await;
            Ok(response)
        })
    }
}

pub async fn listen_for_code(port: u32) -> Result<ReceivedCode> {
    let bind = format!("127.0.0.1:{}", port);
    log::info!("Listening on: http://{}", bind);
    let addr: SocketAddr = str::parse(&bind)?;
    let listener = TcpListener::bind(addr).await?;
    let (tx, rx) = oneshot::channel();
    let tx = Arc::new(Mutex::new(Some(tx)));

    // Accept a single connection
    let (stream, _) = listener.accept().await?;

    let io = TokioIo::new(stream);

    // Create the service
    let service = OAuthService { tx: tx.clone() };

    // Process the connection with our service
    let handle = tokio::spawn(async move {
        if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
            eprintln!("Error serving connection: {}", err);
        }
    });

    // Wait for the callback to be processed
    let received_code = rx.await?;
    log::info!("Authorization code received - closing server");
    handle.abort();
    Ok(received_code)
}
