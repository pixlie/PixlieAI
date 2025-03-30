import { Component, createMemo, For } from "solid-js";
import { useEngine } from "../../stores/engine";
import { useParams } from "@solidjs/router";
import { INodeItemDisplayProps } from "../../utils/types";
import ContentNode from "./ContentNode";
import LinkNode from "./LinkNode";

const PREVIEW_TRUNCATION_LENGTH = 500;

const ContentContainerNode: Component<INodeItemDisplayProps> = (props) => {
  const [_engine, { getRelatedNodes, getNodeById }] = useEngine();
  const params = useParams();

  const getPartialNodeIds = createMemo<number[]>(() =>
    getRelatedNodes(
      params.projectId,
      props.nodeId,
      "ParentOf",
      (node) =>
        node.labels.includes("Partial") &&
        (props.mode === "regular" || !node.labels.includes("Title")) &&
        node.payload.type === "Text" &&
        node.payload.data.trim().length > 0,
    ).map((node) => node.id),
  );

  const getPreviewContent = createMemo<string>(() => {
    const content = getPartialNodeIds()
      .map((id) =>
        (
          getNodeById(params.projectId, id)?.payload.data as string | undefined
        )?.trim(),
      )
      .filter((data) => data)
      .join(" ");
    return (
      content.slice(0, PREVIEW_TRUNCATION_LENGTH) +
      (content.length > PREVIEW_TRUNCATION_LENGTH ? "..." : "")
    );
  });

  const getLinkNodeId = createMemo<number | undefined>(() => {
    return getRelatedNodes(params.projectId, props.nodeId, "ContentOf")?.[0]
      ?.id;
  });

  return !!getLinkNodeId() || getPartialNodeIds().length > 0 ? (
    <div class="flex flex-col px-3 py-1">
      {getLinkNodeId() ? (
        <LinkNode nodeId={getLinkNodeId()!} showFlags={!!props.showFlags} />
      ) : (
        <div class="text-sm">Content from unknown source</div>
      )}
      {props.mode == "regular" && (
        <For each={getPartialNodeIds()}>
          {(nodeId) => (
            <ContentNode nodeId={nodeId as number} mode={props.mode} />
          )}
        </For>
      )}
      {props.mode == "preview" && getPreviewContent().length && (
        <div class="text-sm max-w-[780px]">{getPreviewContent()}</div>
      )}
    </div>
  ) : null;
};

export default ContentContainerNode;
