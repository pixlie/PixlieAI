import { Component } from "solid-js";
import Heading from "../../widgets/typography/Heading";
import LocalPythonEnv from "../../widgets/settings/LocalPythonEnv";
import MQTTBroker from "../../widgets/settings/MQTTBroker";
import Ollama from "../../widgets/settings/Ollama";
import Markdown from "../../widgets/typography/Markdown";
import StorageDir from "../../widgets/settings/StorageDir";

const setupIntroduction = `
Let up walk through the initial setup process for Pixlie AI.
We will need a local installation of [Python](https://www.python.org/) and an MQTT server, like [Mosquitto](https://mosquitto.org/).

Don't worry if you are unsure about these, we will walk you through the process.
`;

const Setup: Component = () => {
  return (
    <>
      <Heading size={2}>Setup</Heading>
      <Markdown text={setupIntroduction} />

      <div class="mb-4" />
      <StorageDir />

      <div class="mb-4" />
      <LocalPythonEnv />

      <div class="mb-4" />
      <MQTTBroker />

      <div class="mb-4" />
      <Ollama />

      <div class="mb-4" />
    </>
  );
};

export default Setup;
