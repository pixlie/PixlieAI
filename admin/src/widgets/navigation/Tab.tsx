import { useNavigate, useSearchParams } from "@solidjs/router";
import { Component } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses.tsx";

interface TabItemProps {
  label: string;
  href?: string;
  searchParamKey?: string; // Set either href or searchParamKey (key=Label)
  searchParamValue?: string; // Set when searchParamKey is set
  isActive?: boolean;
  count?: number;
}

const TabItem: Component<TabItemProps> = (props) => {
  const [_searchParams, setSearchParams] = useSearchParams();
  const navigate = useNavigate();
  const [_, { getColors }] = useUIClasses();

  const handleClick = () => {
    if (!!props.href) {
      navigate(props.href);
    } else if (!!props.searchParamKey && !!props.searchParamValue) {
      setSearchParams({ [props.searchParamKey]: props.searchParamValue });
    }
  };

  return (
    // Current: "border-indigo-500 text-indigo-600", Default: "border-transparent text-gray-500 hover:border-gray-200 hover:text-gray-700"
    <a
      onClick={handleClick}
      class={
        "flex whitespace-nowrap px-4 py-2 rounded-t text-sm font-medium cursor-pointer border " +
        getColors()["tabs.link"]
      }
    >
      {props.label}
      {/* Current: "bg-indigo-100 text-indigo-600", Default: "bg-gray-100 text-gray-900" */}
      {!!props.count ? (
        <span class="ml-3 hidden rounded-full bg-gray-100 px-2.5 py-0.5 text-xs font-medium text-gray-900 md:inline-block">
          {props.count}
        </span>
      ) : null}
    </a>
  );
};

interface TabProps {
  tabs: Array<TabItemProps>;
}

const Tabs: Component<TabProps> = (props) => {
  const [_, { getColors }] = useUIClasses();

  return (
    <div class={"border-b px-4 " + getColors()["tabs"]}>
      <nav class="-mb-px flex space-x-4" aria-label="Tabs">
        {props.tabs.map((tab) => (
          <TabItem {...tab} />
        ))}
      </nav>
    </div>
  );
};

export default Tabs;
