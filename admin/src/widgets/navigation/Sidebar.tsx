import { Component, For } from "solid-js";
import SidebarLink from "./SidebarLink";
import {
  getGlobalRoutes,
  getPerProjectRoutes,
} from "../../routes/routeList.tsx";
import { useUIClasses } from "../../stores/UIClasses";
import { useWorkspace } from "../../stores/workspace";
import { useParams } from "@solidjs/router";

const Sidebar: Component = () => {
  const [_, { getColors }] = useUIClasses();
  const [workspace] = useWorkspace();
  const params = useParams();

  return (
    <div
      class={
        "fixed w-48 inset-y-0 z-50 flex flex-col " + getColors()["sideBar"]
      }
    >
      <div class="flex items-center p-4">
        <a
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
        </a>
      </div>

      <div class="grow">
        <nav class="flex flex-col px-2">
          {workspace.isReady &&
          workspace.settingsStatus?.type === "Complete" ? (
            <>
              <For each={getGlobalRoutes()}>
                {(item) => <SidebarLink {...item} />}
              </For>
              <span class="block my-3" />

              {!!params.projectId ? (
                <For each={getPerProjectRoutes()}>
                  {(item) => <SidebarLink {...item} />}
                </For>
              ) : null}
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
