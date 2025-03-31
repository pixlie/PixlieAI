import { Component, createSignal } from "solid-js";
import TextInput from "../interactable/TextInput";
import { useWorkspace } from "../../stores/workspace";
import { createStore } from "solid-js/store";
import { IFormFieldValue } from "../../utils/types";
import Icon from "../interactable/Icon";
import ToolTip from "../navigation/ToolTip";
import { useUIClasses } from "../../stores/UIClasses";

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

  const handleChange = (name: string, value: IFormFieldValue) => {
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
        <li>- Enter the path to your new directory below</li>
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
        {!saved() ? (
          <button onClick={handleSubmit} class=" -mr-2">
            <ToolTip text="Save">
              <div class="p-2 text-gray-800 hover:text-gray-950 hover:bg-slate-200 rounded-full">
                <Icon name="save" />
              </div>
            </ToolTip>
          </button>
        ) : (
          <div class="p-2 -mr-2">
            <Icon name="check" color="#00C853" />
          </div>
        )}
      </div>
      {!!errorMessage && (
        <small class={getColors()["textDanger"]}>{errorMessage()}</small>
      )}
    </div>
  );
};

export default StorageDir;
