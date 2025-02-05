import { Component, createEffect, createMemo, onMount } from "solid-js";
import Heading from "../../widgets/typography/Heading";
import Tabs from "../../widgets/navigation/Tab";
import { useEngine } from "../../stores/engine.tsx";
import { useParams, useSearchParams } from "@solidjs/router";
import NodeListItem from "../../widgets/node/ListItem.tsx";
import TextInput from "../../widgets/interactable/TextInput.tsx";
import { createStore } from "solid-js/store";
import Button from "../../widgets/interactable/Button.tsx";
import { NodeWrite } from "../../api_types/NodeWrite.ts";
import { IFormFieldValue } from "../../utils/types.tsx";

const labelTypes: string[] = ["Link", "TextClassification"];
type LabelType = (typeof labelTypes)[number];

interface ILinkFormData {
  url: string;
}

const LinkForm: Component = () => {
  const [_, { insertNode }] = useEngine();
  const params = useParams();
  const [formData, setFormData] = createStore<ILinkFormData>({
    url: "",
  });

  const handleChange = (_: any, value: IFormFieldValue) => {
    setFormData((existing) => ({
      ...existing,
      url: value as string,
    }));
  };

  const handleSubmit = async () => {
    insertNode(params.projectId, {
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
  const params = useParams();

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
      fetchNodesByLabel(params.projectId, searchParams.label as LabelType).then(
        (_) => {},
      );
    } else {
      fetchNodesByLabel(params.projectId, "Link").then((_) => {});
    }
  });

  createEffect(() => {
    if (!!searchParams.label) {
      fetchNodesByLabel(params.projectId, searchParams.label as LabelType).then(
        (_) => {},
      );
    } else {
      fetchNodesByLabel(params.projectId, "Link").then((_) => {});
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
