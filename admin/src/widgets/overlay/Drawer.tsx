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
      class="relative z-10"
      aria-labelledby="slide-over-title"
      role="dialog"
      aria-modal="true"
    >
      {/*Background backdrop, show/hide based on slide-over state.*/}
      <div class="fixed inset-0 bg-gray-500/75 transition-opacity"></div>

      <div class="fixed inset-0 overflow-hidden">
        <div class="absolute inset-0 overflow-hidden">
          <div class="pointer-events-none fixed inset-y-0 right-0 flex max-w-full pl-10 sm:pl-16">
            {/*Slide-over panel, show/hide based on slide-over state.*/}

            {/*Entering: "transform transition ease-in-out duration-500 sm:duration-700"*/}
            {/*  From: "translate-x-full"*/}
            {/*  To: "translate-x-0"*/}
            {/*Leaving: "transform transition ease-in-out duration-500 sm:duration-700"*/}
            {/*  From: "translate-x-0"*/}
            {/*  To: "translate-x-full"*/}
            <div class="pointer-events-auto w-screen max-w-md">
              <div class="flex h-full flex-col divide-y divide-gray-200 bg-white shadow-xl">
                <div class="h-0 flex-1 overflow-y-auto">
                  <div class="bg-indigo-700 px-4 py-6 sm:px-6">
                    <div class="flex items-center justify-between">
                      <h2
                        class="text-base font-semibold text-white"
                        id="slide-over-title"
                      >
                        {props.title}
                      </h2>
                      <div class="ml-3 flex h-7 items-center">
                        <button
                          type="button"
                          class="relative rounded-md bg-indigo-700 text-indigo-200 hover:text-white focus:outline-none focus:ring-2 focus:ring-white"
                          onClick={handleClose}
                        >
                          <span class="absolute -inset-2.5"></span>
                          <span class="sr-only">Close panel</span>
                          <svg
                            class="size-6"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke-width="1.5"
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
                    </div>
                    <div class="mt-1">
                      <p class="text-sm text-indigo-300">{props.subtitle}</p>
                    </div>
                  </div>
                  <div class="flex flex-1 flex-col justify-between">
                    <div class="divide-y divide-gray-200 px-4 sm:px-6">
                      <div class="pb-5 pt-6">{props.content}</div>
                    </div>
                  </div>
                </div>
                <div class="flex shrink-0 justify-end px-4 py-4">
                  {props.footer}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Drawer;
