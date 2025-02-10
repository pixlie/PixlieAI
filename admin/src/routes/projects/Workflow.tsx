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
import { insertNode } from "../../utils/api.ts";

const labelTypes: string[] = ["Link", "Domain"];
type LabelType = (typeof labelTypes)[number];

interface ILinkFormData {
  url: string;
}

const LinkForm: Component = () => {
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
    } as NodeWrite);
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

  onMount(() => {
    fetchNodesByLabel(params.projectId, "AddedByUser");
  });

  // Nodes that have the label "AddedByUser" are the nodes that are in the workflow
  const getNodesInWorkflow = createMemo(() =>
    engine.isReady && "AddedByUser" in engine.nodeIdsByLabel
      ? engine.nodeIdsByLabel["AddedByUser"].map((x) => {
          if ("Link" in engine.nodes[x].payload) {
            return "Link";
          } else if ("Domain" in engine.nodes[x].payload) {
            return "Domain";
          }
        })
      : [],
  );

  const getTabs = createMemo(() =>
    getNodesInWorkflow().map((l) => ({
      label: `${l}(s)`,
      searchParamKey: "label",
      searchParamValue: l,
    })),
  );

  createEffect(() => {
    if (!!searchParams.label) {
      fetchNodesByLabel(params.projectId, searchParams.label as LabelType);
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

      {searchParams.label === "Link" ? (
        <div class="mt-6">
          <LinkForm />
        </div>
      ) : null}
    </div>
  );
};

export default Workflow;
