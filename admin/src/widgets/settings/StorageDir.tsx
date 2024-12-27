import { Component } from "solid-js";
import Heading from "../../widgets/typography/Heading";
import Markdown from "../typography/Markdown";
import TextInput from "../interactable/TextInput";
import Button from "../interactable/Button";
import { useWorkspace } from "../../stores/Workspace";
import { createStore } from "solid-js/store";
import { IFormFieldValue } from "../../utils/types";

const help = `
The storage directory is where we store the graph data, some AI/ML model data, etc.
[Gliner](https://github.com/gliner/gliner), which is one of the AI/ML tools, needs about 6 GB of space.

Please copy and paste the path to the directory where you want to store the data.
`;

interface IStorageDirFormData {
  pathToStorageDir: string;
}

const StorageDir: Component = () => {
  const [workspace, { fetchSettings, saveSettings }] = useWorkspace();
  const [formData, setFormData] = createStore<IStorageDirFormData>({
    pathToStorageDir: workspace.settings?.pathToStorageDir || "",
  });

  const handleChange = (name: string, value: IFormFieldValue) => {
    if (!!value && typeof value === "string") {
      setFormData((existing) => ({
        ...existing,
        [name]: value,
      }));
    }
  };

  const handleSubmit = async () => {
    await saveSettings({
      ...formData,
    });
    await fetchSettings();
  };

  // The user has to set the storage directory
  return (
    <>
      <Heading size={3}>Storage Directory</Heading>
      <Markdown text={help} />

      <div class="flex flex-col gap-y-2">
        <TextInput
          name="path_to_storage_dir"
          isEditable
          onChange={handleChange}
          value={formData.pathToStorageDir}
        />
        <div>
          <Button label="Set storage directory" onClick={handleSubmit} />
        </div>
      </div>
    </>
  );
};

export default StorageDir;
