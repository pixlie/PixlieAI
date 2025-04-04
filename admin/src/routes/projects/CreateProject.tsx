import { Component, createSignal, For } from "solid-js";
import LinkForm from "../../widgets/nodeForm/LinkForm";
import Heading from "../../widgets/typography/Heading";
import { WorkflowSidebar } from "./Workflow";
import { getPixlieAIAPIRoot, createNode, createEdge } from "../../utils/api";
import { ProjectCreate } from "../../api_types/ProjectCreate";
import { Project } from "../../api_types/Project";
import { NodeWrite } from "../../api_types/NodeWrite";
import { useNavigate } from "@solidjs/router";
import Paragraph from "../../widgets/typography/Paragraph";
import TextArea from "../../widgets/interactable/TextArea";
import Button from "../../widgets/interactable/Button";
import Toggle from "../../widgets/interactable/Toggle";
import FormLabel from "../../widgets/interactable/FormLabel";
import FormError from "../../widgets/interactable/FormError.tsx";

interface IFormData {
  objective: string;
  onlyExtractDataFromSpecifiedLinks: boolean;
  onlyCrawlWithinDomainsOfSpecifiedLinks: boolean;
  onlyCrawlDirectLinksFromSpecifiedLinks: boolean;
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
    onlyExtractDataFromSpecifiedLinks: false,
    onlyCrawlWithinDomainsOfSpecifiedLinks: false,
    onlyCrawlDirectLinksFromSpecifiedLinks: false,
    startingLinks: [],
  });
  const [formErrors, setFormErrors] = createSignal<IError>({});

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

    if (formData().onlyCrawlWithinDomainsOfSpecifiedLinks) {
      if (formData().startingLinks.length === 0) {
        setFormErrors({
          ...formErrors(),
          links: "Please add at least one link",
        });
      } else {
        setFormErrors(
          Object.fromEntries(
            Object.entries(formErrors()).filter(([key]) => key !== "links"),
          ),
        );
      }
    } else {
      setFormErrors(
        Object.fromEntries(
          Object.entries(formErrors()).filter(([key]) => key !== "links"),
        ),
      );
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
    if (
      formData().onlyCrawlWithinDomainsOfSpecifiedLinks &&
      formData().startingLinks.length > 0
    ) {
      // Create a node for ProjectSettings
      let projectSettingsNodeId = await createNode(project.uuid, {
        ProjectSettings: {
          extract_data_only_from_specified_links:
            formData().onlyExtractDataFromSpecifiedLinks,
          crawl_direct_links_from_specified_links:
            formData().onlyCrawlDirectLinksFromSpecifiedLinks,
          crawl_within_domains_of_specified_links:
            formData().onlyCrawlWithinDomainsOfSpecifiedLinks,
        },
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
    } else {
      // Create a node for Objective without ProjectSettings
      await createNode(project.uuid, {
        Objective: formData().objective,
      } as NodeWrite);
    }

    navigate(`/p/${project.uuid}/workflow`);
  };

  return (
    <div class="flex gap-x-12">
      <div class="w-48" />

      <div class="flex-1 flex flex-col">
        <div class="max-w-screen-md space-y-4">
          <Heading size={1}>Objective</Heading>

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
          <FormError name="objective" errors={formErrors} />

          <div class="flex items-center gap-x-2">
            <Toggle
              name="onlyExtractDataFromSpecifiedLinks"
              value={formData().onlyExtractDataFromSpecifiedLinks}
              onChange={handleToggle}
            />
            <FormLabel
              label="Only extract data from specified links"
              for="onlyExtractDataFromSpecifiedLinks"
            />
          </div>

          {!formData().onlyExtractDataFromSpecifiedLinks && (
            <>
              <div class="flex items-center gap-x-2">
                <Toggle
                  name="onlyCrawlWithinDomainsOfSpecifiedLinks"
                  value={formData().onlyCrawlWithinDomainsOfSpecifiedLinks}
                  onChange={handleToggle}
                />
                <FormLabel
                  label="Only crawl within domains of specified links"
                  for="onlyCrawlWithinDomainsOfSpecifiedLinks"
                />
              </div>

              <div class="flex items-center gap-x-2">
                <Toggle
                  name="onlyCrawlDirectLinksFromSpecifiedLinks"
                  value={formData().onlyCrawlDirectLinksFromSpecifiedLinks}
                  onChange={handleToggle}
                />
                <FormLabel
                  label="Only crawl direct links from specified links"
                  for="onlyCrawlDirectLinksFromSpecifiedLinks"
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
