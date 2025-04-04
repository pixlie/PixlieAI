import { Component, createContext, useContext } from "solid-js";
import { createStore } from "solid-js/store";
import { IEngineStore, INodeItem, IProviderPropTypes } from "../utils/types";
import { getPixlieAIAPIRoot } from "../utils/api";
import { EngineResponsePayload } from "../api_types/EngineResponsePayload.ts";
import { APINodeItem } from "../api_types/APINodeItem.ts";
import { APINodeEdges } from "../api_types/APINodeEdges.ts";
import { EdgeLabel } from "../api_types/EdgeLabel.ts";

const makeStore = () => {
  const [store, setStore] = createStore<IEngineStore>({
    projects: {},
    sync: [],
  });

  const setProjectId = (projectId: string) => {
    if (!!store.projects[projectId]) {
      return;
    }
    setStore((existing: IEngineStore) => ({
      ...existing,
      projects: {
        ...existing.projects,
        [projectId]: {
          nodes: {},
          edges: {},
          nodesFetchedAt: 0,
          edgesFetchedAt: 0,
          isFetching: false,
        },
      },
    }));
  };

  const fetchNodes = (projectId: string) => {
    let pixlieAIAPIRoot = getPixlieAIAPIRoot();
    fetch(
      `${pixlieAIAPIRoot}/api/engine/${projectId}/nodes?since=${store.projects[projectId].nodesFetchedAt}`,
      {
        headers: {
          "Content-Type": "application/json",
        },
      },
    ).then((response) => {
      if (!response.ok) {
        throw new Error("Failed to fetch nodes");
      }
      response.json().then((responsePayload: EngineResponsePayload) => {
        if (responsePayload.type === "Nodes") {
          setStore((existing: IEngineStore) => ({
            ...existing,
            projects: {
              ...existing.projects,
              [projectId]: {
                ...existing.projects[projectId],
                nodes: {
                  ...existing.projects[projectId].nodes,
                  ...responsePayload.data.reduce(
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
                nodesFetchedAt: Date.now(),
              },
            },
          }));
        }
      });
    });
  };

  const fetchEdges = (projectId: string) => {
    let pixlieAIAPIRoot = getPixlieAIAPIRoot();
    fetch(
      `${pixlieAIAPIRoot}/api/engine/${projectId}/edges?since=${store.projects[projectId].edgesFetchedAt}`,
      {
        headers: {
          "Content-Type": "application/json",
        },
      },
    ).then((response) => {
      if (!response.ok) {
        throw new Error("Failed to fetch edges");
      }
      response.json().then((responsePayload: EngineResponsePayload) => {
        if (responsePayload.type === "Edges") {
          setStore((existing: IEngineStore) => ({
            ...existing,
            projects: {
              ...existing.projects,
              [projectId]: {
                ...existing.projects[projectId],
                edges: {
                  ...existing.projects[projectId].edges,
                  ...(responsePayload.data as {
                    [nodeId: number]: APINodeEdges;
                  }),
                },
                edgesFetchedAt: Date.now(),
              },
            },
          }));
        }
      });
    });
  };

  const getNodeById = (
    projectId: string,
    nodeId: number,
  ): APINodeItem | undefined => {
    if (
      projectId in store.projects &&
      nodeId in store.projects[projectId].nodes
    ) {
      return store.projects[projectId].nodes[nodeId];
    }
    return undefined;
  };

  const getNodes = (
    projectId: string,
    filterFn: (node: APINodeItem) => boolean,
  ): Array<APINodeItem> => {
    if (projectId in store.projects) {
      return Object.values(store.projects[projectId]?.nodes || {})
        .filter((node) => filterFn(node))
        .map((node) => node);
    }
    return [];
  };

  const getRelatedNodeIds = (
    projectId: string,
    nodeId: number,
    relatedNodeTypes: EdgeLabel,
  ): Array<number> => {
    if (
      projectId in store.projects &&
      nodeId in store.projects[projectId].nodes &&
      nodeId in store.projects[projectId].edges
    ) {
      return store.projects[projectId].edges[nodeId].edges
        .filter(
          ([relatedNodeId, edgeLabel]) =>
            relatedNodeTypes === edgeLabel &&
            relatedNodeId in store.projects[projectId].nodes,
        )
        .map(([relatedNodeId, _]) => relatedNodeId);
    }
    return [];
  };

  const getRelatedNodes = (
    projectId: string,
    nodeId: number,
    relatedNodeTypes: EdgeLabel,
    filterFn?: (node: APINodeItem) => boolean,
  ): Array<APINodeItem> => {
    return getRelatedNodeIds(projectId, nodeId, relatedNodeTypes)
      .map((id) => getNodeById(projectId, id) as APINodeItem)
      .filter((node) => !filterFn || filterFn(node));
  };

  const sync = (projectId: string) => {
    if (store.sync.filter((x) => x === projectId).length > 0) {
      return;
    }

    const fetcher = (projectId: string) => {
      return () => {
        if (!store.projects[projectId]) {
          return;
        }
        if (store.projects[projectId].isFetching) {
          return;
        }
        setStore((existing: IEngineStore) => ({
          ...existing,
          projects: {
            ...existing.projects,
            [projectId]: {
              ...existing.projects[projectId],
              isFetching: true,
            },
          },
        }));
        fetchNodes(projectId);
        fetchEdges(projectId);
        setStore((existing: IEngineStore) => ({
          ...existing,
          projects: {
            ...existing.projects,
            [projectId]: {
              ...existing.projects[projectId],
              isFetching: false,
            },
          },
        }));
        if (
          store.sync.length > 0 &&
          store.sync.filter((x) => x === projectId).length > 0
        ) {
          window.setTimeout(fetcher(projectId), 2000);
        }
      };
    };

    setStore((existing: IEngineStore) => ({
      ...existing,
      sync: [...existing.sync, projectId],
    }));
    window.setTimeout(fetcher(projectId), 100);
  };

  const stopSync = (projectId?: string) => {
    if (!!projectId) {
      setStore((existing: IEngineStore) => ({
        ...existing,
        sync: [...existing.sync.filter((x) => x !== projectId)],
      }));
    } else {
      setStore((existing: IEngineStore) => ({
        ...existing,
        sync: [],
      }));
    }
  };

  return [
    store,
    {
      setProjectId,
      sync,
      stopSync,
      getNodeById,
      getNodes,
      getRelatedNodeIds,
      getRelatedNodes,
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
