use clap::{
    crate_version, crate_authors, crate_description, crate_name,
    App, Arg
};
use std::net::SocketAddr;
use std::convert::Infallible;
use hyper::{Server, Response, Body, Request,
    Method, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use futures::TryStreamExt;

#[tokio::main]
async fn main() {
    //Parse command line options
    let matches = App::new(crate_name!())
        .author(crate_authors!())
        .about(crate_description!())
        .version(crate_version!())
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .help("Port to run the server on")
                .takes_value(true)
        )
        .get_matches();

    let port = matches.value_of("port").unwrap_or("8080").parse::<u16>().unwrap();
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let mk_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(hello_world))
    });

    let server = Server::bind(&addr).serve(mk_svc);
    if let Err(e) = server.await {
        eprintln!("{:?}", e);
    }
}

async fn hello_world(req: Request<Body>) -> Result<Response<Body>, Infallible>
{
    let mut response = Response::new(Body::empty());

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            *response.body_mut() = Body::from("Only POST requests are processed");
        },
        (&Method::POST, "/echo") => {
            *response.body_mut() = req.into_body();
        },
        (&Method::POST, "/echo_up") => {
            let mapping = req.into_body()
                .map_ok(|chunk| {
                    chunk.iter()
                        .map(|byte| {
                            byte.to_ascii_uppercase()
                        })
                        .collect::<Vec<u8>>()
                });

            *response.body_mut() = Body::wrap_stream(mapping);
        },
        (&Method::POST, "/echo_rev") => {
            let full_body = hyper::body::to_bytes(req.into_body()).await;

            let full_body = full_body.unwrap();

            let reversed = full_body.iter()
                .rev()
                .cloned()
                .collect::<Vec<u8>>();
            *response.body_mut() = reversed.into();
        },
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };

    Ok(response)
}