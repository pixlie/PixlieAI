import { Component } from "solid-js";
import LoaderIcon from "../../assets/icons/tabler-loader.svg";
import { useUIClasses } from "../../stores/UIClasses";

interface IPropTypes {
  count: number;
}

const ResultsCount: Component<IPropTypes> = (props) => {
  const [_, { getColors }] = useUIClasses();
  return (
    <div class={"flex items-center gap-1 text-md leading-none cursor-default " + getColors()["textLight"]}>
      {props.count === 0 ? (
        <div
          class={
            "w-4 h-4 flex justify-center items-center " +
            getColors()["textMedium"]
          }
        >
          <LoaderIcon />
        </div>
      ) : (
        <p>{props.count}</p>
      )}
      <p>Results</p>
    </div>
  );
};

export default ResultsCount;
