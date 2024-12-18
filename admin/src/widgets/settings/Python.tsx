import { Component } from "solid-js";
import Heading from "../../widgets/typography/Heading";
import { useWorkspace } from "../../stores/Workspace";
import Markdown from "../typography/Markdown";
import Paragraph from "../typography/Paragraph";

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

      {workspace.settingsStatus?.type === "Incomplete" &&
      workspace.settingsStatus?.data.includes("PythonNotAvailable") ? (
        <Paragraph>
          We need Python, version 3.9 or above, installed on this computer
        </Paragraph>
      ) : (
        <Paragraph>Python is available</Paragraph>
      )}
    </>
  );
};

export default LocalPythonEnv;
