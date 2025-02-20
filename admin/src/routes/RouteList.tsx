import { Component, createMemo, For } from "solid-js";
import { useWorkspace } from "../stores/workspace.tsx";
import { useLocation, useParams } from "@solidjs/router";
import SidebarLink from "../widgets/navigation/SidebarLink.tsx";

interface IRoute {
  label: string;
  icon?: string;
  href?: string;
  isActive?: boolean;
}

const GlobalRoutes: Component = () => {
  let routes: Array<IRoute> = [
    {
      label: "Projects",
      href: "/p",
    },
  ];

  return <For each={routes}>{(item) => <SidebarLink {...item} />}</For>;
};

const PerProjectRoutes: Component = () => {
  const [workspace] = useWorkspace();
  const params = useParams();
  const location = useLocation();

  const getProject = createMemo(() => {
    if (params.projectId && workspace.isReady && workspace.projects) {
      return workspace.projects.find(
        (project) => project.uuid === params.projectId,
      );
    }
  });

  const getRoutes = createMemo(() =>
    params.projectId && !!getProject()
      ? [
          {
            label: getProject()!.name,
            isActive: true,
          },
          {
            label: "Workflow",
            href: `/p/${params.projectId}/workflow`,
            isActive: location.pathname.startsWith(
              `/p/${params.projectId}/workflow`,
            ),
          },
          // {
          //   label: "Insights",
          //   href: `/p/${params.projectId}/insights`,
          //   isActive: location.pathname.startsWith(`/p/${params.projectId}/insights`),
          // },
          {
            label: "Graph",
            href: `/p/${params.projectId}/graph`,
            isActive: location.pathname.startsWith(
              `/p/${params.projectId}/graph`,
            ),
          },
          {
            label: "Query",
            href: `/p/${params.projectId}/searchResults`,
            isActive: location.pathname.startsWith(
              `/p/${params.projectId}/searchResults`,
            ),
          },
          {
            label: "Crawl",
            href: `/p/${params.projectId}/crawl`,
            isActive: location.pathname.startsWith(
              `/p/${params.projectId}/crawl`,
            ),
          },
        ]
      : [],
  );

  return <For each={getRoutes()}>{(item) => <SidebarLink {...item} />}</For>;
};

export type { IRoute };
export { GlobalRoutes, PerProjectRoutes };
