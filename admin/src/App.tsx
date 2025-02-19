import { Component, JSX } from "solid-js";
import Sidebar from "./widgets/navigation/Sidebar";
import { WorkspaceProvider } from "./stores/workspace";
import InitialCheckAndLoad from "./utils/InitialCheckAndLoad.tsx";
import { UIClassesProvider, useUIClasses } from "./stores/UIClasses";
import { RouteSectionProps } from "@solidjs/router";

interface AppInnerProps {
  children: JSX.Element;
}

const AppInner: Component<AppInnerProps> = (props) => {
  const [_, { getColors }] = useUIClasses();

  return (
    <InitialCheckAndLoad>
      <div class={`relative isolate flex min-h-svh w-full ${getColors().app}`}>
        <Sidebar />

        <div class="ml-48 p-6 flex-1 ">{props.children}</div>
      </div>
    </InitialCheckAndLoad>
  );
};

const App: Component<RouteSectionProps> = (props) => {
  return (
    <UIClassesProvider>
      <WorkspaceProvider>
        <AppInner children={props.children} />
      </WorkspaceProvider>
    </UIClassesProvider>
  );
};

export default App;
