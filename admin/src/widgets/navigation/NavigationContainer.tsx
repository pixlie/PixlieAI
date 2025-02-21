import { Component, JSX } from "solid-js";
import Navbar from "./Navbar.tsx";
import { useUIClasses } from "../../stores/UIClasses.tsx";
import Sidebar from "./Sidebar.tsx";

interface NavigationContainerProps {
  children: JSX.Element;
}

const NavigationContainer: Component<NavigationContainerProps> = (props) => {
  const [_, { getColors }] = useUIClasses();

  return (
    <div class={"relative isolate flex h-dvh w-dvw " + getColors().app}>
      <Navbar />
      <div class="mt-20 flex-1 flex p-6 gap-6 w-ful">
        <Sidebar />
        <div
          class={
            "flex flex-col flex-1 rounded border " + getColors().mainContent
          }
        >
          {/* <Breadcrumb /> */}
          <div class="flex flex-col flex-1 overflow-scroll p-6 gap-6">
            {props.children}
          </div>
        </div>
      </div>
    </div>
  );
};

export default NavigationContainer;
