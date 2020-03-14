#[tokio::main]
async fn main() {
    use warp::Filter;
    use mmds::filters::{get_mds, patch_mds, put_mds};
    
    let api = get_mds().or(patch_mds()).or(put_mds());

    warp::serve(api).run(([127, 0, 0, 1], 7878)).await;
}

