import { Component, createSignal, Show } from "solid-js";
import ToolTip from "../../widgets/navigation/ToolTip";
import Icon from "../../widgets/interactable/Icon";
import Paragraph from "../../widgets/typography/Paragraph";

const InfoPopOver: Component = () => {
  const [visible, setVisible] = createSignal<boolean>(false);

  return (
    <div class="relative w-10">
      <ToolTip text="Info">
        <button
          onClick={() => setVisible(true)}
          aria-label="Info"
          class="flex items-center p-2 text-gray-800 hover:text-gray-950 hover:bg-slate-200 rounded-full"
        >
          <Icon name="info" />
        </button>
      </ToolTip>
      <Show when={visible()}>
        <button
          class="fixed inset-0 bg-slate-500/20 transition-opacity transition duration-500 ease-in-out z-10"
          onClick={() => setVisible(false)}
        />
        <div class="absolute left-0 mt-1.5 z-20 w-72 rounded-md shadow-md border-slate-200 border bg-white focus:outline-none flex flex-col p-4 gap-4">
          <Paragraph size="sm">
            In plain English, tell Pixlie what you would like to extract from
            the web. Feel free to use topics and keywords that you care about.
          </Paragraph>
          <Paragraph size="sm">
            Pixlie uses your objective to ask AI for starting URLs and keywords
            to monitor on websites. Pixlie will continue crawling the web as
            long as pages match your objective.
          </Paragraph>
          <Paragraph size="sm">
            Pixlie can extract information that is relevant to your objective
            but this feature is still in beta. Pixlie can extract blog posts,
            job posts, people, companies, events, dates and locations.
          </Paragraph>
        </div>
      </Show>
    </div>
  );
};

export default InfoPopOver;
