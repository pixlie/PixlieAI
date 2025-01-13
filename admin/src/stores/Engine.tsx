import { Component, createContext, useContext } from "solid-js";
import { createStore } from "solid-js/store";
import { IProviderPropTypes, IWorkspace } from "../utils/types";
import { camelCasedKeys, getPixlieAIAPIRoot } from "../utils/api";
import { Settings } from "../api_types/Settings";

const makeStore = () => {
  const [store, setStore] = createStore<IWorkspace>({
    isReady: false,
    isFetching: false,
  });

  return [
    store,
    {
      fetchNodesByLabel: async () => {
        setStore((data) => ({ ...data, isFetching: true, isReady: false }));
        let pixieAIAPIRoot = getPixlieAIAPIRoot();
        let response = await fetch(
          `${pixieAIAPIRoot}/engine?` +
            new URLSearchParams({
              label: "domain",
            }).toString(),
        );
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
