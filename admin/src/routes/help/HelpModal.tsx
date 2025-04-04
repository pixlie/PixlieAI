import { Component } from "solid-js";
import { useLocation, useNavigate } from "@solidjs/router";
import Modal from "../../widgets/overlay/Modal";

const HelpModal: Component = () => {
  const navigate = useNavigate();
  const location = useLocation();

  const title = "Help";
  const options = [
    {
      link: "mailto:team@pixlie.com",
      icon: "mail",
      title: "Send a message",
      subtitle: "to team@pixlie.com",
    },
    {
      link: "https://calendly.com/sumitdatta/quick-chat",
      icon: "brand-calendly",
      title: "Schedule a chat",
      subtitle: "on Calendly",
    },
    {
      link: "https://discord.gg/5W9U9RPTGp",
      icon: "brand-discord",
      title: "Join our community",
      subtitle: "on Discord",
    },
    {
      link: "https://github.com/pixlie/PixlieAI/issues",
      icon: "brand-github",
      title: "Report a bug",
      subtitle: "on GitHub",
    },
  ];

  const Content: Component = () => {
    return (
      <div class="flex flex-col pt-6 pb-3 gap-6">
        {options.map(({ link, title, subtitle }) => (
          <a
            href={link}
            class="px-6 py-3 border border-gray-100 hover:bg-gray-50 rounded-lg flex items-center justify-between shadow"
            target="_blank"
            rel="noreferrer"
          >
            <span class="gap-5 flex items-center">
              <span class="flex flex-col">
                <p class="text-gray-700 font-medium">{title}</p>
                <p class="text-gray-400 text-sm">{subtitle}</p>
              </span>
            </span>
          </a>
        ))}
      </div>
    );
  };

  return (
    <>
      <Modal
        title={title}
        content={<Content />}
        onClose={() => navigate(location.pathname)}
      />
    </>
  );
};

export default HelpModal;
