import { Component, onMount } from "solid-js";
import { useEngine } from "../../stores/engine";
import Heading from "../../widgets/typography/Heading";
import NodeListItem from "../../widgets/node/ListItem";

const Crawl: Component = () => {
  const [engine, { fetchNodesByLabel }] = useEngine();

  onMount(() => {
    fetchNodesByLabel("Link");
  });

  return (
    <>
      <Heading size={1}>Crawl</Heading>

      {!engine.isReady ? (
        <>Loading...</>
      ) : (
        <>
          {"Link" in engine.nodeIdsByLabel &&
            engine.nodeIdsByLabel["Link"].map((nodeId) => (
              <NodeListItem nodeId={nodeId} />
            ))}
        </>
      )}
    </>
  );
};

export default Crawl;
