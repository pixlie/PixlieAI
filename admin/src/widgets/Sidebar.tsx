import { Component, For } from "solid-js";
import SidebarLink from "./navigation/SidebarLink";
import { routes } from "../routes/routeList";
import { useUIClasses } from "../stores/UIClasses";
import { useWorkspace } from "../stores/workspace";

const Sidebar: Component = () => {
  const [_, { getColors }] = useUIClasses();
  const [workspace] = useWorkspace();

  return (
    <div
      class={
        "fixed inset-y-0 z-50 w-72 flex flex-col " + getColors()["sideBar"]
      }
    >
      <div class="flex grow flex-col gap-y-5 overflow-y-auto px-6 py-4">
        <div class="flex items-center gap-3">
          <img
            class="h-auto w-7"
            src="https://pixlie.com/images/logo.png"
            alt="Pixlie AI"
          />
          <p class={"text-2xl font-medium " + getColors()["sideBar.link"]}>
            Pixlie AI
          </p>
        </div>

        <nav class="flex flex-1 flex-col">
          <ul role="list" class="flex flex-1 flex-col gap-y-7">
            <li>
              <ul role="list" class="-mx-2 space-y-1">
                {!!workspace.isReady &&
                workspace.settingsStatus?.type === "Complete" ? (
                  <For each={routes}>{(item) => <SidebarLink {...item} />}</For>
                ) : (
                  <SidebarLink
                    label="Setup"
                    icon="cog"
                    href="/settings/setup"
                  />
                )}
              </ul>
            </li>
          </ul>
        </nav>
      </div>
    </div>
  );
};

export default Sidebar;
