import { Component, For } from "solid-js";
import SidebarLink from "./SidebarLink";
import { perProjectRoutes } from "../../routes/routeList";
import { useUIClasses } from "../../stores/UIClasses";
import { useWorkspace } from "../../stores/workspace";
import { useLocation } from "@solidjs/router";

const Sidebar: Component = () => {
  const [_, { getColors }] = useUIClasses();
  const [workspace] = useWorkspace();
  const location = useLocation();

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
              {!!workspace.settings?.currentProject ? (
                <For each={perProjectRoutes}>
                  {(item) => (
                    <SidebarLink
                      label={item.label}
                      href={`/p/${workspace.settings?.currentProject}${item.href}`}
                      isActive={location.pathname.startsWith(
                        `/p/${workspace.settings?.currentProject}${item.href}`,
                      )}
                    />
                  )}
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
