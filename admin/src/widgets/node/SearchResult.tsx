import { Component, createMemo, For } from "solid-js";
import { useEngine } from "../../stores/engine";
import { useParams } from "@solidjs/router";
import { APINodeItem } from "../../api_types/APINodeItem";
import NodeGrid from "./NodeGrid";
import { useUIClasses } from "../../stores/UIClasses";

interface ISearchResultsProps {
  searchTerms: string[];
}

const SearchResults: Component<ISearchResultsProps> = (props) => {
  const [_e, { getNodes, getRelatedNodes }] = useEngine();
  const [_u, { getColors }] = useUIClasses();
  const params = useParams();

  const getContentNodeIds = createMemo<Array<number>>(() => {
    // Later, we can fetch by label Content instead of WebPage
    // and do conditional rendering based on the label type(WebPage, PDFFile, etc)
    return getNodes(params.projectId, (node) => {
      return node.labels.includes("WebPage");
    }).map((node) => node.id);
  });

  const getResultNodeIds = createMemo<
    [number, Record<"partialNodeIds", number[]>][]
  >(() => {
    // We filter out and pass on just relevant content node ids
    // with their related partial node ids.
    let prevNode: APINodeItem | null = null;
    const nodeMatcher = (node: APINodeItem) => {
      return (
        (node.labels.includes("Partial") || node.labels.includes("Metadata")) &&
        node.payload.type === "Text" &&
        node.payload.data.length > 0 &&
        props.searchTerms.some((searchTerm) =>
          (node.payload.data as string)
            .toLowerCase()
            .replace(/[[\s\n][\s\n]+]/, " ")
            .includes(searchTerm.toLowerCase()),
        )
      );
    };
    return getContentNodeIds()
      .map((contentNodeId) => {
        return [
          contentNodeId,
          {
            partialNodeIds: getRelatedNodes(
              params.projectId,
              contentNodeId,
              "ParentOf",
              (contentNode) => {
                const currentNodeMatches = nodeMatcher(contentNode);
                const prevNodeMatches = !!prevNode && nodeMatcher(prevNode);
                prevNode = contentNode;
                return currentNodeMatches || prevNodeMatches;
              },
            ).map((node) => node.id) as number[],
          },
        ] as [number, Record<"partialNodeIds", number[]>];
      })
      .filter((result) => result[1].partialNodeIds.length > 0);
  });

  return (
    <>
      <div class={`text-sm px-3 italic ${getColors().textMuted}`}>
        Found {getResultNodeIds().length} results for{" "}
        <For each={Object.values(props.searchTerms)}>
          {(term, idx) => (
            <>
              {idx() > 0 && " OR "}
              <span class="font-bold">"{term}"</span>
            </>
          )}
        </For>
      </div>
      <NodeGrid
        nodeType="Search"
        source={() => getResultNodeIds().map((r) => r[0])}
        mode="preview"
        data={{
          data: { highlightTerms: props.searchTerms },
          nodeData: Object.fromEntries(getResultNodeIds()),
        }}
      />
    </>
  );
};

export default SearchResults;
