import { Component, createSignal } from "solid-js";
import TextInput from "../interactable/TextInput.tsx";
import TextArea from "../interactable/TextArea.tsx";
import Drawer from "../overlay/Drawer.tsx";
import { DisplayAs, IFormFieldValue } from "../../utils/types.tsx";
import Button from "../interactable/Button.tsx";
import Label from "../interactable/Label.tsx";
import Heading from "../typography/Heading.tsx";
import { useWorkspace } from "../../stores/workspace.tsx";
import { useEngine } from "../../stores/engine.tsx";
import { ProjectCreate } from "../../api_types/ProjectCreate.ts";
import { LinkWrite } from "../../api_types/LinkWrite.ts";
import { useNavigate } from "@solidjs/router";

interface IPropTypes {
  displayAs: DisplayAs;
  onClose?: () => void;
  projectId?: string;
}

interface IProjectFormData {
  name: string;
  description: string;
  startingURLs: string; // One per line
  webpageKeywords: string; // One per line
}

const ProjectForm: Component<IPropTypes> = (props) => {
  const [_w, { createProject }] = useWorkspace();
  const [_e, { insertNode }] = useEngine();
  const navigate = useNavigate();
  const [formData, setFormData] = createSignal<IProjectFormData>({
    name: "",
    description: "",
    startingURLs: "",
    webpageKeywords: "",
  });
  const title = "Create a project";
  const subtitle =
    "Create a project to crawl website(s); monitor keywords or semantic information and extract them.";

  const handleChange = (name: string, value: IFormFieldValue) => {
    setFormData({
      ...formData(),
      [name]: value,
    });
  };

  const handleFormSubmit = () => {
    createProject({
      name: formData().name,
      description: formData().description,
    } as ProjectCreate).then((item) => {
      let promises = [];
      for (const url of formData().startingURLs.split(/[\r\n]+/)) {
        if (!url || url.length === 0) continue;
        promises.push(
          insertNode(item.uuid, {
            Link: {
              url,
            } as LinkWrite,
          }),
        );
      }

      Promise.allSettled(promises).then(() => {
        navigate(`/p/${item.uuid}/workflow`);
      });
    });
  };

  const Content: Component = () => {
    return (
      <div class="space-y-8">
        <div>
          <Label label="Project name" />
          <TextInput
            name="name"
            isEditable
            onChange={handleChange}
            value={formData().name}
          />
        </div>

        <div>
          <Label label={`Starting URLs (one per line)`} />
          <TextArea
            name="startingURLs"
            isEditable
            onChange={handleChange}
            value={formData().startingURLs}
          />
        </div>

        <div>
          <Label label="Keywords of interest (one per line)" />
          <TextArea
            name="webpageKeywords"
            isEditable
            onChange={handleChange}
            value={formData().webpageKeywords}
          />
        </div>
      </div>
    );
  };

  const Footer: Component = () => {
    return (
      <div class="space-x-2">
        <Button size="sm" label="Cancel" onClick={props.onClose} />
        <Button size="sm" label="Save" onClick={handleFormSubmit} />
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
