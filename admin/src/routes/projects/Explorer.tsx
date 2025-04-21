import { Component, createEffect, For, onMount } from "solid-js";
import { ExplorerProvider, useExplorer } from "../../stores/explorer.tsx";
import { APINodeItem } from "../../api_types/APINodeItem.ts";
import Paragraph from "../../widgets/typography/Paragraph.tsx";
import { NodeLabel } from "../../api_types/NodeLabel.ts";
import { useParams } from "@solidjs/router";
import Heading from "../../widgets/typography/Heading.tsx";

const NodeDisplay: Component<APINodeItem> = (props: APINodeItem) => {
  return (
    <>
      {props.labels.includes("Objective" as NodeLabel) && (
        <div class="absolute p-4 shadow inline-block rounded bg-stone-100">
          <Heading size={4}>Objective</Heading>
          <Paragraph>{props.payload.data as string}</Paragraph>
        </div>
      )}
      {props.labels.includes("WebSearch" as NodeLabel) && (
        <div class="absolute p-4 shadow inline-block rounded bg-stone-100">
          <Paragraph size={"sm"}>
            Web Search keywords: {props.payload.data as string}
          </Paragraph>
        </div>
      )}
    </>
  );
};

const Inner: Component = () => {
  const [explorer, { setProjectId, explore }] = useExplorer();
  const params = useParams();

  onMount(() => {
    if (
      !!params.projectId &&
      !Object.keys(explorer.projects).includes(params.projectId)
    ) {
      setProjectId(params.projectId);
      explore(params.projectId);
    }
  });

  createEffect((prevProjectId: string | void) => {
    if (!!params.projectId) {
      if (
        prevProjectId !== params.projectId &&
        !Object.keys(explorer.projects).includes(params.projectId)
      ) {
        // When changing to a new project from projects dropdown
        setProjectId(params.projectId);
        explore(params.projectId);
      }
    }
  }, params.projectId);

  return (
    <>
      {!!params.projectId &&
        Object.keys(explorer.projects).includes(params.projectId) && (
          <For each={explorer.projects[params.projectId].nodes}>
            {(node) => <NodeDisplay {...node} />}
          </For>
        )}
    </>
  );
};

const Explorer: Component = () => {
  // The node editor is created with free positioned divs that can be dragged and dropped.
  // Each node's position can be saved (later). Nodes are connected with edges which are SVG paths

  return (
    <ExplorerProvider>
      <div class="relative">
        <Inner />
      </div>
    </ExplorerProvider>
  );
};

export default Explorer;
