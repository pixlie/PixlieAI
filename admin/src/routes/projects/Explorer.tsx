import { Component, createEffect, createMemo, For, onMount } from "solid-js";
import { ExplorerProvider, useExplorer } from "../../stores/explorer.tsx";
import { APINodeItem } from "../../api_types/APINodeItem.ts";
import Paragraph from "../../widgets/typography/Paragraph.tsx";
import { NodeLabel } from "../../api_types/NodeLabel.ts";
import { useParams } from "@solidjs/router";
import Heading from "../../widgets/typography/Heading.tsx";
import { CrawlerSettings } from "../../api_types/CrawlerSettings.ts";

const NodeDisplay: Component<APINodeItem> = (props: APINodeItem) => {
  const [_, { placeNodeOnCanvas }] = useExplorer();
  const params = useParams();
  let elementRef: HTMLDivElement;

  onMount(() => {
    if (params.projectId && elementRef) {
      let position = placeNodeOnCanvas(
        params.projectId as string,
        elementRef.getBoundingClientRect().width,
        elementRef.getBoundingClientRect().height,
      );

      // Place the node at the given position
      elementRef.style.left = `${position.x1}px`;
      elementRef.style.top = `${position.y1}px`;
    }
  });

  const labelsOfInterest = ["Objective", "CrawlerSettings", "WebSearch"];
  return (
    <>
      {labelsOfInterest.some((label) =>
        props.labels.includes(label as NodeLabel),
      ) ? (
        <div
          class="absolute p-4 shadow inline-block rounded bg-stone-100"
          ref={elementRef}
        >
          {props.labels.includes("Objective" as NodeLabel) && (
            <>
              <Heading size={4}>Objective</Heading>
              <Paragraph>{props.payload.data as string}</Paragraph>
            </>
          )}
          {props.labels.includes("CrawlerSettings" as NodeLabel) && (
            <>
              <Heading size={4}>Crawler Settings</Heading>
              <Paragraph>
                {(
                  props.payload.data as CrawlerSettings
                ).crawl_link_if_anchor_text_has_any_of_these_keywords?.join(
                  ", ",
                )}
              </Paragraph>

              <Paragraph>
                {(
                  props.payload.data as CrawlerSettings
                ).keywords_to_search_the_web_to_get_starting_urls?.join(", ")}
              </Paragraph>
            </>
          )}
          {props.labels.includes("WebSearch" as NodeLabel) && (
            <>
              <Paragraph size={"sm"}>
                Web Search keywords: {props.payload.data as string}
              </Paragraph>
            </>
          )}
        </div>
      ) : (
        <></>
      )}
    </>
  );
};

const NodeGroupDisplay: Component<number[]> = (props: number[]) => {
  const [explorer, { placeNodeOnCanvas }] = useExplorer();
  const params = useParams();
  let elementRef: HTMLDivElement;

  // Get the first node in the group
  const firstNode = createMemo(() => {
    if (
      params.projectId &&
      Object.keys(explorer.projects).includes(params.projectId)
    ) {
      return explorer.projects[params.projectId].nodes.find((node) =>
        props.includes(node.id),
      );
    } else {
      return undefined;
    }
  });

  onMount(() => {
    if (params.projectId && elementRef) {
      let position = placeNodeOnCanvas(
        params.projectId as string,
        elementRef.getBoundingClientRect().width,
        elementRef.getBoundingClientRect().height,
      );

      // Place the node at the given position
      elementRef.style.left = `${position.x1}px`;
      elementRef.style.top = `${position.y1}px`;
    }
  });

  const labelsOfInterest = ["WebSearch"];
  return (
    <>
      {labelsOfInterest.some((label) =>
        firstNode()?.labels.includes(label as NodeLabel),
      ) ? (
        <div
          class="absolute p-4 shadow inline-block rounded bg-stone-100"
          ref={elementRef}
        >
          {firstNode()?.labels.includes("WebSearch" as NodeLabel) && (
            <>
              <Paragraph size={"sm"}>
                Web Search keywords: {firstNode()?.payload.data as string}
              </Paragraph>
            </>
          )}
        </div>
      ) : (
        <></>
      )}
    </>
  );
};

const Inner: Component = () => {
  const [explorer, { setProjectId, explore, setCanvasPosition }] =
    useExplorer();
  const params = useParams();
  let canvasRef: HTMLDivElement;

  onMount(() => {
    if (!!params.projectId && canvasRef) {
      setCanvasPosition(
        params.projectId as string,
        canvasRef.getBoundingClientRect().x,
        canvasRef.getBoundingClientRect().y,
        canvasRef.getBoundingClientRect().width,
        canvasRef.getBoundingClientRect().height,
      );
    }
  });

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

  const getNonSiblingNodes = createMemo<APINodeItem[]>(() => {
    if (
      !!params.projectId &&
      Object.keys(explorer.projects).includes(params.projectId)
    ) {
      // For each node in explorer store, filter only the ones that are not siblings of any other node
      // Create a flat array of all sibling nodes
      const siblingNodes =
        explorer.projects[params.projectId].siblingNodes.flat();
      return explorer.projects[params.projectId].nodes.filter(
        (node) => !siblingNodes.includes(node.id),
      );
    } else {
      return [];
    }
  });

  const getSiblingNodeIds = createMemo<number[][]>(() => {
    if (
      !!params.projectId &&
      Object.keys(explorer.projects).includes(params.projectId)
    ) {
      return explorer.projects[params.projectId].siblingNodes;
    } else {
      return [];
    }
  });

  return (
    <>
      <For each={getNonSiblingNodes()}>
        {(node) => <NodeDisplay {...node} />}
      </For>
      <For each={getSiblingNodeIds()}>
        {(nodeIds) => <NodeGroupDisplay {...nodeIds} />}
      </For>
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
