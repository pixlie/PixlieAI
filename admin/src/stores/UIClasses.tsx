import { Component, createContext, useContext } from "solid-js";
import { createStore } from "solid-js/store";
import { IProviderPropTypes } from "../utils/types";
import lightDefault from "../utils/uIClasses/lightDefault";

interface IStore {
  currentTheme: "lightDefault";
}

const makeStore = () => {
  const [store, setStore] = createStore<IStore>({
    currentTheme: "lightDefault",
  });

  return [
    store,
    {
      setTheme: (theme: "lightDefault") => {
        setStore("currentTheme", theme);
      },
      getColors() {
        return lightDefault["tailwindClasses"];
      },
    },
  ] as const; // `as const` forces tuple type inference
};

type TStoreAndFunctions = ReturnType<typeof makeStore>;
const tailwindClassesStore = makeStore();

const uiClassesContext =
  createContext<TStoreAndFunctions>(tailwindClassesStore);

export const UIClassesProvider: Component<IProviderPropTypes> = (props) => {
  return (
    <uiClassesContext.Provider value={tailwindClassesStore}>
      {props.children}
    </uiClassesContext.Provider>
  );
};

export const useUIClasses = () => useContext(uiClassesContext);
