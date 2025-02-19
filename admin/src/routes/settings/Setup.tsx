import { Component } from "solid-js";
import Ollama from "../../widgets/settings/Ollama";
import Markdown from "../../widgets/typography/Markdown";
import StorageDir from "../../widgets/settings/StorageDir";
import { useWorkspace } from "../../stores/workspace";
import Anthropic from "../../widgets/settings/Anthropic";

const help = `
To run Pixlie AI, we need a storage space on your computer,
[Python](https://www.python.org/) and Ollama (or Anthropic's API key).
`;

const Setup: Component = () => {
  const [workspace] = useWorkspace();

  return (
    <div class="max-w-screen-sm">
      <Markdown text={help} />

      {workspace.isReady ? (
        <>
          <div class="mb-16" />
          <StorageDir />

          {!!workspace.settings?.pathToStorageDir ? (
            <>
              <div class="mb-16" />
              <Ollama />

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
