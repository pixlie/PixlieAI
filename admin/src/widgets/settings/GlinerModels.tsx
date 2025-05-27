import { Component, createSignal } from "solid-js";
import { useWorkspace } from "../../stores/workspace";
import CheckIcon from "../../assets/icons/tabler-check.svg";
import Button from "../interactable/Button";
import LoaderIcon from "../../assets/icons/tabler-loader.svg";

const GlinerModels: Component = () => {
  const [workspace, { glinerSettings, fetchSettingsStatus }] = useWorkspace();

  const handleClick = async () => {
    setIsLoading(true);
    try {
      await glinerSettings();
      fetchSettingsStatus();
      setIsDownloaded(true);
    } catch (e) {
      console.error(e);
    }
    setIsLoading(false);
  };
  const [isLoading, setIsLoading] = createSignal<boolean>(false);
  const [isDownloaded, setIsDownloaded] = createSignal<boolean>(
    workspace.settingsStatus?.type === "Complete"
  );

  return (
    <div>
      <p class="font-medium">Gliner Models</p>
      <div class="mt-2">
        {isLoading() ||
        (workspace.settingsStatus?.type === "Incomplete" &&
          workspace.settingsStatus?.data.includes("GlinerFileNotFound")) ? (
          <div class="flex items-center justify-between gap-2">
            <p>Download In Progress</p>
            <LoaderIcon />
          </div>
        ) : (
          <>
            {isDownloaded() ? (
              <div class="flex items-center justify-between gap-2">
                <p>Download Complete</p>
                <div style={{ color: "#00C853" }}>
                  <CheckIcon />
                </div>
              </div>
            ) : (
              <Button
                label="Download"
                colorTheme="secondary"
                onClick={handleClick}
              />
            )}
          </>
        )}
      </div>
    </div>
  );
};

export default GlinerModels;
