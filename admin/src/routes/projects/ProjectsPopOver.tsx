import { Component, createSignal, For, Show } from "solid-js";
import { useWorkspace } from "../../stores/workspace";
import { A, useNavigate, useParams } from "@solidjs/router";
import BellIcon from "../../assets/icons/tabler-bell.svg";
import ChevronRightIcon from "../../assets/icons/tabler-chevron-right.svg";
import LoaderIcon from "../../assets/icons/tabler-loader.svg";
import IconButton from "../../widgets/interactable/IconButton";

const ProjectsPopOver: Component = () => {
  const [visible, setVisible] = createSignal<boolean>(false);
  const [workspace, { fetchProjects }] = useWorkspace();
  const params = useParams();
  const navigate = useNavigate();

  return (
    <div class="relative w-10">
      <IconButton
        name="Activity"
        icon={BellIcon}
        onClick={() => {
          fetchProjects();
          setVisible(true);
        }}
      />
      <Show when={visible()}>
        <button
          class="fixed inset-0 bg-slate-500/20 transition-opacity transition duration-500 ease-in-out z-10 cursor-default"
          onClick={() => setVisible(false)}
        />
        <div class="absolute right-0 z-20 w-72 max-h-72 rounded-lg overflow-hidden shadow-md border-slate-200 border  bg-white  focus:outline-none flex flex-col overflow-y-auto">
          <For
            each={workspace.projects}
            fallback={
              <p class="text-gray-500 text-center p-4">No activity yet!</p>
            }
          >
            {({ uuid, name }, i) => (
              <>
                {i() > 0 && <hr />}
                <A
                  href={`/p/${uuid}/workflow`}
                  onClick={(event) => {
                    event.preventDefault();
                    setVisible(false);
                    navigate(`/p/${uuid}/workflow`);
                  }}
                  class="block w-full"
                  role="menuitem"
                >
                  <div
                    class={
                      "flex w-full items-center px-4  py-3.5 gap-2 hover:text-slate-950 " +
                      (uuid === params.projectId
                        ? "font-medium bg-blue-100 text-slate-950 hover:bg-blue-100"
                        : "text-slate-800 hover:bg-slate-50")
                    }
                  >
                    <div class="flex-1 overflow-hidden gap-1 flex flex-col items-center-center">
                      <span class="block truncate text-left">
                        {name || "Project"}
                      </span>
                      {/* <p class="text-gray-500 text-xs">{`Status: ${project.isReady ? "Ready" : "In Progress"}`}</p> */}
                    </div>

                    <div class="text-slate-400">
                      {!!name ? (
                        <ChevronRightIcon />
                      ) : (
                        <div class="-mr-1">
                          <LoaderIcon />
                        </div>
                      )}
                    </div>
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
