import { Component, createEffect, createMemo, onMount } from "solid-js";
// import Tabs from "../../widgets/navigation/Tab";
import { useEngine } from "../../stores/engine.tsx";
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

  // type NodesInWorkflow = "Link";
  // Nodes that have the label "AddedByUser" are the nodes that are in the workflow
  // const getNodesInWorkflow = createMemo(
  //   (prev: Array<NodesInWorkflow>): Array<NodesInWorkflow> => {
  //     if ("AddedByUser" in engine.nodeIdsByLabel) {
  //       return engine.nodeIdsByLabel["AddedByUser"]
  //         .map((x) => {
  //           if (engine.nodes[x].payload.type === "Link") {
  //             return "Link";
  //           }
  //         })
  //         .filter((x) => x !== undefined) as Array<NodesInWorkflow>;
  //     }
  //     return prev;
  //   },
  //   [],
  // );

  // const getTabs = createMemo(() =>
  //   getNodesInWorkflow().map((l) => ({
  //     label: `${l}(s)`,
  //     searchParamKey: "label",
  //     searchParamValue: l,
  //   })),
  // );

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
