import { Component, createSignal } from "solid-js";
import { IFormFieldValue } from "../../utils/types";
import { getPixlieAIAPIRoot, insertNode } from "../../utils/api";
import { ProjectCreate } from "../../api_types/ProjectCreate";
import { Project } from "../../api_types/Project";
import { NodeWrite } from "../../api_types/NodeWrite";
import { useNavigate } from "@solidjs/router";
import ToolTip from "../../widgets/navigation/ToolTip";
import InfoPopOver from "./InfoPopOver";
import PromptInput from "../../widgets/interactable/PromptInput";
import SendIcon from "../../assets/icons/tabler-arrow-up.svg";

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
      class="flex flex-col w-full h-full justify-end pb-12 z-20 items-center relative"
      style={{
        "background-image":
          "url('https://pixlie.com/_astro/hero-image.DdgBYhys_2wzNdY.webp')",
        "background-size": "contain",
        "background-position": "center",
        "background-repeat": "no-repeat",
        overflow: "hidden",
      }}
    >
      <div class="relative w-1/2 items-end bg-white/40 backdrop-blur-md rounded-3xl border-slate-100 border shadow-lg p-4 flex flex-col gap-4">
        <PromptInput
          id="projectObjective"
          name="objective"
          placeholder="Describe your objective..."
          isEditable
          onChange={handleChange}
          value={formData().objective}
        />

        <div class="h-10 w-full flex items-center justify-between">
          <InfoPopOver />
          <div
            class="rounded-full shadow transition duration-150 ease-out  scale-95"
            style={{ "background-color": "#00C853" }}
          >
            <ToolTip text="Send">
              <button
                onClick={handleFormSubmit}
                class="rounded-full p-2 self-end w-10 text-white hover:bg-green-600 cursor-pointer"
              >
                <SendIcon />
              </button>
            </ToolTip>
          </div>
        </div>
      </div>
    </div>
  );
};

export default CreateWorkflow;
