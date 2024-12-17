import { Component, createContext, useContext } from "solid-js";
import { createStore } from "solid-js/store";
import { IProviderPropTypes, IWorkspace } from "../utils/types";
import {
  camelCasedKeys,
  getPixlieAIAPIRoot,
  snakeCasedKeys,
} from "../utils/api";
import { SettingsStatus } from "../api_types/SettingsStatus";
import { Settings } from "../api_types/Settings";

const makeStore = () => {
  const [store, setStore] = createStore<IWorkspace>({
    isReady: false,
    isFetching: false,
  });

  return [
    store,
    {
      fetchSettings: async () => {
        setStore((data) => ({ ...data, isFetching: true, isReady: false }));
        let pixieAIAPIRoot = getPixlieAIAPIRoot();
        let response = await fetch(`${pixieAIAPIRoot}/api/settings`);
        if (!response.ok) {
          throw new Error("Failed to fetch settings");
        }
        let settings: Settings = await response.json();
        setStore((data) => ({
          ...data,
          isFetching: false,
          isReady: true,
          settings: camelCasedKeys(settings),
        }));
      },

      fetchSettingsStatus: async () => {
        let pixieAIAPIRoot = getPixlieAIAPIRoot();
        let response = await fetch(`${pixieAIAPIRoot}/api/settings/status`);
        if (!response.ok) {
          throw new Error("Failed to fetch settings status");
        }
        let settingsStatus = await response.json();
        setStore("settingsStatus", settingsStatus as SettingsStatus);
      },

      saveSettings: async (settings: Partial<IWorkspace["settings"]>) => {
        let pixieAIAPIRoot = getPixlieAIAPIRoot();
        let response = await fetch(`${pixieAIAPIRoot}/api/settings`, {
          method: "PUT",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify(snakeCasedKeys(settings)),
        });
        if (!response.ok) {
          throw new Error("Failed to save settings");
        }
        setStore("settings", settings);
      },
    },
  ] as const; // `as const` forces tuple type inference
};

type TStoreAndFunctions = ReturnType<typeof makeStore>;
export const workspaceStore = makeStore();

const WorkspaceContext = createContext<TStoreAndFunctions>(workspaceStore);

export const WorkspaceProvider: Component<IProviderPropTypes> = (props) => {
  return (
    <WorkspaceContext.Provider value={workspaceStore}>
      {props.children}
    </WorkspaceContext.Provider>
  );
};

export const useWorkspace = () => useContext(WorkspaceContext);
