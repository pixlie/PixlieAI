import { Component, createMemo, onMount } from "solid-js";
import { useEngine } from "../../stores/engine.tsx";
import { useParams } from "@solidjs/router";
import NodeGrid from "../../widgets/node/NodeGrid";
import Paragraph from "../../widgets/typography/Paragraph";
import LinkForm from "../../widgets/nodeForm/LinkForm";
import SearchTermForm from "../../widgets/nodeForm/SearchTermForm";
import Heading from "../../widgets/typography/Heading.tsx";
import TopicForm from "../../widgets/nodeForm/TopicForm.tsx";

const Workflow: Component = () => {
  const [engine, { fetchNodes, fetchEdges }] = useEngine();
  const params = useParams();

  onMount(() => {
    fetchNodes(params.projectId);
    fetchEdges(params.projectId);
  });

  const getProject = createMemo(() => {
    if (!!params.projectId && params.projectId in engine.projects) {
      return engine.projects[params.projectId];
    }
    return undefined;
  });

  const getSelectLinkIds = createMemo<number[]>(() => {
    if (getProject()) {
      // Only select nodes that have AddedByUser label
      return Object.values(getProject()!.nodes)
        .filter(
          (x) => x.labels.includes("AddedByUser") && x.labels.includes("Link"),
        )
        .map((x) => x.id);
    } else {
      return [];
    }
  });

  const getSelectSearchTermIds = createMemo<number[]>(() => {
    if (getProject()) {
      // Only select nodes that have AddedByUser label
      return Object.values(getProject()!.nodes)
        .filter(
          (x) =>
            x.labels.includes("AddedByUser") && x.labels.includes("SearchTerm"),
        )
        .map((x) => x.id);
    } else {
      return [];
    }
  });
  
  const getSelectTopicIds = createMemo<number[]>(() => {
    if (getProject()) {
      // Only select nodes that have AddedByUser label
      return Object.values(getProject()!.nodes)
        .filter(
          (x) =>
            x.labels.includes("AddedByUser") && x.labels.includes("Topic"),
        )
        .map((x) => x.id);
    } else {
      return [];
    }
  });

  return (
    <>
      <div class="max-w-screen-sm mb-8">
        <Paragraph>
          Pixlie can monitor keywords on multiple URLs. If you add a URL from a
          website, then Pixlie will crawl all URLs on that website.
        </Paragraph>
      </div>

      <Heading size={3}>Starting links</Heading>
      <NodeGrid nodeType={"Link"} source={getSelectLinkIds} />
      <div class="mt-6 max-w-screen-sm">
        <LinkForm />
      </div>

      <Heading size={3}>Saved search terms</Heading>
      <NodeGrid nodeType={"SearchTerm"} source={getSelectSearchTermIds} />
      <div class="mt-6 max-w-screen-sm">
        <SearchTermForm />
      </div>

      <Heading size={3}>Saved topics</Heading>
      <NodeGrid nodeType={"Topic"} source={getSelectTopicIds} />
      <div class="mt-6 max-w-screen-sm">
        <TopicForm />
      </div>
    </>
  );
};

export default Workflow;
