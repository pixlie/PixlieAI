import { Component, For } from "solid-js";
import LinkNode from "./LinkNode";
import DomainNode from "./DomainNode";
import SearchTermNode from "./SearchTermNode";
import ContentNode from "./ContentNode";

interface NodeListItemProps {
  nodeType?: string;
  source: () => Array<number>;
}

const NodeGrid: Component<NodeListItemProps> = (props) => {
  return (
    <>
      {props.nodeType ? (
        <>
          {props.nodeType === "Link" && (
            <div class="grid grid-cols-[auto_1fr_auto] gap-2">
              <For each={props.source()}>
                {(nodeId) => <LinkNode nodeId={nodeId} />}
              </For>
            </div>
          )}
          {props.nodeType === "Domain" && (
            <div class="flex flex-col gap-6">
              <For each={props.source()}>
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
          {(props.nodeType === "Title" ||
            props.nodeType === "Heading" ||
            props.nodeType === "Paragraph") && (
            <div class="grid grid-cols-1 gap-2">
              <For each={props.source()}>
                {(nodeId) => <ContentNode nodeId={nodeId} />}
              </For>
            </div>
          )}
        </>
      ) : null}
    </>
  );
};

export default NodeGrid;
