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
    <Loader>
      <div class={`relative isolate flex min-h-svh w-full ${getColors().app}`}>
        <Sidebar />

        <div class="ml-48 px-6 flex-1 ">{props.children}</div>
      </div>
    </Loader>
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
