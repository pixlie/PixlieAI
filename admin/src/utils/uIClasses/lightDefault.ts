const lightDefault = {
  name: "Light default",
  type: "light",
  tailwindClasses: {
    app: "bg-stone-100 text-stone-800",

    sideBar: "bg-gray-200",
    "sideBar.logo": "text-gray-800",
    "sideBar.link": "text-gray-800 hover:bg-gray-300",
    "sideBar.link.active": "bg-gray-300",
    "sideBar.label": "text-gray-600",
    "sideBar.label.active": "text-gray-800 border-gray-400",

    mainContent: "",
    heading: "text-gray-800",

    label: "text-gray-600",
    text: "text-gray-700",
    input: "bg-gray-50 text-gray-700 border-gray-300",
    textArea: "bg-gray-50 text-gray-700 border-gray-300",
    button: "bg-green-600 text-white",
    link: "text-blue-600 hover:underline",

    tabs: "border-slate-300",
    "tabs.link":
      "text-gray-500 hover:text-gray-700 border-slate-300 border-b-stone-100",
  },
};

export default lightDefault;
