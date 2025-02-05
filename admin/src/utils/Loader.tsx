import {
  Component,
  createEffect,
  createResource,
  JSX,
  onMount,
} from "solid-js";
import { useWorkspace } from "../stores/workspace";
import { useLocation, useNavigate } from "@solidjs/router";

interface ILoaderProps {
  children: JSX.Element;
}

const Loader: Component<ILoaderProps> = (props) => {
  const [workspace, { fetchSettings, fetchSettingsStatus }] = useWorkspace();
  const [_settings, { refetch }] = createResource(async () => {
    await fetchSettings();
    await fetchSettingsStatus();
  });
  const navigate = useNavigate();
  const location = useLocation();

  onMount(() => {
    refetch();
  });

  createEffect(() => {
    if (workspace.isFetching) {
      return;
    } else if (workspace.isReady) {
      if (location.pathname.startsWith("/settings/setup")) {
        if (workspace.settingsStatus?.type === "Complete") {
          navigate("/");
        }
      } else if (
        workspace.settingsStatus &&
        workspace.settingsStatus.type !== "Complete"
      ) {
        navigate("/settings/setup");
      }
    }
  });

  return (
    <>
      {workspace.isFetching ? (
        <div class="w-screen h-screen">
          <div class="flex items-center justify-center w-full h-full">
            <div
              class={
                "inline-block h-8 w-8 animate-spin rounded-full border-4 border-solid border-current " +
                "border-e-transparent align-[-0.125em] text-surface " +
                "motion-reduce:animate-[spin_1.5s_linear_infinite] dark:text-white"
              }
              role="status"
            >
              <span class="!absolute !-m-px !h-px !w-px !overflow-hidden !whitespace-nowrap !border-0 !p-0 ![clip:rect(0,0,0,0)]">
                Loading...
              </span>
            </div>
          </div>
        </div>
      ) : (
        props.children
      )}
    </>
  );
};

export default Loader;
