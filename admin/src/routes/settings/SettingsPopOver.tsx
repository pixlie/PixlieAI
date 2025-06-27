import {
  Component,
  createEffect,
  createMemo,
  createSignal,
  Show,
} from "solid-js";
import { useWorkspace } from "../../stores/workspace";
import StorageDir from "../../widgets/settings/StorageDir";
import SettingsIcon from "../../assets/icons/tabler-settings-filled.svg";
import IconButton from "../../widgets/interactable/IconButton";
import GlinerModels from "../../widgets/settings/GlinerModels";
import ApiKeys from "../../widgets/settings/ApiKeys";
import Notifications from "../../widgets/settings/Notifications";

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
      !getAPIKeys()?.BraveSearch ||
      !getAPIKeys()?.SendGrid
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
      <IconButton
        name="Settings"
        icon={SettingsIcon}
        onClick={() => setVisible(true)}
        disabled={isActionRequired()}
      />
      <Show when={visible()}>
        <button
          class="fixed inset-0 bg-slate-500/20 transition-opacity transition duration-500 ease-in-out z-10 cursor-default"
          onClick={() => setVisible(false)}
          disabled={isActionRequired()}
        />
        <div class="absolute right-0 z-20">
          <div class="flex-1 w-96 rounded-lg shadow-md border-slate-200 border bg-white focus:outline-none flex flex-col p-4 pt-3 gap-3 flex-1 overflow-y-auto">
            <StorageDir />
            {!!workspace.settings?.pathToStorageDir && (
              <>
                <hr class="mt-1.5 -mx-4" />
                <ApiKeys />
                <hr class="mt-1.5 -mx-4" />
                <Notifications />
                <hr class="mt-1.5 -mx-4" />
                <GlinerModels />
              </>
            )}
            {/* todo: save all settings in one click? */}
            {/* <hr class="mt-1.5 -mx-4" />
              <button class="bg-blue-500 hover:bg-blue-600 text-white font-semibold p-3 mt-1.5 w-full rounded-full text-center">
                Save
              </button> */}
          </div>
        </div>
      </Show>
    </div>
  );
};

export default SettingsPopOver;
