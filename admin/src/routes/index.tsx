import { Component } from "solid-js";
import { Route } from "@solidjs/router";
import Setup from "./settings/Setup";
import SettingsContainer from "./Settings";
import Workflow from "./projects/Workflow";
import Insights from "./projects/Insights";
import Graph from "./projects/Graph";
import Crawl from "./projects/Crawl";
import ProjectList from "./projects/ProjectList";
import PerProjectWrapper from "./projects/PerProject";
import SearchResults from "./projects/SearchResults";

const Routes: Component = () => {
  return (
    <>
      <Route path="/p">
        <Route path="/" component={ProjectList} />
        <Route path="/:projectId" component={PerProjectWrapper}>
          <Route path="/workflow" component={Workflow} />
          <Route path="/insights" component={Insights} />
          <Route path="/graph" component={Graph} />
          <Route path="/searchResults" component={SearchResults} />
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
