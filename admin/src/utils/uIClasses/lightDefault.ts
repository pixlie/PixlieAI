import { Themable } from "./types";

interface ITheme {
  name: string;
  type: string;
  tailwindClasses: Themable;
}

const lightDefault: ITheme = {
  name: "Light default",
  type: "light",
  tailwindClasses: {
    app: "bg-slate-50 text-stone-800",

    navBar: "bg-white",
    // "navBar.logo": "text-blue-700",
    "navBar.logo": "text-violet-700 hover:text-violet-900",
    "navBar.icon": "text-gray-800",
    "navBar.link": "text-gray-800",
    "navBar.link.active": "bg-gray-300",

    sideBar: "bg-white border-slate-200",
    "sideBar.logo": "text-gray-800",
    "sideBar.link": "text-gray-800 hover:bg-gray-100",
    "sideBar.link.active": "bg-gray-100",
    "sideBar.label": "text-gray-600",
    "sideBar.label.active": "text-gray-800 border-gray-400 bg-gray-100",

    mainContent: "bg-white border-slate-200",
    heading: "text-gray-800",

    "form.label": "text-gray-800",
    text: "text-slate-800",
    textDark: "text-slate-950",
    textMedium: "text-slate-500",
    textLight: "text-slate-400",
    textMuted: "text-slate-300",
    textSoft: "text-gray-500",
    textSuccess: "text-green-600",
    textInfo: "text-blue-600",
    textDanger: "text-red-600",
    textWarning: "text-yellow-600",
    input: "bg-slate-100 text-gray-700 border-slate-200",
    textArea: "bg-gray-50 text-gray-700 border-gray-300",
    link: "text-blue-600 hover:underline",
    formError: "text-red-600",

    breadcrumb: "bg-stone-100",
    "breadcrumb.link": "text-gray-400",
    "breadcrumb.link.last": "text-gray-800",

    tabs: "border-stone-300",
    "tabs.link":
      "text-gray-500 hover:text-gray-700 border-slate-300 border-b-stone-100",

    "button.default": "bg-slate-500 hover:bg-slate-400 text-white",
    "button.cancel": "bg-red-600 hover:bg-red-500 text-white",
    "button.secondary": "bg-blue-500 text-white",
    "button.success": "bg-green-700 hover:bg-green-600 text-white",
    "button.muted": "bg-gray-300 text-gray-900",

    highlight: "bg-yellow-200",
  },
};

export default lightDefault;
