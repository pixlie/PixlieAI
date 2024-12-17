import { Component } from "solid-js";
import Heading from "../../widgets/typography/Heading";
import Markdown from "../typography/Markdown";
import TextInput from "../interactable/TextInput";

const setupIntroduction = `
The storage directory is where we store the graph data, some AI/ML model data, etc.
[Gliner](https://github.com/gliner/gliner), which is one of the AI/ML tools, needs about 6 GB of space.

Please copy and paste the path to the directory where you want to store the data.
`;

const StorageDir: Component = () => {
  // The user has to set the storage directory
  return (
    <>
      <Heading size={3}>Storage Directory</Heading>
      <Markdown text={setupIntroduction} />

      <TextInput name="path_to_storage_dir" isEditable />
    </>
  );
};

export default StorageDir;
