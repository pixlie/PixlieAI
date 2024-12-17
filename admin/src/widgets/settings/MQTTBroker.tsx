import { Component } from "solid-js";
import Heading from "../../widgets/typography/Heading";

const MQTTBroker: Component = () => {
  // We will call our API to see if we detect the MQTT broker, else we will ask user to install.
  return (
    <>
      <Heading size={3}>MQTT Server</Heading>
    </>
  );
};

export default MQTTBroker;
