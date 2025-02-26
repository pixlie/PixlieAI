import { RouteSectionProps } from "@solidjs/router";
import { Component } from "solid-js";

const Contact: Component<RouteSectionProps> = () => {
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

  return (
    <>
      <p class="text-lg font-medium">Need help?</p>
      <div class="grid grid-cols-3 gap-6">
        {options.map(({ link, icon, title, subtitle }) => (
          <a
            href={link}
            class="p-6 border border-gray-300 hover:bg-gray-50 rounded-lg"
            target="_blank"
            rel="noreferrer"
          >
            <span class="gap-3 flex items-center justify-center">
              <p class="text-3xl">{icon}</p>
              <span class="gap-1 flex flex-col">
                <p class="text-gray-700">{title}</p>
                <p class="text-gray-400 text-sm font-medium">{subtitle}</p>
              </span>
            </span>
          </a>
        ))}
      </div>
    </>
  );
};

export default Contact;
