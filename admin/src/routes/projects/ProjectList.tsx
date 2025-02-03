import { Component, onMount } from "solid-js";
import { useWorkspace } from "../../stores/workspace";
import ProjectListItem from "../../widgets/projects/ProjectListItem";
import Heading from "../../widgets/typography/Heading";
import ProjectForm from "../../widgets/projects/ProjectForm.tsx";
import Button from "../../widgets/interactable/Button";
import { useLocation, useNavigate } from "@solidjs/router";

const ProjectList: Component = () => {
  const [workspace, { fetchProjects }] = useWorkspace();
  const location = useLocation();
  const navigate = useNavigate();

  onMount(() => {
    fetchProjects().then((_) => {});
  });

  const handleClose = () => {
    if (location.search.length > 0) {
      navigate(`${location.pathname}?${location.search}`);
    } else {
      navigate(location.pathname);
    }
  };

  return (
    <div class="relative">
      <Heading size={1}>Projects</Heading>
      {location.hash === "#createProject" && (
        <ProjectForm displayAs="Drawer" onClose={handleClose} />
      )}
      <div class="my-4">
        <Button label="Create a project" href="#createProject" />
      </div>

      <div class="sm:grid sm:grid-cols-2 sm:gap-4 sm:divide-y-0">
        {workspace.isReady && !!workspace.projects ? (
          <>
            {workspace.projects.map((project) => (
              <ProjectListItem {...project} />
            ))}
          </>
        ) : null}
      </div>
    </div>
  );
};

export default ProjectList;
