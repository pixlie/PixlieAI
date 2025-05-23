import { Component, createSignal, Show } from "solid-js";

import IconButton from "../../widgets/interactable/IconButton";
import HelpIcon from "../../assets/icons/tabler-help.svg";
import MailIcon from "../../assets/icons/tabler-mail.svg";
import CalendlyIcon from "../../assets/icons/brand-calendly.svg";
import DiscordIcon from "../../assets/icons/brand-discord.svg";
import GitHubIcon from "../../assets/icons/brand-github.svg";
import ChevronRightIcon from "../../assets/icons/tabler-chevron-right.svg";

const HelpPopOver: Component = () => {
  const [visible, setVisible] = createSignal<boolean>(false);

  const options = [
    {
      link: "mailto:team@pixlie.com",
      title: "Send a message",
      icon: <MailIcon />,
    },
    {
      link: "https://calendly.com/sumitdatta/quick-chat",
      title: "Schedule a chat",
      icon: <CalendlyIcon />,
    },
    {
      link: "https://discord.gg/5W9U9RPTGp",
      title: "Join our community",
      icon: <DiscordIcon />,
    },
    {
      link: "https://github.com/pixlie/PixlieAI/issues",
      title: "Report a bug",
      icon: <GitHubIcon />,
    },
  ];

  return (
    <div class="relative w-10">
      <IconButton
        name="Help"
        icon={HelpIcon}
        onClick={() => setVisible(true)}
      />
      <Show when={visible()}>
        <button
          class="fixed inset-0 bg-slate-500/20 transition-opacity transition duration-500 ease-in-out z-10 cursor-default"
          onClick={() => setVisible(false)}
        />
        <div class="absolute right-0 z-20 whitespace-nowrap rounded-lg overflow-hidden shadow-md border-slate-200 border bg-white focus:outline-none flex flex-col">
          {options.map(({ link, icon, title }, i) => (
            <>
              {i > 0 && <hr />}
              <a
                href={link}
                class="flex w-full items-center px-4 py-3.5 gap-2 hover:bg-slate-50"
                target="_blank"
                rel="noreferrer"
              >
                <span class="gap-3 flex items-center w-full">
                  <span class="h-6 w-6 flex items-center justify-center">
                    {icon}
                  </span>
                  <p class="flex-1 font-medium text-gray-800 hover:text-gray-950">
                    {title}
                  </p>
                  <div class="text-slate-400">
                    <ChevronRightIcon />
                  </div>
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
