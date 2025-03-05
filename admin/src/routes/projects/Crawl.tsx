import { Component, createMemo, onMount } from "solid-js";
import { useEngine } from "../../stores/engine";
import NodeGrid from "../../widgets/node/NodeGrid";
import { useParams, useSearchParams } from "@solidjs/router";
import Heading from "../../widgets/typography/Heading.tsx";
import Paragraph from "../../widgets/typography/Paragraph.tsx";

const labelTypes: string[] = ["Domain", "Link"];
type LabelType = (typeof labelTypes)[number];

const Crawl: Component = () => {
  const [engine, { fetchNodes, fetchEdges }] = useEngine();
  const [searchParams] = useSearchParams();
  const params = useParams();

  onMount(() => {
    fetchNodes(params.projectId);
    fetchEdges(params.projectId);
  });

  const getProject = createMemo(() => {
    if (!!params.projectId && params.projectId in engine.projects) {
      return engine.projects[params.projectId];
    }
    return undefined;
  });

  const getSelectNodeIds = createMemo<number[]>(() => {
    if (getProject() && !!searchParams.label) {
      return Object.values(getProject()!.nodes)
        .filter((x) => x.payload.type === searchParams.label)
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
      {searchParams.label === "Link" && <Heading size={3}>Links found</Heading>}
      {searchParams.label === "Domain" && (
        <Heading size={3}>Domains found</Heading>
      )}
      <Paragraph>Domains or links found while crawling.</Paragraph>
      <NodeGrid
        nodeType={getNodeTypeFromSearchParam()}
        source={getSelectNodeIds}
      />
    </>
  );
};

export default Crawl;
