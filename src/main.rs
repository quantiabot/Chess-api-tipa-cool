use axum::{Router, routing::get, extract::Query, Json};
use serde::{Deserialize, Serialize};
use shakmaty::{Chess, Position};
use shakmaty::fen::Fen;
use shakmaty::uci::UciMove;

#[derive(Deserialize)]
struct Req{
    fen:String,
}

#[derive(Serialize)]
struct Resp{
    best_move:String,
    new_fen:String,
}

#[derive(Deserialize)]
struct ApiResp{
    r#move: Option<String>,
    bestMove: Option<String>,
}

async fn get_move(Query(req):Query<Req>)->Json<Resp>{

    let mut position = match req.fen.parse::<Fen>() {
        Ok(f) => match f.into_position::<Chess>(shakmaty::CastlingMode::Standard) {
            Ok(p) => p.into(),
            Err(_) => {
                return Json(Resp{
                    best_move:"0000".to_string(),
                    new_fen:req.fen,
                });
            }
        },
        Err(_) => {
            return Json(Resp{
                best_move:"0000".to_string(),
                new_fen:req.fen,
            });
        }
    };

    let url = format!(
        "https://chess-api.com/v1?fen={}",
        urlencoding::encode(&req.fen)
    );

    let response = match ureq::get(&url).call() {
        Ok(r) => r.into_string().unwrap_or_default(),
        Err(_) => {
            return Json(Resp{
                best_move:"0000".to_string(),
                new_fen:req.fen,
            });
        }
    };

    let api: ApiResp = match serde_json::from_str(&response) {
        Ok(v) => v,
        Err(_) => {
            return Json(Resp{
                best_move:"0000".to_string(),
                new_fen:req.fen,
            });
        }
    };

    let best = api.r#move.or(api.bestMove).unwrap_or("0000".to_string());

    let move_played = match best.parse::<UciMove>() {
        Ok(m) => match m.to_move(&position) {
            Ok(mv) => mv,
            Err(_) => {
                return Json(Resp{
                    best_move:best,
                    new_fen:req.fen,
                });
            }
        },
        Err(_) => {
            return Json(Resp{
                best_move:best,
                new_fen:req.fen,
            });
        }
    };

    position = match position.play(move_played) {
        Ok(p) => p,
        Err(_) => position,
    };

    let new_fen = Fen::from_position(
        position,
        shakmaty::EnPassantMode::Legal,
    ).to_string();

    Json(Resp{
        best_move:best,
        new_fen,
    })
}

#[tokio::main]
async fn main(){

    let app = Router::new()
        .route("/move", get(get_move));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
