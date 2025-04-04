import { Component, createMemo, For, Show } from "solid-js";
import { useLocation, useParams } from "@solidjs/router";
import SidebarLink from "../widgets/navigation/SidebarLink.tsx";
import { useEngine } from "../stores/engine.tsx";

interface IRoute {
  label: string;
  icon?: string;
  href?: string;
  isActive?: boolean;
  children?: IRoute[];
  isOpen?: boolean;
}

const SidebarItem: Component<{ route: IRoute }> = (props) => {
  return (
    <>
      <Show when={props.route.children?.length}>
        <div class="flex items-center gap-2 w-full">
          {/* <div class=""> */}
          {props.route.isActive ? (
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
              ></path>
            </svg>
          ) : (
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
                d="M9 5l7 7-7 7"
              ></path>
            </svg>
          )}
          {/* </div> */}
          <SidebarLink {...props.route} />
        </div>
      </Show>

      <Show when={!props.route.children?.length}>
        <SidebarLink {...props.route} />
      </Show>

      <Show when={props.route.isActive && props.route.children?.length}>
        {/* <div class="border-l border-gray-150 ml-8 pl-3 my-3"> */}
        <div class="pl-7 text-sm">
          <For each={props.route.children}>
            {(child) => <SidebarItem route={{ ...child }} />}
          </For>
        </div>
        {/* </div> */}
      </Show>
    </>
  );
};

const GlobalRoutes: Component = () => {
  let routes: Array<IRoute> = [
    {
      label: "Projects",
      href: "/p",
    },
    {
      label: "Settings",
      href: "/settings",
    },
    {
      label: "Help",
      href: "/help",
    },
  ];

  return <For each={routes}>{(item) => <SidebarLink {...item} />}</For>;
};

const PerProjectRoutes: Component = () => {
  const [engine] = useEngine();
  const params = useParams();
  const location = useLocation();

  const getProject = createMemo(() => {
    if (!!params.projectId && params.projectId in engine.projects) {
      return engine.projects[params.projectId];
    }
    return undefined;
  });

  const getRoutes = createMemo(() =>
    !!getProject()
      ? [
          {
            label: "Workflow",
            href: `/p/${params.projectId}/workflow`,
            isActive: location.pathname.startsWith(
              `/p/${params.projectId}/workflow`,
            ),
          },
          {
            label: "Search",
            href: `/p/${params.projectId}/search`,
            isActive: location.pathname.startsWith(
              `/p/${params.projectId}/search`,
            ),
          },
          {
            label: "Graph",
            href: `/p/${params.projectId}/graph`,
            isActive: location.pathname.startsWith(
              `/p/${params.projectId}/graph`
            ),
          },
          {
            label: "Data",
            href: `/p/${params.projectId}/data`,
            isActive: location.pathname.startsWith(
              `/p/${params.projectId}/data`,
            ),
            children: ["WebPage"].map((label) => ({
              label: `${label}s`,
              href: `/p/${params.projectId}/data?label=${label}`,
              isActive: location.pathname.startsWith(
                `/p/${params.projectId}/data?label=${label}`,
              ),
            })),
          },
          {
            label: "Crawl",
            href: `/p/${params.projectId}/crawl`,
            isActive: location.pathname.startsWith(
              `/p/${params.projectId}/crawl`,
            ),
            children: ["Domain", "Link"].map((label) => ({
              label: `${label}s`,
              href: `/p/${params.projectId}/crawl?label=${label}`,
              isActive: location.pathname.startsWith(
                `/p/${params.projectId}/crawl?label=${label}`,
              ),
            })),
          },
        ]
      : [],
  );

  return <For each={getRoutes()}>{(item) => <SidebarItem route={item} />}</For>;
};

export type { IRoute };
export { GlobalRoutes, PerProjectRoutes };
