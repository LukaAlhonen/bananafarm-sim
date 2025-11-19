#[cfg(test)]
mod tests {
    use influxdb3client::SoilMoistureMeasurement;
    use publisher::mqtt_publisher::{MqttPublisher, PublisherParams};
    use publisher::sensor::Sensor;
    use rumqttc::v5::{Event, Incoming};
    use subscriber::mqtt_subscriber::{MqttSubscriber, SubscriberParams};
    use tokio::{sync::mpsc, task, time};

    #[tokio::test]
    async fn test_publish_subscribe() {
        // create subscriber
        let mut sub_client = MqttSubscriber::new(SubscriberParams {
            broker_address: String::from("localhost"),
            broker_port: 1884,
        });

        // create publisher and sensor
        let pub_client = MqttPublisher::new(PublisherParams {
            broker_address: String::from("localhost"),
            broker_port: 1884,
        });
        let sensor = Sensor::new();

        // subscribe and init mpsc channel
        sub_client.subscribe_ack("test/topic").await.unwrap();
        let (tx, mut rx) = mpsc::channel::<SoilMoistureMeasurement>(100);

        // start polling for messages
        task::spawn(async move {
            loop {
                if let Ok(Event::Incoming(Incoming::Publish(publish))) = sub_client.poll().await {
                    if let Ok(message) = String::from_utf8(publish.payload.to_vec()) {
                        if let Ok(parsed_message) = SoilMoistureMeasurement::from_payload(&message)
                        {
                            tx.send(parsed_message).await.unwrap();
                        }
                    }
                }
            }
        });

        // create new measurement and publish
        let data = sensor.read();
        let measurement =
            SoilMoistureMeasurement::new(data, "cb", sensor.get_id_string().as_str(), "loc-1");
        let message = measurement.into_payload().unwrap();
        pub_client.publish("test/topic", &message).await.unwrap();

        // Read received message from buffer
        let mut message_received = None;
        let timeout = time::Duration::from_secs(5);
        let start = time::Instant::now();
        while start.elapsed() < timeout {
            if let Ok(m) = time::timeout(time::Duration::from_millis(500), rx.recv()).await {
                if let Some(m) = m {
                    message_received = Some(m);
                    break;
                }
            }
        }

        // verify message
        assert_eq!(Some(measurement), message_received);
    }
}
