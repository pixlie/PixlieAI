import { Component, createMemo, For } from "solid-js";
import { INodeListItemProps } from "../../utils/types.tsx";
import ContentContainerNode from "./ContentContainerNode.tsx";
import DomainNode from "./DomainNode";
import LinkNode from "./LinkNode";
import ProjectSettingsNode from "./ProjectSettingsNode.tsx";
import SearchTermNode from "./SearchTermNode";
import TopicNode from "./TopicNode";
import URLNode from "./URLNode.tsx";
import WebPageNode from "./WebPageNode.tsx";

const NodeGrid: Component<INodeListItemProps> = (props) => {
  const getN = createMemo<Array<number>>(() => {
    return props.source().slice(0, 100);
  });

  return (
    <>
      {props.nodeType ? (
        <>
          {props.nodeType === "Link" && (
            <div class="grid grid-cols-1 gap-2 pt-1 pb-8">
              <For each={getN()}>
                {(nodeId) => <LinkNode nodeId={nodeId} showFlags={true} />}
              </For>
            </div>
          )}
          {props.nodeType === "Domain" && (
            <div class="grid grid-cols-[1fr_auto] gap-2 pt-1 pb-8">
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
            <div class="columns-1 lg:columns-3 space-y-8 gap-8 mt-2 pb-4">
              <For each={getN()}>
                {(nodeId) => (
                  <div class="break-inside-avoid overflow-visible will-change-transform">
                    <WebPageNode nodeId={nodeId} />
                  </div>
                )}
              </For>
            </div>
          )}
          {props.nodeType === "ProjectSettings" && (
            <div class="grid grid-cols-1 gap-2">
              <For each={getN()}>
                {(nodeId) => <ProjectSettingsNode nodeId={nodeId} />}
              </For>
            </div>
          )}
          {props.nodeType === "URL" && ( // TODO: add URL as a NodeLabel
            <div class="flex-1 flex flex-col -mt-2">
              <For each={getN()}>
                {(nodeId, i) => (
                  <URLNode
                    nodeId={nodeId}
                    showDivider={i() < getN().length - 1}
                  />
                )}
              </For>
            </div>
          )}
        </>
      ) : null}
    </>
  );
};

export default NodeGrid;
