import { Component, onMount } from "solid-js";
import { useWorkspace } from "../../stores/workspace";
// import ProjectListItem from "../../widgets/projects/ProjectListItem";
import ProjectForm from "../../widgets/projects/ProjectForm.tsx";
import { useLocation, useNavigate } from "@solidjs/router";

const ProjectList: Component = () => {
  const [_, { fetchProjects }] = useWorkspace();
  const location = useLocation();
  const navigate = useNavigate();

  onMount(() => {
    fetchProjects();
  });

  const handleClose = () => {
    if (location.search.length > 0) {
      navigate(`${location.pathname}?${location.search}`);
    } else {
      navigate(location.pathname);
      fetchProjects();
    }
  };

  return (
    <>
      <div class="relative">
        {location.hash === "#createProject" && (
          <ProjectForm displayAs="Drawer" onClose={handleClose} />
        )}
      </div>
      {/* <div class="sm:grid sm:grid-cols-2 sm:gap-4 sm:divide-y-0">
        {workspace.isReady && !!workspace.projects ? (
          <>
            {workspace.projects.map((project) => (
              <ProjectListItem {...project} />
            ))}
          </>
        ) : null}
      </div> */}
      <div class="h-full w-full flex overflow-hidden">
        <img
          src="https://pixlie.com/_astro/hero-image.DdgBYhys_WvsQV.webp"
          alt="Pixlie AI"
          height="100%"
          style={{
            "object-fit": "contain",
          }}
        />
      </div>
    </>
  );
};

export default ProjectList;
