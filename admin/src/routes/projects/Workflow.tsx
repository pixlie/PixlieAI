import { Component, createEffect, createMemo, onMount } from "solid-js";
import Heading from "../../widgets/typography/Heading";
import Tabs from "../../widgets/navigation/Tab";
import { useEngine } from "../../stores/engine";
import { useParams, useSearchParams } from "@solidjs/router";
import NodeGrid from "../../widgets/node/NodeGrid";
import Paragraph from "../../widgets/typography/Paragraph";
import LinkForm from "../../widgets/nodeForm/LinkForm";
import SearchTermForm from "../../widgets/nodeForm/SearchTermForm";

const labelTypes: string[] = ["Link", "SearchTerm"];
type LabelType = (typeof labelTypes)[number];

const Workflow: Component = () => {
  const [engine, { fetchNodesByLabel }] = useEngine();
  const [searchParams] = useSearchParams();
  const params = useParams();

  const getProject = createMemo(() => {
    if (!!params.projectId && params.projectId in engine.projects) {
      return engine.projects[params.projectId];
    }
    return undefined;
  });

  const getSelectNodeIds = createMemo<number[]>(() => {
    if (
      getProject() &&
      !!searchParams.label &&
      (searchParams.label as LabelType) in getProject()!.nodeIdsByLabel
    ) {
      // Only select nodes that have AddedByUser label
      return getProject()!.nodeIdsByLabel[
        searchParams.label as LabelType
      ].filter((nodeId) =>
        getProject()!.nodes[nodeId].labels.includes("AddedByUser"),
      );
    } else {
      return [];
    }
  });

  onMount(() => {
    if (params.projectId) {
      fetchNodesByLabel(params.projectId, "AddedByUser");
    }
  });

  const getTabs = createMemo(() =>
    labelTypes.map((l) => ({
      label: `${l}(s)`,
      searchParamKey: "label",
      searchParamValue: l,
    })),
  );

  createEffect(() => {
    if (params.projectId && !!searchParams.label) {
      fetchNodesByLabel(params.projectId, searchParams.label as LabelType);
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
      <Heading size={3}>Workflow</Heading>
      <div class="max-w-screen-sm mb-8">
        <Paragraph>
          Pixlie can monitor keywords on multiple URLs. If you add a URL from a
          website, then Pixlie will crawl all URLs on that website.
        </Paragraph>
      </div>

      <Tabs tabs={getTabs()} />
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
