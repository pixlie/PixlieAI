import { Component } from "solid-js";
import Heading from "../../widgets/typography/Heading";
import Gliner from "../../widgets/settings/Gliner";
import MQTTBroker from "../../widgets/settings/MQTTBroker";
// import Ollama from "../../widgets/settings/Ollama";
import Markdown from "../../widgets/typography/Markdown";
import StorageDir from "../../widgets/settings/StorageDir";
import { useWorkspace } from "../../stores/workspace";
import Anthropic from "../../widgets/settings/Anthropic";

const help = `
Let us walk through the setup process for Pixlie AI.
We will need [Python](https://www.python.org/), Ollama (or Anthropic's API key) and an MQTT server like [Mosquitto](https://mosquitto.org/).
`;

const Setup: Component = () => {
  const [workspace] = useWorkspace();

  return (
    <div class="max-w-screen-sm">
      <Heading size={2}>Setup</Heading>
      <Markdown text={help} />

      {!!workspace.isReady ? (
        <>
          <div class="mb-16" />
          <StorageDir />

          {!!workspace.settings?.pathToStorageDir ? (
            <>
              <div class="mb-16" />
              <Gliner />

              <div class="mb-16" />
              <MQTTBroker />

              {/* <div class="mb-16" />
              <Ollama /> */}

              <div class="mb-16" />
              <Anthropic />

              <div class="mb-16" />
            </>
          ) : (
            <div class="my-12">
              Please set the storage directory to continue.
            </div>
          )}
        </>
      ) : (
        <div class="my-12">Loading...</div>
      )}
    </div>
  );
};

export default Setup;
