import {
  Component,
  createEffect,
  createMemo,
  createSignal,
  Show,
} from "solid-js";
import { useWorkspace } from "../../stores/workspace";
import ToolTip from "../../widgets/navigation/ToolTip";
import Icon from "../../widgets/interactable/Icon";
import StorageDir from "../../widgets/settings/StorageDir";
import Anthropic from "../../widgets/settings/Anthropic";
import BraveSearch from "../../widgets/settings/BraveSearch";

const SettingsPopOver: Component = () => {
  const [visible, setVisible] = createSignal<boolean>(false);
  const [workspace] = useWorkspace();

  const getSettingsStatus = createMemo(() => {
    if (workspace.isReady) {
      return workspace.settingsStatus?.type;
    }
    return undefined;
  });

  const getAPIKeys = createMemo(() => {
    if (workspace.isReady) {
      return workspace.workspace?.apiKeys;
    }
    return undefined;
  });

  const isActionRequired = createMemo(() => {
    if (!getSettingsStatus() || !getAPIKeys()) {
      return false;
    }
    return (
      getSettingsStatus() === "Incomplete" ||
      !getAPIKeys()?.Anthropic ||
      !getAPIKeys()?.BraveSearch
    );
  });

  createEffect(() => {
    if (isActionRequired()) {
      setVisible(true);
    }
  });

  return (
    <div class="relative w-10">
      <Show when={isActionRequired()}>
        <div
          class="absolute top-1.5 right-1.5 w-2.5 h-2.5 bg-red-500 z-10 rounded-full"
          style={{ "background-color": "#D50000" }}
        />
      </Show>
      <ToolTip text="Settings">
        <button
          onClick={() => setVisible(true)}
          aria-label="Settings"
          class="flex items-center p-2 text-gray-800 hover:text-gray-950 hover:bg-slate-200 rounded-full"
          disabled={isActionRequired()}
        >
          <Icon name="settings" />
        </button>
      </ToolTip>
      <Show when={visible()}>
        <button
          class="fixed inset-0 bg-slate-500/20 transition-opacity transition duration-500 ease-in-out z-10"
          onClick={() => setVisible(false)}
          disabled={isActionRequired()}
        />
        <div class="absolute right-0 mt-1.5 z-20 w-96 rounded-md shadow-md border-slate-200 border bg-white focus:outline-none flex flex-col p-4 pt-3 gap-3">
          <StorageDir />
          {!!workspace.settings?.pathToStorageDir && (
            <>
              <hr class="mt-1.5 -mx-4" />
              <Anthropic />
              <hr class="mt-1.5 -mx-4" />
              <BraveSearch />
            </>
          )}
          {/* todo: save all settings in one click? */}
          {/* <hr class="mt-1.5 -mx-4" />
              <button class="bg-blue-500 hover:bg-blue-600 text-white font-semibold p-3 mt-1.5 w-full rounded-full text-center">
                Save
              </button> */}
        </div>
      </Show>
    </div>
  );
};

export default SettingsPopOver;
