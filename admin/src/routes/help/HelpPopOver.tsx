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
                  {icon}
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
