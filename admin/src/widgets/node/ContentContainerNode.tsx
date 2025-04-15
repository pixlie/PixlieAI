import { Component, createMemo, For } from "solid-js";
import { useEngine } from "../../stores/engine";
import { useParams } from "@solidjs/router";
import { INodeItemDisplayProps } from "../../utils/types";
import ContentNode from "./ContentNode";
import LinkNode from "./LinkNode";
import HighlightTerms from "../generic/HighlightTerms";
import { useUIClasses } from "../../stores/UIClasses";

const PREVIEW_TRUNCATION_LENGTH = 500;

const ContentPreview: Component<{
  nodeIds: number[];
  data?: Record<string, any>;
  nodeData?: Record<string, any>;
}> = (props) => {
  const params = useParams();
  const [_e, { getNodeById }] = useEngine();
  const [_s, { getColors }] = useUIClasses();

  const getContent = createMemo<string>(() => {
    return props.nodeIds
      .filter((id) => {
        const node = getNodeById(params.projectId, id);
        return (
          !!node &&
          node.payload.type === "Text" &&
          node.payload.data !== undefined &&
          node.payload.data.trim().length > 0
        );
      })
      .map((id) =>
        (getNodeById(params.projectId, id)!.payload.data as string).trim(),
      )
      .join(" ");
  });

  return (
    <div class={`text-sm ${getColors().textSoft} max-w-[780px]}`}>
      {!!props.data?.highlightTerms ? (
        <HighlightTerms
          terms={props.data?.highlightTerms || []}
          content={getContent()}
        />
      ) : (
        getContent().slice(0, PREVIEW_TRUNCATION_LENGTH) +
        (getContent().length > PREVIEW_TRUNCATION_LENGTH ? "…" : "")
      )}
    </div>
  );
};

const ContentContainerNode: Component<INodeItemDisplayProps> = (props) => {
  const [_, { getRelatedNodes }] = useEngine();
  const params = useParams();

  const getPartialNodeIds = createMemo<number[]>(() =>
    getRelatedNodes(
      params.projectId,
      props.nodeId,
      "ParentOf",
      (node) =>
        props.mode === "regular" ||
        (!node.labels.includes("Title") &&
          (props.nodeData?.partialNodeIds
            ? props.nodeData.partialNodeIds.includes(node.id)
            : node.labels.includes("Metadata") &&
              node.payload.type === "Text" &&
              node.payload.data.trim().length > 0)),
    ).map((node) => node.id),
  );

  const getLinkNodeId = createMemo<number | undefined>(() => {
    return getRelatedNodes(params.projectId, props.nodeId, "ContentOf")?.[0]
      ?.id;
  });

  // TODO: Fix filtering logic resulting in empty containers
  // https://github.com/user-attachments/assets/1d674a3d-af10-4934-8679-6f3d9444319d
  return !!getLinkNodeId() || getPartialNodeIds().length > 0 ? (
    <div class="flex flex-col px-3 py-1">
      {getLinkNodeId() ? (
        <LinkNode
          nodeId={getLinkNodeId()!}
          showFlags={!!props.showFlags}
          data={props.data}
        />
      ) : (
        <div class="text-sm">Content from unknown source</div>
      )}
      {props.mode == "regular" && (
        <For each={getPartialNodeIds()}>
          {(nodeId) => (
            <ContentNode
              nodeId={nodeId as number}
              mode={props.mode}
              data={props.data}
              nodeData={props.nodeData?.[nodeId]}
            />
          )}
        </For>
      )}
      {props.mode == "preview" && (
        <ContentPreview
          nodeIds={getPartialNodeIds()}
          data={props.data}
          nodeData={props.nodeData}
        />
      )}
    </div>
  ) : (
    ""
  );
};

export default ContentContainerNode;
