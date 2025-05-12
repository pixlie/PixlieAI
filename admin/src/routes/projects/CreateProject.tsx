import { useNavigate } from "@solidjs/router";
import { Component, createSignal, For } from "solid-js";
import { NodeWrite } from "../../api_types/NodeWrite";
import { Project } from "../../api_types/Project";
import { ProjectCreate } from "../../api_types/ProjectCreate";
import BackgroundImage from "../../assets/background.webp";
import { createEdge, createNode, getPixlieAIAPIRoot } from "../../utils/api";
import FormError from "../../widgets/interactable/FormError.tsx";
import FormLabel from "../../widgets/interactable/FormLabel";
import TextArea from "../../widgets/interactable/TextArea";
import Toggle from "../../widgets/interactable/Toggle";
// import PromptInput from "../../widgets/interactable/PromptInput.tsx";
import { ProjectSettingsWrite } from "../../api_types/ProjectSettingsWrite.ts";
import SendIcon from "../../assets/icons/tabler-arrow-up.svg";
import IconButton from "../../widgets/interactable/IconButton.tsx";
import LinkForm from "../../widgets/nodeForm/LinkForm.tsx";
import Heading from "../../widgets/typography/Heading.tsx";
import Paragraph from "../../widgets/typography/Paragraph.tsx";
import InfoPopOver from "./InfoPopOver.tsx";

interface IFormData {
  objective: string;
  // onlyExtractDataFromSpecifiedLinks: boolean;
  // onlyCrawlWithinDomainsOfSpecifiedLinks: boolean;
  // onlyCrawlDirectLinksFromSpecifiedLinks: boolean;
  projectSettings: ProjectSettingsWrite;
  startingLinks: string[];
}

interface IError {
  // Keys are field names or __form__ for form level error
  [key: string]: string;
}

