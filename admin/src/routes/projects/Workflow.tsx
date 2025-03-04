import { Component, createMemo } from "solid-js";
import { useEngine } from "../../stores/engine.tsx";
import { useParams, useSearchParams } from "@solidjs/router";
import NodeGrid from "../../widgets/node/NodeGrid";
import Paragraph from "../../widgets/typography/Paragraph";
import LinkForm from "../../widgets/nodeForm/LinkForm";
import SearchTermForm from "../../widgets/nodeForm/SearchTermForm";

const labelTypes: string[] = ["Link", "SearchTerm"];
type LabelType = (typeof labelTypes)[number];

const Workflow: Component = () => {
  const [engine] = useEngine();
  const [searchParams] = useSearchParams();
  const params = useParams();

  const getProject = createMemo(() => {
    if (!!params.projectId && params.projectId in engine.projects) {
      return engine.projects[params.projectId];
    }
    return undefined;
  });

  const getSelectNodeIds = createMemo<number[]>(() => {
    if (getProject() && !!searchParams.label) {
      // Only select nodes that have AddedByUser label
      return Object.values(getProject()!.nodes)
        .filter(
          (x) =>
            x.labels.includes("AddedByUser") &&
            (labelTypes.includes(x.payload.type) ||
              x.labels.filter((label) => labelTypes.includes(label))),
        )
        .map((x) => x.id);
    } else {
      return [];
    }
  });

  const getNodeTypeFromSearchParam = createMemo(() => {
    if (!!searchParams.label) {
      return searchParams.label as LabelType;
    }
    return undefined;
  });

  return (
    <>
      <div class="max-w-screen-sm mb-8">
        <Paragraph>
          Pixlie can monitor keywords on multiple URLs. If you add a URL from a
          website, then Pixlie will crawl all URLs on that website.
        </Paragraph>
      </div>

      {/* <Tabs tabs={getTabs()} /> */}
      <NodeGrid
        nodeType={getNodeTypeFromSearchParam()}
        source={getSelectNodeIds}
      />

      {searchParams.label === "Link" && (
        <div class="mt-6 max-w-screen-sm">
          <LinkForm />
        </div>
      )}
      {searchParams.label === "SearchTerm" && (
        <div class="mt-6 max-w-screen-sm">
          <SearchTermForm />
        </div>
      )}
    </>
  );
};

export default Workflow;
