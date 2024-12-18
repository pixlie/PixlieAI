import { Component } from "solid-js";
import Heading from "../../widgets/typography/Heading";
import Markdown from "../typography/Markdown";

const help = `We need a message queueing server to coordinate the different parts of Pixlie AI.
We use MQTT as the protocol, and we recommend you install [Mosquitto](https://mosquitto.org/) to run the server locally.`;

const MQTTBroker: Component = () => {
  // We will call our API to see if we detect the MQTT broker, else we will ask user to install.
  return (
    <>
      <Heading size={3}>Message Queue</Heading>
      <Markdown text={help} />
    </>
  );
};

export default MQTTBroker;
