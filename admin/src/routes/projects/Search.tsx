import { Component, createMemo, createSignal, For, onMount } from "solid-js";
import { useEngine } from "../../stores/engine";
import Heading from "../../widgets/typography/Heading";
import { useParams } from "@solidjs/router";
import TextInput from "../../widgets/interactable/TextInput";
import SearchResults from "../../widgets/node/SearchResult";
import { IActionsWrapperAction, IFormFieldValue } from "../../utils/types";
import ActionsWrapper from "../../widgets/interactable/ActionsWrapper";
import { slugify } from "../../utils/utils";
import SaveIcon from "../../assets/icons/feather-save.svg";
import DeleteIcon from "../../assets/icons/heroicons-trash.svg";
import RemoveIcon from "../../assets/icons/tabler-cross.svg";
import Label from "../../widgets/generic/Label";
import { useUIClasses } from "../../stores/UIClasses";
import { insertNode } from "../../utils/api";
import { NodeWrite } from "../../api_types/NodeWrite";

interface IFormData {
  searchTerm: string;
}

interface ISearchTerm {
  id: string;
  term: string;
  enabled: boolean;
  active: boolean;
  saved: boolean;
}

interface ISearchTermListProps {
  searchTerms: Record<string, ISearchTerm>;
  type: "active" | "saved";
  actions: {
    [key in "toggle" | "save" | "delete" | "activate" | "deactivate"]: (
      slug: string,
    ) => void;
  };
}

const SearchTermList: Component<ISearchTermListProps> = (
  props: ISearchTermListProps,
) => {
  const [_, { getColors }] = useUIClasses();
  const getWrapperActions = (slug: string): IActionsWrapperAction[] => {
    const wrapperActions: IActionsWrapperAction[] = [
      {
        render: !props.searchTerms[slug].saved,
        color: "textInfo",
        onClick: (_) => props.actions.save(slug),
        tooltip: "Save",
        icon: <SaveIcon />,
      },
      {
        render: props.searchTerms[slug].saved,
        color: "textMuted",
        tooltip: "Delete",
        icon: <DeleteIcon />,
      },
    ];
    if (props.type === "active") {
      wrapperActions.push({
        render: props.searchTerms[slug].active,
        color: "textDanger",
        onClick: (_) => props.actions.delete(slug),
        tooltip: "Remove",
        icon: <RemoveIcon />,
      });
    }
    return wrapperActions;
  };
  const getRenderableSearchTerms = createMemo<ISearchTerm[]>(() => {
    return Object.values(props.searchTerms).filter((term) => term[props.type]);
  });
  return (
    <>
      {getRenderableSearchTerms().length > 0 ? (
        <div class="flex flex-wrap gap-2 items-end px-3">
          <span class={`text-sm font-bold ${getColors().textSoft}`}>
            {props.type === "active" ? "Active" : "Saved"} Keywords:
          </span>
          <For each={getRenderableSearchTerms()}>
            {(savedSearchTerm) => (
              <ActionsWrapper
                id={savedSearchTerm.id}
                onClick={() => props.actions.toggle(savedSearchTerm.id)}
                actions={getWrapperActions(savedSearchTerm.id)}
              >
                <Label
                  color={
                    savedSearchTerm.enabled ? "button.success" : "button.muted"
                  }
                  tooltip={savedSearchTerm.enabled ? "Disable" : "Enable"}
                  onClick={() => props.actions.toggle(savedSearchTerm.id)}
                >
                  {savedSearchTerm.term}
                </Label>
              </ActionsWrapper>
            )}
          </For>
        </div>
      ) : null}
    </>
  );
};

const Search: Component = () => {
  const [_e, { getNodes }] = useEngine();
  const params = useParams();
  const [formData, setFormData] = createSignal<IFormData>({
    searchTerm: "",
  });
  const [searchTerms, setSearchTerms] = createSignal<
    Record<string, ISearchTerm>
  >({});

  const getSavedSearchTermNodes = () =>
    createMemo(() => {
      const searchTermNodes = getNodes(params.projectId, (node) =>
        node.labels.includes("SearchTerm"),
      );
      setSearchTerms((existing) => ({
        ...existing,
        ...Object.fromEntries(
          searchTermNodes.map((node) => {
            const slug = slugify(node.payload.data as string);
            return [
              slug,
              {
                id: slug,
                term: node.payload.data as string,
                enabled: existing[slug]?.enabled || false,
                active: existing[slug]?.active || false,
                saved: true,
              },
            ];
          }),
        ),
      }));
    });

  onMount(() => {
    getSavedSearchTermNodes();
  });

  const getEnabledSearchTerms = createMemo<ISearchTerm[]>(() => {
    return Object.values(searchTerms()).filter((term) => term.enabled);
  });

  const handleSearchTermChange = (_name: string, data: IFormFieldValue) => {
    if (typeof data !== "string" || !data) {
      return;
    }
    setFormData({ ...formData, searchTerm: data });
  };

  const handleKeyUp = (event: KeyboardEvent) => {
    if (event.key === "Enter") {
      const searchTerm = formData().searchTerm;
      const slug = slugify(searchTerm);
      setSearchTerms({
        ...searchTerms(),
        [slug]: {
          id: slug,
          term: searchTerm,
          enabled: true,
          active: true,
          saved: searchTerms()[slug]?.saved || false,
        },
      });
      setFormData({ ...formData, searchTerm: "" });
    }
  };

  const toggleSearchTerm = (slug: string) => {
    setSearchTerms((existing) => ({
      ...existing,
      [slug]: {
        ...existing[slug],
        enabled: !existing[slug].enabled,
      },
    }));
  };

  const deactivateSearchTerm = (slug: string) => {
    setSearchTerms((existing) => {
      if (!existing[slug].saved) {
        delete existing[slug];
        return existing;
      }
      return {
        ...existing,
        [slug]: {
          ...existing[slug],
          active: false,
        },
      };
    });
  };

  const activateSearchTerm = (slug: string) => {
    setSearchTerms((existing) => ({
      ...existing,
      [slug]: {
        ...existing[slug],
        active: true,
        enabled: true,
      },
    }));
  };

  const saveSearchTerm = (slug: string) => {
    insertNode(params.projectId, {
      SearchTerm: searchTerms()[slug].term,
    } as NodeWrite);
    getSavedSearchTermNodes();
  };

  const deleteSearchTerm = (_slug: string) => {
    // Logic to delete search term node
  };

  const searchTermListActions = {
    toggle: toggleSearchTerm,
    save: saveSearchTerm,
    delete: deleteSearchTerm,
    activate: activateSearchTerm,
    deactivate: deactivateSearchTerm,
  };

  return (
    <>
      <Heading size={3}>Search</Heading>
      <TextInput
        name="query"
        placeholder={
          Object.keys(searchTerms()).length === 0
            ? "Search for…"
            : "Add another keyword…"
        }
        value={formData().searchTerm}
        onChange={handleSearchTermChange}
        onKeyUp={handleKeyUp}
        isEditable
      />
      <div class="flex flex-col gap-4">
        <SearchTermList
          searchTerms={searchTerms()}
          type="saved"
          actions={searchTermListActions}
        />
        <SearchTermList
          searchTerms={searchTerms()}
          type="active"
          actions={searchTermListActions}
        />

        {getEnabledSearchTerms().length > 0 ? (
          <SearchResults
            searchTerms={getEnabledSearchTerms().map(
              (searchTerm) => searchTerm.term,
            )}
          />
        ) : (
          <div class="px-3">Enter/enable a search term to get started</div>
        )}
      </div>
    </>
  );
};

export default Search;
