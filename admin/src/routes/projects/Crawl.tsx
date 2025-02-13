import { Component, createEffect, createMemo } from "solid-js";
import { useEngine } from "../../stores/engine";
import Heading from "../../widgets/typography/Heading";
import NodeListItem from "../../widgets/node/ListItem";
import Tabs from "../../widgets/navigation/Tab";
import { useParams, useSearchParams } from "@solidjs/router";

const labelTypes: string[] = ["Domain", "Link"];
type LabelType = (typeof labelTypes)[number];

const Crawl: Component = () => {
  const [engine, { fetchNodesByLabel }] = useEngine();
  const [searchParams] = useSearchParams();
  const params = useParams();

  const getSelectNodeIds = createMemo<number[]>(() =>
    !!searchParams.label &&
    (searchParams.label as LabelType) in engine.nodeIdsByLabel
      ? engine.nodeIdsByLabel[searchParams.label as LabelType]
      : [],
  );

  const getTabs = createMemo(() =>
    labelTypes.map((l) => ({
      label: `${l}(s)`,
      searchParamKey: "label",
      searchParamValue: l,
    })),
  );

  createEffect(() => {
    if (!!searchParams.label) {
      fetchNodesByLabel(params.projectId, searchParams.label as LabelType);
    }
  });

  return (
    <>
      <Heading size={3}>Crawl</Heading>

      <Tabs tabs={getTabs()} />
      {!engine.isReady ? (
        <>Loading...</>
      ) : (
        <>
          {getSelectNodeIds().map((nodeId) => (
            <NodeListItem nodeId={nodeId} />
          ))}
        </>
      )}
    </>
  );
};

export default Crawl;
