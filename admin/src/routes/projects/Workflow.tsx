import { Component, createEffect, createMemo, onMount } from "solid-js";
import Tabs from "../../widgets/navigation/Tab";
import { useEngine } from "../../stores/engine.tsx";
import { useParams, useSearchParams } from "@solidjs/router";
import NodeGrid from "../../widgets/node/NodeGrid.tsx";
import TextInput from "../../widgets/interactable/TextInput.tsx";
import { createStore } from "solid-js/store";
import Button from "../../widgets/interactable/Button.tsx";
import { NodeWrite } from "../../api_types/NodeWrite.ts";
import { IFormFieldValue } from "../../utils/types.tsx";
import { insertNode } from "../../utils/api.ts";
import Paragraph from "../../widgets/typography/Paragraph.tsx";

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

  type NodesInWorkflow = "Link";
  // Nodes that have the label "AddedByUser" are the nodes that are in the workflow
  const getNodesInWorkflow = createMemo(
    (prev: Array<NodesInWorkflow>): Array<NodesInWorkflow> => {
      if ("AddedByUser" in engine.nodeIdsByLabel) {
        return engine.nodeIdsByLabel["AddedByUser"]
          .map((x) => {
            if (engine.nodes[x].payload.type === "Link") {
              return "Link";
            }
          })
          .filter((x) => x !== undefined) as Array<NodesInWorkflow>;
      }
      return prev;
    },
    [],
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

  const getNodeTypeFromSearchParam = createMemo(() => {
    if (!!searchParams.label) {
      return searchParams.label as LabelType;
    }
    return undefined;
  });

  return (
    <>
      <div class="max-w-screen-sm mb-8">
        <Paragraph>
          Pixlie can monitor keywords on multiple URLs. If you add a URL from a
          website, then Pixlie will crawl all URLs on that website.
        </Paragraph>
      </div>

      <Tabs tabs={getTabs()} />
      <NodeGrid
        nodeType={getNodeTypeFromSearchParam()}
        source={getSelectNodeIds}
      />

      {searchParams.label === "Link" ? (
        <div class="mt-6 max-w-screen-sm">
          <LinkForm />
        </div>
      ) : null}
    </>
  );
};

export default Workflow;
