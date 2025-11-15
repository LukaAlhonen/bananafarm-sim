pub mod mqtt_publisher;
pub mod sensor;

use clap::Parser;
use influxdb3client::SoilMoistureMeasurement;
use log::{error, info};
use sensor::Sensor;
use std::env;
use std::time::Duration;
use tokio::time;

use crate::mqtt_publisher::{MqttPublisher, PublisherParams};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    name: String,
    #[arg(long)]
    sensor_id: String,
    #[arg(long)]
    location: String,
}

#[tokio::main]
async fn main() {
    // Init env and logger
    env_logger::init();
    dotenv::from_path("publisher/.env").ok();

    // Load env vars
    let broker_address = env::var("BROKER_ADDRESS").expect("BROKER_ADDRESS MUST BE SET");
    let broker_port = env::var("BROKER_PORT").expect("BROKER_PORT MUST BE SET");
    let topic = env::var("TOPIC").expect("TOPIC MUST BE SET");
    let location = env::var("LOCATION").expect("LOCATION MUST BE SET");

    let client = MqttPublisher::new(PublisherParams {
        broker_address,
        broker_port: broker_port.parse().expect("Failed to parse broker port"),
    });

    let mut sensor = Sensor::new();
    sensor._seed(30.2_f32);

    loop {
        let data = sensor.read();
        let measurement: SoilMoistureMeasurement =
            SoilMoistureMeasurement::new(data, "cb", &sensor.get_id_string(), &location);

        match measurement.into_payload() {
            Ok(message) => match &client.publish(&topic, &message).await {
                Ok(_) => info!("Published message to {}", &topic),
                Err(e) => error!("Error when publishing message: {}", e.to_string()),
            },
            Err(err) => error!("Error parsing measurement: {}", err),
        }

        // wait for 1 minute before sending new message
        time::sleep(Duration::from_millis(6000)).await;
    }
}
