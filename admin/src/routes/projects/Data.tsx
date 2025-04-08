import { Component, createMemo } from "solid-js";
import { useEngine } from "../../stores/engine";
import NodeGrid from "../../widgets/node/NodeGrid";
import { useParams, useSearchParams } from "@solidjs/router";
import Heading from "../../widgets/typography/Heading.tsx";
import { NodeLabel } from "../../api_types/NodeLabel.ts";
import ResultsCount from "../../widgets/generic/ResultsCount.tsx";

const labelTypes: string[] = ["SearchResults"];
type LabelType = (typeof labelTypes)[number];

const Data: Component = () => {
  const [_, { getNodes }] = useEngine();
  const [searchParams] = useSearchParams();
  const params = useParams();
  const getSelectNodeIds = createMemo<number[]>(() => {
    // Later, we can fetch by label Content instead of WebPage
    // and do conditional rendering based on the label type(WebPage, PDFFile, etc)
    return getNodes(params.projectId, (node) =>
      node.labels.includes(searchParams.label as NodeLabel),
    ).map((node) => node.id);
  });

  const getNodeTypeFromSearchParam = createMemo(() => {
    if (!!searchParams.label) {
      return searchParams.label as LabelType;
    }
    return undefined;
  });

  return (
    <>
      {searchParams.label === "WebPage" && (
        <>
          <Heading size={3}>Web Pages</Heading>
          <ResultsCount count={getSelectNodeIds().length} />
        </>
      )}
      <NodeGrid
        nodeType={getNodeTypeFromSearchParam()}
        source={getSelectNodeIds}
        mode="preview"
      />
    </>
  );
};

export default Data;
