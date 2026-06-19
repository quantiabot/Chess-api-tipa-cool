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
    best_move:Option<String>,
    new_fen:String,
    error:Option<String>,
}

#[derive(Deserialize)]
struct ApiResp{
    r#move:Option<String>,
}

async fn get_move(Query(req):Query<Req>)->Json<Resp>{

    let mut position:Chess = match req.fen.parse::<Fen>() {
        Ok(fen) => match fen.into_position::<Chess>(shakmaty::CastlingMode::Standard) {
            Ok(p) => p.into(),
            Err(_) => {
                return Json(Resp{
                    best_move:None,
                    new_fen:req.fen,
                    error:Some("invalid fen".to_string()),
                });
            }
        },
        Err(_) => {
            return Json(Resp{
                best_move:None,
                new_fen:req.fen,
                error:Some("fen parse error".to_string()),
            });
        }
    };

    let response = match ureq::post("https://chess-api.com/v1")
        .set("Content-Type", "application/json")
        .send_json(ureq::json!({
            "fen": req.fen
        })) {
            Ok(r) => r.into_string().unwrap_or_default(),
            Err(_) => {
                return Json(Resp{
                    best_move:None,
                    new_fen:req.fen,
                    error:Some("api request failed".to_string()),
                });
            }
    };

    let api: ApiResp = match serde_json::from_str(&response) {
        Ok(v) => v,
        Err(_) => {
            return Json(Resp{
                best_move:None,
                new_fen:req.fen,
                error:Some("invalid api response".to_string()),
            });
        }
    };

    let best = match api.r#move {
        Some(m) => m,
        None => {
            return Json(Resp{
                best_move:None,
                new_fen:req.fen,
                error:Some("no move returned".to_string()),
            });
        }
    };

    let move_played = match best.parse::<UciMove>() {
        Ok(m) => match m.to_move(&position) {
            Ok(mv) => mv,
            Err(_) => {
                return Json(Resp{
                    best_move:Some(best),
                    new_fen:req.fen,
                    error:Some("illegal move".to_string()),
                });
            }
        },
        Err(_) => {
            return Json(Resp{
                best_move:Some(best),
                new_fen:req.fen,
                error:Some("move parse error".to_string()),
            });
        }
    };

    position = match position.clone().play(move_played) {
        Ok(p) => p,
        Err(_) => position,
    };

    let new_fen = Fen::from_position(
        &position,
        shakmaty::EnPassantMode::Legal,
    ).to_string();

    Json(Resp{
        best_move:Some(best),
        new_fen,
        error:None,
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
