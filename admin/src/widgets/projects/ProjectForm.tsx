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
import { TopicWrite } from "../../api_types/TopicWrite.ts";
import { SearchTerm } from "../../api_types/SearchTerm.ts";


interface IProjectFormData {
  name: string;
  description: string;
  startingURLs: string; // One per line
  topics: string;
  searchTerms: string; // One per line
}

const ProjectForm: Component = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const [formData, setFormData] = createSignal<IProjectFormData>({
    name: "",
    description: "",
    startingURLs: "",
    topics: "",
    searchTerms: "",
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
        for (const topic of formData().topics.split(/[\r\n]+/)) {
          if (!!topic) {
            insertNode(item.uuid, {
              Topic: {
                topic,
              } as TopicWrite,
            } as NodeWrite);
          }
        }
        for (const searchTerm of formData().searchTerms.split(/[\r\n]+/)) {
          if (!!searchTerm) {
            insertNode(item.uuid, {
              SearchTerm: searchTerm as SearchTerm,
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
          <Label label="Topics to track (one per line)" for="createProjectTopics" />
          <TextArea
            id="createProjectTopics"
            name="topics"
            isEditable
            onChange={handleChange}
            value={formData().topics}
          />
        </div>

        <div>
          <Label label="Search terms of interest (one per line)" for="createProjectSearchTerms" />
          <TextArea
            id="createProjectSearchTerms"
            name="searchTerms"
            isEditable
            onChange={handleChange}
            value={formData().searchTerms}
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
