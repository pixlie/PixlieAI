import {
  Component,
  createSignal,
} from "solid-js";
import TextInput from "../interactable/TextInput";
import { useWorkspace } from "../../stores/workspace";
import { createStore } from "solid-js/store";
import SaveIcon from "../../assets/icons/tabler-device-floppy.svg";
import CheckIcon from "../../assets/icons/tabler-check.svg";
import IconButton from "../interactable/IconButton";

interface IFormData {
  sendgrid_sender_email: string;
  sendgrid_receiver_email: string;
}

const Notifications: Component = () => {
  const [workspace, { fetchWorkspace, saveWorkspace }] = useWorkspace();
  const [formData, setFormData] = createStore<IFormData>({
    sendgrid_sender_email: workspace.workspace?.sendgrid_sender_email || "",
    sendgrid_receiver_email: workspace.workspace?.sendgrid_receiver_email || "",
  });
  const [errorMessage, setErrorMessage] = createSignal<string>("");
  const [senderSaved, setSenderSaved] = createSignal<boolean>(false);
  const [receiverSaved, setReceiverSaved] = createSignal<boolean>(false);

  const handleChange = (name: string, value: string | number) => {
    setErrorMessage("");
    if (!!value && typeof value === "string") {
      setFormData((existing) => ({
        ...existing,
        [name]: value,
      }));
    }
  };

  const onSaveReceiverEmail = async () => {
    if (!formData.sendgrid_receiver_email) {
      setErrorMessage("Please enter a receiver email address");
      return;
    }

    // Basic email validation
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    if (!emailRegex.test(formData.sendgrid_receiver_email)) {
      setErrorMessage("Please enter a valid receiver email address");
      return;
    }

    saveWorkspace({
      sendgrid_receiver_email: formData.sendgrid_receiver_email,
    });
    fetchWorkspace();
    setReceiverSaved(true);
  };

  const onSaveSenderEmail = async () => {
    if (!formData.sendgrid_sender_email) {
      setErrorMessage("Please enter a sender email address");
      return;
    }

    // Basic email validation
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    if (!emailRegex.test(formData.sendgrid_sender_email)) {
      setErrorMessage("Please enter a valid sender email address");
      return;
    }
    saveWorkspace({
      sendgrid_sender_email: formData.sendgrid_sender_email,
    });
    fetchWorkspace();
    setSenderSaved(true);
  };

  return (
    <div class="space-y-4">
      <div class="">
        <h3 class="text-lg font-medium text-slate-800 mb-2 flex items-center gap-2">
          {/* <svg class="w-5 h-5 text-slate-800" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 4.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
          </svg> */}
          Email Notifications
        </h3>
        {/* <p class="text-slate-600 text-sm">Configure email addresses for SendGrid notifications when content changes are detected.</p> */}
      </div>

      {workspace.isFetching ? (
        <div class="bg-white rounded-lg border border-slate-200 p-6">
          <div class="flex items-center justify-center">
            <div class="animate-spin rounded-full h-5 w-5 border-b-2 border-blue-600"></div>
            <span class="ml-2 text-sm text-slate-500">Loading...</span>
          </div>
        </div>
      ) : (
        <div class="bg-white rounded-lg border border-slate-200 p-4 hover:border-slate-300 transition-colors">
          <div class="flex items-center gap-3 mb-3">
            <div>
              <h4 class="font-medium text-slate-800">SendGrid</h4>
              {/* <p class="text-sm text-slate-500">Setup sender and receiver email addresses</p> */}
            </div>
          </div>
          <div class="mb-4">
            <div class="text-xs text-slate-600 space-y-1">
              <div class="flex items-center gap-1">
                <span class="text-slate-400">1.</span>
                Create an account{" "}
                <a
                  href="https://signup.sendgrid.com/"
                  target="_blank"
                  rel="noreferrer"
                  class="inline-flex items-center gap-1 text-blue-600 hover:text-blue-700 font-medium"
                >
                  here
                  <svg
                    class="w-3 h-3"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
                    />
                  </svg>
                </a>
              </div>
              <div class="flex items-center gap-1">
                <span class="text-slate-400">2.</span>
                Verify your sender email in SendGrid
              </div>
              <div class="flex items-center gap-1">
                <span class="text-slate-400">3.</span>
                Enter both email addresses below:
              </div>
            </div>
          </div>

          <div class="space-y-4">
            <div>
              <div class="flex items-center gap-3">
                <label class="block text-sm font-medium text-slate-700 w-10">
                  To:
                </label>
                <div class="flex-1">
                  <TextInput
                    name="sendgrid_receiver_email"
                    isEditable
                    onChange={handleChange}
                    onFocus={() => setReceiverSaved(false)}
                    value={formData.sendgrid_receiver_email}
                  />
                </div>
                <div class="flex-shrink-0">
                  {!receiverSaved() ? (
                    <IconButton
                      name="Save"
                      icon={SaveIcon}
                      onClick={onSaveReceiverEmail}
                    />
                  ) : (
                    <div class="p-2 text-green-600">
                      <CheckIcon />
                    </div>
                  )}
                </div>
              </div>
            </div>
            <div>
              <div class="flex items-center gap-3">
                <label class="block text-sm font-medium text-slate-700 w-10">
                  From:
                </label>
                <div class="flex-1">
                  <TextInput
                    name="sendgrid_sender_email"
                    isEditable
                    onChange={handleChange}
                    onFocus={() => setSenderSaved(false)}
                    value={formData.sendgrid_sender_email}
                  />
                </div>
                <div class="flex-shrink-0">
                  {!senderSaved() ? (
                    <IconButton
                      name="Save"
                      icon={SaveIcon}
                      onClick={onSaveSenderEmail}
                    />
                  ) : (
                    <div class="p-2 text-green-600">
                      <CheckIcon />
                    </div>
                  )}
                </div>
              </div>
            </div>
          </div>

          {!!errorMessage() && (
            <div class="flex items-center gap-2 text-red-600 text-sm bg-red-50 p-2 rounded-md mt-4">
              <svg
                class="w-4 h-4 flex-shrink-0"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
              {errorMessage()}
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export default Notifications;
