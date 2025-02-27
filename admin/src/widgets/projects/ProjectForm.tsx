import { Component, createSignal } from "solid-js";
import TextInput from "../interactable/TextInput.tsx";
import TextArea from "../interactable/TextArea.tsx";
import Drawer from "../overlay/Drawer.tsx";
import { DisplayAs, IFormFieldValue } from "../../utils/types.tsx";
import Button from "../interactable/Button.tsx";
import Label from "../interactable/Label.tsx";
import Heading from "../typography/Heading.tsx";
import { ProjectCreate } from "../../api_types/ProjectCreate.ts";
import { LinkWrite } from "../../api_types/LinkWrite.ts";
import { useNavigate } from "@solidjs/router";
import { getPixlieAIAPIRoot, insertNode } from "../../utils/api.ts";
import { Project } from "../../api_types/Project.ts";
import { NodeWrite } from "../../api_types/NodeWrite.ts";
import { TopicWrite } from "../../api_types/TopicWrite.ts";

interface IPropTypes {
  displayAs: DisplayAs;
  onClose?: () => void;
  projectId?: string;
}

interface IProjectFormData {
  name: string;
  description: string;
  startingURLs: string; // One per line
  topic: string;
  webpageKeywords: string; // One per line
}

const ProjectForm: Component<IPropTypes> = (props) => {
  const navigate = useNavigate();
  const [formData, setFormData] = createSignal<IProjectFormData>({
    name: "",
    description: "",
    startingURLs: "",
    topic: "",
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
    let pixlieAIAPIRoot = getPixlieAIAPIRoot();
    fetch(`${pixlieAIAPIRoot}/api/projects`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        name: formData().name,
        description: formData().description,
      } as ProjectCreate),
    }).then((response) => {
      if (!response.ok) {
        throw new Error("Failed to save settings");
      }
      response.json().then((item: Project) => {
        for (const url of formData().startingURLs.split(/[\r\n]+/)) {
          if (!url || url.length === 0) continue;
          insertNode(item.uuid, {
            Link: {
              url,
            } as LinkWrite,
          } as NodeWrite);
        }

        if (!!formData().topic) {
          insertNode(item.uuid, {
            Topic: {
              topic: formData().topic
            } as TopicWrite,
          } as NodeWrite);
        }

        navigate(`/p/${item.uuid}/workflow`);
      });
    });
  };

  const Content: Component = () => {
    return (
      <div class="space-y-8">
        <div>
          <Label label="Project name" for="createProjectName" />
          <TextInput
            id="createProjectName"
            name="name"
            isEditable
            onChange={handleChange}
            value={formData().name}
            autocomplete={false}
          />
        </div>

        <div>
          <Label label={`Starting URLs (one per line)`} for="createProjectStartingURLs" />
          <TextArea
            id="createProjectStartingURLs"
            name="startingURLs"
            isEditable
            onChange={handleChange}
            value={formData().startingURLs}
          />
        </div>

        <div>
          <Label label="Topic to track" for="createProjectTopic" />
          <TextInput
            id="createProjectTopic"
            name="topic"
            isEditable
            onChange={handleChange}
            value={formData().topic}
            autocomplete={false}
          />
        </div>

        {/* <div>
          <Label label="Keywords of interest (one per line)" />
          <TextArea
            name="webpageKeywords"
            isEditable
            onChange={handleChange}
            value={formData().webpageKeywords}
          />
        </div> */}
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
