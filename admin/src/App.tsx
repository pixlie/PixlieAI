import { Component } from "solid-js";
import { WorkspaceProvider } from "./stores/workspace";
import InitialCheckAndLoad from "./utils/InitialCheckAndLoad.tsx";
import { UIClassesProvider } from "./stores/UIClasses";
import { RouteSectionProps } from "@solidjs/router";
import NavigationContainer from "./widgets/navigation/NavigationContainer.tsx";

const App: Component<RouteSectionProps> = (props) => {
  return (
    <UIClassesProvider>
      <WorkspaceProvider>
        <InitialCheckAndLoad>
          <NavigationContainer children={props.children} />
        </InitialCheckAndLoad>
      </WorkspaceProvider>
    </UIClassesProvider>
  );
};

export default App;
