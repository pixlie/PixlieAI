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
import SaveIcon from "../../assets/icons/tabler-device-floppy.svg";
import CheckIcon from "../../assets/icons/tabler-check.svg";
import IconButton from "../interactable/IconButton";

interface IFormData {
  braveSearchApiKey: string;
}

const BraveSearch: Component = () => {
  const [workspace, { fetchWorkspace, saveWorkspace }] = useWorkspace();
  const [formData, setFormData] = createStore<IFormData>({
    braveSearchApiKey:
      workspace.workspace?.apiKeys["BraveSearch" as APIProvider] || "",
  });
  const [_, { getColors }] = useUIClasses();
  const [errorMessage, setErrorMessage] = createSignal<string>("");
  const [saved, setSaved] = createSignal<boolean>(
    !!workspace.workspace?.apiKeys["BraveSearch" as APIProvider]
  );

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
    setSaved(true);
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

  createEffect(() => {
    setFormData((existing) => ({
      ...existing,
      braveSearchApiKey: getBraveSearchApiKey() || "",
    }));
  });

  return (
    <div>
      <p class="font-medium">Brave Search API Key</p>

      {workspace.isFetching ? (
        <div>Loading...</div>
      ) : (
        <>
          <ol class="text-gray-500 py-1 flex flex-col">
            <li>
              - Create an account{" "}
              <a
                href="https://brave.com/search/api/"
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
                href="https://api-dashboard.search.brave.com/app/keys"
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
              name="braveSearchApiKey"
              isEditable
              onChange={handleChange}
              onFocus={() => {
                setFormData({ braveSearchApiKey: "" });
                setSaved(false);
              }}
              value={formData.braveSearchApiKey}
            />
            <div class="-mr-2">
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
          {!!errorMessage && (
            <small class={getColors()["textDanger"]}>{errorMessage()}</small>
          )}
        </>
      )}
    </div>
  );
};

export default BraveSearch;
