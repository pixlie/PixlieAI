use crate::error::PiResult;
use log::debug;
use rumqttc::v5::{mqttbytes::QoS, MqttOptions};
use rumqttc::v5::{AsyncClient, Incoming};
use std::time::Duration;
use tokio::{task, time};

pub async fn check_mqtt_broker() -> PiResult<()> {
    // We test the connection to the MQTT broker by subscribing to a topic and waiting for a response.
    let mqtt_options = MqttOptions::new("pixlieai_test", "localhost", 1883);
    let (client, mut connection) = AsyncClient::new(mqtt_options, 10);
    client.subscribe("test/topic", QoS::AtMostOnce).await?;
    debug!("Subscribed to test/topic on MQTT broker");
    task::spawn(async move {
        publish(client).await;
    });

    // Iterate to poll the eventloop for connection progress
    loop {
        let event = connection.poll().await;
        match event {
            Ok(rumqttc::v5::Event::Incoming(Incoming::ConnAck(_))) => {
                debug!("Connected to MQTT broker");
                break;
            }
            Err(e) => {
                debug!("Connection error: {:?}", e);
                return Err(e.into());
            }
            _ => {}
        }
        break;
    }
    Ok(())
}

async fn publish(client: AsyncClient) {
    client
        .subscribe("test/topic", QoS::AtMostOnce)
        .await
        .unwrap();
    for i in 0..2_usize {
        let payload = vec![1; i];
        let topic = format!("test/topic");
        let qos = QoS::AtLeastOnce;

        let _ = client.publish(topic, qos, true, payload).await;
    }

    time::sleep(Duration::from_secs(1)).await;
}
