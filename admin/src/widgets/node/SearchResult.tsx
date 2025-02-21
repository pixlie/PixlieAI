import {
  Component,
  createMemo,
  createSignal,
  For,
  JSX,
  onMount,
} from "solid-js";
import { useEngine } from "../../stores/engine.tsx";
import { useParams } from "@solidjs/router";
import { IEngine } from "../../utils/types.tsx";
import { SearchTerm } from "../../api_types/SearchTerm";
import { getPixlieAIAPIRoot } from "../../utils/api.ts";
import { EngineResponsePayload } from "../../api_types/EngineResponsePayload.ts";
import { APINodeItem } from "../../api_types/APINodeItem.ts";
import Heading from "../typography/Heading.tsx";
import Paragraph from "../typography/Paragraph.tsx";

interface INodeProps {
  nodeId: number;
}

function highlightText(text: string, searchTerm: string): JSX.Element {
  const regex = new RegExp(searchTerm, "gi");
  const parts = text.split(regex);
  return (
    <>
      {parts.map((part, index) => (
        <>
          {index > 0 && <span class="bg-yellow-200 px-0.5">{searchTerm}</span>}
          {part}
        </>
      ))}
    </>
  );
}

interface ISearchResultItemsProps extends INodeProps {
  searchTerm: string;
}

const SearchResultItems: Component<ISearchResultItemsProps> = (props) => {
  const params = useParams();
  const [results, setResults] = createSignal<Array<APINodeItem>>([]);

  const getSearchResults = (projectId: string, nodeId: number) => {
    let pixlieAIAPIRoot = getPixlieAIAPIRoot();
    fetch(`${pixlieAIAPIRoot}/api/engine/${projectId}/query/${nodeId}`, {
      headers: {
        "Content-type": "application/json",
      },
    }).then((response) => {
      if (!response.ok) {
        throw new Error("Failed to fetch query results");
      }

      // Store nodes into the store. These will be a mix of different payload types
      response.json().then((responsePayload: EngineResponsePayload) => {
        if (responsePayload.type === "Results") {
          setResults((existing: Array<APINodeItem>) => ([
            ...existing,
            ...responsePayload.data.nodes,
          ]));
        }
      });
    });
  };

  onMount(() => {
    getSearchResults(params.projectId, props.nodeId);
  });

  return (
    <>
    <For each={results()}>
      {(result) => (
        <div class="mt-2">
            <span class={
              "text-xs bg-gray-300 rounded px-2 py-0.5" + (result.payload.type === "Paragraph" && " float-left mr-1")
            }>{result.payload.type}</span>
          {result.payload.type === "Heading" && (
            <Heading size={3}>{highlightText(result.payload.data, props.searchTerm)}</Heading>
          )}
          {result.payload.type === "Title" && (
            <Heading size={3}>{highlightText(result.payload.data, props.searchTerm)}</Heading>
          )}
          {result.payload.type === "Paragraph" && (
            <Paragraph>{result.payload.data}</Paragraph>
          )}
        </div>
      )}
    </For>
    </>
  );
};

const SearchResultNode: Component<INodeProps> = (props) => {
  const [engine] = useEngine();
  const params = useParams();

  const getProject = createMemo<IEngine | undefined>(() => {
    if (!!params.projectId && params.projectId in engine.projects) {
      return engine.projects[params.projectId];
    }
    return undefined;
  });

  const getPayload = createMemo<SearchTerm | undefined>(() => {
    if (getProject() && props.nodeId in getProject()!.nodes) {
      return getProject()!.nodes[props.nodeId].payload.data as SearchTerm;
    }
    return undefined;
  });

  return (
    <>
      {!!getPayload() ? (
        <div class="border border-gray-300 p-4 rounded-lg">
          <div>
            Search Term:{" "}
            <span class="font-bold text-xs bg-gray-300 rounded px-2 py-0.5">
              {getPayload()!}
            </span>
          </div>
          <div>
            <SearchResultItems nodeId={props.nodeId} searchTerm={getPayload() || ""} />
          </div>
        </div>
      ) : (
        "This term was not found in the search results"
      )}
    </>
  );
};

export default SearchResultNode;
