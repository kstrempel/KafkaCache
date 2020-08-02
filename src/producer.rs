#[macro_use]
extern crate log;

use rdkafka::config::ClientConfig;
use rdkafka::error::KafkaError;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rand::Rng;


fn kafka_producer(brokers: &str) -> Result<FutureProducer, KafkaError> {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .create()
}

async fn publish_line_to_topic(topic: String, line: String, key: String, producer: FutureProducer) {
    let rec = FutureRecord::to(topic.as_str())
        .payload(line.as_str())
        .key(key.as_str());
    match producer.clone().send(rec, 0).await {
        Ok(Ok(_)) => {}
        Ok(Err((kakfa_error, _))) => error!(
            "{}",
            format!("failed to publish to kafka, {}", kakfa_error.to_string())
        ),
        Err(cancelled_err) => error!("{}", format!("cancelled, {}", cancelled_err.to_string())),
    }
}


#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    info!("Starting to inject data updates");

    let producer = kafka_producer("localhost:9092").unwrap();

    loop {
        let id = rand::thread_rng().gen_range(0, 1000000);
        let value = rand::thread_rng().gen_range(0, 1000000000);
        tokio::spawn(publish_line_to_topic(String::from("test"), 
                                           value.to_string(), 
                                           id.to_string(), 
                                           producer.clone()));
    }
}