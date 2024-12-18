import { Component } from "solid-js";
import Heading from "../../widgets/typography/Heading";
import { useWorkspace } from "../../stores/Workspace";
import Markdown from "../typography/Markdown";

const help = `
Pixlie AI needs a Python environment to run some of the AI/ML tools.
`;

const LocalPythonEnv: Component = () => {
  // We need a local Python virtual environment. We are our API if it can detect system Python and venv.
  const [workspace] = useWorkspace();

  return (
    <>
      <Heading size={3}>Python</Heading>
      <Markdown text={help} />

      <div class="mt-4">
        {workspace.settingsStatus?.type === "Incomplete" &&
        workspace.settingsStatus?.data.includes("PythonNotAvailable") ? (
          <>We need Python, version 3.9 or above, installed on this computer</>
        ) : (
          <>Python is available</>
        )}
      </div>
    </>
  );
};

export default LocalPythonEnv;
