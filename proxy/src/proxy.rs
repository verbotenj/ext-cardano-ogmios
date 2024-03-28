use futures_util::future::select;
use futures_util::SinkExt;
use futures_util::StreamExt;
use futures_util::TryStreamExt;
use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::body::Incoming;
use hyper::client::conn::http1 as http1_client;
use hyper::header::{
    HeaderValue, CONNECTION, HOST, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY, UPGRADE,
};
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use rustls::ServerConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use std::error::Error;
use std::fmt::Display;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::{fs, io};
use tokio::net::{TcpListener, TcpStream};
use tokio::pin;
use tokio_rustls::TlsAcceptor;
use tokio_tungstenite::tungstenite::handshake::derive_accept_key;
use tokio_tungstenite::tungstenite::protocol::Role;
use tokio_tungstenite::{connect_async, WebSocketStream};
use tracing::{error, info};
use url::Url;

use crate::limiter::limiter;
use crate::utils::{full, get_header, ProxyResponse, DMTR_API_KEY};
use crate::{Consumer, State};

pub async fn start(state: Arc<State>) {
    let addr_result = SocketAddr::from_str(&state.config.proxy_addr);
    if let Err(err) = addr_result {
        error!(error = err.to_string(), "invalid proxy addr");
        std::process::exit(1);
    }
    let addr = addr_result.unwrap();

    let listener_result = TcpListener::bind(addr).await;
    if let Err(err) = listener_result {
        error!(error = err.to_string(), "fail to bind tcp server listener");
        std::process::exit(1);
    }
    let listener = listener_result.unwrap();

    let tls_acceptor_result = build_tls_acceptor(&state);
    if let Err(err) = tls_acceptor_result {
        error!(error = err.to_string(), "fail to load tls");
        std::process::exit(1);
    }
    let tls_acceptor = tls_acceptor_result.unwrap();

    info!(addr = state.config.proxy_addr, "proxy listening");

    loop {
        let state = state.clone();
        let accept_result = listener.accept().await;
        if let Err(err) = accept_result {
            error!(error = err.to_string(), "fail to accept client");
            continue;
        }
        let (stream, _) = accept_result.unwrap();

        let tls_acceptor = tls_acceptor.clone();

        tokio::spawn(async move {
            let tls_stream = match tls_acceptor.accept(stream).await {
                Ok(tls_stream) => tls_stream,
                Err(err) => {
                    error!(error = err.to_string(), "failed to perform tls handshake");
                    return;
                }
            };

            let io = TokioIo::new(tls_stream);

            let service = service_fn(move |req| handle(req, state.clone()));

            if let Err(err) = Builder::new(TokioExecutor::new())
                .serve_connection_with_upgrades(io, service)
                .await
            {
                error!(error = err.to_string(), "failed proxy server connection");
            }
        });
    }
}

async fn handle(
    mut hyper_req: Request<Incoming>,
    state: Arc<State>,
) -> Result<ProxyResponse, hyper::Error> {
    match (hyper_req.method(), hyper_req.uri().path()) {
        (&Method::GET, "/healthz") => handle_healthz().await,
        _ => {
            let proxy_req_result = ProxyRequest::new(&mut hyper_req, &state).await;
            if proxy_req_result.is_none() {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_GATEWAY)
                    .body(full("Invalid hostname"))
                    .unwrap());
            }

            let proxy_req = proxy_req_result.unwrap();
            if proxy_req.consumer.is_none() {
                return Ok(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(full("Unauthorized"))
                    .unwrap());
            }

            let response_result = match proxy_req.protocol {
                Protocol::Http => handle_http(hyper_req, &proxy_req).await,
                Protocol::Websocket => handle_websocket(hyper_req, &proxy_req, state.clone()).await,
            };

            match &response_result {
                Ok(response) => {
                    state
                        .metrics
                        .count_http_total_request(&proxy_req, response.status());
                }
                Err(_) => todo!("send error to prometheus"),
            };

            response_result
        }
    }
}

async fn handle_http(
    hyper_req: Request<Incoming>,
    proxy_req: &ProxyRequest,
) -> Result<ProxyResponse, hyper::Error> {
    let stream = TcpStream::connect(&proxy_req.instance).await.unwrap();
    let io: TokioIo<TcpStream> = TokioIo::new(stream);

    let (mut sender, conn) = http1_client::Builder::new()
        .preserve_header_case(true)
        .title_case_headers(true)
        .handshake(io)
        .await?;

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    let resp = sender.send_request(hyper_req).await?;
    Ok(resp.map(|b| b.boxed()))
}

