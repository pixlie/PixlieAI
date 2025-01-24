import { Component, createEffect, createMemo, onMount } from "solid-js";
import Heading from "../../widgets/typography/Heading";
import Tabs from "../../widgets/navigation/Tab";
import { useEngine } from "../../stores/engine.tsx";
import { useSearchParams } from "@solidjs/router";
import NodeListItem from "../../widgets/node/ListItem.tsx";
import TextInput from "../../widgets/interactable/TextInput.tsx";
import { createStore } from "solid-js/store";
import { Link } from "../../api_types/Link";
import Button from "../../widgets/interactable/Button.tsx";

const labelTypes: string[] = ["Link", "TextClassification"];
type LabelType = (typeof labelTypes)[number];

interface ILinkFormData {
  url: string;
}

const LinkForm: Component = () => {
  const [_, { insertNode }] = useEngine();
  const [formData, setFormData] = createStore<ILinkFormData>({
    url: "",
  });

  const handleChange = (_, value: string) => {
    setFormData((existing) => ({
      ...existing,
      url: value,
    }));
  };

  const handleSubmit = async () => {
    insertNode({
      Link: formData,
    } as NodeWrite).then((_) => {});
  };

  return (
    <div class="flex flex-col gap-y-2">
      <TextInput
        name="url"
        placeholder="https://"
        isEditable
        onChange={handleChange}
        value={formData.url}
      />
      <div>
        <Button label="Add a Link" onClick={handleSubmit} />
      </div>
    </div>
  );
};

const Workflow: Component = () => {
  const [engine, { fetchNodesByLabel }] = useEngine();
  const [searchParams] = useSearchParams();

  const getSelectNodeIds = createMemo<number[]>(() =>
    !!searchParams.label &&
    (searchParams.label as LabelType) in engine.nodeIdsByLabel
      ? engine.nodeIdsByLabel[searchParams.label as LabelType]
      : [],
  );

  const getTabs = createMemo(() =>
    labelTypes.map((l) => ({
      label: `${l}(s)`,
      searchParamKey: "label",
      searchParamValue: l,
    })),
  );

  onMount(() => {
    if (!!searchParams.label) {
      fetchNodesByLabel(searchParams.label as LabelType).then((_) => {});
    } else {
      fetchNodesByLabel("Link").then((_) => {});
    }
  });

  createEffect(() => {
    if (!!searchParams.label) {
      fetchNodesByLabel(searchParams.label as LabelType).then((_) => {});
    } else {
      fetchNodesByLabel("Link").then((_) => {});
    }
  });

  return (
    <div class="max-w-screen-sm">
      <Heading size={1}>Workflow</Heading>

      <Tabs tabs={getTabs()} />
      {!engine.isReady ? (
        <>Loading...</>
      ) : (
        <>
          {getSelectNodeIds().map((nodeId) => (
            <NodeListItem nodeId={nodeId} />
          ))}
        </>
      )}

      {searchParams.label === "Link" ? <LinkForm /> : null}
    </div>
  );
};

export default Workflow;
