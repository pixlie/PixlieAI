import { Component } from "solid-js";
import Heading from "../../widgets/typography/Heading";
import Markdown from "../typography/Markdown";
import TextInput from "../interactable/TextInput";
import { createStore } from "solid-js/store";
import { useWorkspace } from "../../stores/Workspace";
import { IFormFieldValue } from "../../utils/types";
import Button from "../interactable/Button";

const help = `Right now Pixlie AI only supports using Anthropic Claude. Please copy and paste your API key.
We are working on supporting Ollama so that you can use open source AI models.`;

interface IFormData {
  anthropicApiKey: string;
}

const Anthropic: Component = () => {
  const [workspace, { fetchSettings, saveSettings }] = useWorkspace();
  const [formData, setFormData] = createStore<IFormData>({
    anthropicApiKey: workspace.settings?.anthropicApiKey || "",
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

  return (
    <>
      <Heading size={3}>Anthropic</Heading>
      <Markdown text={help} />

      <div class="flex flex-col gap-y-2">
        <TextInput
          name="anthropic_api_key"
          isEditable
          onChange={handleChange}
          value={formData.anthropicApiKey}
        />
        <div>
          <Button label="Set Anthropic API key" onClick={handleSubmit} />
        </div>
      </div>
    </>
  );
};

export default Anthropic;
