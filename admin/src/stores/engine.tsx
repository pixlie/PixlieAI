import { Component, createContext, useContext } from "solid-js";
import { createStore } from "solid-js/store";
import { IEngine, IProviderPropTypes } from "../utils/types";
import { getPixlieAIAPIRoot } from "../utils/api";
import { EngineApiResponse } from "../api_types/EngineApiResponse";
import { NodeWrite } from "../api_types/NodeWrite.ts";

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
      setCurrentProject: async (projectId: string) => {
        let pixlieAIAPIRoot = getPixlieAIAPIRoot();
        let response = await fetch(`${pixlieAIAPIRoot}/api/settings`, {
          method: "PUT",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            current_project: projectId,
          }),
        });
        if (!response.ok) {
          throw new Error("Failed to select project");
        }
      },

      fetchNodesByLabel: async (label: string) => {
        setStore((data) => ({ ...data, isFetching: true, isReady: false }));
        let pixlieAIAPIRoot = getPixlieAIAPIRoot();
        let response = await fetch(
          `${pixlieAIAPIRoot}/api/engine/nodes?` +
            new URLSearchParams({
              label,
            }).toString(),
        );
        if (!response.ok) {
          throw new Error("Failed to fetch nodes");
        }
        let engineResponse: EngineApiResponse = await response.json();
        if (engineResponse.type === "Results") {
          setStore((state: IEngine) => ({
            ...state,
            nodes: engineResponse.data.nodes.reduce(
              (acc, item) => ({
                ...acc,
                [item.id]: item,
              }),
              state.nodes,
            ),
            // nodeIdsByLabel: {
            //   ...state.nodeIdsByLabel,
            //   [label]:
            //     "NodeIdsByLabel" in engineResponse.data.query_type
            //       ? engineResponse.data.query_type.NodeIdsByLabel[1]
            //       : [],
            // },
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
      },

      insertNode: async (node: NodeWrite) => {
        let pixlieAIAPIRoot = getPixlieAIAPIRoot();
        let response = await fetch(`${pixlieAIAPIRoot}/api/engine/nodes`, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify(node),
        });
        if (!response.ok) {
          throw new Error("Failed to insert node");
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
