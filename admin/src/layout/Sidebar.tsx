import { Component, For } from "solid-js";
import SidebarLink from "../widgets/navigation/SidebarLink";
import { routes } from "../routes/routeList";
import { useTailwindClasses } from "../stores/TailwindClasses";

const Sidebar: Component = () => {
  const [_, { getClasses }] = useTailwindClasses();

  return (
    <div
      class={
        "fixed inset-y-0 z-50 w-72 flex flex-col " + getClasses()["sideBar"]
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
                <For each={routes}>{(item) => <SidebarLink {...item} />}</For>
              </ul>
            </li>
          </ul>
        </nav>
      </div>
    </div>
  );
};

export default Sidebar;
