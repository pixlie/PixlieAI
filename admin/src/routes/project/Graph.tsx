import { Component, onMount } from "solid-js";
import Screen from "../../layout/Screen";
import { useEngine } from "../../stores/Engine";

const Graph: Component = () => {
  const [_engine, { fetchNodesByLabel }] = useEngine();

  onMount(() => {
    fetchNodesByLabel();
  });

  return <Screen title="Graph" />;
};

export default Graph;
