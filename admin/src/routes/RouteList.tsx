import { Component, createMemo, For } from "solid-js";
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
  isChild?: boolean;
}

const SidebarItem: Component<{ route: IRoute }> = (props) => {
  return (
    <>
      <SidebarLink {...props.route} />
      <For each={props.route.children}>
        {(child) => <SidebarItem route={{ ...child, isChild: true }} />}
      </For>
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
            children: ["WebPage"].map((label) => ({
              label: `${label}s`.replace(/([a-z])([A-Z])/g, "$1 $2"),
              href: `/p/${params.projectId}/data?label=${label}`,
              isActive:
                location.pathname.startsWith(`/p/${params.projectId}/data`) &&
                location.search.includes(`label=${label}`),
            })),
          },
          {
            label: "Crawl",
            children: ["Domain", "Link"].map((label) => ({
              label: `${label}s`,
              href: `/p/${params.projectId}/crawl?label=${label}`,
              isActive:
                location.pathname.startsWith(`/p/${params.projectId}/crawl`) &&
                location.search.includes(`label=${label}`),
            })),
          },
        ]
      : [],
  );

  return <For each={getRoutes()}>{(item) => <SidebarItem route={item} />}</For>;
};

export type { IRoute };
export { GlobalRoutes, PerProjectRoutes };