const CreateProject: Component = () => {
  const navigate = useNavigate();
  const [formData, setFormData] = createSignal<IFormData>({
    objective: "",
    projectSettings: {
      extract_data_only_from_specified_links: false,
      crawl_within_domains_of_specified_links: false,
      crawl_direct_links_from_specified_links: false,
    },
    // onlyExtractDataFromSpecifiedLinks: false,
    // onlyCrawlWithinDomainsOfSpecifiedLinks: false,
    // onlyCrawlDirectLinksFromSpecifiedLinks: false,
    startingLinks: [],
  });
  const [formErrors, setFormErrors] = createSignal<IError>({});

  const handleTextChange = (name: string, value: string | number) => {
    setFormData({
      ...formData(),
      [name]: value,
    });
  };

  const handleProjectSettingsToggle = (name: string, value: boolean) => {
    setFormData({
      ...formData(),
      projectSettings: {
        ...formData().projectSettings,
        [name]: value,
      },
    });
  };

  const addLink = (_name: string, value: string) => {
    if (formData().startingLinks.includes(value)) {
      return;
    }

    setFormData({
      ...formData(),
      startingLinks: [...formData().startingLinks, value],
    });
  };

  const handleFormSubmit = async () => {
    if (!formData().objective || formData().objective.length === 0) {
      setFormErrors({
        ...formErrors(),
        objective: "Please enter an objective",
      });
    }

    if (Object.values(formData().projectSettings).some((value) => value)) {
      if (formData().startingLinks.length === 0) {
        setFormErrors({
          ...formErrors(),
          links: "Please add at least one link",
        });
      } else {
        // setFormErrors(
        //   Object.fromEntries(
        //     Object.entries(formErrors()).filter(([key]) => key !== "links"),
        //   ),
        // );
      }
    } else {
      // setFormErrors(
      //   Object.fromEntries(
      //     Object.entries(formErrors()).filter(([key]) => key !== "links"),
      //   ),
      // );
    }

    if (Object.keys(formErrors()).length > 0) {
      return;
    }

    let pixlieAIAPIRoot = getPixlieAIAPIRoot();
    let response = await fetch(`${pixlieAIAPIRoot}/api/projects`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({} as ProjectCreate),
    });
    if (!response.ok) {
      throw new Error("Failed to create project");
    }
    let project: Project = await response.json();
    // if (
    //   formData().onlyCrawlWithinDomainsOfSpecifiedLinks &&
    //   formData().startingLinks.length > 0
    // ) {
    // Create a node for ProjectSettings
    let projectSettingsNodeId = await createNode(project.uuid, {
      ProjectSettings: formData().projectSettings,
    });

    if (projectSettingsNodeId !== undefined) {
      // Create a node per link
      for (const link in formData().startingLinks) {
        let linkNodeId = await createNode(project.uuid, {
          Link: {
            url: link,
          },
        } as NodeWrite);

        if (linkNodeId !== undefined) {
          createEdge(project.uuid, {
            node_ids: [projectSettingsNodeId, linkNodeId],
            edge_labels: ["RelatedTo", "RelatedTo"],
          });
        }
      }

      // Create a node for Objective
      let objectiveNodeId = await createNode(project.uuid, {
        Objective: formData().objective,
      } as NodeWrite);

      if (objectiveNodeId !== undefined) {
        createEdge(project.uuid, {
          node_ids: [projectSettingsNodeId, objectiveNodeId],
          edge_labels: ["RelatedTo", "RelatedTo"],
        });
      }
    }

    navigate(`/p/${project.uuid}/workflow`);
  };

  return (
    <div class="flex flex-col w-full h-full justify-end pb-12 items-center relative">
      <div
        class="absolute inset-0 bg-center bg-contain bg-no-repeat opacity-50 z-0"
        style={{
          "background-image": `url(${BackgroundImage}`,
          "background-size": "contain",
          "background-position": "center",
          "background-repeat": "no-repeat",
          overflow: "hidden",
        }}
      />
      <div class="relative w-3/4 bg-white/40 backdrop-blur-lg rounded-3xl border-slate-100 border shadow-lg p-5 flex flex-col gap-4">
        <Heading size={5}>Start a new Pixlie project</Heading>
        {/* <Heading size={6}>What do you want to extract from the web?</Heading> */}
        {/* <Paragraph size="sm">
          State your objective in plain English. Feel free to use topics and
          keywords that you care about. Pixlie will continue crawling the web as
          long as pages match your objective.
          Your objective in plain English using relevant topics &amp; keywords.
          Pixlie will find web pages matching your objective.
        </Paragraph> */}

        <TextArea
          id="projectObjective"
          name="objective"
          placeholder={
            "Describe your objective to search for web pages & identify " +
            "if they are relevant."
          }
          isEditable
          isRequired
          rows={4}
          onChange={handleTextChange}
          value={formData().objective}
        />
        <Paragraph size="sm">
          Feel free to specify known relevant links or include topics &amp;
          keywords you care about.
        </Paragraph>

        {/* <PromptInput
          id="projectObjective"
          name="objective"
          placeholder="Describe your objective..."
          isEditable
          onChange={handleTextChange}
          value={formData().objective}
        /> */}

        <div class="h-10 w-full flex items-center justify-between">
          <InfoPopOver />
          <IconButton
            onClick={() => handleFormSubmit()}
            name="Start a new project"
            icon={SendIcon}
            position="top"
            color="button.success"
          />
        </div>

        <FormError name="objective" errors={formErrors} />

        <div class="flex items-center gap-x-2">
          <Toggle
            name="extract_data_only_from_specified_links"
            value={
              formData().projectSettings.extract_data_only_from_specified_links
            }
            onChange={handleProjectSettingsToggle}
          />
          <FormLabel
            label="Only extract data from specified links"
            for="extract_data_only_from_specified_links"
          />
        </div>

        {!formData().projectSettings.extract_data_only_from_specified_links && (
          <>
            <div class="flex items-center gap-x-2">
              <Toggle
                name="crawl_within_domains_of_specified_links"
                value={
                  formData().projectSettings
                    .crawl_within_domains_of_specified_links
                }
                onChange={handleProjectSettingsToggle}
              />
              <FormLabel
                label="Only crawl within domains of specified links"
                for="crawl_within_domains_of_specified_links"
              />
            </div>

            <div class="flex items-center gap-x-2">
              <Toggle
                name="crawl_direct_links_from_specified_links"
                value={
                  formData().projectSettings
                    .crawl_direct_links_from_specified_links
                }
                onChange={handleProjectSettingsToggle}
              />
              <FormLabel
                label="Only crawl direct links from specified links"
                for="crawl_direct_links_from_specified_links"
              />
            </div>
          </>
        )}

        <div class="max-w-screen-sm flex-col space-y-2">
          <Heading size={3}>Links to crawl</Heading>
          <Paragraph size="sm">
            Optionally, you may specify a list of links and limit the crawl to
            these or their domains or links directly linked from these pages.
          </Paragraph>
          {formData().startingLinks.length > 0 && (
            <div class="flex flex-col gap-y-2 my-2">
              <For each={formData().startingLinks}>
                {(link) => <span class="">{link}</span>}
              </For>
            </div>
          )}

          <LinkForm name="url" onChange={addLink} />
        </div>

        <FormError name="links" errors={formErrors} />
      </div>
    </div>
  );
};

export default CreateProject;
