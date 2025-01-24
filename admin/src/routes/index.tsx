import { Component } from "solid-js";
import { Route } from "@solidjs/router";
import Setup from "./settings/Setup";
import SettingsContainer from "./Settings";
import Workflow from "./perProject/Workflow";
import Insights from "./perProject/Insights";
import Graph from "./perProject/Graph";
import Crawl from "./perProject/Crawl";
import ProjectList from "../widgets/projects/ProjectList";
import PerProjectWrapper from "./perProject";

const Routes: Component = () => {
  return (
    <>
      <Route path="/p">
        <Route path="/" component={ProjectList} />
        <Route path="/:projectId" component={PerProjectWrapper}>
          <Route path="/workflow" component={Workflow} />
          <Route path="/insights" component={Insights} />
          <Route path="/graph" component={Graph} />
          <Route path="/crawl" component={Crawl} />
        </Route>
      </Route>
      <Route path="/settings" component={SettingsContainer}>
        <>
          <Route path="/setup" component={Setup} />
          <Route path="" component={Setup} />
        </>
      </Route>
    </>
  );
};

export default Routes;
