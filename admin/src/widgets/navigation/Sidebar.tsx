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
          "w-48 rounded-lg border flex flex-col pl-5 pr-3 py-3 " +
          getColors()["sideBar"]
        }
      >
        <PerProjectRoutes />
      </div>
    </Show>
  );
};

export default Sidebar;
