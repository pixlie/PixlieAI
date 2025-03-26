import { Component, createMemo, For } from "solid-js";
import { useEngine } from "../../stores/engine.tsx";
import { useParams } from "@solidjs/router";
import NodeGrid from "../../widgets/node/NodeGrid";
import Paragraph from "../../widgets/typography/Paragraph";
import LinkForm from "../../widgets/nodeForm/LinkForm";
import Heading from "../../widgets/typography/Heading.tsx";
import { NodeLabel } from "../../api_types/NodeLabel.ts";
import { APINodeItem } from "../../api_types/APINodeItem.ts";

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
        .map((x) => x.id);
    } else {
      return [];
    }
  });

  return (
    <div class="flex gap-x-12">
      <div class="flex-1 flex flex-col gap-y-6">
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

        <div>
          <Heading size={3}>Starting links</Heading>
          <NodeGrid nodeType={"Link"} source={getStartingLinkIds} />
          <div class="pt-6 max-w-screen-sm">
            <LinkForm />
          </div>
        </div>
      </div>

      <WorkflowSidebar />
    </div>
  );
};

export const WorkflowSidebar: Component = () => {
  return (
    <div class="w-96 bg-stone-100 rounded-md p-6 drop-shadow flex flex-col gap-6">
      <Paragraph size={"sm"}>
        Pixlie uses your objective to ask AI for starting URLs and keywords to
        monitor on websites. Pixlie will continue crawling the web as long as
        pages match your objective.
      </Paragraph>

      <Paragraph size={"sm"}>
        Pixlie can extract information that is relevant to your objective but
        this feature is still in beta. Pixlie can extract blog posts, job posts,
        people, companies, events, dates and locations.
      </Paragraph>
    </div>
  );
};

export default Workflow;
