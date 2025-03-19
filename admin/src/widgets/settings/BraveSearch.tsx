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
Pixlie uses Brave Search API to search the web.

Please copy and paste your API key. You can obtain one by signing up
on [Brave Search API](https://brave.com/search/api/).
`;

interface IFormData {
  braveSearchApiKey: string;
}

const BraveSearch: Component = () => {
  const [workspace, { fetchWorkspace, saveWorkspace }] = useWorkspace();
  const [formData, setFormData] = createStore<IFormData>({
    braveSearchApiKey: "",
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
    if (!formData.braveSearchApiKey) {
      setErrorMessage("Please enter an API key");
      return;
    }
    if (
      formData.braveSearchApiKey.length < 30 ||
      !formData.braveSearchApiKey.startsWith("BSA")
    ) {
      setErrorMessage("Please enter a valid Brave Search API key");
      return;
    }
    saveWorkspace({
      brave_search_api_key: formData.braveSearchApiKey,
    });
    setFormData("braveSearchApiKey", "");
    fetchWorkspace();
  };

  onMount(() => {
    fetchWorkspace();
  });

  const getBraveSearchApiKey = createMemo<string | undefined>(() => {
    if (workspace.workspace?.apiKeys) {
      return workspace.workspace.apiKeys["BraveSearch" as APIProvider];
    }
    return undefined;
  });

  return (
    <div class="flex flex-col gap-y-2">
      <Heading size={3}>Brave Search API Key</Heading>
      <Markdown text={help} />

      {workspace.isFetching ? (
        <div>Loading...</div>
      ) : (
        <>
          {getBraveSearchApiKey() && (
            <small class={getColors()["textInfo"]}>
              You have already saved a Brave Search API key. You can replace it
              by entering entering a new one.
            </small>
          )}
          <TextInput
            name="braveSearchApiKey"
            placeholder={getBraveSearchApiKey() || "Your Brave Search API Key"}
            isEditable
            onChange={handleChange}
            value={formData.braveSearchApiKey}
          />
          {!!errorMessage && (
            <small class={getColors()["textDanger"]}>{errorMessage()}</small>
          )}
          <div>
            <Button
              label={`${getBraveSearchApiKey() ? "Update" : "Save"} Brave Search API key`}
              onClick={handleSubmit}
            />
          </div>
        </>
      )}
    </div>
  );
};

export default BraveSearch;
