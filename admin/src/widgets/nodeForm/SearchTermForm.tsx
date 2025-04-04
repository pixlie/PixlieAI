import { Component } from "solid-js";
import { useParams } from "@solidjs/router";
import { createStore } from "solid-js/store";
import { createNode } from "../../utils/api";
import { NodeWrite } from "../../api_types/NodeWrite";
import TextInput from "../interactable/TextInput";
import Button from "../interactable/Button";

interface IFormData {
  term: string;
}

const SearchTermForm: Component = () => {
  const params = useParams();
  const [formData, setFormData] = createStore<IFormData>({
    term: "",
  });

  const handleChange = (_: any, value: string | number) => {
    setFormData((existing) => ({
      ...existing,
      term: value as string,
    }));
  };

  const handleSubmit = async () => {
    await createNode(params.projectId, {
      SearchTerm: formData.term,
    } as NodeWrite);
  };

  return (
    <div class="grid grid-cols-[1fr_auto] gap-x-2">
      <TextInput
        name="url"
        placeholder="search term"
        isEditable
        onChange={handleChange}
        value={formData.term}
      />

      <Button label="Add a search term" onClick={handleSubmit} />
    </div>
  );
};

export default SearchTermForm;
