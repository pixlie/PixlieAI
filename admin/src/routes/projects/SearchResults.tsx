import { Component, createMemo } from "solid-js";
import { useEngine } from "../../stores/engine";
import Heading from "../../widgets/typography/Heading";
import { useParams } from "@solidjs/router";
import SearchResult from "../../widgets/node/SearchResult.tsx";

const SearchResults: Component = () => {
  const [engine] = useEngine();
  const params = useParams();

  const getProject = createMemo(() => {
    if (!!params.projectId && params.projectId in engine.projects) {
      return engine.projects[params.projectId];
    }
    return undefined;
  });

  // We have to display each `SearchTerm` and then the results per `SearchTerm`
  // Each result node can have different payload types, like Title, Heading, Paragraph, etc.
  return (
    <>
      <Heading size={3}>Search results</Heading>

      <div class="flex flex-col space-y-4">
        {!!getProject()
          ? Object.values(getProject()!.nodes).map((node) => {
              if (node.payload.type === "SearchTerm") {
                return <SearchResult nodeId={node.id} />;
              }
            })
          : null}
      </div>
    </>
  );
};

export default SearchResults;
