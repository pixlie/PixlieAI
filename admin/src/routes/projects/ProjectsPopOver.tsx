import { Component, createMemo, createSignal, For, Show } from "solid-js";
import { useWorkspace } from "../../stores/workspace";
import { A, useNavigate, useParams } from "@solidjs/router";
import ToolTip from "../../widgets/navigation/ToolTip";
import Icon from "../../widgets/interactable/Icon";

const ProjectsPopOver: Component = () => {
  const [visible, setVisible] = createSignal<boolean>(false);
  const [workspace] = useWorkspace();
  const params = useParams();
  const navigate = useNavigate();

  const getProjects = createMemo(() => {
    if (workspace.projects) {
      return workspace.projects.map((project) => ({
        isActive: project.uuid === params.projectId,
        isReady: !!project.name,
        name: project.name || "Project",
        uuid: project.uuid,
      }));
    }
    return [];
  });

  return (
    <div class="relative w-10">
      <ToolTip text="Activity">
        <button
          onClick={() => setVisible(true)}
          aria-label="Activity"
          class="flex items-center p-2 text-gray-800 hover:text-gray-950 hover:bg-slate-200 rounded-full"
        >
          <Icon name="bell" />
        </button>
      </ToolTip>
      <Show when={visible()}>
        <button
          class="fixed inset-0 bg-slate-500/20 transition-opacity transition duration-500 ease-in-out z-10"
          onClick={() => setVisible(false)}
        />
        <div class="absolute right-0 mt-1.5 z-20 w-72 rounded-md shadow-md  border-slate-200 border  bg-white  focus:outline-none flex flex-col py-2 gap-2">
          <For each={getProjects()}>
            {(project, i) => (
              <>
                {i() > 0 && <hr />}
                <A
                  href={`/p/${project.uuid}/workflow`}
                  onClick={(event) => {
                    event.preventDefault();
                    setVisible(false);
                    navigate(`/p/${project.uuid}/workflow`);
                  }}
                  class="block w-full"
                  role="menuitem"
                >
                  <div
                    class={
                      "flex w-full items-center px-4 py-1 gap-2 hover:bg-blue-100 hover:text-gray-950 " +
                      (project.isActive
                        ? "bg-slate-200 text-gray-950"
                        : "text-gray-800")
                    }
                  >
                    <div class="flex-1 overflow-hidden gap-1 flex flex-col">
                      <span class="block font-medium truncate text-left">
                        {project.name}
                      </span>
                      {/* <p class="text-gray-500 text-xs">{`Status: ${project.isReady ? "Ready" : "In Progress"}`}</p> */}
                    </div>
                    {project.isReady ? (
                      <Icon
                        name="chevron-right"
                        size={20}
                        class="text-slate-400 -mr-1.5"
                      />
                    ) : (
                      <Icon
                        name="loader"
                        size={16}
                        class="text-slate-400 animate-spin"
                      />
                    )}
                  </div>
                </A>
              </>
            )}
          </For>
        </div>
      </Show>
    </div>
  );
};

export default ProjectsPopOver;
