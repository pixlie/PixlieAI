import { Component } from "solid-js";
import { useLocation, useNavigate } from "@solidjs/router";
import Drawer from "../../widgets/overlay/Drawer";

const HelpModal: Component = () => {
  const navigate = useNavigate();
  const location = useLocation();

  const title = "Need help?";
  const options = [
    {
      link: "mailto:team@pixlie.com",
      icon: "✏️",
      title: "Send a message",
      subtitle: "to team@pixliie.com",
    },
    {
      link: "https://calendly.com/sumitdatta/quick-chat",
      icon: "📞",
      title: "Schedule a chat",
      subtitle: "with Sumit",
    },
    {
      link: "https://github.com/orgs/pixlie/projects/5",
      icon: "🐛",
      title: "Report a bug",
      subtitle: "on GitHub",
    },
  ];

  const Content: Component = () => {
    return (
      <div class="flex flex-col gap-6">
        {options.map(({ link, icon, title, subtitle }) => (
          <a
            href={link}
            class="p-6 border border-gray-300 hover:bg-gray-50 rounded-lg flex items-center justify-between"
            target="_blank"
            rel="noreferrer"
          >
            <span class="gap-6 flex items-center">
              <p class="text-3xl">{icon}</p>
              <span class="gap-1 flex flex-col">
                <p class="text-gray-700 font-medium">{title}</p>
                <p class="text-gray-400 text-sm">{subtitle}</p>
              </span>
            </span>
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="w-6 h-6"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
              <polyline points="15 3 21 3 21 9"></polyline>
              <line x1="10" y1="14" x2="21" y2="3"></line>
            </svg>
          </a>
        ))}
      </div>
    );
  };

  return (
    <>
      <div class="relative">
        <Drawer
          title={title}
          content={<Content />}
          onClose={() => navigate(location.pathname)}
        />
      </div>
    </>
  );
};

export default HelpModal;
