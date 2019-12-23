use headers::{
    authorization::Credentials,
    Authorization,
};

use hyper::{
    client::HttpConnector,
    header,
    service::{
        make_service_fn,
        service_fn,
    },
    Client,
    Error,
    Server,
    Uri,
};

use hyper_tls::HttpsConnector;
use log::*;
use std::net::SocketAddr;
use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
#[structopt(about, rename_all = "kebab-case")]
struct Opt {
    /// Socket address to listen on.
    #[structopt(long, short = "L", default_value = "127.0.0.1:3001")]
    listen_addr: SocketAddr,
    /// Remote URL to proxy to.
    remote_url: Uri,
    /// Target host username.
    #[structopt(long)]
    username: Option<String>,
    /// Target host password.
    #[structopt(long)]
    password: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let opt = Opt::from_args();

    pretty_env_logger::init();

    let mut http = HttpConnector::new();
    http.enforce_http(false);

    let tls = native_tls::TlsConnector::builder()
        // .danger_accept_invalid_certs(true)
        .build()?;

    let https = HttpsConnector::from((http, tls.into()));
    let client = Client::builder().build(https);

    let out_uri_clone = opt.remote_url.clone();
    let authz = if let Some(ref username) = opt.username {
        let authz = Authorization::basic(username, &opt.password.unwrap_or_default());
        Some(authz.0.encode())
    } else {
        None
    };

    let make_service = make_service_fn(move |_| {
        let client = client.clone();
        let out_uri = out_uri_clone.clone();
        let authz = authz.clone();

        async move {
            Ok::<_, Error>(service_fn(move |mut req| {
                let mut uri_parts = out_uri.clone().into_parts();
                if let Some(path_and_query) = req.uri().path_and_query() {
                    uri_parts.path_and_query = Some(path_and_query.clone());
                }

                let uri = Uri::from_parts(uri_parts).unwrap();
                *req.uri_mut() = uri;

                let headers = req.headers_mut();
                headers.remove(header::HOST);

                if let Some(ref authz) = authz {
                    headers.insert(header::AUTHORIZATION, authz.clone());
                }

                info!("{:?}", req);
                client.request(req)
            }))
        }
    });

    let server = Server::bind(&opt.listen_addr).serve(make_service);

    info!("Listening on http://{}", &opt.listen_addr);
    info!("Proxying to {}", &opt.remote_url);

    server.await.map_err(|e| e.into())
}
