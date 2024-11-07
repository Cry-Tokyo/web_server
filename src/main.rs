use http::{Method, Request, Response, StatusCode, Uri, Version};
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::service::service_fn;
use hyper_util::{
    rt::tokio::{TokioExecutor, TokioIo},
    server::conn::auto::Builder,
};
use rustls_pemfile::{certs, private_key};
use std::io::{self, BufReader};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::{fs, net::TcpListener};
use tokio_rustls::{
    rustls::{pki_types::CertificateDer, ServerConfig},
    TlsAcceptor,
};

async fn get_resource(uri: &Uri) -> io::Result<Full<Bytes>> {
    match fs::read(uri.path()).await {
        Ok(o) => Ok(Full::from(o)),
        Err(e) => Err(e),
    }
}
async fn tunnel() {}

async fn handle_request(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, http::Error> {
    match *req.method() {
        Method::POST => Response::builder()
            .status(StatusCode::OK)
            .version(Version::HTTP_3)
            .body(Full::from(_404)),
        Method::GET => match get_resource(req.uri()).await {
            Ok(o) => Response::builder()
                .status(StatusCode::OK)
                .version(Version::HTTP_3)
                .body(o),
            Err(e) => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .version(Version::HTTP_3)
                .body(Full::from(_404)),
        },
        Method::CONNECT => Response::builder()
            .status(StatusCode::OK)
            .version(Version::HTTP_3)
            .body(Full::from(_404)),

        _ => Response::builder().body(Full::from(_404)),
    }
}

fn load_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = "[::]:4433".parse()?;

    Ok(())
}
const _404: Bytes = Bytes::from_static(&[
    60, 33, 100, 111, 99, 116, 121, 112, 101, 32, 104, 116, 109, 108, 62, 10, 60, 104, 116, 109,
    108, 62, 10, 32, 32, 32, 32, 60, 104, 101, 97, 100, 62, 60, 47, 104, 101, 97, 100, 62, 10, 32,
    32, 32, 32, 60, 98, 111, 100, 121, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 60, 104, 49, 62, 60,
    98, 62, 78, 79, 84, 32, 70, 79, 85, 78, 68, 60, 47, 98, 62, 60, 47, 104, 49, 62, 10, 32, 32,
    32, 32, 60, 47, 98, 111, 100, 121, 62, 10, 60, 47, 104, 116, 109, 108, 62, 10,
]);
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = "[::]:4433".parse()?;
    let key = private_key(&mut BufReader::new(std::fs::File::open("key.pem")?))?;
    let cert: io::Result<Vec<CertificateDer>> =
        certs(&mut BufReader::new(std::fs::File::open("cert.pem")?)).collect();
    let mut config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert?, key.unwrap())?;
    config.max_early_data_size = u32::MAX;
    config.alpn_protocols = vec![
        b"h3".to_vec(),
        b"h2".to_vec(),
        b"http/1.1".to_vec(),
        b"http/1.0".to_vec(),
    ];
    let acceptor = TlsAcceptor::from(Arc::new(config));
    let listner = TcpListener::bind(addr).await?;
    let service = service_fn(handle_request);
    let root: Arc<PathBuf> = Arc::new(PathBuf::from("assets/"));

    loop {
        let (stream, _) = listner.accept().await?;
        let acceptor = acceptor.clone();
        tokio::spawn(async move {
            let stream = acceptor.accept(stream).await.unwrap();

            if let Err(err) = Builder::new(TokioExecutor::new())
                .serve_connection(TokioIo::new(stream), service)
                .await
            {
                eprintln!("{}", err)
            }
        });
    }
}
