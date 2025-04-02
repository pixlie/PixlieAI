import { Component } from "solid-js";
import { useParams } from "@solidjs/router";
import { createStore } from "solid-js/store";
import { createNode } from "../../utils/api";
import { NodeWrite } from "../../api_types/NodeWrite";
import TextInput from "../interactable/TextInput";
import Button from "../interactable/Button";

interface IFormData {
  topic: string;
}

const ObjectiveForm: Component = () => {
  const params = useParams();
  const [formData, setFormData] = createStore<IFormData>({
    topic: "",
  });

  const handleChange = (_: any, value: string | number) => {
    setFormData((existing) => ({
      ...existing,
      topic: value as string,
    }));
  };

  const handleSubmit = async () => {
    createNode(params.projectId, {
      Objective: formData.topic,
    } as NodeWrite);
  };

  return (
    <div class="grid grid-cols-[1fr_auto] gap-x-2">
      <TextInput
        name="topic"
        placeholder="topic"
        isEditable
        onChange={handleChange}
        value={formData.topic}
      />

      <Button label="Add a topic" onClick={handleSubmit} />
    </div>
  );
};

export default ObjectiveForm;
