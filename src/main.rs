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

    let mut position:Chess=req.fen
        .parse::<Fen>()
        .unwrap()
        .into_position::<Chess>(shakmaty::CastlingMode::Standard)
        .unwrap()
        .into();

    let url=format!(
        "https://chess-api.com/v1?fen={}",
        urlencoding::encode(&req.fen)
    );

    let response=ureq::get(&url)
        .call()
        .unwrap()
        .into_string()
        .unwrap();

    let api:ApiResp=serde_json::from_str(&response).unwrap();

    let best=api.r#move
        .or(api.bestMove)
        .unwrap_or("0000".to_string());

    let move_played=best
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
        best_move:best,
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
