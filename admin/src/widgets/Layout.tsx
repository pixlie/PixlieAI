import { Component, JSX } from "solid-js";
import Navbar from "./navigation/Navbar.tsx";
import { useUIClasses } from "../stores/UIClasses.tsx";
import Sidebar from "./navigation/Sidebar.tsx";
import { useLocation } from "@solidjs/router";
import HelpModal from "../routes/help/HelpModal.tsx";
import SettingsModal from "../routes/settings/SettingsModal.tsx";

interface ILayoutProps {
  children: JSX.Element;
}

const Layout: Component<ILayoutProps> = (props) => {
  const [_, { getColors }] = useUIClasses();
  const location = useLocation();

  return (
    <div class={"relative isolate flex h-dvh w-dvw " + getColors().app}>
      <Navbar />
      <div class="relative">{location.hash === "#help" && <HelpModal />}</div>
      <div class="relative">
        {location.hash === "#settings" && <SettingsModal />}
      </div>
      <div class="mt-14 flex-1 flex p-8 gap-8 w-full">
        <Sidebar />
        <div
          class={
            "flex flex-col flex-1 rounded-lg border " + getColors().mainContent
          }
        >
          {/* <Breadcrumb /> */}
          <div class="flex flex-col flex-1 overflow-scroll p-8 gap-4">
            {props.children}
          </div>
        </div>
      </div>
    </div>
  );
};

export default Layout;
