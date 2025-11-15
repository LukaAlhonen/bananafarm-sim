use std::time::Duration;

use log::{error, info};
use rumqttc::v5::{
    mqttbytes::QoS, AsyncClient, ClientError, ConnectionError, Event, EventLoop, Incoming,
    MqttOptions,
};
use uuid::Uuid;

pub struct MqttSubscriber {
    client: AsyncClient,
    eventloop: EventLoop,
}

pub struct SubscriberParams {
    pub broker_address: String,
    pub broker_port: u16,
}

impl MqttSubscriber {
    pub fn new(
        SubscriberParams {
            broker_address,
            broker_port,
        }: SubscriberParams,
    ) -> Self {
        let name = format!("sub:{}", Uuid::new_v4());
        let mut mqttoptions = MqttOptions::new(&name, &broker_address, broker_port);
        mqttoptions.set_keep_alive(Duration::from_secs(5));
        let (client, eventloop) = AsyncClient::new(mqttoptions, 10);
        MqttSubscriber { client, eventloop }
    }

    pub async fn subscribe(&self, topic: &str) -> Result<(), ClientError> {
        self.client.subscribe(topic, QoS::AtLeastOnce).await?;

        Ok(())
    }

    // subscribes to a topic and waits for a suback from the broker
    pub async fn subscribe_ack(&mut self, topic: &str) -> Result<(), ClientError> {
        self.subscribe(topic).await?;

        // loop untill suback recieved
        loop {
            match self.poll().await {
                Ok(Event::Incoming(Incoming::SubAck(_))) => {
                    info!("subscribed to {}", topic);
                    break;
                }
                Ok(_) => {}
                Err(err) => {
                    error!("Error subscribing: {}", err)
                }
            }
        }

        Ok(())
    }

    pub async fn poll(&mut self) -> Result<Event, ConnectionError> {
        self.eventloop.poll().await
    }
}
