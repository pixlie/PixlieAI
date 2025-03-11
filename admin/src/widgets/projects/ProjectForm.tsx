import { Component, createSignal } from "solid-js";
import TextInput from "../interactable/TextInput.tsx";
import TextArea from "../interactable/TextArea.tsx";
import Drawer from "../overlay/Drawer.tsx";
import { IFormFieldValue } from "../../utils/types.tsx";
import Button from "../interactable/Button.tsx";
import Label from "../interactable/Label.tsx";
import { ProjectCreate } from "../../api_types/ProjectCreate.ts";
import { LinkWrite } from "../../api_types/LinkWrite.ts";
import { useLocation, useNavigate } from "@solidjs/router";
import { getPixlieAIAPIRoot, insertNode } from "../../utils/api.ts";
import { Project } from "../../api_types/Project.ts";
import { NodeWrite } from "../../api_types/NodeWrite.ts";
import { Topic } from "../../api_types/Topic.ts";
import Paragraph from "../typography/Paragraph.tsx";

interface IProjectFormData {
  objective: string;
  startingURLs: string; // One per line
}

const ProjectForm: Component = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const [formData, setFormData] = createSignal<IProjectFormData>({
    objective: "",
    startingURLs: "",
  });
  const title = "Start a web research project";
  const subtitle =
    "Set an objective which will guide the crawler and data extraction.";

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
      body: JSON.stringify({} as ProjectCreate),
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
        for (const topic of formData().objective.split(/[\r\n]+/)) {
          if (!!topic) {
            insertNode(item.uuid, {
              Topic: topic as Topic,
            } as NodeWrite);
          }
        }

        navigate(`/p/${item.uuid}/workflow`);
      });
    });
  };

  const Content: Component = () => {
    return (
      <div class="space-y-8">
        <Paragraph size="sm">
          What do you want to extract from the web? You may state this in plain
          English. Feel free to use keywords, topics, or search terms. Pixlie
          will crawl the web and extract information that relates to your
          objective.
        </Paragraph>

        <div>
          <Label label="Objective" for="projectObjective" />
          <TextArea
            id="projectObjective"
            name="objective"
            isEditable
            onChange={handleChange}
            value={formData().objective}
          />
        </div>

        <div>
          <Label
            label={`Starting URLs (optional, one per line)`}
            for="createProjectStartingURLs"
          />
          <TextArea
            id="createProjectStartingURLs"
            name="startingURLs"
            isEditable
            onChange={handleChange}
            value={formData().startingURLs}
          />
        </div>
      </div>
    );
  };

  const Footer: Component = () => {
    return (
      <div class="space-x-3">
        <Button
          size="sm"
          label="Cancel"
          color="bg-red-500"
          onClick={() => navigate(location.pathname)}
        />
        <Button size="sm" label="Save" onClick={handleFormSubmit} />
      </div>
    );
  };

  return (
    <>
      <div class="relative">
        <Drawer
          title={title}
          subtitle={subtitle}
          content={<Content />}
          footer={<Footer />}
          onClose={() => navigate(location.pathname)}
        />
      </div>
    </>
  );
};

export default ProjectForm;
