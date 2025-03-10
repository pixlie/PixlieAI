import { Component, createContext, useContext } from "solid-js";
import { createStore } from "solid-js/store";
import { IEngineStore, INodeItem, IProviderPropTypes } from "../utils/types";
import { getPixlieAIAPIRoot } from "../utils/api";
import { EngineResponsePayload } from "../api_types/EngineResponsePayload.ts";
import { APINodeItem } from "../api_types/APINodeItem.ts";

const makeStore = () => {
  const [store, setStore] = createStore<IEngineStore>({
    projects: {},
  });

  const setProjectId = (projectId: string) => {
    setStore((existing: IEngineStore) => ({
      ...existing,
      projects: {
        ...existing.projects,
        [projectId]: {
          nodes: {},
          nodeIdsByLabel: {},
          edges: {},
        },
      },
    }));
  };

  const fetchNodes = (projectId: string) => {
    let pixlieAIAPIRoot = getPixlieAIAPIRoot();
    fetch(`${pixlieAIAPIRoot}/api/engine/${projectId}/nodes`, {
      headers: {
        "Content-Type": "application/json",
      },
    }).then((response) => {
      if (!response.ok) {
        throw new Error("Failed to fetch nodes");
      }
      response.json().then((responsePayload: EngineResponsePayload) => {
        if (responsePayload.type === "Results") {
          setStore((existing: IEngineStore) => ({
            ...existing,
            projects: {
              ...existing.projects,
              [projectId]: {
                ...existing.projects[projectId],
                nodes: {
                  ...existing.projects[projectId].nodes,
                  ...responsePayload.data.nodes.reduce(
                    (map: { [k: number]: INodeItem }, item) => ({
                      ...map,
                      [item.id]: {
                        ...item,
                        isFetching: false,
                      },
                    }),
                    {},
                  ),
                },
              },
            },
          }));
        }
      });
    });
  };

  const fetchEdges = (projectId: string) => {
    let pixlieAIAPIRoot = getPixlieAIAPIRoot();
    fetch(`${pixlieAIAPIRoot}/api/engine/${projectId}/edges`, {
      headers: {
        "Content-Type": "application/json",
      },
    }).then((response) => {
      if (!response.ok) {
        throw new Error("Failed to fetch edges");
      }
      response.json().then((responsePayload: EngineResponsePayload) => {
        if (responsePayload.type === "Results") {
          setStore((existing: IEngineStore) => ({
            ...existing,
            projects: {
              ...existing.projects,
              [projectId]: {
                ...existing.projects[projectId],
                edges: responsePayload.data.edges,
              },
            },
          }));
        }
      });
    });
  };

  const getRelatedNodes = (
    projectId: string,
    nodeId: number,
    relatedNodeType: string,
  ): Array<APINodeItem> => {
    if (nodeId in store.projects[projectId].nodes) {
      if (nodeId in store.projects[projectId].edges) {
        let nodes: Array<APINodeItem> = [];
        for (const edge of store.projects[projectId].edges[nodeId]) {
          let [nId, edgeLabel]: [number, string] = edge;
          if (edgeLabel === relatedNodeType) {
            if (nId in store.projects[projectId].nodes) {
              nodes.push(store.projects[projectId].nodes[nId]);
            }
          }
        }
        return nodes;
      }
      return [];
    }
    return [];
  };

  const toggleCrawl = (projectId: string, nodeId: number) => {
    setStore("projects", projectId, "nodes", nodeId, "isFetching", true);
    let pixlieAIAPIRoot = getPixlieAIAPIRoot();
    fetch(`${pixlieAIAPIRoot}/api/engine/${projectId}/domain/${nodeId}`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
    }).then((response) => {
      if (!response.ok) {
        setStore("projects", projectId, "nodes", nodeId, "isFetching", false);
        throw new Error("Failed to toggle crawl");
      }
    });
  };

  return [
    store,
    {
      setProjectId,
      fetchNodes,
      fetchEdges,
      getRelatedNodes,
      toggleCrawl,
    },
  ] as const; // `as const` forces tuple type inference
};

type TStoreAndFunctions = ReturnType<typeof makeStore>;
export const engineStore = makeStore();

const EngineContext = createContext<TStoreAndFunctions>(engineStore);

export const EngineProvider: Component<IProviderPropTypes> = (props) => {
  return (
    <EngineContext.Provider value={engineStore}>
      {props.children}
    </EngineContext.Provider>
  );
};

export const useEngine = () => useContext(EngineContext);
