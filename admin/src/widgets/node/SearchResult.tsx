import { Component, createMemo, For, JSX } from "solid-js";
import { useEngine } from "../../stores/engine";
import { useParams } from "@solidjs/router";
import { APINodeItem } from "../../api_types/APINodeItem";
import Heading from "../typography/Heading";
import Paragraph from "../typography/Paragraph";

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

interface ISearchResultsProps {
  searchTerm: string;
}

const SearchResults: Component<ISearchResultsProps> = (props) => {
  const [engine] = useEngine();
  const params = useParams();

  const getResults = createMemo<Array<APINodeItem>>(() => {
    if (params.projectId in engine.projects) {
      return Object.entries(engine.projects[params.projectId].nodes)
        .filter(([_key, value]) => {
          return (
            value.payload.type === "Text" &&
            (value.labels.includes("Title") ||
              value.labels.includes("Heading") ||
              value.labels.includes("Paragraph")) &&
            value.payload.data
              .toLowerCase()
              .includes(props.searchTerm.toLowerCase())
          );
        })
        .map(([_key, value]) => value as APINodeItem);
    }
    return [];
  });

  return (
    <For each={getResults()}>
      {(result) => (
        <div class="mt-2">
          <span
            class={
              "text-xs bg-gray-300 rounded px-2 py-0.5" +
              (result.payload.type === "Text" && " float-left mr-1")
            }
          >
            {result.labels.includes("Title") && "Title"}
            {result.labels.includes("Heading") && "Heading"}
            {result.labels.includes("Paragraph") && "Paragraph"}
          </span>
          {result.payload.type === "Text" &&
            result.labels.includes("Heading") && (
              <Heading size={3}>
                {highlightText(result.payload.data, props.searchTerm)}
              </Heading>
            )}
          {result.payload.type === "Text" &&
            result.labels.includes("Title") && (
              <Heading size={3}>
                {highlightText(result.payload.data, props.searchTerm)}
              </Heading>
            )}
          {result.payload.type === "Text" &&
            result.labels.includes("Paragraph") && (
              <Paragraph>
                {highlightText(result.payload.data, props.searchTerm)}
              </Paragraph>
            )}
        </div>
      )}
    </For>
  );
};

export default SearchResults;
