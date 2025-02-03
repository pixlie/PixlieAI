import { Component } from "solid-js";
import TextInput from "../interactable/TextInput.tsx";
import TextArea from "../interactable/TextArea.tsx";
import Drawer from "../overlay/Drawer.tsx";
import { DisplayAs } from "../../utils/types.tsx";
import Button from "../interactable/Button.tsx";
import Label from "../interactable/Label.tsx";
import Heading from "../typography/Heading.tsx";

interface IPropTypes {
  displayAs: DisplayAs;
  onClose?: () => void;
  projectId?: string;
}

const ProjectForm: Component<IPropTypes> = (props) => {
  const title = "Create a project";
  const subtitle =
    "Create a project to crawl website(s); monitor keywords or semantic information and extract them.";

  const Content: Component = () => {
    return (
      <div class="space-y-8">
        <div>
          <Label label="Project name" />
          <TextInput name="project_name" isEditable />
        </div>

        <div>
          <Label label={`Starting URLs (one per line)`} />
          <TextArea name="starting_urls" isEditable />
        </div>

        <div>
          <Label label="Keywords of interest (one per line)" />
          <TextArea name="webpage_keywords" isEditable />
        </div>
      </div>
    );
  };

  const Footer: Component = () => {
    return (
      <div class="space-x-2">
        <Button size="sm" label="Cancel" onClick={props.onClose} />
        <Button size="sm" label="Save" onClick={() => {}} />
      </div>
    );
  };

  return (
    <>
      {props.displayAs === "Drawer" ? (
        <Drawer
          title={title}
          subtitle={subtitle}
          content={<Content />}
          footer={<Footer />}
          onClose={props.onClose}
        />
      ) : (
        <>
          <Heading size={2}>{title}</Heading>
          <Content />
        </>
      )}
    </>
  );
};

export default ProjectForm;
