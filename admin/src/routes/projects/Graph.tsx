import { Component, createMemo } from "solid-js";
import { useEngine } from "../../stores/engine";
import NodeGrid from "../../widgets/node/NodeGrid";
import { useParams, useSearchParams } from "@solidjs/router";
import Heading from "../../widgets/typography/Heading.tsx";
import { NodeLabel } from "../../api_types/NodeLabel.ts";

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
      if (typeof searchParams.label === "string") {
        return Object.values(getProject()!.nodes)
          .filter((x) => x.labels.includes(searchParams.label as NodeLabel))
          .map((x) => x.id);
      } else if (Array.isArray(searchParams.label)) {
        return Object.values(getProject()!.nodes)
          .filter((x) =>
            x.labels.filter((label) =>
              (searchParams.label as string[]).includes(label),
            ),
          )
          .map((x) => x.id);
      }
      return [];
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
      {searchParams.label === "Title" && <Heading size={3}>Titles</Heading>}
      {searchParams.label === "Paragraph" && (
        <Heading size={3}>Paragraphs</Heading>
      )}
      {searchParams.label === "Heading" && <Heading size={3}>Headings</Heading>}
      {searchParams.label === "BulletPoint" && (
        <Heading size={3}>Bullet points</Heading>
      )}
      {searchParams.label === "OrderedPoint" && (
        <Heading size={3}>Ordered points</Heading>
      )}
      <NodeGrid
        nodeType={getNodeTypeFromSearchParam()}
        source={getSelectNodeIds}
      />
    </>
  );
};

export default Graph;
