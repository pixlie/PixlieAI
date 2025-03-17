> [!NOTE]  
> Pixlie is currently in **beta**. We value your feedback as we progress towards a stable release.

# Pixlie

AI powered knowledge graphs for semantically accurate insights. From personal research to semantic search in your apps.

## What is Pixlie?

Pixlie helps you create knowledge graphs that store semantic information about your data.

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

Choose a domain to crawl:
![Set your domain](https://pixlie.com/images/screenshots/pixlie-screenshot-website.png)

Set your keywords:
![Set your kewyords](https://pixlie.com/images/screenshots/pixlie-screenshot-searchterm.png)

See the domains on the website:
![Website's domain](https://pixlie.com/images/screenshots/pixlie-screenshot-domains.png)

See the links on the website:
![Website's outbound link](https://pixlie.com/images/screenshots/pixlie-screenshot-links.png)

See the search results:
![Search Results](https://pixlie.com/images/screenshots/pixlie-screenshot-search-results.png)

## How can I use Pixlie?

Pixlie is open source and is under active development. Our work is done in public, please star this repository, it
means a lot to us.
If you want to be notified when Pixlie is ready for use, please subscribe to
our [insights newsletter](https://pixlie.com/insights).

## Documentation for developers

If you want to develop on Pixlie, please see the [DEVELOP.md](DEVELOP.md) file.

## Documentation for users

If you want to use Pixlie, please see the [USE.md](USE.md) file.

## REST API

Pixlie has a REST API that you can use to interact with the graph. We use Bruno to document the API.
You can find the API spec in the `rest_api` directory.

## Is Pixlie an alternative to using vector databases?

Yes, Pixlie is an alternative to using vector databases. Vector databases are good for storing and querying semantic
data,
but they do not model the underlying data accurately. In Pixlie, we use LLMs to classify individual pieces of
semantically
meaningful data. Each individual entity, such as a person, place, date, event, etc., is stored separately in the graph,
along with its relationships to other entities. This makes our graph based approach better where accuracy is
important.

## License

- Pixlie is licensed under the GNU General Public License version 3.0
- See the [LICENSE](LICENSE) file for details
