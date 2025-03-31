import {
  Component,
  createEffect,
  createMemo,
  createSignal,
  onMount,
} from "solid-js";
import TextInput from "../interactable/TextInput";
import { createStore } from "solid-js/store";
import { useWorkspace } from "../../stores/workspace";
import { IFormFieldValue } from "../../utils/types";
import { useUIClasses } from "../../stores/UIClasses";
import { APIProvider } from "../../api_types/APIProvider";
import Icon from "../interactable/Icon";
import ToolTip from "../navigation/ToolTip";

interface IFormData {
  anthropicApiKey: string;
}

const Anthropic: Component = () => {
  const [workspace, { fetchWorkspace, saveWorkspace }] = useWorkspace();
  const [formData, setFormData] = createStore<IFormData>({
    anthropicApiKey:
      workspace.workspace?.apiKeys["Anthropic" as APIProvider] || "",
  });
  const [saved, setSaved] = createSignal<boolean>(
    !!workspace.workspace?.apiKeys["Anthropic" as APIProvider]
  );

  const [_, { getColors }] = useUIClasses();
  const [errorMessage, setErrorMessage] = createSignal<string>("");

  const handleChange = (name: string, value: IFormFieldValue) => {
    setErrorMessage("");
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
    setSaved(true);
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

  createEffect(() => {
    setFormData((existing) => ({
      ...existing,
      anthropicApiKey: getAnthropicApiKey() || "",
    }));
  });

  return (
    <div>
      <p class="font-medium">Anthropic API Key</p>
      {workspace.isFetching ? (
        <div>Loading...</div>
      ) : (
        <>
          <ol class="text-gray-500 py-1 flex flex-col">
            <li>
              - Create an account{" "}
              <a
                href="https://console.anthropic.com/login"
                target="_blank"
                rel="noreferrer"
                class="underline text-blue-500 hover:text-blue-600 font-semibold"
              >
                here
              </a>
            </li>
            <li>
              - Create a new key{" "}
              <a
                href="https://console.anthropic.com/settings/keys"
                target="_blank"
                rel="noreferrer"
                class="underline text-blue-500 hover:text-blue-600 font-semibold"
              >
                here
              </a>
            </li>
            <li>- Enter your new key below:</li>
          </ol>
          <div class="flex items-center gap-2 pt-2">
            <TextInput
              name="anthropicApiKey"
              isEditable
              onChange={handleChange}
              onFocus={() => {
                setFormData({ anthropicApiKey: "" });
                setSaved(false);
              }}
              value={formData.anthropicApiKey}
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
        </>
      )}
    </div>
  );
};

export default Anthropic;
