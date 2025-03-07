import { Component, createMemo, createSignal, For, onMount } from "solid-js";
import { useEngine } from "../../stores/engine";
import Heading from "../../widgets/typography/Heading";
import { useParams } from "@solidjs/router";
import TextInput from "../../widgets/interactable/TextInput";
import { APINodeItem } from "../../api_types/APINodeItem";
import SearchResults from "../../widgets/node/SearchResult";
import { IFormFieldValue } from "../../utils/types";

interface IFormData {
  searchTerm: string;
}

const Search: Component = () => {
  const [engine, { fetchNodes, fetchEdges }] = useEngine();
  const params = useParams();
  const [formData, setFormData] = createSignal<IFormData>({
    searchTerm: "",
  });
  const [searchTerm, setSearchTerm] = createSignal<string>("");

  onMount(() => {
    fetchNodes(params.projectId);
    fetchEdges(params.projectId);
  });

  const getSearchTerms = createMemo<Array<APINodeItem>>(() => {
    if (!!params.projectId && params.projectId in engine.projects) {
      return Object.values(engine.projects[params.projectId].nodes).filter(
        (node) => node.labels.includes("SearchTerm"),
      );
    }
    return [];
  });

  const handleSearchTermChange = (_name: string, data: IFormFieldValue) => {
    if (typeof data !== "string" || !data) {
      return;
    }
    setFormData({ ...formData, searchTerm: data });
  };

  // Save the typed search term to the searchTerm field on Enter key press
  const handleKeyUp = (event: KeyboardEvent) => {
    if (event.key === "Enter") {
      setSearchTerm(formData().searchTerm);
    }
  };

  const handleClickSavedSearchTerm = (data: string) => {
    setSearchTerm(data);
  };

  return (
    <>
      <Heading size={3}>Search</Heading>
      <TextInput
        name="query"
        placeholder="Search for..."
        value={formData().searchTerm}
        onChange={handleSearchTermChange}
        onKeyUp={handleKeyUp}
        isEditable
      />

      <Heading size={5}>Saved search terms</Heading>
      <div class="flex flex-row space-x-2">
        <For each={getSearchTerms()}>
          {(node) => (
            <span
              class="bg-indigo-700 text-white px-2 rounded cursor-pointer"
              onClick={() =>
                handleClickSavedSearchTerm(node.payload.data as string)
              }
            >
              {node.payload.data as string}
            </span>
          )}
        </For>
      </div>

      {!!searchTerm() && (
        <>
          <Heading size={5}>Searching for {searchTerm()}</Heading>
          <SearchResults searchTerm={searchTerm()} />
        </>
      )}
    </>
  );
};

export default Search;
