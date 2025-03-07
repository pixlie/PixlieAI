import { Component, createSignal, onMount } from "solid-js";
import Heading from "../../widgets/typography/Heading";
import Markdown from "../typography/Markdown";
import TextInput from "../interactable/TextInput";
import { createStore } from "solid-js/store";
import { useWorkspace } from "../../stores/workspace";
import { IFormFieldValue } from "../../utils/types";
import Button from "../interactable/Button";
import { useUIClasses } from "../../stores/UIClasses";

const help = `
Right now Pixlie AI only supports using Anthropic Claude for analysis.

Please copy and paste your API key. You can obtain one by signing up
on [https://console.anthropic.com/](https://console.anthropic.com/).
`;

interface IFormData {
  anthropicApiKey: string;
}

const Anthropic: Component = () => {
  const [workspace, { fetchWorkspace, saveWorkspace }] = useWorkspace();
  const [formData, setFormData] = createStore<IFormData>({
    anthropicApiKey: "",
  });
  const [_, { getColors }] = useUIClasses();
  const [errorMessage, setErrorMessage] = createSignal<string>("");

  const handleChange = (name: string, value: IFormFieldValue) => {
    if (!!value && typeof value === "string") {
      setFormData((existing) => ({
        ...existing,
        [name]: value,
      }));
    }
  };

  const handleSubmit = async () => {
    if (!formData.anthropicApiKey) {
      setErrorMessage("Please enter an API key");
      return;
    }
    if (
      formData.anthropicApiKey.length < 64 ||
      !formData.anthropicApiKey.startsWith("sk-ant-")
    ) {
      setErrorMessage("Please enter a valid Anthropic API key");
      return;
    }
    saveWorkspace({
      ...formData,
    });
    fetchWorkspace();
  };

  onMount(() => {
    fetchWorkspace();
  });

  return (
    <>
      <Heading size={3}>Anthropic</Heading>
      <Markdown text={help} />

      <div class="flex flex-col gap-y-2">
        {workspace.isFetching ? (
          <div>Loading...</div>
        ) : (
          <>
            {workspace.workspace?.anthropicApiKey && (
              <small class={getColors()["textInfo"]}>
                You already have an Anthropic API key saved. You can replace it
                by entering a new one.
              </small>
            )}
            <TextInput
              name="anthropicApiKey"
              placeholder={
                workspace.workspace?.anthropicApiKey
                  ? `${workspace.workspace.anthropicApiKey.slice(0, 9)}***${workspace.workspace.anthropicApiKey.slice(-4)}`
                  : "Your Anthropic API Key"
              }
              isEditable
              onChange={handleChange}
              value={formData.anthropicApiKey}
            />
            {!!errorMessage && (
              <small class={getColors()["textDanger"]}>{errorMessage()}</small>
            )}
            <div>
              <Button
                label={`${workspace.workspace?.anthropicApiKey ? "Update" : "Save"} Anthropic API key`}
                onClick={handleSubmit}
              />
            </div>
          </>
        )}
      </div>
    </>
  );
};

export default Anthropic;
