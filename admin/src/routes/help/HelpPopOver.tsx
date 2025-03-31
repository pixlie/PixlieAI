import { Component, createSignal, Show } from "solid-js";
import ToolTip from "../../widgets/navigation/ToolTip";
import Icon from "../../widgets/interactable/Icon";

const HelpPopOver: Component = () => {
  const [visible, setVisible] = createSignal<boolean>(false);

  const options = [
    {
      link: "mailto:team@pixlie.com",
      icon: "mail",
      title: "Send a message",
    },
    {
      link: "https://calendly.com/sumitdatta/quick-chat",
      icon: "brand-calendly",
      title: "Schedule a chat",
    },
    {
      link: "https://discord.gg/5W9U9RPTGp",
      icon: "brand-discord",
      title: "Join our community",
    },
    {
      link: "https://github.com/pixlie/PixlieAI/issues",
      icon: "brand-github",
      title: "Report a bug",
    },
  ];

  return (
    <div class="relative w-10">
      <ToolTip text="Help">
        <button
          onClick={() => setVisible(true)}
          aria-label="Help"
          class="flex items-center p-2 text-gray-800 hover:text-gray-950 hover:bg-slate-200 rounded-full"
        >
          <Icon name="help" />
        </button>
      </ToolTip>
      <Show when={visible()}>
        <button
          class="fixed inset-0 bg-slate-500/20 transition-opacity transition duration-500 ease-in-out z-10"
          onClick={() => setVisible(false)}
        />
        <div class="absolute right-0 mt-1.5 z-20 w-72 rounded-md shadow-md border-slate-200 border bg-white focus:outline-none flex flex-col py-2 gap-2">
          {options.map(({ link, icon, title }, i) => (
            <>
              {i > 0 && <hr />}
              <a
                href={link}
                class="flex w-full items-center px-4 py-1 gap-2 hover:bg-blue-100"
                target="_blank"
                rel="noreferrer"
              >
                <span class="gap-3 flex items-center w-full">
                  <Icon name={icon} />
                  <p class="flex-1 font-medium text-gray-800 hover:text-gray-950">
                    {title}
                  </p>
                  <Icon
                    name="chevron-right"
                    size={20}
                    class="text-slate-400 -mr-1.5"
                  />
                </span>
              </a>
            </>
          ))}
        </div>
      </Show>
    </div>
  );
};

export default HelpPopOver;
