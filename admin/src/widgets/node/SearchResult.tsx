import {
  Component,
  createEffect,
  createMemo,
  createSignal,
  For,
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

interface INodeProps {
  nodeId: number;
}

const SearchResultItems: Component<INodeProps> = (props) => {
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
          setResults((existing: Array<APINodeItem>) => ({
            ...existing,
            ...responsePayload.data.nodes,
          }));
        }
      });
    });
  };

  onMount(() => {
    getSearchResults(params.projectId, props.nodeId);
  });

  createEffect(() => {
    console.log(results());
  });

  return (
    <For each={results()}>
      {(result) => (
        <>
          {result.payload.type === "Heading" && (
            <Heading size={3}>{result.payload.data}</Heading>
          )}
          {result.payload.type === "Title" && (
            <Heading size={1}>{result.payload.data}</Heading>
          )}
        </>
      )}
    </For>
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
        <>
          <span class="font-bold">{getPayload()!}</span>
          <div>
            <SearchResultItems nodeId={props.nodeId} />
          </div>
        </>
      ) : null}
    </>
  );
};

export default SearchResultNode;
