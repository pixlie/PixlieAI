import { Component, createMemo, createSignal, For, Show } from "solid-js";
import { useWorkspace } from "../../stores/workspace";
import { A, useNavigate, useParams } from "@solidjs/router";

const NavbarMenu: Component = () => {
  const [visible, setVisible] = createSignal<boolean>(false);
  const [workspace] = useWorkspace();
  const params = useParams();
  const navigate = useNavigate();

  const getProject = createMemo(() => {
    if (params.projectId && workspace.isReady && workspace.projects) {
      return workspace.projects.find(
        (project) => project.uuid === params.projectId,
      );
    }
    return undefined;
  });

  return (
    <div class="relative w-full">
      <Show
        when={
          workspace.isReady && workspace.settingsStatus?.type === "Complete"
        }
      >
        <button
          type="button"
          onClick={() => setVisible(!visible())}
          class="inline-flex items-center justify-between w-full gap-5 pl-6 pr-5 rounded-md border shadow-sm py-2.5 bg-white hover:bg-gray-50 focus:outline-none focus:ring-offset-gray-100"
          id="options-menu"
          aria-expanded="true"
          aria-haspopup="true"
        >
          <p class="flex-1 truncate text-left text-sm text-gray-800 hover:text-gray-900 font-medium">
            {getProject()?.name || "Projects"}
          </p>
          <svg
            class="w-6 h-6"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M19 9l-7 7-7-7"
              transform={visible() ? "rotate(180 12 12)" : ""}
            ></path>
          </svg>
        </button>
      </Show>

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
            <div class="px-3.5 py-1.5 flex flex-col" role="none">
              <A
                href="/p/create"
                onClick={() => setVisible(false)}
                class="flex items-center rounded p-1.5 pl-2 gap-0.5 text-blue-600 hover:bg-blue-100"
                role="menuitem"
              >
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
                <p class="text-sm font-semibold">New Project</p>
              </A>
              <Show
                when={!!workspace.projects && workspace.projects.length > 0}
              >
                <div class="border-b my-1.5" />
              </Show>
              <For
                each={workspace.projects?.filter(
                  (project) => project.name !== getProject()?.name,
                )}
              >
                {(project) => (
                  <A
                    href={`/p/${project.uuid}/workflow`}
                    onClick={(event) => {
                      event.preventDefault();
                      setVisible(false);
                      navigate(`/p/${project.uuid}/workflow`);
                    }}
                    class="block w-full rounded px-3 py-1.5 hover:bg-gray-100"
                    role="menuitem"
                  >
                    <span class="block truncate text-left text-sm text-gray-800 hover:text-gray-900 font-medium">
                      {project.name}
                    </span>
                  </A>
                )}
              </For>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};

export default NavbarMenu;
