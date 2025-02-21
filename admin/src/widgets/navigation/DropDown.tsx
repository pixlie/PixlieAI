import { Component, createMemo, createSignal, For, Show } from "solid-js";
import { useWorkspace } from "../../stores/workspace";
import { useParams } from "@solidjs/router";

const DropDown: Component = () => {
  const [visible, setVisible] = createSignal<boolean>(false);
  const [workspace] = useWorkspace();
  const params = useParams();
  const getProject = createMemo(() => {
    if (params.projectId && workspace.isReady && workspace.projects) {
      return workspace.projects.find(
        (project) => project.uuid === params.projectId
      );
    }
  });
  return (
    <div class="relative w-48">
      <button
        type="button"
        onClick={() => setVisible(!visible())}
        class="inline-flex items-center justify-between w-full px-3 rounded-md border border-gray-300 shadow-sm py-2.5 bg-white font-semibold text-md text-gray-800 hover:bg-gray-50 focus:outline-none focus:ring-offset-gray-100"
        id="options-menu"
        aria-expanded="true"
        aria-haspopup="true"
      >
        {getProject()?.name || "Projects"}
        <svg
          class="w-5 h-5"
          fill="none"
        //   stroke="oklch(0.551 0.027 264.364)"
            stroke="currentColor"
          viewBox="0 0 24 24"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            // stroke-width="1"
            stroke-width="2.5"
            d="M19 9l-7 7-7-7"
            transform={visible() ? "rotate(180 12 12)" : ""}
          ></path>
        </svg>
      </button>

      <Show when={visible()}>
        <div
          class="origin-top-right absolute right-0 mt-1.5 w-full"
          role="menu"
          aria-orientation="vertical"
          aria-labelledby="options-menu"
        >
          <div
            class="w-full rounded-md shadow-lg bg-white ring-1 ring-black ring-opacity-5 focus:outline-none"
            role="menu"
            aria-orientation="vertical"
            aria-labelledby="options-menu"
          >
            <div class="p-1.5 flex flex-col" role="none">
              <For each={workspace.projects?.filter((project) => project.name !== getProject()?.name)}>
                {(project) => (
                  <a
                    href={`/p/${project.uuid}/workflow`}
                    onClick={() => setVisible(false)}
                    class="block w-full rounded p-1.5 text-md text-gray-700 hover:bg-gray-100 hover:text-gray-900"
                    role="menuitem"
                  >
                    {project.name}
                  </a>
                )}
              </For>
              <a
                href="/p#createProject"
                onClick={() => setVisible(false)}
                class="block flex items-center w-full rounded p-1.5 pl-1 gap-0.5 justify-center text-md font-bold text-violet-800 hover:bg-indigo-100"
                role="menuitem"
              >
                {/* plus icon */}
                <svg
                  class="w-5 h-5"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                  xmlns="http://www.w3.org/2000/svg"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2.5"
                    d="M12 6v6m0 0v6m0-6h6m-6 0H6"
                  ></path>
                </svg>
                New Project
              </a>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};

export default DropDown;