async fn handle_websocket(
    mut hyper_req: Request<Incoming>,
    proxy_req: &ProxyRequest,
    state: Arc<State>,
) -> Result<ProxyResponse, hyper::Error> {
    let headers = hyper_req.headers();
    let upgrade = HeaderValue::from_static("Upgrade");
    let websocket = HeaderValue::from_static("websocket");
    let key = headers.get(SEC_WEBSOCKET_KEY);
    let derived = key.map(|k| derive_accept_key(k.as_bytes()));
    let version = hyper_req.version();

    let proxy_req = proxy_req.clone();
    let state = state.clone();

    tokio::task::spawn(async move {
        match hyper::upgrade::on(&mut hyper_req).await {
            Ok(upgraded) => {
                let upgraded = TokioIo::new(upgraded);
                let client_stream =
                    WebSocketStream::from_raw_socket(upgraded, Role::Server, None).await;
                let (client_outgoing, mut client_incoming) = client_stream.split();

                let url =
                    Url::parse(&format!("ws://{}{}", proxy_req.instance, hyper_req.uri())).unwrap();
                let connection_result = connect_async(url).await;
                if let Err(err) = connection_result {
                    error!(error = err.to_string(), "fail to connect to the instance");
                    return;
                }
                let (instance_stream, _) = connection_result.unwrap();
                let (mut instance_outgoing, instance_incoming) = instance_stream.split();

                state.metrics.inc_ws_total_connection(&proxy_req);

                let client_in = async {
                    loop {
                        let result = client_incoming.try_next().await;
                        match result {
                            Ok(data) => {
                                if let Some(data) = data {
                                    limiter(state.clone(), proxy_req.consumer.as_ref().unwrap())
                                        .await;
                                    if let Err(err) = instance_outgoing.send(data).await {
                                        error!(
                                            error = err.to_string(),
                                            "fail to send data to instance"
                                        );
                                        break;
                                    }
                                }
                            }
                            Err(err) => {
                                error!(error = err.to_string(), "stream client incoming");
                                break;
                            }
                        }
                    }
                };
                pin!(client_in);

                let instance_in = instance_incoming
                    .inspect_ok(|_| state.metrics.count_ws_total_frame(&proxy_req))
                    .forward(client_outgoing);

                select(client_in, instance_in).await;

                state.metrics.dec_ws_total_connection(&proxy_req);
            }
            Err(err) => {
                error!(error = err.to_string(), "upgrade error");
            }
        }
    });

    let mut res = Response::new(BoxBody::default());
    *res.status_mut() = StatusCode::SWITCHING_PROTOCOLS;
    *res.version_mut() = version;
    res.headers_mut().append(CONNECTION, upgrade);
    res.headers_mut().append(UPGRADE, websocket);
    res.headers_mut()
        .append(SEC_WEBSOCKET_ACCEPT, derived.unwrap().parse().unwrap());

    Ok(res)
}

async fn handle_healthz() -> Result<ProxyResponse, hyper::Error> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(full("pong"))
        .unwrap())
}

#[derive(Debug, Clone)]
pub enum Protocol {
    Http,
    Websocket,
}
impl Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Http => write!(f, "http"),
            Protocol::Websocket => write!(f, "websocket"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProxyRequest {
    pub namespace: String,
    pub host: String,
    pub instance: String,
    pub consumer: Option<Consumer>,
    pub protocol: Protocol,
}
impl ProxyRequest {
    pub async fn new(hyper_req: &mut Request<Incoming>, state: &State) -> Option<Self> {
        let mut host = get_header(hyper_req, HOST.as_str())?;
        let host_regex = host.clone();

        let captures = state.host_regex.captures(&host_regex)?;
        let network = captures.get(2)?.as_str().to_string();
        let version = captures.get(3)?.as_str().to_string();

        let instance = format!(
            "ogmios-{network}-{version}.{}:{}",
            state.config.ogmios_dns, state.config.ogmios_port
        );

        let namespace = state.config.proxy_namespace.clone();

        let protocol = get_header(hyper_req, UPGRADE.as_str())
            .map(|h| {
                if h.eq_ignore_ascii_case("websocket") {
                    return Protocol::Websocket;
                }

                Protocol::Http
            })
            .unwrap_or(Protocol::Http);

        if let Some(key) = captures.get(1) {
            let key = key.as_str();
            hyper_req
                .headers_mut()
                .insert(DMTR_API_KEY, HeaderValue::from_str(key).unwrap());
            host = host.replace(&format!("{key}."), "");
        }

        let token = get_header(hyper_req, DMTR_API_KEY).unwrap_or_default();
        let consumer = state.get_consumer(&network, &version, &token).await;

        Some(Self {
            namespace,
            instance,
            consumer,
            protocol,
            host,
        })
    }
}

fn build_tls_acceptor(state: &State) -> Result<TlsAcceptor, Box<dyn Error>> {
    let certs = load_certs(&state.config.ssl_crt_path)?;

    let key = load_private_key(&state.config.ssl_key_path)?;

    let server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .unwrap();

    let tls_acceptor = TlsAcceptor::from(Arc::new(server_config));
    Ok(tls_acceptor)
}

fn load_certs(path: &PathBuf) -> io::Result<Vec<CertificateDer<'static>>> {
    let cert_file = fs::File::open(path)?;
    let mut reader = io::BufReader::new(cert_file);
    rustls_pemfile::certs(&mut reader).collect()
}

fn load_private_key(path: &PathBuf) -> io::Result<PrivateKeyDer<'static>> {
    let key_file = fs::File::open(path)?;
    let mut reader = io::BufReader::new(key_file);
    rustls_pemfile::private_key(&mut reader).map(|key| key.unwrap())
}
