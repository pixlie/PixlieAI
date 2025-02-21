import { createResource, Component } from "solid-js";

interface IData {
  type: TypeChoices;
  insight: string;
  items: IItem[];
}

interface IItem {
  name: string;
  value: number;
  news?: string;
  url?: string;
}

enum TypeChoices {
  average = "average",
  percentage = "percentage",
  ratio = "ratio",
  list = "list",
}

const DUMMY_DATA: IData[] = [
  {
    type: TypeChoices.average,
    insight: "📊 Average of startups with [blank]",
    items: [{ name: "X", value: 50 }],
  },
  {
    type: TypeChoices.percentage,
    insight: "✨ [blank]% of startups have [blank].",
    items: [
      { name: "X", value: 75 },
      { name: "Y", value: 50 },
      { name: "Z", value: 25 },
    ],
  },
  {
    type: TypeChoices.ratio,
    insight: "💪 [blank] startups have twice as many [A] compared to [B].",
    items: [
      { name: "A", value: 50 },
      { name: "B", value: 25 },
    ],
  },
  {
    type: TypeChoices.list,
    insight: "🏆 Top 10 [blank] this week:",
    items: [
      { name: "Name", news: "[Recent news]", value: 1, url: "[URL]" },
      { name: "Name", news: "[Recent news]", value: 2, url: "[URL]" },
      { name: "Name", news: "[Recent news]", value: 3, url: "[URL]" },
      { name: "Name", news: "[Recent news]", value: 4, url: "[URL]" },
      { name: "Name", news: "[Recent news]", value: 5, url: "[URL]" },
      { name: "Name", news: "[Recent news]", value: 6, url: "[URL]" },
      { name: "Name", news: "[Recent news]", value: 7, url: "[URL]" },
      { name: "Name", news: "[Recent news]", value: 8, url: "[URL]" },
      { name: "Name", news: "[Recent news]", value: 9, url: "[URL]" },
      { name: "Name", news: "[Recent news]", value: 10, url: "[URL]" },
    ],
  },
];

const Average = ({ items }: { items: IItem[] }) => {
  return (
    <div class="flex h-full w-auto aspect-square rounded-full items-center justify-center bg-blue-100 shadow">
      <p class="text-center text-xl font-semibold">{`${items[0]?.name}`}</p>
    </div>
  );
};

const Percentage = ({ items = [] }: { items: IItem[] }) => {
  const colors = [
    "rgb(219 234 254)",
    "rgb(191 219 254)",
    "rgb(147 197 253)",
    "rgb(96 165 250)",
    "rgb(59 130 246)",
    "rgb(37 99 235)",
    "rgb(29 78 216)",
    "rgb(30 64 175)",
    "rgb(30 58 138)",
  ];
  return (
    <div class="flex flex-col h-full w-auto aspect-square items-center justify-center gap-2">
      <p class="text-center text-xl font-semibold">All Startups</p>
      <div class="h-full w-auto aspect-square rounded-full relative bg-gray-100 shadow">
        {items.map(({ name, value }, i) => (
          <div class="absolute bottom-0 flex flex-col h-full w-full items-center justify-end gap-2">
            <p class="text-center text-xl font-semibold">
              All Startups{" "}
              <span class="underline underline-offset-2">{`with ${name}`}</span>
            </p>
            <div
              class="flex items-center justify-center rounded-full shadow"
              style={{
                "background-color": colors[i],
                height: `${value}%`,
                width: `${value}%`,
              }}
            >
              {/* <p class="text-center text-xl font-semibold">{`${value}%`}</p> */}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

const Ratio = ({ items = [] }: { items: IItem[] }) => {
  const maxValue = Math.max(...items.map((item) => item.value));
  const size = (value: number) => Math.max((value / maxValue) * 100, 1);
  return (
    <div class="flex h-full w-auto items-center justify-center gap-6">
      {items.map(({ name, value }) => (
        <div
          class="flex h-full w-auto aspect-square items-center justify-center relative rounded-full bg-blue-100 shadow"
          style={{
            height: `${size(value)}%`,
            width: `${size(value)}%`,
          }}
        >
          <p class="text-center text-xl font-semibold">{name}</p>
        </div>
      ))}
    </div>
  );
};

const List = ({ items = [] }: { items: IItem[] }) => (
  <div class="flex flex-col gap-6">
    {items.map(({ name, value, news, url }) => (
      <div class="flex justify-start items-center gap-6">
        <span class="flex w-16 h-16 items-center justify-center rounded-full bg-blue-100 text-xl font-semibold">
          {value}
        </span>
        <div>
          <a
            href={`https://www.google.com/search?q=${name}`}
            target="_blank"
            rel="noreferrer"
            style={{ flex: 1 }}
            class="text-xl font-semibold"
          >
            {name}
          </a>
          <p class="text-xl">{news}</p>
        </div>
        {url && (
          <a
            href={url}
            target="_blank"
            rel="noreferrer"
            class="flex-1 text-right text-xl"
          >
            🔗
          </a>
        )}
      </div>
    ))}
  </div>
);

const Insights: Component = () => {
  // TODO: get actual data and replace dummy data
  const getData = async () => {
    try {
      const response = await fetch("", {
        method: "GET",
        headers: {
          "Content-Type": "application/json",
        },
      });
      if (response.ok) {
        return await response.json();
      }
      return DUMMY_DATA;
    } catch {
      return DUMMY_DATA;
    }
  };
  const [data] = createResource<IData[]>(() => getData());

  return (
    <>
      {data()?.map(({ type, insight, items }) => (
        <div class="flex flex-col min-h-dvh w-full justify-center items-center gap-16 p-16">
          <h2 class="flex flex-wrap font-semibold leading-tighter tracking-tighter text-center text-4xl">
            {insight}
          </h2>
          <div class="flex-1">
            {type === TypeChoices.average && <Average items={items} />}
            {type === TypeChoices.percentage && <Percentage items={items} />}
            {type === TypeChoices.ratio && <Ratio items={items} />}
            {type === TypeChoices.list && <List items={items} />}
          </div>
        </div>
      ))}
    </>
  );
};

export default Insights;
