use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use temperature_tools::TimedTemp;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[get("/")]
async fn temps_index() -> impl Responder {
    let all_temps = temperature_tools::all_temps().await;

    match all_temps {
        Ok(temps) => {
            HttpResponse::Ok().json(temps)
        },
        Err(err) => {
            HttpResponse::BadRequest().body(format!("{}", err))
        }
    }
}

#[get("/latest")]
async fn temps_latest() -> impl Responder {
    let last_temp = temperature_tools::last_temp().await;
    match last_temp {
        Ok(TimedTemp{timestamp, centigrade}) => {
            HttpResponse::Ok().body(format!("{} centigrade, at {}", centigrade, timestamp))
        },
        Err(err) => {
            HttpResponse::BadRequest().body(format!("{}", err))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let address = dotenv::var("HOME_ADDRESS").unwrap_or("127.0.0.1:8080".to_string());
    println!("Serving on : {}", address);

    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .service(web::scope("/temperature").service(temps_latest).service(temps_index))
            .route("/hey", web::get().to(manual_hello))
    })
        .bind(address)?
        .run()
        .await
}
