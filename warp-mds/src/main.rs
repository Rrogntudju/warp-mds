#[tokio::main]
async fn main() {
    let api = filters::sudoku();

    warp::serve(api).run(([127, 0, 0, 1], 7878)).await;
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SudokuRequest {
    puzzle: String,
}

#[derive(Serialize)]
struct SolveResponse {
    status: String,
    data: String,
    message: String,
}

#[derive(Serialize)]
struct DisplayResponse {
    status: String,
    data: Vec<String>,
    message: String,
}

mod filters {
    use super::{handlers, SudokuRequest};
    use warp::Filter;

    pub fn sudoku() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("api")
            .and(warp::post())
            .and(solve().or(display()))
    }

    pub fn solve() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("solve")
            .and(warp::path::end())
            .and(json_body())
            .and_then(handlers::solve)
    }

    pub fn display() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("display")
            .and(warp::path::end())
            .and(json_body())
            .and_then(handlers::display)
    }

    fn json_body() -> impl Filter<Extract = (SudokuRequest,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(150).and(warp::body::json())
    }
}
