import { Component, createMemo, Show } from "solid-js";
import { useLocation, useParams, useSearchParams } from "@solidjs/router";

import { useWorkspace } from "../../stores/workspace";
import BreadcrumbLabel from "./BreadcrumbLabel";

const labelTypes: string[] = ["Domains, Links, WebPages, URLs, SearchResults"];
type LabelType = (typeof labelTypes)[number];

const Breadcrumb: Component = () => {
  const location = useLocation();
  const [searchParams] = useSearchParams();
  const params = useParams();
  const [workspace] = useWorkspace();

  const getNodeTypeFromSearchParam = createMemo(() => {
    if (!!searchParams.label) {
      return `${searchParams.label}s` as LabelType;
    }
    return undefined;
  });

  const getProject = createMemo(() => {
    if (params.projectId && workspace.isReady && workspace.projects) {
      return workspace.projects.find(
        (project) => project.uuid === params.projectId
      );
    }
    return undefined;
  });

  return (
    <div class=" flex items-center gap-2 h-4">
      <Show
        when={
          workspace.isReady &&
          workspace.settingsStatus?.type === "Complete" &&
          params.projectId
        }
      >
        <BreadcrumbLabel label="Projects" />
        <BreadcrumbLabel label={getProject()?.name || "Project"} />
        <BreadcrumbLabel
          label={location.pathname
            .split("/")
            .slice(-1)[0]
            .replace(/-/g, " ")
            .replace(/\b\w/g, (char) => char.toUpperCase())}
          isLast={!getNodeTypeFromSearchParam()}
        />
        <Show when={!!getNodeTypeFromSearchParam()}>
          <BreadcrumbLabel
            label={getNodeTypeFromSearchParam()?.replace(
              /([a-z])([A-Z])/g,
              "$1 $2"
            )}
            isLast
          />
        </Show>
      </Show>
    </div>
  );
};

export default Breadcrumb;
