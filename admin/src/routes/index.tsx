import { Component } from "solid-js";
import { Route } from "@solidjs/router";
import PerProjectWrapper from "./projects/PerProject";
import HelpContainer from "./help/HelpContainer";
import Contact from "./help/Contact";
import Terminal from "./projects/Terminal.tsx";
import Workflow from "./projects/Workflow";
import Insights from "./projects/Insights";
import Crawl from "./projects/Crawl";
import Search from "./projects/Search";
import Graph from "./projects/Graph.tsx";
import Explorer from "./projects/Explorer.tsx";
import Results from "./projects/Results.tsx";

const Routes: Component = () => {
  return (
    <>
      <Route path="/">
        <Route path="" component={Terminal} />

        <Route path="/p" component={PerProjectWrapper}>
          <Route path="/:projectId">
            <Route path="" component={Terminal} />
            <Route path="/workflow" component={Workflow} />
            <Route path="/explorer" component={Explorer} />
            <Route path="/insights" component={Insights} />
            <Route path="/results" component={Results} />
            <Route path="/search" component={Search} />
            <Route path="/crawl" component={Crawl} />
            <Route path="/graph" component={Graph} />
          </Route>
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
