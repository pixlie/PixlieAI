import { batch, Component, createContext, useContext } from "solid-js";
import { createStore } from "solid-js/store";
import { APINodeItem } from "../api_types/APINodeItem.ts";
import { EdgeLabel } from "../api_types/EdgeLabel.ts";
import { EngineResponsePayload } from "../api_types/EngineResponsePayload.ts";
import { getPixlieAIAPIRoot } from "../utils/api";
import {
  IEngineEdges,
  IEngineNodes,
  IEngineStore,
  INodeItem,
  IProviderPropTypes,
} from "../utils/types";
import { APINodeEdges } from "../api_types/APINodeEdges.ts";

const makeStore = () => {
  const [store, setStore] = createStore<IEngineStore>({
    projects: {},
    sync: [],
  });

  const setProjectId = (projectId: string) => {
    if (!!store.projects[projectId]) {
      return;
    }
    setStore("projects", projectId, {
      nodes: {},
      edges: {},
      nodesFetchedUpto: BigInt(0),
      edgesFetchedUpto: BigInt(0),
      isFetching: false,
    });
  };

  const fetchNodes = (projectId: string) => {
    let pixlieAIAPIRoot = getPixlieAIAPIRoot();
    fetch(
      `${pixlieAIAPIRoot}/api/engine/${projectId}/nodes?since=${store.projects[projectId].nodesFetchedUpto}`,
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
          let maxNodeWrittenAt = store.projects[projectId].nodesFetchedUpto;
          let nodes: IEngineNodes = Object.fromEntries(
            responsePayload.data.map((node) => {
              const writtenAt = node.written_at;
              if (writtenAt > maxNodeWrittenAt) {
                maxNodeWrittenAt = writtenAt;
              }
              let newNode = node as INodeItem;
              newNode.isFetching = false;
              return [newNode.id, newNode];
            }),
          );
          setStore("projects", projectId, "nodes", nodes);
          setStore("projects", projectId, "nodesFetchedUpto", maxNodeWrittenAt);
        }
      });
    });
  };

  const fetchEdges = (projectId: string) => {
    let pixlieAIAPIRoot = getPixlieAIAPIRoot();
    fetch(
      `${pixlieAIAPIRoot}/api/engine/${projectId}/edges?since=${store.projects[projectId].edgesFetchedUpto}`,
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
          let maxEdgeWrittenAt = store.projects[projectId].edgesFetchedUpto;

          let edges: IEngineEdges = Object.fromEntries(
            Object.entries(responsePayload.data).map(([nodeId, apiEdges]) => {
              if (apiEdges === undefined) {
                return [Number(nodeId), {} as APINodeEdges];
              }
              const writtenAt = apiEdges.written_at;
              if (writtenAt > maxEdgeWrittenAt) {
                maxEdgeWrittenAt = writtenAt;
              }
              return [Number(nodeId), apiEdges];
            }),
          );
          setStore("projects", projectId, "edges", edges);
          setStore("projects", projectId, "edgesFetchedUpto", maxEdgeWrittenAt);
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
    if (store.sync.includes(projectId)) {
      return;
    }
    setStore("sync", (existing) => [...existing, projectId]);
    const fetcher = (projectId: string) => {
      return () => {
        if (
          !store.projects[projectId] ||
          store.projects[projectId].isFetching
        ) {
          return;
        }
        setStore("projects", projectId, "isFetching", true);
        batch(() => {
          fetchNodes(projectId);
          fetchEdges(projectId);
          setStore("projects", projectId, "isFetching", false);
        });
        if (store.sync.length > 0 && store.sync.includes(projectId)) {
          window.setTimeout(fetcher(projectId), 2000);
        }
      };
    };
    window.setTimeout(fetcher(projectId), 100);
  };

  const stopSync = (projectId?: string) => {
    if (!!projectId) {
      setStore("sync", (existing) => existing.filter((id) => id !== projectId));
    } else {
      setStore("sync", []);
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
