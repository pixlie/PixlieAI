import { Component, createMemo, onMount } from "solid-js";
import { useEngine } from "../../stores/engine";
import NodeGrid from "../../widgets/node/NodeGrid";
import { useParams, useSearchParams } from "@solidjs/router";

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
    <NodeGrid
      nodeType={getNodeTypeFromSearchParam()}
      source={getSelectNodeIds}
    />
  );
};

export default Crawl;
