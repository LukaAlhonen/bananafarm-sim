use log::{error, info};
use rumqttc::v5::{mqttbytes::QoS, AsyncClient, ClientError, Event, MqttOptions};
use std::time::Duration;
use tokio::task;
use uuid::Uuid;

pub struct PublisherParams {
    pub broker_address: String,
    pub broker_port: u16,
}

// Represents a rumqtt AsyncClient that acts as a publisher
pub struct MqttPublisher {
    client: AsyncClient,
}

impl MqttPublisher {
    // pub fn new<S: Into<String>>(name: S, broker_address: S, broker_port: u16) -> Self {
    pub fn new(
        PublisherParams {
            broker_address,
            broker_port,
        }: PublisherParams,
    ) -> Self {
        let name = format!("pub:{}", Uuid::new_v4());
        let mut mqttoptions = MqttOptions::new(&name, &broker_address, broker_port);
        mqttoptions.set_keep_alive(Duration::from_secs(5));
        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

        // start background poll when client is created
        task::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(Event::Incoming(incoming)) => info!("Incoming: {:?}", incoming),
                    Ok(Event::Outgoing(outgoing)) => info!("Outgoing: {:?}", outgoing),
                    Err(err) => error!("Error: {}", err),
                }
            }
        });

        MqttPublisher { client: client }
    }

    // publish new message to a chosen topic
    pub async fn publish<S: Into<String>>(&self, topic: S, message: S) -> Result<(), ClientError> {
        self.client
            .publish(topic.into(), QoS::AtLeastOnce, false, message.into())
            .await
    }
}
