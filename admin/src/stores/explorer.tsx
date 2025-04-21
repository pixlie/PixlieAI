import { batch, Component, createContext, useContext } from "solid-js";
import { createStore } from "solid-js/store";
import { EngineResponsePayload } from "../api_types/EngineResponsePayload.ts";
import { getPixlieAIAPIRoot } from "../utils/api";
import { IExplorerStore, IProviderPropTypes } from "../utils/types";

const makeStore = () => {
  const [store, setStore] = createStore<IExplorerStore>({
    projects: {},
  });

  const setProjectId = (projectId: string) => {
    if (!Object.keys(store.projects).includes(projectId)) {
      setStore("projects", projectId, {
        nodes: [],
        edges: {},
      });
    }
  };

  const explore = (projectId: string) => {
    let pixlieAIAPIRoot = getPixlieAIAPIRoot();
    fetch(`${pixlieAIAPIRoot}/api/engine/${projectId}/explore`, {
      headers: {
        "Content-Type": "application/json",
      },
    })
      .then((response) => {
        if (!response.ok) {
          throw new Error("Failed to fetch nodes");
        }
        return response.json();
      })
      .then((response: EngineResponsePayload) => {
        if (response.type === "Explore") {
          batch(() => {
            for (const node of response.data.nodes) {
              setStore(
                "projects",
                projectId,
                "nodes",
                store.projects[projectId].nodes.length,
                node,
              );
            }
            for (const [nodeId, edges] of Object.entries(response.data.edges)) {
              if (!!edges) {
                setStore(
                  "projects",
                  projectId,
                  "edges",
                  parseInt(nodeId),
                  edges,
                );
              }
            }
          });
        }
      });
  };

  return [
    store,
    {
      setProjectId,
      explore,
    },
  ] as const; // `as const` forces tuple type inference
};

type TStoreAndFunctions = ReturnType<typeof makeStore>;
export const explorerStore = makeStore();

const ExplorerContext = createContext<TStoreAndFunctions>(explorerStore);

export const ExplorerProvider: Component<IProviderPropTypes> = (props) => {
  return (
    <ExplorerContext.Provider value={explorerStore}>
      {props.children}
    </ExplorerContext.Provider>
  );
};

export const useExplorer = () => useContext(ExplorerContext);
