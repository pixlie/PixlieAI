import { Component, createSignal, Show } from "solid-js";
import Paragraph from "../../widgets/typography/Paragraph";
import IconButton from "../../widgets/interactable/IconButton";
import InfoIcon from "../../assets/icons/tabler-info-circle.svg";

const InfoPopOver: Component = () => {
  const [visible, setVisible] = createSignal<boolean>(false);

  return (
    <div class="relative w-10">
      <div class="text-gray-400">
        <IconButton
          name="Learn More"
          icon={InfoIcon}
          onClick={() => setVisible(true)}
        />
      </div>
      <Show when={visible()}>
        <button
          class="fixed inset-0 bg-slate-500/20 transition-opacity transition duration-500 ease-in-out z-10"
          onClick={() => setVisible(false)}
        />
        <div class="absolute left-0 ml-2 mb-1.5 bottom-full z-50 w-72 rounded-md shadow-md border-slate-200 border bg-white focus:outline-none flex flex-col p-4 gap-4">
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
