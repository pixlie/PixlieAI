import { Component, JSX } from "solid-js";
import Navbar from "./navigation/Navbar.tsx";
import { useUIClasses } from "../stores/UIClasses.tsx";
import Sidebar from "./navigation/Sidebar.tsx";
import { useLocation } from "@solidjs/router";
import HelpModal from "../routes/help/HelpModal.tsx";
import SettingsModal from "../routes/settings/SettingsModal.tsx";
import ProjectForm from "./projects/ProjectForm.tsx";

interface ILayoutProps {
  children: JSX.Element;
}

const Layout: Component<ILayoutProps> = (props) => {
  const [_, { getColors }] = useUIClasses();
  const location = useLocation();

  return (
    <div class={"relative isolate flex h-dvh w-dvw " + getColors().app}>
      <Navbar />
      <div class="relative">{location.hash === "#create" && <ProjectForm />}</div>
      <div class="relative">{location.hash === "#help" && <HelpModal />}</div>
      <div class="relative">{location.hash === "#settings" && <SettingsModal />}</div>
      <div class="mt-20 flex-1 flex p-6 gap-6 w-full">
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

export default Layout;
