import { Component, createMemo, For } from "solid-js";
import LinkNode from "./LinkNode";
import DomainNode from "./DomainNode";
import SearchTermNode from "./SearchTermNode";
import TopicNode from "./TopicNode";
import ContentContainerNode from "./ContentContainerNode.tsx";

interface NodeListItemProps {
  nodeType?: string;
  source: () => Array<number>;
}

const NodeGrid: Component<NodeListItemProps> = (props) => {
  const getN = createMemo<Array<number>>(() => {
    return props.source().slice(0, 100);
  });

  return (
    <>
      {props.nodeType ? (
        <>
          {props.nodeType === "Link" && (
            <div class="grid grid-cols-[auto_1fr_auto] gap-2">
              <For each={getN()}>
                {(nodeId) => <LinkNode nodeId={nodeId} />}
              </For>
            </div>
          )}
          {props.nodeType === "Domain" && (
            <div class="grid grid-cols-[1fr_auto] gap-2">
              <For each={getN()}>
                {(nodeId) => <DomainNode nodeId={nodeId} />}
              </For>
            </div>
          )}
          {props.nodeType === "SearchTerm" && (
            <div class="grid grid-cols-[1fr_auto] gap-2">
              <For each={props.source()}>
                {(nodeId) => <SearchTermNode nodeId={nodeId} />}
              </For>
            </div>
          )}
          {props.nodeType === "Topic" && (
            <div class="grid grid-cols-[1fr_auto] gap-2">
              <For each={props.source()}>
                {(nodeId) => <TopicNode nodeId={nodeId} />}
              </For>
            </div>
          )}
          {props.nodeType === "WebPage" && (
            <div class="grid grid-cols-2 gap-6">
              <For each={props.source()}>
                {(nodeId) => <ContentContainerNode nodeId={nodeId} />}
              </For>
            </div>
          )}
        </>
      ) : null}
    </>
  );
};

export default NodeGrid;
