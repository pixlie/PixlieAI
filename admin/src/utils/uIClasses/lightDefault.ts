interface ITheme {
  name: string;
  type: string;
  tailwindClasses: { [key: string]: string };
}

const lightDefault: ITheme = {
  name: "Light default",
  type: "light",
  tailwindClasses: {
    app: "bg-stone-100 text-stone-800",

    navBar: "bg-white",
    // "navBar.logo": "text-blue-700",
    "navBar.logo": "text-violet-700 hover:text-violet-900",
    "navBar.icon": "text-gray-800",
    "navBar.link": "text-gray-800",
    "navBar.link.active": "bg-gray-300",

    sideBar: "bg-white",
    "sideBar.logo": "text-gray-800",
    "sideBar.link": "text-gray-800 hover:bg-gray-100",
    "sideBar.link.active": "bg-gray-100",
    "sideBar.label": "text-gray-600",
    "sideBar.label.active": "text-gray-800 border-gray-400 bg-gray-100",

    mainContent: "bg-white",
    heading: "text-gray-800",

    label: "text-gray-800",
    text: "text-gray-700",
    textSuccess: "text-green-600",
    textInfo: "text-blue-600",
    textDanger: "text-red-600",
    textMuted: "text-gray-400",
    textWarning: "text-yellow-600",
    input: "bg-gray-50 text-gray-700 border-gray-300",
    textArea: "bg-gray-50 text-gray-700 border-gray-300",
    link: "text-blue-600 hover:underline",

    breadcrumb: "bg-stone-100",
    "breadcrumb.link": "text-gray-400",
    "breadcrumb.link.last": "text-gray-800",

    tabs: "border-stone-300",
    "tabs.link":
      "text-gray-500 hover:text-gray-700 border-slate-300 border-b-stone-100",

    "button.default": "bg-slate-500 text-white",
    "button.cancel": "bg-red-500 text-white",
    "button.secondary": "bg-blue-500 text-white",
    "button.success": "bg-green-600 text-white",
  },
};

export default lightDefault;
