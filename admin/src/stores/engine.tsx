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
      fetchNodesByLabel: async (label: string) => {
        setStore((data) => ({ ...data, isFetching: true, isReady: false }));
        let pixieAIAPIRoot = getPixlieAIAPIRoot();
        let response = await fetch(
          `${pixieAIAPIRoot}/api/engine/nodes?` +
            new URLSearchParams({
              label,
            }).toString(),
        );
        if (!response.ok) {
          console.error("Failed to fetch settings");
        }
        let engineResponse: EngineApiResponse = await response.json();
        if (engineResponse.type === "Results") {
          setStore((state) => ({
            ...state,
            nodes: engineResponse.data.nodes,
            nodeIdsByLabel: {
              ...state.nodeIdsByLabel,
              [label]: engineResponse.data.query_type.NodeIdsByLabel[1],
            },
            isFetching: false,
            isReady: true,
          }));
        }
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
