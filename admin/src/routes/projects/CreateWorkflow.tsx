import { Component, createSignal } from "solid-js";
import { IFormFieldValue } from "../../utils/types";
import { getPixlieAIAPIRoot, insertNode } from "../../utils/api";
import { ProjectCreate } from "../../api_types/ProjectCreate";
import { Project } from "../../api_types/Project";
import { NodeWrite } from "../../api_types/NodeWrite";
import { useNavigate } from "@solidjs/router";
import ToolTip from "../../widgets/navigation/ToolTip";
import Icon from "../../widgets/interactable/Icon";
import InfoPopOver from "./InfoPopOver";
import PromptInput from "../../widgets/interactable/PromptInput";

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
    <div
      class="flex w-full h-full justify-center items-end grid grid-cols-3 grid-rows-3 relative"
      style={{
        "background-image":
          "url('https://pixlie.com/_astro/hero-image.DdgBYhys_2wzNdY.webp')",
        "background-size": "contain",
        "background-position": "center",
        "background-repeat": "no-repeat",
        overflow: "hidden",
      }}
    >
      <div />
      {/* top row center column */}
      <div class="flex items-center mb-1 gap-2">
        <p class="font-medium bg-white rounded-full">Objective</p>
        <div class="bg-white rounded-full">
          <InfoPopOver />
        </div>
      </div>
      <div />
      <div />
      {/* center row center column */}
      <div class="flex justify-center items-center w-full h-full">
        <PromptInput
          id="projectObjective"
          name="objective"
          placeholder="Describe your objective..."
          isEditable
          onChange={handleChange}
          value={formData().objective}
        />
      </div>
      {/* center row left column */}
      <div class="m-2 bg-white rounded-full">
        <button onClick={handleFormSubmit} aria-label="Send">
          <ToolTip text="Send">
            <div class="flex items-center p-2 hover:bg-slate-200 rounded-full">
              <div class="bg-blue-600 h-6 w-6 rounded-full flex text-white justify-center items-center">
                <Icon name="arrow-up" size={16} />
              </div>
            </div>
          </ToolTip>
        </button>
      </div>
      <div />
      <div />
      <div />
    </div>
  );
};

export default CreateWorkflow;
