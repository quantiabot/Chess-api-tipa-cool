use axum::{Router, routing::post, Json};
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

async fn get_move(Json(req):Json<Req>)->Json<Resp>{
        let mut position=req.fen.parse::<Fen>()
        .unwrap()
        .into_position::<Chess>(shakmaty::CastlingMode::Standard)
        .unwrap()
        .into();

        let lichess_call=format!(
                "https://lichess.org/api/cloud-eval?fen={}",
                urlencoding::encode(&req.fen)
        );

        let response=ureq::get(&lichess_call)
        .call().unwrap()
        .into_string().unwrap();

        let lichess_response:RespLichess=
                serde_json::from_str(&response).unwrap();

        let best_move=lichess_response.pvs[0]
        .moves
        .split_whitespace()
        .next()
        .unwrap();

        let move_played=best_move
        .parse::<UciMove>()
        .unwrap()
        .to_move(&position)
        .unwrap();

        position=position.play(move_played).unwrap();

        let new_fen=Fen::from_position(
                position.clone(),
                shakmaty::EnPassantMode::Legal,
        ).to_string();

        Json(Resp{
                best_move:best_move.to_string(),
                new_fen
        })
}

#[derive(Deserialize)]
struct Pv{
    moves:String,
}

#[derive(Deserialize)]
struct RespLichess{
    pvs:Vec<Pv>,
}

#[tokio::main]
async fn main(){
        let app=Router::new()
        .route("/move",post(get_move));

        let listener=tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

        axum::serve(listener,app).await.unwrap();
}
