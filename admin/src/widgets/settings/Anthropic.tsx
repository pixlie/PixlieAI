import { Component, createMemo, createSignal, onMount } from "solid-js";
import Heading from "../../widgets/typography/Heading";
import Markdown from "../typography/Markdown";
import TextInput from "../interactable/TextInput";
import { createStore } from "solid-js/store";
import { useWorkspace } from "../../stores/workspace";
import { IFormFieldValue } from "../../utils/types";
import Button from "../interactable/Button";
import { useUIClasses } from "../../stores/UIClasses";
import { APIProvider } from "../../api_types/APIProvider";

const help = `
Pixlie only supports using Anthropic Claude as an AI model at this moment.

Please copy and paste your API key. You can obtain one by signing up
on [Anthropic Console](https://www.anthropic.com/api).
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
      anthropic_api_key: formData.anthropicApiKey,
    });
    setFormData("anthropicApiKey", "");
    fetchWorkspace();
  };

  onMount(() => {
    fetchWorkspace();
  });

  const getAnthropicApiKey = createMemo<string | undefined>(() => {
    if (workspace.workspace?.apiKeys) {
      return workspace.workspace.apiKeys["Anthropic" as APIProvider];
    }
    return undefined;
  });

  return (
    <div class="flex flex-col gap-y-2">
      <Heading size={3}>Anthropic API Key</Heading>
      <Markdown text={help} />

      {workspace.isFetching ? (
        <div>Loading...</div>
      ) : (
        <>
          {getAnthropicApiKey() && (
            <small class={getColors()["textInfo"]}>
              You have already saved an Anthropic API key. You can replace it by
              entering entering a new one.
            </small>
          )}
          <TextInput
            name="anthropicApiKey"
            placeholder={getAnthropicApiKey() || "Your Anthropic API Key"}
            isEditable
            onChange={handleChange}
            value={formData.anthropicApiKey}
          />
          {!!errorMessage && (
            <small class={getColors()["textDanger"]}>{errorMessage()}</small>
          )}
          <div>
            <Button
              label={`${getAnthropicApiKey() ? "Update" : "Save"} Anthropic API key`}
              onClick={handleSubmit}
            />
          </div>
        </>
      )}
    </div>
  );
};

export default Anthropic;
