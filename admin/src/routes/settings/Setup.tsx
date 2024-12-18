import { Component } from "solid-js";
import Heading from "../../widgets/typography/Heading";
import LocalPythonEnv from "../../widgets/settings/Python";
import MQTTBroker from "../../widgets/settings/MQTTBroker";
import Ollama from "../../widgets/settings/Ollama";
import Markdown from "../../widgets/typography/Markdown";
import StorageDir from "../../widgets/settings/StorageDir";
import { useWorkspace } from "../../stores/Workspace";
import Anthropic from "../../widgets/settings/Anthropic";

const help = `
Let up walk through the initial setup process for Pixlie AI.
We will need a local installation of [Python](https://www.python.org/) and an MQTT server, like [Mosquitto](https://mosquitto.org/).

Don't worry if you are unsure about these, we will walk you through the process.
`;

const Setup: Component = () => {
  const [workspace] = useWorkspace();

  return (
    <div class="max-w-screen-sm">
      <Heading size={2}>Setup</Heading>
      <Markdown text={help} />

      {!!workspace.isReady ? (
        <>
          <div class="mb-12" />
          <StorageDir />

          <div class="mb-12" />
          <LocalPythonEnv />

          <div class="mb-12" />
          <MQTTBroker />

          <div class="mb-12" />
          <Ollama />

          <div class="mb-12" />
          <Anthropic />
        </>
      ) : (
        <div class="my-12">Loading...</div>
      )}
    </div>
  );
};

export default Setup;
