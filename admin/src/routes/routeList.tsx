import { createMemo } from "solid-js";
import { useWorkspace } from "../stores/workspace.tsx";
import { useLocation, useParams } from "@solidjs/router";

interface IRoute {
  label: string;
  icon?: string;
  href?: string;
  isActive?: boolean;
}

const getGlobalRoutes = (): Array<IRoute> => {
  return [
    {
      label: "Projects",
      href: "/p",
    },
  ];
};

const getPerProjectRoutes = (): Array<IRoute> => {
  const [workspace] = useWorkspace();
  const params = useParams();
  const location = useLocation();

  const getProject = createMemo(() => {
    if (workspace.isReady && workspace.projects && params.projectId) {
      return workspace.projects.find(
        (project) => project.uuid === params.projectId,
      );
    }
  });

  return [
    {
      label: getProject()!.name,
      isActive: true,
    },
    {
      label: "Workflow",
      href: `/p/${params.projectId}/workflow`,
      isActive: location.pathname.startsWith(`/p/${params.projectId}/workflow`),
    },
    {
      label: "Insights",
      href: `/p/${params.projectId}/insights`,
      isActive: location.pathname.startsWith(`/p/${params.projectId}/insights`),
    },
    {
      label: "Graph",
      href: `/p/${params.projectId}/graph`,
      isActive: location.pathname.startsWith(`/p/${params.projectId}/graph`),
    },
    {
      label: "Crawl",
      href: `/p/${params.projectId}/crawl`,
      isActive: location.pathname.startsWith(`/p/${params.projectId}/crawl`),
    },
  ];
};

export type { IRoute };
export { getGlobalRoutes, getPerProjectRoutes };
