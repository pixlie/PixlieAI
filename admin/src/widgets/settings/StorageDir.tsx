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
    !!workspace.settings?.pathToStorageDir,
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
    <div>
      <p class="font-medium">Storage Directory</p>
      <ol class="text-gray-500 py-1 flex flex-col">
        <li>- Create a new directory on your computer</li>
        <li>- Enter the path to your new directory below:</li>
      </ol>
      <div class="flex items-center gap-2 pt-2">
        <TextInput
          name="pathToStorageDir"
          isEditable
          onChange={handleChange}
          onFocus={() => {
            setFormData({ pathToStorageDir: "" });
            setSaved(false);
          }}
          value={formData.pathToStorageDir}
        />
        <div class="-mr-2">
          {!saved() ? (
            <IconButton name="Save" icon={SaveIcon} onClick={handleSubmit} />
          ) : (
            <div class="p-2" style={{ color: "#00C853" }}>
              <CheckIcon />
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
