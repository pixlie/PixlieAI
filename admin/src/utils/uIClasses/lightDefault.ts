const lightDefault = {
  name: "Light default",
  type: "light",
  tailwindClasses: {
    app: "bg-stone-100 text-stone-800",

    sideBar: "bg-gray-200",
    "sideBar.logo": "text-gray-800",
    "sideBar.link": "text-gray-800 hover:bg-gray-300",
    "sideBar.link.active": "bg-gray-300",

    mainContent: "",
    heading: "text-gray-800",

    label: "text-gray-600",
    text: "text-gray-700",
    input: "bg-gray-50 text-gray-700 border-gray-300",
    textArea: "bg-gray-50 text-gray-700 border-gray-300",
    button: "bg-green-600 text-white",

    tabs: "border-stone-300",
    "tabs.link":
      "bg-stone-200 text-gray-500 hover:border-gray-200 hover:text-gray-700",
  },
};

export default lightDefault;
