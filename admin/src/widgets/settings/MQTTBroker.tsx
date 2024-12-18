import { Component, createResource } from "solid-js";
import Heading from "../../widgets/typography/Heading";
import Markdown from "../typography/Markdown";
import { getPixlieAIAPIRoot } from "../../utils/api";

const help = `We need a message queueing server to coordinate the different parts of Pixlie AI.
We use MQTT as the protocol, and we recommend you install [Mosquitto](https://mosquitto.org/) to run the server locally.`;

const MQTTBroker: Component = () => {
  // We will call our API to see if we detect the MQTT broker, else we will ask user to install.
  // We create a SolidJS resource and call our API check_mqtt_broker_connection
  const [data] = createResource(async () => {
    let pixieAIAPIRoot = getPixlieAIAPIRoot();
    const res = await fetch(`${pixieAIAPIRoot}/api/settings/check_mqtt_broker`);
    return await res.text();
  });

  return (
    <>
      <Heading size={3}>Message Queue</Heading>
      <Markdown text={help} />

      <div class="mt-4">
        {data() === "OK" ? (
          <>MQTT Server found</>
        ) : (
          <>
            No MQTT Server found. We suggest you install and run Mosquitto and
            then refresh this page.
          </>
        )}
      </div>
    </>
  );
};

export default MQTTBroker;
