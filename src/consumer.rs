  
use std::time::Duration;
use rdkafka::config::ClientConfig;
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::error::KafkaError;
use rdkafka::message::{BorrowedMessage, Message as KafkaMessage};
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::{Arc};
use actix_web::web;


#[derive(Debug, Clone)]
pub struct MessagePayload(String);

pub struct AppStateMemory {
    pub memory: Arc<HashMap<u32, &'static str>>
}

/// generic way to turn a borrowed message into a (wrapped) string
impl<'a> From<&'a BorrowedMessage<'a>> for MessagePayload {
    fn from(bm: &'a BorrowedMessage) -> Self {
        match bm.payload_view::<str>() {
            Some(Ok(s)) => MessagePayload(String::from(s)),
            Some(Err(e)) => MessagePayload(format!("{:?}", e)),
            None => MessagePayload(String::from("")),
        }
    }
}

pub struct IngestConsumer {
    consumer: StreamConsumer,
}

impl IngestConsumer {
    pub fn new() -> Result<Self, KafkaError> {
        info!("create consumer");
        let topics = vec!["test"];
        let consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", "test-group")
            .set("bootstrap.servers", &"localhost:9092")
            .set("auto.offset.reset", "latest")
            .set("enable.partition.eof", "true")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .set_log_level(RDKafkaLogLevel::Debug)
            .create()?;
        consumer.subscribe(topics.as_slice())?;
        Ok(IngestConsumer { consumer })
    }

    pub async fn run(&self, data: web::Data<AppStateMemory>) {
        let mut stream = self.consumer.start_with(Duration::from_millis(50), false);
        loop {
            match stream.next().await {
                Some(Ok(borrowed_message)) => {
                    let message_payload = MessagePayload::from(&borrowed_message);
                    let key = borrowed_message.key();
                    info!("Received message {:?}-{:?}", key, message_payload);
                    {}
                }
                Some(Err(kafka_error)) => match kafka_error {
                    KafkaError::PartitionEOF(partition) => {
                        info!("at end of partition {:?}", partition);
                    }
                    _ => {},
                },
                None => {}
            }
        }
    }
}