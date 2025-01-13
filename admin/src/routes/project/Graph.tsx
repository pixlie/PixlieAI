import { Component, onMount } from "solid-js";
import { useEngine } from "../../stores/engine";
import Heading from "../../widgets/typography/Heading";
import NodeListItem from "../../widgets/node/ListItem";

const Graph: Component = () => {
  const [engine, { fetchNodesByLabel }] = useEngine();

  onMount(() => {
    fetchNodesByLabel("Link");
  });

  return (
    <>
      <Heading size={1}>Graph</Heading>

      {!engine.isReady ? (
        <>Loading...</>
      ) : (
        <>
          {Object.keys(engine.nodes).map((nodeId) => (
            <NodeListItem nodeId={nodeId} />
          ))}
        </>
      )}
    </>
  );
};

export default Graph;
