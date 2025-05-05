import { Component, createResource, For, Show } from "solid-js";
import { useParams, useSearchParams } from "@solidjs/router";

import { useEngine } from "../../stores/engine";
import Heading from "../../widgets/typography/Heading.tsx";
import ResultsCount from "../../widgets/generic/ResultsCount.tsx";
import MatchResult from "../../widgets/node/MatchResult.tsx";

const Data: Component = () => {
  const [_, { getMatches }] = useEngine();
  const [searchParams] = useSearchParams();
  const params = useParams();
  const [matches] = createResource(() => params.projectId, getMatches);

  return (
    <>
      {searchParams.label === "WebPage" && (
        <>
          <Heading size={3}>Web Pages</Heading>
          <ResultsCount count={matches()?.length || 0} />
          <Show when={!!matches() && matches()!.length > 0}>
            <div class="columns-1 lg:columns-3 space-y-8 gap-8">
              <For each={matches()!.slice(0, 100)}>
                {(match) => <MatchResult match={match} />}
              </For>
            </div>
          </Show>
        </>
      )}
    </>
  );
};

export default Data;
