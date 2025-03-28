import { Component, createMemo, For } from "solid-js";
import { useEngine } from "../../stores/engine";
import { useParams } from "@solidjs/router";
import { INodeItemDisplayProps } from "../../utils/types";
import ContentNode from "./ContentNode";
import LinkNode from "./LinkNode";

const PREVIEW_TRUNCATION_LENGTH = 500;

const ContentContainerNode: Component<INodeItemDisplayProps> = (props) => {
  const [_engine, { getRelatedNodes }] = useEngine();
  const params = useParams();

  interface IRelevantNodeIds {
    "link": number | null;
    "content": number[] | string;
    "render": () => boolean;
  }

  const getRelevantNodeIds = createMemo<IRelevantNodeIds>(() => {

    const relevantNodeIds = getRelatedNodes(
      params.projectId,
      props.nodeId,
      "ParentOf"
    ).reduce(
      (acc, node) => {
        if (node.payload.type !== "Text" || node.payload.data.length === 0) {
          return acc;
        }
        if (node.labels.includes("Partial") && !node.labels.includes("Title")) {
          if (props.mode == "regular") {
            (acc.content as number[]).push(node.id);
          }
          else if (props.mode == "preview") {
            if (acc.content.length < PREVIEW_TRUNCATION_LENGTH) {
              acc.content += " " + node.payload.data;
              if (acc.content.length >= PREVIEW_TRUNCATION_LENGTH) {
                acc.content =
                  (acc.content as string)
                    .trim()
                    .substring(0, PREVIEW_TRUNCATION_LENGTH) + "...";
              }
            }
          }
        }
        return acc;
      },
      {
        link: null,
        title: null,
        content: props.mode === "regular" ? [] as number[] : [] as string[],
        render: function () {
          return this.link || this.content.length > 0;
        },
      } as IRelevantNodeIds
    );
    if (props.mode == "regular") {
      (relevantNodeIds.content as number[]).sort();
    }
    const linkNodeIds = getRelatedNodes(
      params.projectId,
      props.nodeId,
      "ContentOf"
    ).filter((node) => node.labels.includes("Link")).map((node) => node.id);
    if (linkNodeIds.length > 0) {
      relevantNodeIds.link = linkNodeIds[0];
    }
    return relevantNodeIds;
  });


  return (
    <>
      {getRelevantNodeIds().render() ? (
        <div class="flex flex-col px-3 py-1">
          <LinkNode
            nodeId={getRelevantNodeIds().link!}
            showFlags={false}
          />
          {props.mode == "regular" && (
            <For each={getRelevantNodeIds().content as number[]}>
              {(nodeId) => <ContentNode nodeId={nodeId as number} mode={props.mode} />}
            </For>
          )}
          {props.mode == "preview" && (
            <div class="text-sm max-w-[780px]">
              {getRelevantNodeIds().content}
            </div>
          )}
        </div>
      ) : null}
    </>
  );
};

export default ContentContainerNode;
