import { Component, createMemo } from "solid-js";
import { useEngine } from "../../stores/engine";
import NodeGrid from "../../widgets/node/NodeGrid.tsx";
import { useParams, useSearchParams } from "@solidjs/router";
// import Tabs from "../../widgets/navigation/Tab";

const labelTypes: string[] = [
  "Title",
  "Paragraph",
  "Heading",
  "BulletPoints",
  "OrderedPoints",
  "SearchResults",
];
type LabelType = (typeof labelTypes)[number];

const Graph: Component = () => {
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
        .filter((x) => labelTypes.includes(x.payload.type))
        .map((x) => x.id);
    } else {
      return [];
    }
  });

  // const getTabs = createMemo(() =>
  //   labelTypes.map((l) => ({
  //     label: `${l}(s)`,
  //     searchParamKey: "label",
  //     searchParamValue: l,
  //   })),
  // );

  const getNodeTypeFromSearchParam = createMemo(() => {
    if (!!searchParams.label) {
      return searchParams.label as LabelType;
    }
    return undefined;
  });

  return (
    <>
      {/* <Tabs tabs={getTabs()} /> */}
      <NodeGrid
        nodeType={getNodeTypeFromSearchParam()}
        source={getSelectNodeIds}
      />
    </>
  );
};

export default Graph;
