import { Component, Show } from "solid-js";
import { PerProjectRoutes } from "../../routes/RouteList.tsx";
import { useUIClasses } from "../../stores/UIClasses";
import { useWorkspace } from "../../stores/workspace";
import { useParams } from "@solidjs/router";

const Sidebar: Component = () => {
  const [_, { getColors }] = useUIClasses();
  const [workspace] = useWorkspace();
  const params = useParams();

  return (
    <Show
      when={
        workspace.isReady &&
        workspace.settingsStatus?.type === "Complete" &&
        workspace.projects &&
        !!params.projectId
      }
    >
      <div
        class={
          "w-48 rounded border flex flex-col pl-5 pr-3 py-3 " +
          getColors()["sideBar"]
        }
      >
        <PerProjectRoutes />
      </div>
        {/* Changes from feature/keyword-match in merge commit */}
      {/* <div class="grow">
        <nav class="grid grid-cols-1 gap-y-0.5 px-2">
          {workspace.isReady &&
          workspace.settingsStatus?.type === "Complete" ? (
            <>
              <GlobalRoutes />
              <span class="block my-2" />

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
    </div> */}
    </Show>
  );
};

export default Sidebar;
