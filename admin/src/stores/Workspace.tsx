import { Component, createContext, useContext } from "solid-js";
import { createStore } from "solid-js/store";
import { IProviderPropTypes, IWorkspace } from "../utils/types";
import { getPixlieAIAPIRoot } from "../utils/api";
import { SettingsStatus } from "../api_types/SettingsStatus";

const makeStore = () => {
  const [store, setStore] = createStore<IWorkspace>({
    isReady: false,
    isFetching: false,
  });

  return [
    store,
    {
      fetchSettings: async () => {
        let pixieAIAPIRoot = getPixlieAIAPIRoot();
        let response = await fetch(`${pixieAIAPIRoot}/api/settings`);
        if (!response.ok) {
          throw new Error("Failed to fetch settings");
        }
        let settings = await response.json();
        setStore("settings", settings);
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

      saveSettings: async (settings: IWorkspace["settings"]) => {
        let pixieAIAPIRoot = getPixlieAIAPIRoot();
        let response = await fetch(`${pixieAIAPIRoot}/api/settings`, {
          method: "POST",
          body: JSON.stringify(settings),
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
