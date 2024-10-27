use http::{Method, Request, Response, StatusCode, Version};
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
use std::sync::{Arc, LazyLock};
use tokio::{fs, net::TcpListener};
use tokio_rustls::{
    rustls::{pki_types::CertificateDer, ServerConfig},
    TlsAcceptor,
};

async fn get_resource(uri: &'static str) -> io::Result<Full<Bytes>> {
    match fs::read(uri).await {
        Ok(o) => Ok(Full::from(o)),
        Err(e) => Err(e),
    }
}
async fn tunnel() {}

async fn handle_request(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, http::Error> {
    let mut res = Response::builder();
    match req.method() {
        &Method::POST => {
            res.status(StatusCode::OK);
            res.version(Version::HTTP_3);
            //res.header(key, value);
            return res.body(());
        }
        &Method::GET => match get_resource(req.uri().path()).await {
            Ok(o) => {
                res.status(StatusCode::OK);
                res.version(Version::HTTP_3);
                //res.header(key, value);
                return res.body(o);
            }
            Err(e) => {
                res.status(StatusCode::NOT_FOUND);
                res.version(Version::HTTP_3);
                return res.body(_404);
            }
        },
        &Method::CONNECT => {
            res.status(StatusCode::OK);
            res.version(Version::HTTP_3);
            //res.header(key, value);
            return res.body(o);
        }
    }
}

//const _404 = [];
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
