import { Component, createSignal } from "solid-js";
import TextInput from "../interactable/TextInput";
import { useWorkspace } from "../../stores/workspace";
import { createStore } from "solid-js/store";
import { useUIClasses } from "../../stores/UIClasses";
import SaveIcon from "../../assets/icons/tabler-device-floppy.svg";
import CheckIcon from "../../assets/icons/tabler-check.svg";
import IconButton from "../interactable/IconButton";

interface IFormData {
  pathToStorageDir: string;
}

const StorageDir: Component = () => {
  const [workspace, { fetchSettings, saveSettings }] = useWorkspace();
  const [formData, setFormData] = createStore<IFormData>({
    pathToStorageDir: workspace.settings?.pathToStorageDir || "",
  });
  const [_, { getColors }] = useUIClasses();
  const [errorMessage, setErrorMessage] = createSignal<string>("");
  const [saved, setSaved] = createSignal<boolean>(
    !!workspace.settings?.pathToStorageDir
  );

  const handleChange = (name: string, value: string | number) => {
    if (!!value && typeof value === "string") {
      setFormData((existing) => ({
        ...existing,
        [name]: value,
      }));
    }
  };

  const handleSubmit = async () => {
    if (!formData.pathToStorageDir) {
      setErrorMessage("Please enter a path");
      return;
    }
    saveSettings({
      ...formData,
    });
    fetchSettings();
    setSaved(true);
  };

  // The user has to set the storage directory
  return (
    <div class="space-y-4">
      <div class="">
        <h3 class="text-lg font-medium text-slate-800 flex items-center gap-2">
          {/* <svg class="w-5 h-5 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2H5a2 2 0 00-2-2v0a2 2 0 012-2h6l2 2h6a2 2 0 012 2v1M3 7l3 3 3-3" />
          </svg> */}
          Storage
        </h3>
        {/* <p class="text-slate-600 text-sm">Configure where Pixlie stores your project data and AI models.</p> */}
      </div>

      <div class="bg-white rounded-lg border border-slate-200 p-4 hover:border-slate-300 transition-colors">
        <div class="flex items-center gap-3 mb-3">
            <h4 class="font-medium text-slate-800">Directory Path</h4>
        </div>

        <div class="mb-4">
          <div class="text-xs text-slate-600 space-y-1">
            <div class="flex items-center gap-1">
              <span class="text-slate-400">1.</span>
              Create a new directory on your computer
            </div>
            <div class="flex items-center gap-1">
              <span class="text-slate-400">2.</span>
              Enter the path to your new directory below:
            </div>
          </div>
        </div>

        <div class="space-y-3">
          <div class="flex items-center gap-3">
            <div class="flex-1">
              <TextInput
                name="pathToStorageDir"
                isEditable
                onChange={handleChange}
                onFocus={() => {
                  setFormData({ pathToStorageDir: "" });
                  setSaved(false);
                }}
                value={formData.pathToStorageDir}
                placeholder="/Users/yourusername/pixlie-data"
              />
            </div>
            <div class="flex-shrink-0">
              {!saved() ? (
                <IconButton
                  name="Save"
                  icon={SaveIcon}
                  onClick={handleSubmit}
                />
              ) : (
  <div class="p-2" style={{ color: "#00C853" }}>
                  <CheckIcon />
                </div>
              )}
            </div>
          </div>
          {!!errorMessage() && (
            <div class="flex items-center gap-2 text-red-600 text-sm bg-red-50 p-2 rounded-md">
              <svg
                class="w-4 h-4 flex-shrink-0"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
              {errorMessage()}
            </div>
          )}
        </div>
      </div>
      {!!errorMessage && (
        <small class={getColors()["textDanger"]}>{errorMessage()}</small>
      )}
    </div>
  );
};

export default StorageDir;
