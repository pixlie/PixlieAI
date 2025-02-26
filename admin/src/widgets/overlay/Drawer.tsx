import { Component, JSX } from "solid-js";

interface IPropTypes {
  title: string;
  subtitle?: string;
  onClose?: () => void;
  content: JSX.Element;
  footer?: JSX.Element;
}

const Drawer: Component<IPropTypes> = (props) => {
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
      {/*Background backdrop, show/hide based on slide-over state.*/}
      <div class="fixed inset-0 bg-gray-500/75 transition-opacity"></div>

      <div class="fixed inset-0 overflow-hidden">
        <div class="absolute inset-0 overflow-hidden">
          <div class="pointer-events-none fixed w-full justify-center items-center inset-y-0 flex ">
            {/*Slide-over panel, show/hide based on slide-over state.*/}

            {/*Entering: "transform transition ease-in-out duration-500 sm:duration-700"*/}
            {/*  From: "translate-x-full"*/}
            {/*  To: "translate-x-0"*/}
            {/*Leaving: "transform transition ease-in-out duration-500 sm:duration-700"*/}
            {/*  From: "translate-x-0"*/}
            {/*  To: "translate-x-full"*/}
            <div class="pointer-events-auto w-1/2 flex m-6 justify-center items-center ">
              <div class="flex h-full w-full flex-col divide-y divide-gray-200 rounded-md overflow-hidden bg-white shadow-xl">
                <div class="h-0 flex-1 overflow-y-auto">
                  <div class="bg-violet-800 p-6">
                    <div class="flex items-center justify-between">
                      <h2
                        class="text-base font-semibold text-white"
                        id="slide-over-title"
                      >
                        {props.title}
                      </h2>
                      <button
                        type="button"
                        class="text-white -mr-1"
                        onClick={handleClose}
                      >
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
                    <div class="mt-1">
                      <p class="text-sm text-violet-300">{props.subtitle}</p>
                    </div>
                  </div>
                  <div class="flex flex-1 flex-col justify-between">
                    <div class="divide-y divide-gray-200 px-6">
                      <div class="pb-6 pt-5">{props.content}</div>
                    </div>
                  </div>
                </div>
                {props.footer && (
                  
                <div class="flex shrink-0 justify-end p-6">{props.footer}</div>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Drawer;
