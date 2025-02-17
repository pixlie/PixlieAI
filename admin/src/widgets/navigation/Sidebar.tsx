import { Component } from "solid-js";
import SidebarLink from "./SidebarLink";
import { GlobalRoutes, PerProjectRoutes } from "../../routes/RouteList.tsx";
import { useUIClasses } from "../../stores/UIClasses";
import { useWorkspace } from "../../stores/workspace";
import { A } from "@solidjs/router";

const Sidebar: Component = () => {
  const [_, { getColors }] = useUIClasses();
  const [workspace] = useWorkspace();

  return (
    <div
      class={
        "fixed w-48 inset-y-0 z-50 flex flex-col " + getColors()["sideBar"]
      }
    >
      <div class="flex items-center p-4">
        <A
          href="/p"
          class={
            "text-2xl font-medium flex gap-2 " + getColors()["sideBar.logo"]
          }
        >
          <img
            class="h-auto w-7"
            src="https://pixlie.com/images/logo.png"
            alt="Pixlie AI"
          />
          Pixlie AI
        </A>
      </div>

      <div class="grow">
        <nav class="flex flex-col px-2">
          {workspace.isReady &&
          workspace.settingsStatus?.type === "Complete" ? (
            <>
              <GlobalRoutes />
              <span class="block my-3" />

              <PerProjectRoutes />
            </>
          ) : (
            <SidebarLink label="Setup" href="/settings/setup" />
          )}
        </nav>
      </div>

      <div class="mb-2 px-2">
        <SidebarLink label="Settings" href="/settings" />
      </div>
    </div>
  );
};

export default Sidebar;
