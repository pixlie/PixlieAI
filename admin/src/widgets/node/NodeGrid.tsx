import { Component, createMemo, For } from "solid-js";
import LinkNode from "./LinkNode";
import DomainNode from "./DomainNode";
import SearchTermNode from "./SearchTermNode";
import TopicNode from "./TopicNode";
import ContentContainerNode from "./ContentContainerNode.tsx";
import { INodeListItemProps } from "../../utils/types.tsx";
import WebPageNode from "./WebPageNode.tsx";
import URLNode from "./URLNode.tsx";

const NodeGrid: Component<INodeListItemProps> = (props) => {
  const getN = createMemo<Array<number>>(() => {
    return props.source().slice(0, 100);
  });

  return (
    <>
      {props.nodeType ? (
        <>
          {props.nodeType === "Link" && (
            <div class="grid grid-cols-1 gap-2">
              <For each={getN()}>
                {(nodeId) => <LinkNode nodeId={nodeId} showFlags={true} />}
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
                {(nodeId) => (
                  <SearchTermNode nodeId={nodeId} mode={props.mode} />
                )}
              </For>
            </div>
          )}
          {props.nodeType === "Topic" && (
            <div class="grid grid-cols-[1fr_auto] gap-2">
              <For each={props.source()}>
                {(nodeId) => <TopicNode nodeId={nodeId} mode={props.mode} />}
              </For>
            </div>
          )}
          {props.nodeType === "Search" && (
            <div class="grid grid-cols-1 gap-2">
              <For each={getN()}>
                {(nodeId) => (
                  <ContentContainerNode
                    nodeId={nodeId}
                    mode={props.mode}
                    data={props.data?.data}
                    nodeData={props.data?.nodeData?.[nodeId]}
                  />
                )}
              </For>
            </div>
          )}
          {props.nodeType === "WebPage" && (
            <div class="columns-1 lg:columns-3 space-y-8 gap-8">
              <For each={getN()}>
                {(nodeId) => (
                  <div class="break-inside-avoid overflow-visible will-change-transform">
                    <WebPageNode nodeId={nodeId} />
                  </div>
                )}
              </For>
            </div>
          )}
          {props.nodeType === "URL" && (  // TODO: add URL as a NodeLabel
            <div class="flex-1 flex flex-col -mt-4">
              <For each={getN()}>{(nodeId) => <URLNode nodeId={nodeId} />}</For>
            </div>
          )}
        </>
      ) : null}
    </>
  );
};

export default NodeGrid;
