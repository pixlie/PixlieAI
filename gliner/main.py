from gliner import GLiNER
import paho.mqtt.client as mqtt
import paho.mqtt.enums as paho_enums
import logging
import sys
import json

logging.basicConfig(stream=sys.stdout, level=logging.INFO)
log = logging.getLogger(__name__)


MQTT_TOPIC = "pixlieai/extract_named_entities_gliner"


class ExtractionRequest:
    text: str
    labels: list[str]

    def __init__(self, payload: bytes):
        self.__dict__ = json.loads(payload)


# The callback for when the client receives a CONNACK response from the server.
def on_connect(client, userdata, flags, reason_code, properties):
    log.info(f"Connected to message broker with result code")
    log.info(f"Listening for entity extraction requests from Pixlie AI")
    # Subscribing in on_connect() means that if we lose the connection and
    # reconnect then subscriptions will be renewed.
    client.subscribe("{}/requests".format(MQTT_TOPIC))


# The callback for when a PUBLISH message is received from the server.
def on_message(client: mqtt.Client, userdata, msg: mqtt.MQTTMessage):
    request = ExtractionRequest(msg.payload)
    log.info("Received extraction request: {}".format(request))
    response = extract_entities(request)
    client.publish(
        "{}/responses".format(MQTT_TOPIC),
        json.dumps(response),
        qos=2,
        retain=False,
    )


def extract_entities(extraction_request: ExtractionRequest):
    # Initialize GLiNER with the base model
    # model = GLiNER.from_pretrained("urchade/gliner_mediumv2.1")
    model = GLiNER.from_pretrained("EmergentMethods/gliner_medium_news-v2.1")

    # Perform entity prediction
    entities = model.predict_entities(
        extraction_request.text, extraction_request.labels, threshold=0.5
    )

    return entities


def main():
    log.info("Starting GLiNER")

    client = mqtt.Client(
        callback_api_version=paho_enums.CallbackAPIVersion.VERSION2,
        client_id="pixlieai_gliner_subscriber",
        protocol=paho_enums.MQTTProtocolVersion.MQTTv5,
    )
    client.on_connect = on_connect
    client.on_message = on_message

    try:
        client.connect("localhost", 1883, 60)
    except ConnectionRefusedError:
        log.error("Connection to MQTT server failed, is it running?")
        exit(1)

    # Blocking call that processes network traffic, dispatches callbacks and
    # handles reconnecting.
    # Other loop*() functions are available that give a threaded interface and a
    # manual interface.
    client.loop_forever()


if __name__ == "__main__":
    main()
