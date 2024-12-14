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

pub async fn listen_for_code(port: u32) -> Result<ReceivedCode, ()> {
    let bind = format!("127.0.0.1:{}", port);
    log::info!("Listening on: http://{}", bind);
    let addr: SocketAddr = match str::parse(&bind) {
        Ok(addr) => addr,
        Err(_) => {
            log::error!("Invalid address: {}", bind);
            return Err(());
        }
    };
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(err) => {
            log::error!("Failed to bind: {}", err);
            return Err(());
        }
    };
    let (tx, mut rx) = oneshot::channel();
    let tx = Arc::new(Mutex::new(Some(tx)));

    let handle = tokio::spawn(async move {
        // Accept a single connection
        let (stream, _) = match listener.accept().await {
            Ok((stream, _)) => (stream, addr),
            Err(err) => {
                eprintln!("Error accepting connection: {}", err);
                return;
            }
        };
        let io = TokioIo::new(stream);

        // Create the service
        let service = OAuthService { tx: tx.clone() };
        // Process the connection with our service
        if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
            eprintln!("Error serving connection: {}", err);
        }
    });

    let mut received_code = None;
    // Wait for the callback to be processed
    loop {
        tokio::select! {
            v1 = (&mut rx) => {
                if received_code.is_none(){
                    received_code = Some(v1);
                    break;
                }
            },
            _ = tokio::signal::ctrl_c()  => {
                println!("CTRL+C was used");
                break;
            }
        }
    }
    println!("Closing server");
    handle.abort();
    match received_code {
        Some(v) => match v {
            Ok(v) => {
                log::info!("Authorization code received - closing server");
                Ok(v)
            }
            Err(_) => {
                log::info!("Error receiving authorization code");
                Err(())
            }
        },
        None => {
            log::info!("CTRL+C was used");
            Err(())
        }
    }
}
