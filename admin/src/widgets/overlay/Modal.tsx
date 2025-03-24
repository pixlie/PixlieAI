import { Component, JSX } from "solid-js";

interface IPropTypes {
  title: string;
  content: JSX.Element;
  onClose?: () => void;
}

const Modal: Component<IPropTypes> = (props) => {
  const handleClose = () => {
    if (props.onClose) {
      props.onClose();
    }
  };

  return (
    <div
      class="relative z-50"
      aria-labelledby="slide-over-title"
      role="dialog"
      aria-modal="true"
    >
      {/* Background */}
      <div class="fixed inset-0 bg-gray-500/75 transition-opacity" />
      <div class="fixed inset-0 overflow-hidden">
        <div class="absolute inset-0 overflow-hidden">
          <div class="pointer-events-none fixed w-full justify-center items-center inset-y-0 flex ">
            <div class="pointer-events-auto w-1/3 flex m-6 justify-center items-center ">
              {/* Modal */}
              <div class="flex h-full w-full flex-col rounded-md overflow-hidden bg-white text-black shadow-xl p-6">
                {/* Header */}
                <div class="flex justify-between items-center">
                  <h1 class="text-xl font-medium" id="slide-over-title">
                    {props.title}
                  </h1>
                  <button type="button" class="-m-1" onClick={handleClose}>
                    <svg
                      class="size-6"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke-width="2"
                      stroke="currentColor"
                      aria-hidden="true"
                      data-slot="icon"
                    >
                      <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        d="M6 18 18 6M6 6l12 12"
                      />
                    </svg>
                  </button>
                </div>
                {/* Content */}
                <div class="flex flex-col gap-4">{props.content}</div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Modal;
