#[tokio::main]
async fn main() {
    use mmds::filters::{get_mds, patch_mds, put_mds};
    use std::env::args;
    use std::net::SocketAddr;
    use warp::Filter;

    let api = get_mds().or(patch_mds()).or(put_mds());

    let arg = match args().nth(1) {
        Some(arg) => arg,
        None => "127.0.0.1:8080".to_string(),
    };

    match arg.parse::<SocketAddr>() {
        Ok(addr) => warp::serve(api).run(addr).await,
        Err(e) => eprintln!("{}", e.to_string()),
    }
}
