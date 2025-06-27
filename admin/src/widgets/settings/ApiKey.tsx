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
import { APIProvider } from "../../api_types/APIProvider";
import SaveIcon from "../../assets/icons/tabler-device-floppy.svg";
import CheckIcon from "../../assets/icons/tabler-check.svg";
import IconButton from "../interactable/IconButton";

interface ApiKeyProps {
  provider: APIProvider;
  title: string;
  instructions: {
    accountUrl: string;
    accountText: string;
    keyUrl: string;
    keyText: string;
    placeholder: string;
    validate?: (key: string) => { isValid: boolean; errorMessage?: string };
  };
}

interface IFormData {
  apiKey: string;
}

const ApiKey: Component<ApiKeyProps> = (props) => {
  const [workspace, { fetchWorkspace, saveWorkspace }] = useWorkspace();
  const [formData, setFormData] = createStore<IFormData>({
    apiKey: workspace.workspace?.apiKeys?.[props.provider] || "",
  });
  const [saved, setSaved] = createSignal<boolean>(false);
  const [wasCleared, setWasCleared] = createSignal<boolean>(false);

  const [errorMessage, setErrorMessage] = createSignal<string>("");

  const handleChange = (name: string, value: string | number) => {
    setErrorMessage("");
    setWasCleared(false); // Reset the cleared flag when user actually types something
    if (!!value && typeof value === "string") {
      setFormData((existing) => ({
        ...existing,
        [name]: value,
      }));
    }
  };

  const handleSubmit = async () => {
    if (!formData.apiKey) {
      setErrorMessage(`Please enter a ${props.title} API key`);
      return;
    }

    // Run custom validation if provided
    if (props.instructions.validate) {
      const validation = props.instructions.validate(formData.apiKey);
      if (!validation.isValid) {
        setErrorMessage(validation.errorMessage || `Please enter a valid ${props.title} API key`);
        return;
      }
    }

    // Create the update object with the correct field name
    let updateKey: string;
    switch (props.provider) {
      case "BraveSearch":
        updateKey = "brave_search_api_key";
        break;
      case "Anthropic":
        updateKey = "anthropic_api_key";
        break;
      case "SendGrid":
        updateKey = "sendgrid_api_key";
        break;
    }
    
    const updateObject = {
      [updateKey]: formData.apiKey,
    };

    saveWorkspace(updateObject);
    setFormData("apiKey", "");
    fetchWorkspace();
    setSaved(true);
  };

  onMount(() => {
    fetchWorkspace();
  });

  const getApiKey = createMemo<string | undefined>(() => {
    if (workspace.workspace?.apiKeys) {
      return workspace.workspace.apiKeys[props.provider];
    }
    return undefined;
  });

  const isApiKeyMasked = createMemo(() => {
    const key = getApiKey();
    return key && key.includes("******");
  });

  createEffect(() => {
    const apiKey = getApiKey();
    // Show masked API keys in the input field as preview
    if (apiKey && apiKey.includes("******")) {
      setFormData((existing) => ({
        ...existing,
        apiKey: apiKey,
      }));
    } else if (apiKey && !apiKey.includes("******")) {
      setFormData((existing) => ({
        ...existing,
        apiKey: apiKey,
      }));
    } else if (!apiKey) {
      setFormData((existing) => ({
        ...existing,
        apiKey: "",
      }));
    }
  });

  // Update saved state when workspace changes
  createEffect(() => {
    setSaved(!!getApiKey());
  });

  return (
     <div class="bg-white rounded-lg border border-slate-200 p-4 hover:border-slate-300 transition-colors">
      <div class="flex items-center gap-3 mb-3">
        <div>
          <h4 class="font-medium text-slate-800">{`${props.title} API Key`}</h4>
        </div>
      </div>

      {workspace.isFetching ? (
        <div class="flex items-center justify-center py-4">
          <div class="animate-spin rounded-full h-5 w-5 border-b-2 border-blue-600"></div>
          <span class="ml-2 text-sm text-slate-500">Loading...</span>
        </div>
      ) : (
        <>
          <div class="mb-4">
            <div class="text-xs text-slate-600 space-y-1">
              <div class="flex items-center gap-1">
                <span class="text-slate-400">1.</span>
                Create an account{" "}
                <a
                  href={props.instructions.accountUrl}
                  target="_blank"
                  rel="noreferrer"
                  class="inline-flex items-center gap-1 text-blue-600 hover:text-blue-700 font-medium"
                >
                  {props.instructions.accountText}
                  <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                  </svg>
                </a>
              </div>
              <div class="flex items-center gap-1">
                <span class="text-slate-400">2.</span>
                Generate a new API key{" "}
                <a
                  href={props.instructions.keyUrl}
                  target="_blank"
                  rel="noreferrer"
                  class="inline-flex items-center gap-1 text-blue-600 hover:text-blue-700 font-medium"
                >
                  {props.instructions.keyText}
                  <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                  </svg>
                </a>
              </div>
              <div class="flex items-center gap-1">
                <span class="text-slate-400">3.</span>
                Paste your new API key below:
              </div>
            </div>
          </div>

          <div class="space-y-3">
            {/* <label class="block text-sm font-medium text-slate-700">API Key</label> */}
            <div class="flex items-center gap-3">
              <div class="flex-1">
                <TextInput
                  name="apiKey"
                  isEditable
                  onChange={handleChange}
                  onFocus={() => {
                    // Clear the input when user focuses, especially if it contains masked key
                    if (isApiKeyMasked() || formData.apiKey.includes("******")) {
                      setFormData({ apiKey: "" });
                      setWasCleared(true);
                    }
                  }}
                  value={formData.apiKey}
                //   placeholder={isApiKeyMasked() ? "Enter new API key to replace current one" : props.instructions.placeholder}
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
                  <div class="p-2 text-green-600">
                    <CheckIcon />
                  </div>
                )}
              </div>
            </div>
            {!!errorMessage() && (
              <div class="flex items-center gap-2 text-red-600 text-sm bg-red-50 p-2 rounded-md">
                <svg class="w-4 h-4 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                {errorMessage()}
              </div>
            )}
          </div>
        </>
      )}
    </div>
  );
};

export default ApiKey;
