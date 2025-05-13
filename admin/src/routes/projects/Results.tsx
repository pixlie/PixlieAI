import { Component, createMemo } from "solid-js";
import { useEngine } from "../../stores/engine";
import { useParams, useSearchParams } from "@solidjs/router";
import Heading from "../../widgets/typography/Heading.tsx";
import ResultsCount from "../../widgets/generic/ResultsCount.tsx";

import NodeGrid from "../../widgets/node/NodeGrid.tsx";

const Results: Component = () => {
  const [engine, { getRelatedNodes }] = useEngine();
  const [searchParams] = useSearchParams();
  const params = useParams();

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
        ?.query_to_classify_content_as_relevant_or_irrelevant_to_objective;
    }
    return undefined;
  });

  const getRelevantNodeIds = createMemo<Array<number>>(() => {
    const projectNodes = getProjectNodes();
    if (!!projectNodes) {
      return Object.values(projectNodes)
        ?.filter((x) => x.labels.includes("Insight"))
        ?.map(
          (x) =>
            getRelatedNodes(params.projectId, x.id, "MatchedFor", (n) =>
              n.labels.includes("WebPage")
            )[0]?.id
        );
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
          <div class="border-l-2 border-green-500 pl-4 flex flex-col gap-1.5 mb-2">
            <p class="font-medium text-green-600">Match Criteria</p>
            <p class="text-slate-700">{getMatchCriteria()}</p>
          </div>
        )}

        <ResultsCount count={getRelevantNodeIds()?.length} />

        {!!getRelevantNodeIds() && getRelevantNodeIds()!.length > 0 ? (
          <NodeGrid
            nodeType={searchParams.label as string}
            source={getRelevantNodeIds}
            mode="preview"
          />
        ) : (
          <div class="h-full w-full flex justify-center items-center">
            <p class="text-md text-slate-400 text-center">
              No matches found yet!
            </p>
          </div>
        )}
      </div>
    </div>
  );
};

export default Results;
