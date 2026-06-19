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
struct Pv{
    moves:String,
}

#[derive(Deserialize)]
struct LichessResp{
    pvs:Vec<Pv>,
}

async fn get_move(Query(req):Query<Req>)->Json<Resp>{

        let mut position:Chess=req.fen
        .parse::<Fen>()
        .unwrap()
        .into_position::<Chess>(shakmaty::CastlingMode::Standard)
        .unwrap()
        .into();

        let lichess_call=format!(
                "https://lichess.org/api/cloud-eval?fen={}",
                urlencoding::encode(&req.fen)
        );

        let response=ureq::get(&lichess_call)
        .call()
        .unwrap()
        .into_string()
        .unwrap();

        let lichess:LichessResp=
        serde_json::from_str(&response)
        .unwrap();

        let best_move=lichess.pvs[0]
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

#[tokio::main]
async fn main(){
        let app=Router::new()
        .route("/move",get(get_move));

        let listener=tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

        axum::serve(listener,app).await.unwrap();
}
