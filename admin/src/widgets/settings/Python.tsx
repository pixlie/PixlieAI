import { Component, createResource } from "solid-js";
import Heading from "../../widgets/typography/Heading";
import { useWorkspace } from "../../stores/Workspace";
import Markdown from "../typography/Markdown";
import Button from "../interactable/Button";
import { getPixlieAIAPIRoot } from "../../utils/api";
import { onMount } from "solid-js";

const help = `
Pixlie AI needs a Python environment to run some of the AI/ML tools.
`;

const PythonEnv: Component = () => {
  // We need a local Python virtual environment. We are our API if it can detect system Python and venv.
  const [workspace, { fetchSettingsStatus }] = useWorkspace();
  const [_settings, { refetch }] = createResource(async () => {
    await fetchSettingsStatus();
  });

  onMount(() => {
    refetch();
  });

  const handleSetupGliner = async () => {
    let pixieAIAPIRoot = getPixlieAIAPIRoot();
    const res = await fetch(`${pixieAIAPIRoot}/api/settings/setup_gliner`, {
      method: "POST",
    });
    return await res.text();
  };

  return (
    <>
      <Heading size={3}>Python</Heading>
      <Markdown text={help} />

      <div class="mt-4">
        {!workspace.isReady ? (
          <>Loading...</>
        ) : (
          <>
            {workspace.settingsStatus?.type === "Complete" && (
              <>Everything looks fine.</>
            )}

            {workspace.settingsStatus?.type === "Incomplete" &&
              !workspace.settingsStatus?.data.includes("PythonNotAvailable") &&
              !workspace.settingsStatus?.data.includes(
                "PythonPipNotAvailable",
              ) &&
              !workspace.settingsStatus?.data.includes(
                "PythonVenvNotAvailable",
              ) &&
              !workspace.settingsStatus?.data.includes("GlinerNotSetup") && (
                <>Everything looks fine.</>
              )}

            {workspace.settingsStatus?.type === "Incomplete" && (
              <>
                {workspace.settingsStatus?.data.includes(
                  "PythonNotAvailable",
                ) && (
                  <>
                    We need Python, version 3.9 or above, installed on this
                    computer. Please install Python using your system's package
                    manager and refresh this page.
                  </>
                )}
                {workspace.settingsStatus?.data.includes(
                  "PythonVenvNotAvailable",
                ) && (
                  <>
                    We need Python virtual environment (venv) installed on this
                    computer. Please install Python venv using your system's
                    package manager and refresh this page.
                  </>
                )}
                {workspace.settingsStatus?.data.includes(
                  "PythonPipNotAvailable",
                ) && (
                  <>
                    We need Python pip installed on this computer. Please
                    install Python pip using your system's package manager and
                    refresh this page.
                  </>
                )}
                {workspace.settingsStatus?.data.includes("GlinerNotSetup") && (
                  <>
                    We need GLiNER installed on this computer.{" "}
                    <Button label="Setup Gliner" onClick={handleSetupGliner} />
                  </>
                )}
              </>
            )}
          </>
        )}
      </div>
    </>
  );
};

export default PythonEnv;
