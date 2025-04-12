import { batch, Component, createContext, useContext } from "solid-js";
import { createStore } from "solid-js/store";
import { IProviderPropTypes, IWorkspace } from "../utils/types";
import { getPixlieAIAPIRoot } from "../utils/api";
import { SettingsStatus } from "../api_types/SettingsStatus";
import { Settings } from "../api_types/Settings";
import { Project } from "../api_types/Project";
import { Workspace } from "../api_types/Workspace";
import { WorkspaceUpdate } from "../api_types/WorkspaceUpdate.ts";
import {
  camelCasedToSnakeCasedKeys,
  snakeCasedToCamelCasedKeys,
} from "../utils/utils.ts";
import { useEngine } from "./engine.tsx";

const makeStore = () => {
  const [store, setStore] = createStore<IWorkspace>({
    isReady: false,
    isFetching: false,
    fetchingFlags: {
      projects: false,
    },
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
          settings: snakeCasedToCamelCasedKeys(settings),
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

  const saveSettings = (workspace: Partial<IWorkspace["settings"]>) => {
    let pixlieAIAPIRoot = getPixlieAIAPIRoot();
    fetch(`${pixlieAIAPIRoot}/api/settings`, {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(camelCasedToSnakeCasedKeys(workspace)),
    }).then((response) => {
      if (!response.ok) {
        throw new Error("Failed to save settings");
      }
      setStore("settings", workspace);
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
          workspace: snakeCasedToCamelCasedKeys(workspace),
        }));
      });
    });
  };

  const saveWorkspace = (update: Partial<WorkspaceUpdate>) => {
    if (!store.workspace || !store.workspace.uuid) {
      return;
    }
    let pixlieAIAPIRoot = getPixlieAIAPIRoot();
    fetch(`${pixlieAIAPIRoot}/api/workspace/${store.workspace.uuid}`, {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(camelCasedToSnakeCasedKeys(update)),
    }).then((response) => {
      if (!response.ok) {
        throw new Error("Failed to save workspace");
      }
      fetchWorkspace();
    });
  };

  const fetchProjects = () => {
    if (store.fetchingFlags.projects) {
      return;
    }
    setStore("fetchingFlags", "projects", true);
    let pixlieAIAPIRoot = getPixlieAIAPIRoot();
    fetch(`${pixlieAIAPIRoot}/api/projects`).then((response) => {
      if (!response.ok) {
        throw new Error("Failed to fetch projects");
      }
      response.json().then((projects: Array<Project>) => {
        const [engine, { removeProject }] = useEngine();
        // Remove projects that are not in the response
        Object.keys(engine.projects).forEach((projectId) => {
          if (!projects.some((project) => project.uuid === projectId)) {
            removeProject(projectId);
          }
        });
        batch(() => {
          setStore("projects", projects);
          setStore("fetchingFlags", "projects", false);
        });
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
