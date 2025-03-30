import { Component, createSignal } from "solid-js";
import LinkForm from "../../widgets/nodeForm/LinkForm";
import Heading from "../../widgets/typography/Heading";
import { WorkflowSidebar } from "./Workflow";
import { getPixlieAIAPIRoot, insertNode } from "../../utils/api";
import { ProjectCreate } from "../../api_types/ProjectCreate";
import { Project } from "../../api_types/Project";
import { NodeWrite } from "../../api_types/NodeWrite";
import { useNavigate } from "@solidjs/router";
import Paragraph from "../../widgets/typography/Paragraph";
import TextArea from "../../widgets/interactable/TextArea";
import Button from "../../widgets/interactable/Button";
import Toggle from "../../widgets/interactable/Toggle";
import Label from "../../widgets/interactable/Label.tsx";

interface IFormData {
  objective: string;
  hasStartingLinks: boolean;
  startingLinks: string[];
}

const CreateProject: Component = () => {
  const navigate = useNavigate();
  const [formData, setFormData] = createSignal<IFormData>({
    objective: "",
    hasStartingLinks: false,
    startingLinks: [],
  });

  const handleTextChange = (name: string, value: string | number) => {
    setFormData({
      ...formData(),
      [name]: value,
    });
  };

  const handleToggle = (name: string, value: boolean) => {
    setFormData({
      ...formData(),
      [name]: value,
    });
  };

  const addLink = (_name: string, value: string) => {
    setFormData({
      ...formData(),
      startingLinks: [...formData().startingLinks, value],
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
        for (const topic of formData().objective.split(/[\r\n]+/)) {
          if (!!topic) {
            insertNode(item.uuid, {
              Objective: topic as string,
            } as NodeWrite);
          }
        }

        navigate(`/p/${item.uuid}/workflow`);
      });
    });
  };

  return (
    <div class="flex gap-x-12">
      <div class="w-48" />

      <div class="flex-1 flex flex-col">
        <div class="max-w-screen-md space-y-4">
          <Heading size={3}>Objective</Heading>

          <Paragraph size="sm">
            What do you want to extract from the web? You may state this in
            plain English. Feel free to use topics and keywords that you care
            about. Pixlie will continue crawling the web as long as pages match
            your objective.
          </Paragraph>

          <TextArea
            id="projectObjective"
            name="objective"
            isEditable
            onChange={handleTextChange}
            value={formData().objective}
          />

          <div class="flex items-center gap-x-2">
            <Toggle
              name="hasStartingLinks"
              value={formData().hasStartingLinks}
              onChange={handleToggle}
            />
            <Label
              label="Manually specify links to crawl"
              for="hasStartingLinks"
            />
          </div>

          {formData().hasStartingLinks && (
            <div class="max-w-screen-sm">
              <Heading size={3}>Links to crawl</Heading>
              <Paragraph size="sm">Please add one link per line.</Paragraph>
              <LinkForm name="url" onChange={addLink} />
            </div>
          )}

          <div class="pt-6 flex space-x-3">
            <div class="flex-1" />
            <Button label="Cancel" href="/" />
            <Button
              label="Save"
              colorTheme="success"
              onClick={handleFormSubmit}
            />
          </div>
        </div>
      </div>

      <WorkflowSidebar />
    </div>
  );
};

export default CreateProject;
