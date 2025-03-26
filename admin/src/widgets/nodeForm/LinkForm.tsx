import { Component } from "solid-js";
import { useParams } from "@solidjs/router";
import { createStore } from "solid-js/store";
import { IFormFieldValue } from "../../utils/types";
import { insertNode } from "../../utils/api";
import { NodeWrite } from "../../api_types/NodeWrite";
import TextInput from "../interactable/TextInput";
import Button from "../interactable/Button";

interface IFormData {
  url: string;
}

const LinkForm: Component = () => {
  const params = useParams();
  const [formData, setFormData] = createStore<IFormData>({
    url: "",
  });

  const handleChange = (_: any, value: IFormFieldValue) => {
    setFormData((existing) => ({
      ...existing,
      term: value as string,
    }));
  };

  const handleSubmit = async () => {
    insertNode(params.projectId, {
      Link: formData,
    } as NodeWrite);
  };

  return (
    <div class="grid grid-cols-[1fr_auto] gap-x-2">
      <TextInput
        name="url"
        placeholder="https://"
        isEditable
        onChange={handleChange}
        value={formData.url}
      />

      <Button
        label="Add a link"
        colorTheme="secondary"
        onClick={handleSubmit}
      />
    </div>
  );
};

export default LinkForm;
