from gliner import GLiNER
import paho.mqtt.client as mqtt
import paho.mqtt.enums as paho_enums
import logging
import sys
import json
import random

from paho.mqtt.reasoncodes import ReasonCode

logging.basicConfig(stream=sys.stdout, level=logging.INFO)
log = logging.getLogger(__name__)


MQTT_TOPIC = "extract_named_entities_gliner"


class ExtractionRequest:
    text: str
    labels: list[str]

    def __init__(self, payload: bytes):
        self.__dict__ = json.loads(payload)


# The callback for when the client receives a CONNACK response from the server.
def on_connect(
    client: mqtt.Client, userdata, flags, reason_code: ReasonCode, properties
):
    if reason_code != 0:
        log.error(f"Could not connect to MQTT server: {reason_code}")
        return

    client.subscribe(topic="pixlieai/{}/requests/+".format(MQTT_TOPIC), qos=2)


# The callback for when a PUBLISH message is received from the server.
def on_message(client: mqtt.Client, userdata, msg: mqtt.MQTTMessage):
    request = ExtractionRequest(msg.payload)
    # log.info("Received extraction request: {}".format(request))
    response = extract_entities(request)
    random_id = msg.topic.split("/")[-1]
    client.publish(
        "pixlieai/{}/responses/{}".format(MQTT_TOPIC, random_id),
        json.dumps(response),
        qos=2,
        retain=False,
    )
    log.info(
        "GLiNER worker: Extracted {} entities for request {}\n\n".format(
            len(response), random_id
        )
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
    client_random_id = random.randint(0, 1000)
    client = mqtt.Client(
        callback_api_version=paho_enums.CallbackAPIVersion.VERSION2,
        client_id="{}_worker_{}".format(MQTT_TOPIC, client_random_id),
        protocol=paho_enums.MQTTProtocolVersion.MQTTv5,
    )
    client.on_connect = on_connect
    client.on_message = on_message

    try:
        client.connect("51.159.172.85", 1883, 60)
        log.info("GLiNER worker {}: Connected to MQTT server".format(client_random_id))
    except ConnectionRefusedError:
        log.error(
            "GLiNER worker {}: Connection to MQTT server failed, is it running?".format(
                client_random_id
            )
        )
        exit(1)

    # Blocking call that processes network traffic, dispatches callbacks and
    # handles reconnecting.
    # Other loop*() functions are available that give a threaded interface and a
    # manual interface.
    client.loop_forever()


if __name__ == "__main__":
    main()
