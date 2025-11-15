pub mod mqtt_subscriber;

use influxdb3client::{InfluxDB3Client, SoilMoistureMeasurement};
use log::{error, info};
use mqtt_subscriber::{MqttSubscriber, SubscriberParams};
use std::env;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    // Load env
    dotenv::from_path("subscriber/.env").ok();

    // Db env vars
    let token = env::var("INFLUXDB_TOKEN").expect("DB_TOKEN MUST BE SET");
    let db_address = env::var("DB_ADDRESS").expect("DB_ADDRESS MUST BE SET");
    let table = env::var("TABLE").expect("TABLE MUST BE SET");

    // Rumqtt env vars
    let topic = env::var("TOPIC").expect("TOPIC MUST BE SET");
    let broker_address = env::var("BROKER_ADDRESS").expect("BROKER_ADDRESS MUST BE SET");
    let broker_port = env::var("BROKER_PORT").expect("BROKER_PORT MUST BE SET");

    let db_client = InfluxDB3Client::new(db_address, token, table);
    let mut client = MqttSubscriber::new(SubscriberParams {
        broker_address,
        broker_port: broker_port.parse().expect("Failed to parse broker port"),
    });
    env_logger::init();

    match client.subscribe_ack(&topic).await {
        Ok(_) => info!("subscribed to topic: {}", &topic),
        Err(err) => error!("error subscribing to topic {}: {}", &topic, err),
    }

    if let Ok(event) = client.poll().await {
        info!("After subscribe got event: {:?}", event);
    }

    let (tx, mut rx) = mpsc::channel::<SoilMoistureMeasurement>(100);

    // separate thread for writing to database
    tokio::spawn(async move {
        while let Some(measurement) = rx.recv().await {
            // write to db
            match db_client.write_query_with_retry(&measurement, 5).await {
                Ok(_) => info!("message written to db"),
                Err(err) => error!("error writing message to db: {}", err),
            }
        }
    });

    loop {
        let notification = client.poll().await;
        match notification {
            // If message recieved on subscribed topic -> write to db
            Ok(rumqttc::v5::Event::Incoming(rumqttc::v5::Incoming::Publish(publish))) => {
                if let Ok(message) = String::from_utf8(publish.payload.to_vec()) {
                    match SoilMoistureMeasurement::from_payload(&message) {
                        // write message to database
                        Ok(parsed_message) => {
                            if let Err(err) = tx.send(parsed_message).await {
                                error!("Error sending message to mpsc channel: {}", err)
                            }
                        }
                        Err(err) => error!("Error parsing message: {}", err),
                    }
                } else {
                    error!("Error decoding message payload as utf-8");
                }
            }
            Ok(_) => {
                info!("{:?}", notification);
            }
            Err(e) => {
                error!("Connecion error: {:?}", e);
                break;
            }
        }
    }
}
