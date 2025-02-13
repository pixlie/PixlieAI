import { createMemo } from "solid-js";
import { useWorkspace } from "../stores/workspace.tsx";
import { useParams } from "@solidjs/router";

interface Route {
  label: string;
  icon: string;
  href: string;
}

const getGlobalRoutes = () => {
  return [
    {
      label: "Projects",
      href: "/p",
    },
  ];
};

const getPerProjectRoutes = () => {
  const [workspace] = useWorkspace();
  const params = useParams();

  const getProject = createMemo(() => {
    if (workspace.isReady && workspace.projects && params.projectId) {
      return workspace.projects.find(
        (project) => project.uuid === params.projectId,
      );
    }
  });

  let routes = [
    {
      label: "Insights",
      href: "/insights",
    },
    {
      label: "Graph",
      href: "/graph",
    },
    {
      label: "Crawl",
      href: "/crawl",
    },
  ];

  if (getProject()) {
    routes = [
      {
        label: getProject()!.name,
        href: `/workflow`,
      },
      ...routes,
    ];
  }

  return routes;
};

export type { Route };
export { getGlobalRoutes, getPerProjectRoutes };
