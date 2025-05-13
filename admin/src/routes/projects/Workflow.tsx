import { Component, createMemo, For } from "solid-js";
import { useEngine } from "../../stores/engine.tsx";
import { useParams } from "@solidjs/router";
import NodeGrid from "../../widgets/node/NodeGrid";
import Paragraph from "../../widgets/typography/Paragraph";
import LinkForm from "../../widgets/nodeForm/LinkForm";
import Heading from "../../widgets/typography/Heading.tsx";
import { NodeLabel } from "../../api_types/NodeLabel.ts";
import { APINodeItem } from "../../api_types/APINodeItem.ts";
import { createNode } from "../../utils/api.ts";
import { NodeWrite } from "../../api_types/NodeWrite.ts";
import ResultsCount from "../../widgets/generic/ResultsCount.tsx";

const Workflow: Component = () => {
  const [engine] = useEngine();
  const params = useParams();

  const getProject = createMemo(() => {
    if (!!params.projectId && params.projectId in engine.projects) {
      return engine.projects[params.projectId];
    }
    return undefined;
  });

  const getObjectives = createMemo<Array<APINodeItem> | undefined>(() => {
    if (!!getProject()) {
      return Object.values(getProject()!.nodes).filter((x) =>
        x.labels.includes("Objective"),
      );
    }
  });

  const getStartingLinkIds = createMemo<number[]>(() => {
    if (getProject()) {
      // Only select nodes that have AddedByUser label
      return Object.values(getProject()!.nodes)
        .filter(
          (x) =>
            x.labels.includes("Link" as NodeLabel) &&
            (x.labels.includes("AddedByUser" as NodeLabel) ||
              x.labels.includes("AddedByAI" as NodeLabel) ||
              x.labels.includes("AddedByWebSearch" as NodeLabel)),
        )
        .sort((a, b) => a.id - b.id)
        .slice(0, 10)
        .map((x) => x.id);
    } else {
      return [];
    }
  });

  const addLink = (_name: string, value: string) => {
    createNode(params.projectId, {
      Link: {
        url: value,
      },
    } as NodeWrite).then((_) => {});
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
        </div>

        <div>
          <Heading size={3}>Crawler Settings</Heading>
          <NodeGrid
            nodeType="CrawlerSettings"
            source={() => []}
            mode="regular"
          />
        </div> */}
<div class="flex flex-col gap-2 pb-2">
        <Heading size={3}>Starting links</Heading>
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
