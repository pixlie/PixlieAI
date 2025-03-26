import { Component, createSignal } from "solid-js";
// import LinkForm from "../../widgets/nodeForm/LinkForm";
import Heading from "../../widgets/typography/Heading";
import { WorkflowSidebar } from "./Workflow";
import { IFormFieldValue } from "../../utils/types";
import { getPixlieAIAPIRoot, insertNode } from "../../utils/api";
import { ProjectCreate } from "../../api_types/ProjectCreate";
import { Project } from "../../api_types/Project";
import { NodeWrite } from "../../api_types/NodeWrite";
import { useNavigate } from "@solidjs/router";
import Paragraph from "../../widgets/typography/Paragraph";
import TextArea from "../../widgets/interactable/TextArea";
import Button from "../../widgets/interactable/Button";
// import Toggle from "../../widgets/interactable/Toggle";

interface IFormData {
  objective: string;
  hasStartingLinks: boolean;
  startingLinks: string[];
}

const CreateWorkflow: Component = () => {
  const navigate = useNavigate();
  const [formData, setFormData] = createSignal<IFormData>({
    objective: "",
    hasStartingLinks: false,
    startingLinks: [],
  });

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
        <div class="max-w-screen-md space-y-2">
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
            onChange={handleChange}
            value={formData().objective}
          />

          {/* <div class="flex items-center gap-x-2">
            <Toggle />
            Specify links to start crawling from.
          </div>

          <Heading size={3}>Starting links</Heading>
          <NodeGrid nodeType={"Link"} source={} />
          <div class="max-w-screen-sm">
            <LinkForm />
          </div> */}

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

export default CreateWorkflow;
