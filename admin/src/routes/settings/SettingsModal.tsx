import { Component } from "solid-js";
import { useLocation, useNavigate } from "@solidjs/router";
import Modal from "../../widgets/overlay/Modal";

import { useWorkspace } from "../../stores/workspace";
import StorageDir from "../../widgets/settings/StorageDir";
import Anthropic from "../../widgets/settings/Anthropic";
import BraveSearch from "../../widgets/settings/BraveSearch";

const SettingsModal: Component = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const [workspace] = useWorkspace();

  const Content: Component = () => {
    return (
      <div class="flex flex-col pt-4 pb-3 gap-5">
        {workspace.isReady ? (
          <>
            <StorageDir />
            {!!workspace.settings?.pathToStorageDir && (
              <>
                <Anthropic />
                <BraveSearch />
              </>
            )}
          </>
        ) : (
          <div class="my-12">Loading...</div>
        )}
      </div>
    );
  };

  return (
    <Modal
      title="Settings"
      content={<Content />}
      onClose={() => navigate(location.pathname)}
    />
  );
};

export default SettingsModal;
