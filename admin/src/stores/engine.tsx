import { Component, createContext, useContext } from "solid-js";
import { createStore } from "solid-js/store";
import { IEngine, INodeItem, IProviderPropTypes } from "../utils/types";
import { getPixlieAIAPIRoot } from "../utils/api";
import { EngineResponsePayload } from "../api_types/EngineResponsePayload.ts";
import { APINodeItem } from "../api_types/APINodeItem.ts";

const makeStore = () => {
  const [store, setStore] = createStore<IEngine>({
    nodes: {},
    nodeIdsByLabel: {},
  });

  const setProjectId = (projectId: string) => {
    setStore((existing: IEngine) => ({
      ...existing,
      projectId,
    }));
  };

  const fetchNodesByLabel = (label: string) => {
    let pixlieAIAPIRoot = getPixlieAIAPIRoot();
    fetch(
      `${pixlieAIAPIRoot}/api/engine/${store.projectId}/nodes?` +
        new URLSearchParams({
          label,
        }).toString(),
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
        if (responsePayload.type === "Results") {
          setStore((existing: IEngine) => ({
            ...existing,
            nodes: responsePayload.data.nodes.reduce(
              (acc, item) => ({
                ...acc,
                [item.id]: {
                  ...item,
                  isFetching: false,
                },
              }),
              existing.nodes,
            ),
            nodeIdsByLabel: {
              ...existing.nodeIdsByLabel,
              [label]:
                responsePayload.data.node_ids_by_label &&
                label in responsePayload.data.node_ids_by_label
                  ? responsePayload.data.node_ids_by_label[label]
                  : [],
            },
          }));
        }
      });
    });
  };

  const fetchNodesByIds = (ids: Array<number>) => {
    let pixlieAIAPIRoot = getPixlieAIAPIRoot();
    fetch(
      `${pixlieAIAPIRoot}/api/engine/${store.projectId}/nodes?` +
        new URLSearchParams({
          ids: ids.join(","),
        }).toString(),
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
        if (responsePayload.type === "Results") {
          setStore((existing: IEngine) => ({
            ...existing,
            nodes: {
              ...existing.nodes,
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
          }));
        }
      });
    });
  };

  const getRelatedNodes = (
    nodeId: number,
    relatedNodeType: string,
  ): Array<APINodeItem> => {
    if (nodeId in store.nodes) {
      if (
        relatedNodeType in store.nodes[nodeId].edges &&
        store.nodes[nodeId].edges[relatedNodeType]
      ) {
        let nodes: Array<APINodeItem> = [];
        let nodesToBeFetched: Array<number> = [];
        store.nodes[nodeId].edges[relatedNodeType]?.forEach((nId: number) => {
          if (nId in store.nodes) {
            nodes.push(store.nodes[nId]);
          } else {
            nodesToBeFetched.push(nId);
          }
        });
        if (nodesToBeFetched.length > 0) {
          fetchNodesByIds(nodesToBeFetched);
        }
        return nodes;
      }
      return [];
    }
    return [];
  };

  return [
    store,
    {
      setProjectId,
      fetchNodesByLabel,
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
