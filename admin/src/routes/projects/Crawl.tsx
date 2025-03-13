import { Component, createMemo, createSignal, onMount } from "solid-js";
import { useEngine } from "../../stores/engine";
import NodeGrid from "../../widgets/node/NodeGrid";
import { useParams, useSearchParams } from "@solidjs/router";
import Heading from "../../widgets/typography/Heading";
import Paragraph from "../../widgets/typography/Paragraph";
import { NodeLabel } from "../../api_types/NodeLabel";

const labelTypes: string[] = ["Domain", "Link"];
type LabelType = (typeof labelTypes)[number];

const Crawl: Component = () => {
  const [engine, { fetchNodes, fetchEdges }] = useEngine();
  const [searchParams] = useSearchParams();
  const params = useParams();
  const [linkCount, setLinkCount] = createSignal(0);
  const [domainCount, setDomainCount] = createSignal(0);

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
    let items: number[] = [];

    if (getProject() && !!searchParams.label) {
      items = Object.values(getProject()!.nodes)
        .filter((x) => x.labels.includes(searchParams.label as NodeLabel))
        .map((x) => x.id);
    }
    if (searchParams.label === "Link") {
      setLinkCount(items.length);
    }
    if (searchParams.label === "Domain") {
      setDomainCount(items.length);
    }
    return items;
  });

  const getNodeTypeFromSearchParam = createMemo(() => {
    if (!!searchParams.label) {
      return searchParams.label as LabelType;
    }
    return undefined;
  });

  return (
    <>
      {searchParams.label === "Link" && (
        <Heading size={3}>Links found: {linkCount()}</Heading>
      )}
      {searchParams.label === "Domain" && (
        <Heading size={3}>Domains found: {domainCount()}</Heading>
      )}
      <Paragraph>
        {searchParams.label ? searchParams.label + "s" : "Domains or links"}{" "}
        found while crawling.
      </Paragraph>
      <NodeGrid
        nodeType={getNodeTypeFromSearchParam()}
        source={getSelectNodeIds}
      />
    </>
  );
};

export default Crawl;
