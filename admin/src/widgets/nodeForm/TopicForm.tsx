import { Component } from "solid-js";
import { useParams } from "@solidjs/router";
import { createStore } from "solid-js/store";
import { IFormFieldValue } from "../../utils/types";
import { insertNode } from "../../utils/api";
import { NodeWrite } from "../../api_types/NodeWrite";
import TextInput from "../interactable/TextInput";
import Button from "../interactable/Button";

interface IFormData {
  topic: string;
}

const TopicForm: Component = () => {
  const params = useParams();
  const [formData, setFormData] = createStore<IFormData>({
    topic: "",
  });

  const handleChange = (_: any, value: IFormFieldValue) => {
    setFormData((existing) => ({
      ...existing,
      topic: value as string,
    }));
  };

  const handleSubmit = async () => {
    insertNode(params.projectId, {
      Topic: formData.topic,
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

export default TopicForm;
