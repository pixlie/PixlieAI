import { Component, createSignal } from "solid-js";
import TextArea from "../interactable/TextArea";
import Drawer from "../overlay/Drawer";
import { IFormFieldValue } from "../../utils/types";
import Button from "../interactable/Button";
import Label from "../interactable/Label";
import { ProjectCreate } from "../../api_types/ProjectCreate";
import { useLocation, useNavigate } from "@solidjs/router";
import { getPixlieAIAPIRoot, insertNode } from "../../utils/api";
import { Project } from "../../api_types/Project";
import { NodeWrite } from "../../api_types/NodeWrite";
import Paragraph from "../typography/Paragraph";

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

  const Content: Component = () => {
    return (
      <div class="space-y-2">
        <Label label="Objective" for="projectObjective" />
        <Paragraph size="sm">
          What do you want to extract from the web? You may state this in plain
          English. Feel free to use topics and keywords that you care about.
          Pixlie will continue crawling the web as long as pages match your
          objective.
        </Paragraph>
        <TextArea
          id="projectObjective"
          name="objective"
          isEditable
          onChange={handleChange}
          value={formData().objective}
        />

        {/*<div>*/}
        {/*  <Label*/}
        {/*    label={`Starting URLs (optional, one per line)`}*/}
        {/*    for="createProjectStartingURLs"*/}
        {/*  />*/}
        {/*  <TextArea*/}
        {/*    id="createProjectStartingURLs"*/}
        {/*    name="startingURLs"*/}
        {/*    isEditable*/}
        {/*    onChange={handleChange}*/}
        {/*    value={formData().startingURLs}*/}
        {/*  />*/}
        {/*</div>*/}
      </div>
    );
  };

  const Footer: Component = () => {
    return (
      <div class="space-x-3">
        <Button size="sm" label="Cancel" color="bg-red-500" href="/" />
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
