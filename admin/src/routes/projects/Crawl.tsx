import { Component, createMemo, createSignal } from "solid-js";
import { useEngine } from "../../stores/engine";
import NodeGrid from "../../widgets/node/NodeGrid";
import { useParams, useSearchParams } from "@solidjs/router";
import Heading from "../../widgets/typography/Heading";
import { NodeLabel } from "../../api_types/NodeLabel";
import ResultsCount from "../../widgets/generic/ResultsCount";

const labelTypes: string[] = ["Domain", "Link"];
type LabelType = (typeof labelTypes)[number];

const Crawl: Component = () => {
  const [engine] = useEngine();
  const [searchParams] = useSearchParams();
  const params = useParams();
  const [linkCount, setLinkCount] = createSignal(0);
  const [domainCount, setDomainCount] = createSignal(0);

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
    <div class="relative flex-1">
      <div class="absolute inset-0 flex flex-col gap-4">
        {searchParams.label === "Link" && (
          <>
            <Heading size={3}>Links</Heading>
            <ResultsCount count={linkCount()} />
          </>
        )}
        {searchParams.label === "Domain" && (
          <>
            <Heading size={3}>Domains</Heading>
            <ResultsCount count={domainCount()} />
          </>
        )}
        <NodeGrid
          nodeType={getNodeTypeFromSearchParam()}
          source={getSelectNodeIds}
          mode="regular"
        />
      </div>
    </div>
  );
};

export default Crawl;
