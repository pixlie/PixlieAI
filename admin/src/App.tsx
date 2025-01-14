import { Component, JSX } from "solid-js";
import Sidebar from "./widgets/navigation/Sidebar";
import { WorkspaceProvider } from "./stores/workspace";
import Loader from "./utils/Loader";
import { UIClassesProvider, useUIClasses } from "./stores/UIClasses";
import { RouteSectionProps } from "@solidjs/router";

interface AppInnerProps {
  children: JSX.Element;
}

const AppInner: Component<AppInnerProps> = (props) => {
  const [_, { getColors }] = useUIClasses();

  return (
    <div class={`relative isolate flex min-h-svh w-full ${getColors().app}`}>
      <Loader />
      <Sidebar />

      <div class="pl-72 ml-6 flex-1">{props.children}</div>
    </div>
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
