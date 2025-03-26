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
import Home from "./Home";
import CreateWorkflow from "./projects/CreateWorkflow.tsx";

const Routes: Component = () => {
  return (
    <>
      <Route path="/">
        <Route path="" component={Home} />

        <Route path="/p">
          <Route path="/create" component={CreateWorkflow} />
          <Route path="/:projectId" component={PerProjectWrapper}>
            <Route path="/workflow" component={Workflow} />
            <Route path="/insights" component={Insights} />
            <Route path="/data" component={Data} />
            <Route path="/search" component={Search} />
            <Route path="/crawl" component={Crawl} />
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
