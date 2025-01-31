import { Component } from "solid-js";
import TextInput from "../interactable/TextInput.tsx";

const CreateProject: Component = () => {
  return (
    <div>
      <TextInput name="project_name" />
    </div>
  );
};

export default CreateProject;
