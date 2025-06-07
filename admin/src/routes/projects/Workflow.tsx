import { useParams } from "@solidjs/router";
import { Component, createMemo, For } from "solid-js";
import { APINodeItem } from "../../api_types/APINodeItem.ts";
import { CrawlerSettings } from "../../api_types/CrawlerSettings.ts";
import { EntityName } from "../../api_types/EntityName.ts";
import { NodeLabel } from "../../api_types/NodeLabel.ts";
import { NodeWrite } from "../../api_types/NodeWrite.ts";
import { useEngine } from "../../stores/engine.tsx";
import { createNode } from "../../utils/api.ts";
import ResultsCount from "../../widgets/generic/ResultsCount.tsx";
import NodeGrid from "../../widgets/node/NodeGrid";
import LinkForm from "../../widgets/nodeForm/LinkForm";
import Heading from "../../widgets/typography/Heading.tsx";
import Paragraph from "../../widgets/typography/Paragraph";
import { Conclusion } from "../../api_types/Conclusion.ts";

const Workflow: Component = () => {
  const [_, { getNodes }] = useEngine();
  const params = useParams();

  const getObjectives = createMemo<Array<APINodeItem> | undefined>(() => {
    return getNodes(params.projectId, (node) =>
      node.labels.includes("Objective")
    );
  });

  const getStartingLinkIds = createMemo<number[]>(() => {
    return getNodes(
      params.projectId,
      (node) =>
        node.labels.includes("Link" as NodeLabel) &&
        (node.labels.includes("AddedByUser" as NodeLabel) ||
          node.labels.includes("AddedByAI" as NodeLabel) ||
          node.labels.includes("AddedByWebSearch" as NodeLabel))
    )
      .sort((a, b) => a.id - b.id)
      .slice(0, 100)
      .map((x) => x.id);
  });

  const getCrawlerSettings = createMemo<CrawlerSettings | undefined>(() => {
    return getNodes(
      params.projectId,
      (node) => node.payload.type === "CrawlerSettings"
    )[0]?.payload.data as CrawlerSettings | undefined;
  });

  const getNamedEntitiesToExtract = createMemo<Array<EntityName> | undefined>(
    () => {
      return getNodes(
        params.projectId,
        (node) => node.payload.type === "NamedEntitiesToExtract"
      )[0]?.payload.data as Array<EntityName> | undefined;
    }
  );

  const getConclusion = createMemo<Conclusion | undefined>(() => {
    return getNodes(
      params.projectId,
      (node) => node.payload.type === "Conclusion"
    )[0]?.payload.data as Conclusion | undefined;
  });

  const addLink = (_name: string, value: string) => {
    createNode(params.projectId, {
      Link: {
        url: value,
      },
    } as NodeWrite).then((_) => {
      // TODO: Handle error in creating node
    });
  };

  return (
    <div class="flex gap-x-12">
      <div class="flex-1 flex flex-col gap-y-4">
        {!!params.projectId ? (
          <div>
            <Heading size={3}>Objective</Heading>
            <For each={getObjectives()}>
              {(node) => <Paragraph>{node.payload.data as string}</Paragraph>}
            </For>
          </div>
        ) : (
          <div class="max-w-screen-sm">
            <Heading size={3}>Objective</Heading>
            {/* <ProjectForm /> */}
          </div>
        )}

        {/* <div>
          <Heading size={3}>Project Settings</Heading>
          <NodeGrid
            nodeType="ProjectSettings"
            source={() => []}
            mode="regular"
          />
        </div> */}

        <div>
          <Heading size={3}>Conclusion</Heading>
          <p>
            {getConclusion()?.conclusion_to_objective_based_on_insights ||
              "No conclusion yet."}
          </p>
        </div>

        <div>
          <Heading size={3}>Keywords</Heading>
          <p>
            {getCrawlerSettings()?.keywords_to_get_accurate_results_from_web_search?.join(
              ", "
            )}
          </p>
        </div>
        <div>
          <Heading size={3}>Entities</Heading>
          <p>{getNamedEntitiesToExtract()?.join(", ")}</p>
        </div>
        <div class="flex flex-col gap-2 pb-2">
          <Heading size={3}>Starting URLs</Heading>
          <div class="max-w-screen-sm">
            <LinkForm name="url" onChange={addLink} />
          </div>
        </div>
        <ResultsCount count={getStartingLinkIds()?.length} />
        {!!getStartingLinkIds() && getStartingLinkIds()?.length > 0 && (
          <NodeGrid
            nodeType={"Link"}
            source={getStartingLinkIds}
            mode="regular"
          />
        )}
      </div>

      <WorkflowSidebar />
    </div>
  );
};

export const WorkflowSidebar: Component = () => {
  return (
    <div class="w-96">
      <div class="flex flex-col gap-y-6 bg-stone-100 rounded-md drop-shadow p-6">
        <Paragraph size={"sm"}>
          Pixlie uses your objective to ask AI for starting URLs and keywords to
          monitor on websites. Pixlie will continue crawling the web as long as
          pages match your objective.
        </Paragraph>

        <Paragraph size={"sm"}>
          Pixlie can extract information that is relevant to your objective but
          this feature is still in beta. Pixlie can extract blog posts, job
          posts, people, companies, events, dates and locations.
        </Paragraph>
      </div>
    </div>
  );
};

export default Workflow;
