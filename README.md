> [!NOTE]  
> Pixlie is currently in **beta**. We value your feedback as we progress towards a stable release.

# Pixlie

AI powered knowledge graphs for semantically accurate insights. From online search to private document search to semantic search in your apps.

## What is Pixlie?

Point Pixlie at any website and describe what you need - from pricing data to project details. Our open-source intelligent crawler does the rest, no coding required.

## Features

- Objective driven projects to create knowledge graphs ("Track companies on Indian stock exchanges")
- Use Anthropic Claude (other models' support coming soon) with your own API keys
- Use Brave Search API for web search with your own API keys
- Built-in simple web crawler
- Runs on your laptop or on the cloud
- Collaborate with your team (paid feature)
- Extract entities like people, places, dates, events, etc. (some paid features)
- Search your knowledge graph
- Use as your default search engine
- Simple to download and install
- Completely UI driven


Watch this video to see Pixlie in action:

[![Pixlie Latest Release](https://img.youtube.com/vi/mF9KuFYNF4s/0.jpg)](https://www.youtube.com/watch?v=mF9KuFYNF4s)

Choose your objective - what do you want to discover with Pixlie?:
![Set your objective](https://pixlie.com/images/screenshots/pixlie-screenshot-objective.png)

Information will instantly be populated. Let Pixlie search a while and see...


See the domains discovered with the Brave API and AI based on your objective:
![domains crawled](https://pixlie.com/images/screenshots/pixlie-screenshot-domains.png?v=2)

A list of outbound links found on these domains:
![links found](https://pixlie.com/images/screenshots/pixlie-screenshot-links.png?v=2)

The content extracted from these domains:
![Web pages relevant to your objective](https://pixlie.com/images/screenshots/pixlie-screenshot-webpages.png)

Search through the results for specifics:
![Search Results](https://pixlie.com/images/screenshots/pixlie-screenshot-search-resultsv0-2-0.png)

## How can I use Pixlie?

At the moment Pixlie can be used on your laptop for personal web research (desktop app coming soon!). Pixlie is open source, with the option of enterprise licensing for those who want to utilize Pixlie's search and knowledge graph within their product. 

Pixlie is open source and is under active development. Our work is done in public, please star this repository, it
means a lot to us.

## Documentation for developers

If you want to develop on Pixlie, please see the [DEVELOP.md](DEVELOP.md) file.

## Documentation for users

If you want to use Pixlie, please see the [USE.md](USE.md) file.

## REST API

Pixlie has a REST API that you can use to interact with the graph. We use Bruno to document the API.
You can find the API spec in the `rest_api` directory.

## Is Pixlie an alternative to using vector databases?

Yes, Pixlie is an alternative to using vector databases. Vector databases are good for storing and querying semantic
data, but they do not model the underlying data accurately. In Pixlie, we use LLMs to classify individual pieces of
semantically meaningful data. Each individual entity, such as a person, place, date, event, etc., is stored separately in the graph,
along with its relationships to other entities. This makes our graph based approach better where accuracy is important.

## Stay in Touch

If you want to be notified when Pixlie is ready for use, please subscribe to
our [insights newsletter](https://pixlie.com/insights).

Join the discussion or get support on [Discord](https://discord.gg/5W9U9RPTGp).

## License

- Pixlie is licensed under the GNU General Public License version 3.0
- See the [LICENSE](LICENSE) file for details
