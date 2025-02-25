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
import { Project } from "../api_types/Project";
import { Workspace } from "../api_types/Workspace";

const makeStore = () => {
  const [store, setStore] = createStore<IWorkspace>({
    isReady: false,
    isFetching: false,
  });

  const fetchSettings = () => {
    setStore((data) => ({ ...data, isFetching: true, isReady: false }));
    let pixlieAIAPIRoot = getPixlieAIAPIRoot();
    fetch(`${pixlieAIAPIRoot}/api/settings`).then((response) => {
      if (!response.ok) {
        console.error("Failed to fetch settings");
      }
      response.json().then((settings: Settings) => {
        setStore((data) => ({
          ...data,
          isFetching: false,
          isReady: true,
          settings: camelCasedKeys(settings),
        }));
      });
    });
  };

  const fetchSettingsStatus = () => {
    let pixlieAIAPIRoot = getPixlieAIAPIRoot();
    fetch(`${pixlieAIAPIRoot}/api/settings/status`).then((response) => {
      if (!response.ok) {
        throw new Error("Failed to fetch settings status");
      }
      response.json().then((settingsStatus: SettingsStatus) => {
        setStore("settingsStatus", settingsStatus as SettingsStatus);
      });
    });
  };

  const saveSettings = (settings: Partial<IWorkspace["settings"]>) => {
    let pixlieAIAPIRoot = getPixlieAIAPIRoot();
    fetch(`${pixlieAIAPIRoot}/api/settings`, {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(snakeCasedKeys(settings)),
    }).then((response) => {
      if (!response.ok) {
        throw new Error("Failed to save settings");
      }
      setStore("settings", settings);
    });
  };

  const fetchWorkspace = () => {
    let pixlieAIAPIRoot = getPixlieAIAPIRoot();
    fetch(`${pixlieAIAPIRoot}/api/workspace`).then((response) => {
      if (!response.ok) {
        throw new Error("Failed to fetch workspace");
      }
      response.json().then((workspace: Workspace) => {
        setStore((data) => ({
          ...data,
          isFetching: false,
          isReady: true,
          workspace: camelCasedKeys(workspace),
        }));
      });
    });
  };

  const saveWorkspace = (workspace: Partial<Workspace>) => {
    if (!store.workspace || !store.workspace.uuid) {
      return;
    }
    let pixlieAIAPIRoot = getPixlieAIAPIRoot();
    fetch(`${pixlieAIAPIRoot}/api/workspace/${store.workspace.uuid}`, {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(snakeCasedKeys(workspace)),
    }).then((response) => {
      if (!response.ok) {
        throw new Error("Failed to save workspace");
      }
      setStore("workspace", workspace);
    });
  };

  const fetchProjects = () => {
    let pixlieAIAPIRoot = getPixlieAIAPIRoot();
    fetch(`${pixlieAIAPIRoot}/api/projects`).then((response) => {
      if (!response.ok) {
        throw new Error("Failed to fetch projects");
      }
      response.json().then((projects: Array<Project>) => {
        setStore("projects", projects);
      });
    });
  };

  return [
    store,
    {
      fetchSettings,
      fetchSettingsStatus,
      saveSettings,
      fetchWorkspace,
      saveWorkspace,
      fetchProjects,
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
