mod consumer;

#[macro_use]
extern crate log;

use actix_web::{HttpResponse, Responder, web};
use failure::Fail;
use crate::consumer::{IngestConsumer, AppStateMemory};
use std::collections::HashMap;
use std::sync::{Arc};


#[derive(Fail, Debug)]
#[fail(display = "kafkacache")]
pub struct KCError {
    name: &'static str,
}

async fn get_key(key: web::Path<(u32,)>, data: web::Data<AppStateMemory>) -> HttpResponse {
    let memory = data.memory.clone();
    match memory.get(&key.0) {
        Some(v) => HttpResponse::Ok().body(*v),
        None => HttpResponse::NotFound().body("can't found key")
    }           
}

async fn info(data: web::Data<AppStateMemory>) -> impl Responder {
/*    let mut memory = Arc::get_mut(&mut data.memory).unwrap();
    memory.insert(0, "Hello");
    memory.insert(1, "Hallo"); */
    HttpResponse::Ok().body("Kafka Cache Server v1.0")
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    use actix_web::{middleware::Logger, web, App, HttpServer};

    std::env::set_var("RUST_LOG", "server=debug,actix_web=debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let memory = web::Data::new(AppStateMemory {
        memory: Arc::new(HashMap::new())
    });

    match IngestConsumer::new() {
        Ok(consumer) =>  {
            info!("Start listener");
            actix_rt::spawn(async move { consumer.run(memory).await });
        },
        Err(e) => info!("Eror in starting listener {:?}", e),
    }
    
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(memory.clone())
            .route("/key/{key}", web::get().to(get_key))
            .route("/info", web::get().to(info))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}