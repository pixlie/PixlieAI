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
import { Project } from "../api_types/Project.ts";
import { ProjectCreate } from "../api_types/ProjectCreate.ts";

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
        let pixlieAIAPIRoot = getPixlieAIAPIRoot();
        let response = await fetch(`${pixlieAIAPIRoot}/api/settings`);
        if (!response.ok) {
          console.error("Failed to fetch settings");
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
        let pixlieAIAPIRoot = getPixlieAIAPIRoot();
        let response = await fetch(`${pixlieAIAPIRoot}/api/settings/status`);
        if (!response.ok) {
          throw new Error("Failed to fetch settings status");
        }
        let settingsStatus = await response.json();
        setStore("settingsStatus", settingsStatus as SettingsStatus);
      },

      saveSettings: async (settings: Partial<IWorkspace["settings"]>) => {
        let pixlieAIAPIRoot = getPixlieAIAPIRoot();
        let response = await fetch(`${pixlieAIAPIRoot}/api/settings`, {
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

      fetchProjects: async () => {
        let pixlieAIAPIRoot = getPixlieAIAPIRoot();
        let response = await fetch(`${pixlieAIAPIRoot}/api/projects`);
        if (!response.ok) {
          throw new Error("Failed to fetch projects");
        }
        let projects: Array<Project> = await response.json();
        setStore("projects", projects);
      },

      createProject: async (project: ProjectCreate) => {
        let pixlieAIAPIRoot = getPixlieAIAPIRoot();
        let response = await fetch(`${pixlieAIAPIRoot}/api/projects`, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify(project),
        });
        if (!response.ok) {
          throw new Error("Failed to save settings");
        }
        return (await response.json()) as Project;
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
