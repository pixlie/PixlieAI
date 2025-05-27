import { Component, createMemo, createSignal } from "solid-js";
import { useParams, useSearchParams } from "@solidjs/router";

import { Classification } from "../../api_types/Classification.ts";
import { useEngine } from "../../stores/engine";
import Heading from "../../widgets/typography/Heading.tsx";
import ResultsCount from "../../widgets/generic/ResultsCount.tsx";
import NodeGrid from "../../widgets/node/NodeGrid.tsx";

const Results: Component = () => {
  const [engine, { getRelatedNodes }] = useEngine();
  const [searchParams] = useSearchParams();
  const params = useParams();

  const [onlyShowMatches, setOnlyShowMatches] = createSignal<boolean>(false);

  const getProject = createMemo(() => {
    if (!!params.projectId && params.projectId in engine.projects) {
      return engine.projects[params.projectId];
    }
    return undefined;
  });

  const getProjectNodes = createMemo(() => {
    const project = getProject();
    if (!!project) {
      return project!.nodes;
    }
    return undefined;
  });

  const getMatchCriteria = createMemo(() => {
    const projectNodes = getProjectNodes();
    if (!!projectNodes) {
      return Object.values(projectNodes)
        .filter((x) => x.payload.type === "ClassifierSettings")
        .map((x) => x)[0]?.payload.data
        ?.prompt_to_classify_content_as_relevant_to_objective_or_not;
    }
    return undefined;
  });

  const getRelevantNodeIds = createMemo<Array<number>>(() => {
    const projectNodes = getProjectNodes();
    if (!!projectNodes) {
      return Object.values(projectNodes)
        ?.filter((x) => x.labels.includes("WebPage"))
        ?.filter((x) => {
          const classification = getRelatedNodes(
            params.projectId,
            x.id,
            "Classifies",
            (n) => n.labels.includes("Classification")
          )[0]?.payload.data as Classification | null;
          return (
            classification && (!onlyShowMatches() || classification.is_relevant)
          );
        })
        ?.map((x) => x.id);
    }
    return [];
  });

  return (
    <div class="relative flex-1">
      <div class="absolute inset-0 flex flex-col gap-4">
        {searchParams.label === "WebPage" && (
          <Heading size={3}>Web Pages</Heading>
        )}
        {searchParams.label === "URL" && <Heading size={3}>URLs</Heading>}

        {!!getMatchCriteria() && (
          <div class="border-l-2 border-green-600 pl-4 flex flex-col gap-1 mt-1 mb-2">
            <p class="font-medium text-green-600">Match Criteria</p>
            <p class="text-slate-700">
              {getMatchCriteria().replace("webpage", "content")}
            </p>
          </div>
        )}

        <div class="flex items-center justify-between">
          <ResultsCount count={getRelevantNodeIds()?.length} />
          <div class="flex items-center gap-3 mb-2">
            <span class="text-md font-medium text-slate-400">
              Only Show Matches:{" "}
            </span>
            <div
              class={
                "relative inline-flex h-6 w-10 items-center rounded-full cursor-pointer " +
                (!onlyShowMatches() ? "bg-slate-200" : "bg-green-600")
              }
              onClick={() => setOnlyShowMatches(!onlyShowMatches())}
            >
              <span
                class="absolute left-1 inline-block h-4 w-4 transform rounded-full bg-white shadow-md transition-transform"
                classList={{
                  "translate-x-0": !onlyShowMatches(),
                  "translate-x-4": onlyShowMatches(),
                }}
              />
            </div>
          </div>
        </div>

        {!!getRelevantNodeIds() && getRelevantNodeIds()!.length > 0 ? (
          <NodeGrid
            nodeType={searchParams.label as string}
            source={getRelevantNodeIds}
            mode="preview"
          />
        ) : (
          <div class="h-full w-full flex justify-center items-center">
            <p class="text-md text-slate-400 text-center">
              No results found yet!
            </p>
          </div>
        )}
      </div>
    </div>
  );
};

export default Results;
