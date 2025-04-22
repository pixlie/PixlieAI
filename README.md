> [!NOTE]  
> Pixlie is currently in **beta**. We value your feedback as we progress towards a stable release.

# Pixlie

AI powered knowledge graphs for semantically accurate insights. From online search to private document search to semantic search in your apps.

## What is Pixlie?

Pixlie is an autonomous knowledge construction system that transforms how you interact with information. Beyond simple data retrieval, Pixlie builds rich semantic understanding through dynamic knowledge graphs.
At its core, Pixlie employs a multi-agent AI architecture that:

- **Understands your objective** - Start with a simple prompt or question
- **Autonomously discovers relevant information** - Our intelligent agents extract, connect, and contextualize data
- **Constructs a living knowledge graph** - Entities and relationships are explicitly modeled, not just vectorized
- **Self-improves as it learns** - Internal prompts continuously optimize based on new discoveries

Unlike vector databases that flatten semantic meaning into numerical proximities, Pixlie preserves the true structure of information - people connect to organizations, events happen at specific times, concepts relate to each other through defined relationships - creating a foundation for genuinely intelligent insights.

## What makes Pixlie different?

### Beyond Vector Databases
Vector databases are good for storing and querying semantic data, but they don't model the underlying data accurately. Pixlie uses a sophisticated hybrid approach:

- **Multi-model intelligence** - We leverage both large language models and specialized smaller models tailored to specific tasks (entity recognition, classification, relationship extraction)
- **Task-specific agents** - Different components of our system use the most appropriate model for each job, balancing accuracy with efficiency
- **Granular semantic modeling** - Each entity (person, place, date, event) is stored as a distinct node in the graph with explicit relationships to other entities
- **Adaptive processing pipeline** - The system selects the optimal processing approach based on the content and objective

This architecture makes our approach more accurate for complex information needs while remaining computationally efficient and cost effective.

### Privacy-First
We're committed to true data privacy. Pixlie is completely self-hosted with no data leaving your environment - a crucial advantage in today's regulatory landscape.

### Transparent
We believe in transparency and community-driven development. Our core offering is open source with optional paid features for enterprise needs and we are building in public.

## Features

- **Objective driven projects** to create knowledge graphs ("Track companies on Indian stock exchanges")
- **Privacy-first architecture** - completely self-hosted with no data leaving your environment
- **Use Anthropic Claude** (other models' support coming soon) with your own API keys
- **Use Brave Search API** for web search with your own API keys
- Built-in simple web crawler
- **Runs anywhere** - on your laptop or on the cloud
- Team collaboration (paid feature)
- **Entity extraction** - identify people, places, dates, events, etc. (some paid features)
- Explore your knowledge graph (coming soon)
- **Open source** with transparent development


## See how Pixlie currently works

Choose your objective - what do you want to discover with Pixlie?:
![Set your objective](https://pixlie.com/images/screenshots/pixlie-screenshot-objective.png)

Information will instantly be populated. Let Pixlie search a while and see...


A list of links discovered based on your objective:
![links found](https://pixlie.com/images/screenshots/pixlie-screenshot-links.png?v=2)

A list of domains (as many links may come from one domain):
![domains crawled](https://pixlie.com/images/screenshots/pixlie-screenshot-domains.png?v=2)

The content extracted from these links:
![Web pages relevant to your objective](https://pixlie.com/images/screenshots/pixlie-screenshot-webpages.png)

Search through the results for specifics:
![Search Results](https://pixlie.com/images/screenshots/pixlie-screenshot-search-resultsv0-2-0.png)

## 🚀 Roadmap
We're actively developing Pixlie with these exciting features on the horizon:

### Coming Soon

- **Knowledge Graph Explorer** - Interactive visual interface to browse, explore, and edit your knowledge graph
- **Self-Correcting Prompt System** - AI that automatically refines its prompts as it learns from new information
- **Multi-Agent Architecture** - Specialized agents working together (crawler, classifier, analyzer) to build comprehensive knowledge graphs
- **Notifications** of new matches on monitored websites
- **Desktop Application** - Native experience across platforms
- **Advanced Document Support** - Ingest CSV, JSON, Markdown, and more file formats beyond web content

## How can I use Pixlie?

At the moment Pixlie can be used on your laptop for personal web research (desktop app coming soon!). Pixlie is open source, with the option of enterprise licensing for those who want to utilize Pixlie's search and knowledge graph within their product. 

Pixlie is under active development. Our work is done in public, please star this repository, it means a lot to us.

## Documentation for developers

If you want to develop on Pixlie, please see the [DEVELOP.md](DEVELOP.md) file.

## Documentation for users

If you want to use Pixlie, please see the [USE.md](USE.md) file.

## REST API

Pixlie has a REST API that you can use to interact with the graph. We use Bruno to document the API.

You can find the API spec in the `rest_api` directory.

## Contribute

We are happily taking contributions! If you are interested in contributing to Pixlie, please join our [Discord Channel](https://discord.gg/5W9U9RPTGp) or reach out to us to set up a call. Share where you want to contribute. We have a lot of _to dos_ stuck in our head that we are happy to put on our [project board](https://github.com/orgs/pixlie/projects/5) if we know someone is interested in taking it on. 

## Stay in Touch

If you want to be notified when Pixlie is ready for use, please subscribe to
our [insights newsletter](https://pixlie.com/insights).

Join the discussion or get support on [Discord](https://discord.gg/5W9U9RPTGp).

## License

- Pixlie is licensed under the GNU General Public License version 3.0
- See the [LICENSE](LICENSE) file for details

## FAQ

### Is Pixlie an alternative to using vector databases?

Yes, Pixlie is an alternative to using vector databases. Vector databases are good for storing and querying semantic
data, but they do not model the underlying data accurately. In Pixlie, we leverage both LLMs and specialized smaller models tailored to specific tasks (entity recognition, classification, relationship extraction). Each individual entity, such as a person, place, date, event, etc., is stored separately in the graph, along with its relationships to other entities. This makes our graph based approach better where accuracy is important.

### What is the difference between Domains, Links and WebPages?
**Link Nodes** are the URLs we discover using Brave & AI based on the Objective. When each Link Node is processed, we store domains for these Links in our database as separate nodes - one node for every domain.

We connect **Domain Nodes** to their Link Nodes in the following way:

Domain `—OwnerOf—>` Link(s)

Link `—BelongsTo—>` a Domain

As a part of the processing of Link Nodes, we then fetch their raw HTML content. Once we successfully fetch the content, we create a **WebPage Node** to store that content and then connect it to the Link Node:

Link `—PathOf—>` WebPage

and vice-versa

WebPage `—ContentOf—>` Link
