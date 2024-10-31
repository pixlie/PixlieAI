from gliner import GLiNER
import paho.mqtt.client as mqtt
import paho.mqtt.enums as paho_enums
import logging
import sys
# from dataclasses import dataclass

# @dataclass
# class Extracted:
#     start: int
#     end: int
#     text: str
#     label: str
#     score: float

logging.basicConfig(stream=sys.stdout, level=logging.INFO)
log = logging.getLogger(__name__)

# The callback for when the client receives a CONNACK response from the server.
def on_connect(client, userdata, flags, reason_code, properties):
    log.info(f"Connected to message broker with result code")
    log.info(f"Listening for entity extraction requests from Pixlie AI")
    # Subscribing in on_connect() means that if we lose the connection and
    # reconnect then subscriptions will be renewed.
    client.subscribe("pixlieai/extract_entities_gliner")

# The callback for when a PUBLISH message is received from the server.
def on_message(client, userdata, msg):
    print(msg.topic+" "+str(msg.payload))

def extract_entities(text, labels):
    # Initialize GLiNER with the base model
    # model = GLiNER.from_pretrained("urchade/gliner_mediumv2.1")
    model = GLiNER.from_pretrained("EmergentMethods/gliner_medium_news-v2.1")

    # Perform entity prediction
    entities = model.predict_entities(text, labels, threshold=0.5)

    return entities

def main():
    log.info("Starting GLiNER")

    mqttc = mqtt.Client(paho_enums.CallbackAPIVersion.VERSION2)
    mqttc.on_connect = on_connect
    mqttc.on_message = on_message

    try:
        mqttc.connect("localhost", 1883, 60)
    except ConnectionRefusedError:
        log.error("Connection to MQTT server failed, is it running?")
        exit(1)

    # Blocking call that processes network traffic, dispatches callbacks and
    # handles reconnecting.
    # Other loop*() functions are available that give a threaded interface and a
    # manual interface.
    mqttc.loop_forever()


if __name__ == "__main__":
    main()
