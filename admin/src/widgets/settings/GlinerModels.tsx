import { Component, createSignal } from "solid-js";
import { useWorkspace } from "../../stores/workspace";
import CheckIcon from "../../assets/icons/tabler-check.svg";
import Button from "../interactable/Button";

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
    <div class="space-y-4">
      <div class="">
        <h3 class="text-lg font-medium text-slate-800 mb-2 flex items-center gap-2">
          {/* <svg class="w-5 h-5 text-orange-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
          </svg> */}
          GLiNER Models
        </h3>
        {/* <p class="text-slate-600 text-sm">Download AI models required for named entity recognition and data extraction.</p> */}
      </div>
      
 <div class="bg-white rounded-lg border border-slate-200 p-4 hover:border-slate-300 transition-colors">
        {isLoading() ||
        (workspace.settingsStatus?.type === "Incomplete" &&
          workspace.settingsStatus?.data.includes("GlinerFileNotFound")) ? (
          <div class="flex items-center gap-3">
            <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-600"></div>
            <span class="text-sm text-slate-600">Downloading</span>
          </div>
        ) : (
          <>
            {isDownloaded() ? (
              <div class="flex items-center justify-between gap-2">
                <p class="text-sm text-slate-600">Downloaded</p>
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