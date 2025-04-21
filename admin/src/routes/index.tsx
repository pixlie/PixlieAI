import { Component } from "solid-js";
import { Route } from "@solidjs/router";
import Setup from "./settings/Setup";
import SettingsContainer from "./settings/SettingsContainer";
import Workflow from "./projects/Workflow";
import Insights from "./projects/Insights";
import Data from "./projects/Data.tsx";
import Crawl from "./projects/Crawl";
import Search from "./projects/Search";
import PerProjectWrapper from "./projects/PerProject";
import HelpContainer from "./help/HelpContainer";
import Contact from "./help/Contact";
import CreateProject from "./projects/CreateProject.tsx";
import Graph from "./projects/Graph.tsx";
import Explorer from "./projects/Explorer.tsx";

const Routes: Component = () => {
  return (
    <>
      <Route path="/">
        <Route path="" component={CreateProject} />

        <Route path="/p">
          <Route path="/create" component={CreateProject} />
          <Route path="/:projectId" component={PerProjectWrapper}>
            <Route path="/workflow" component={Workflow} />
            <Route path="/explorer" component={Explorer} />
            <Route path="/insights" component={Insights} />
            <Route path="/data" component={Data} />
            <Route path="/search" component={Search} />
            <Route path="/crawl" component={Crawl} />
            <Route path="/graph" component={Graph} />
          </Route>
        </Route>

        <Route path="/settings" component={SettingsContainer}>
          <>
            <Route path="/setup" component={Setup} />
            <Route path="" component={Setup} />
          </>
        </Route>

        <Route path="/help" component={HelpContainer}>
          <>
            <Route path="/contact" component={Contact} />
            <Route path="" component={Contact} />
          </>
        </Route>
      </Route>
    </>
  );
};

export default Routes;
