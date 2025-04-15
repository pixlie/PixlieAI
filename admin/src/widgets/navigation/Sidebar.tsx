import { Component, For, Show, createSignal } from "solid-js";
import { PerProjectRoutes } from "../../routes/RouteList.tsx";
import { useUIClasses } from "../../stores/UIClasses";
import { useWorkspace } from "../../stores/workspace";
import { useParams } from "@solidjs/router";
import SidebarLink from "./SidebarLink.tsx";
import CollapseSidebarIcon from "../../assets/icons/tabler-layout-sidebar-inactive.svg";
import ExpandSidebarIcon from "../../assets/icons/tabler-layout-sidebar.svg";
import IconButton from "../interactable/IconButton.tsx";

const Sidebar: Component = () => {
  const [_, { getColors }] = useUIClasses();
  const [workspace] = useWorkspace();
  const params = useParams();
  const [collapsed, setCollapsed] = createSignal(false);

  return (
    <Show
      when={
        workspace.isReady &&
        workspace.settingsStatus?.type === "Complete" &&
        workspace.projects
      }
    >
      <div
        class={
          `transition-all duration-300 flex flex-col border max-h-full rounded-lg py-5 ` +
          (collapsed() ? "w-20" : "w-60") +
          " " +
          getColors()["sideBar"]
        }
      >
        <div class="self-end mr-5 pb-2">
          {/* TODO: show projects dropdown for per project routes when sidebar expanded */}
          <IconButton
            name={collapsed() ? "Expand" : "Collapse"}
            icon={collapsed() ? <ExpandSidebarIcon /> : <CollapseSidebarIcon />}
            onClick={() => setCollapsed(!collapsed())}
          />
        </div>
        <div class="overflow-y-auto ">
          <Show when={!collapsed()}>
            {!!params.projectId ? (
              <PerProjectRoutes />
            ) : (
           
              <For each={workspace.projects}>
                {(project) => (
                  <SidebarLink
                    label={project.name || "Untitled"}
                    href={`/p/${project.uuid}/workflow`}
                    isActive={project.uuid === params.projectId}
                  />
                )}
              </For>
           
            )}
          </Show>
        </div>
      </div>
    </Show>
  );
};

export default Sidebar;
