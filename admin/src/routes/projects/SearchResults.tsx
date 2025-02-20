import { Component, createMemo, onMount } from "solid-js";
import { useEngine } from "../../stores/engine";
import Heading from "../../widgets/typography/Heading";
// import NodeGrid from "../../widgets/node/NodeGrid.tsx";
import { useParams } from "@solidjs/router";

const SearchResults: Component = () => {
  const [engine, { fetchNodesByLabel, getQueryResults }] = useEngine();
  const params = useParams();

  const getProject = createMemo(() => {
    if (!!params.projectId && params.projectId in engine.projects) {
      return engine.projects[params.projectId];
    }
    return undefined;
  });

  onMount(() => {
    // First, we fetch all the nodes with the label `SearchTerm`
    // Then, we fetch query results for each of the nodes with label `SearchTerm`
  });

  // We have to display each `SearchTerm` and then the results per `SearchTerm`
  // Each result node can have different payload types, like Title, Heading, Paragraph, etc.
  return (
    <>
      <Heading size={3}>Search results</Heading>

      {/*<NodeGrid />*/}
    </>
  );
};

export default SearchResults;
