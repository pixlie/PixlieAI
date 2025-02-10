import { Component, createContext, useContext } from "solid-js";
import { createStore } from "solid-js/store";
import { IEngine, IProviderPropTypes } from "../utils/types";
import { getPixlieAIAPIRoot } from "../utils/api";
import { EngineApiResponse } from "../api_types/EngineApiResponse";

const makeStore = () => {
  const [store, setStore] = createStore<IEngine>({
    nodes: {},
    nodeIdsByLabel: {},

    isReady: false,
    isFetching: false,
  });

  return [
    store,
    {
      fetchNodesByLabel: (projectId: string, label: string) => {
        setStore((data) => ({ ...data, isFetching: true, isReady: false }));
        let pixlieAIAPIRoot = getPixlieAIAPIRoot();
        fetch(
          `${pixlieAIAPIRoot}/api/engine/${projectId}/nodes?` +
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
          response.json().then((engineResponse: EngineApiResponse) => {
            if (engineResponse.type === "Results") {
              setStore((existing: IEngine) => ({
                ...existing,
                nodes: engineResponse.data.nodes.reduce(
                  (acc, item) => ({
                    ...acc,
                    [item.id]: item,
                  }),
                  existing.nodes,
                ),
                nodeIdsByLabel: {
                  ...existing.nodeIdsByLabel,
                  [label]:
                    engineResponse.data.node_ids_by_label &&
                    label in engineResponse.data.node_ids_by_label
                      ? engineResponse.data.node_ids_by_label[label]
                      : [],
                },
                isFetching: false,
                isReady: true,
              }));
            } else {
              setStore((data) => ({
                ...data,
                isFetching: false,
                isReady: true,
              }));
            }
          });
        });
      },
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
