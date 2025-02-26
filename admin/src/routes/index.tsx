import { Component } from "solid-js";
import { Route } from "@solidjs/router";
import Setup from "./settings/Setup";
import SettingsContainer from "./settings/SettingsContainer.tsx";
import Workflow from "./projects/Workflow";
import Insights from "./projects/Insights";
import Graph from "./projects/Graph";
import Crawl from "./projects/Crawl";
import SearchResults from "./projects/SearchResults";
import PerProjectWrapper from "./projects/PerProject.tsx";
import HelpContainer from "./help/HelpContainer.tsx";
import Contact from "./help/Contact.tsx";
import Home from "./Home.tsx";

const Routes: Component = () => {
  return (
    <>
      <Route path="/">
        <Route path="" component={Home} />

        <Route path="/p">
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
