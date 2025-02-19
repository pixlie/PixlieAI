import { Component, createEffect, createMemo, onMount } from "solid-js";
import Heading from "../../widgets/typography/Heading";
import Tabs from "../../widgets/navigation/Tab";
import { useEngine } from "../../stores/engine";
import { useParams, useSearchParams } from "@solidjs/router";
import NodeGrid from "../../widgets/node/NodeGrid.tsx";
import TextInput from "../../widgets/interactable/TextInput.tsx";
import { createStore } from "solid-js/store";
import Button from "../../widgets/interactable/Button.tsx";
import { NodeWrite } from "../../api_types/NodeWrite.ts";
import { IFormFieldValue } from "../../utils/types.tsx";
import { insertNode } from "../../utils/api.ts";
import Paragraph from "../../widgets/typography/Paragraph.tsx";

const labelTypes: string[] = ["Link", "SearchKeyword"];
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

  const getProject = createMemo(() => {
    if (!!params.projectId && params.projectId in engine.projects) {
      return engine.projects[params.projectId];
    }
    return undefined;
  });

  const getSelectNodeIds = createMemo<number[]>(() => {
    if (
      getProject() &&
      !!searchParams.label &&
      (searchParams.label as LabelType) in getProject()!.nodeIdsByLabel
    ) {
      // Only select nodes that have AddedByUser label
      return getProject()!.nodeIdsByLabel[
        searchParams.label as LabelType
      ].filter((nodeId) =>
        getProject()!.nodes[nodeId].labels.includes("AddedByUser"),
      );
    } else {
      return [];
    }
  });

  onMount(() => {
    if (params.projectId) {
      fetchNodesByLabel(params.projectId, "AddedByUser");
    }
  });

  const getTabs = createMemo(() =>
    labelTypes.map((l) => ({
      label: `${l}(s)`,
      searchParamKey: "label",
      searchParamValue: l,
    })),
  );

  createEffect(() => {
    if (params.projectId && !!searchParams.label) {
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
      <Heading size={3}>Workflow</Heading>
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
