import { Component, For } from "solid-js";
import SidebarLink from "../widgets/navigation/SidebarLink";
import { routes } from "../routes/routeList";
import { useUIClasses } from "../stores/UIClasses";
import { useWorkspace } from "../stores/Workspace";

const Sidebar: Component = () => {
  const [_, { getColors }] = useUIClasses();
  const [workspace] = useWorkspace();

  return (
    <div
      class={
        "fixed inset-y-0 z-50 w-72 flex flex-col " + getColors()["sideBar"]
      }
    >
      <div class="flex grow flex-col gap-y-5 overflow-y-auto px-6 pb-4">
        <div class="flex h-16 shrink-0 items-center">
          <img
            class="h-8 w-auto"
            src="https://pixlie.com/images/logo.png"
            alt="Pixlie AI"
          />
          &nbsp; Pixlie AI
        </div>

        <nav class="flex flex-1 flex-col">
          <ul role="list" class="flex flex-1 flex-col gap-y-7">
            <li>
              <ul role="list" class="-mx-2 space-y-1">
                {workspace.isReady && !!workspace.settings ? (
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
