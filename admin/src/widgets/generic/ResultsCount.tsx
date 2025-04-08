import { Component } from "solid-js";
import LoaderIcon from "../../assets/icons/tabler-loader.svg";
import { useUIClasses } from "../../stores/UIClasses";

interface IPropTypes {
  count: number;
}

const ResultsCount: Component<IPropTypes> = (props) => {
  const [_, { getColors }] = useUIClasses();
  return (
    <div class={"flex items-center gap-2 text-md " + getColors()["textMedium"]}>
      <p>{`${props.count} Results`}</p>
      {props.count === 0 && (
        <div
          class={
            "w-4 h-4 flex justify-center items-center " +
            getColors()["textMedium"]
          }
        >
          <LoaderIcon />
        </div>
      )}
    </div>
  );
};

export default ResultsCount;
