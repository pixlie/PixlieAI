# Pixlie AI
AI powered knowledge graphs for semantically accurate insights. From personal research to semantic search in your apps.

## What is Pixlie AI?
Pixlie AI helps you create knowledge graphs that store semantic information about your data.
It uses a combination of AI/ML models like GLiNER or Anthropic's Claude to extract semantics at a low cost.
The extracted semantics are stored in the graph, retaining the rich real world context of your data.
You can then get insights from the graph, either visually or programmatically.

Here is how it works:
- Setup Pixlie AI on your computer (or on the cloud)
- Start with a problem or question that you want to deep dive into
- Share your data with Pixlie AI or crawl the web
- Pixlie AI uses LLMs (Anthropic's Claude) to classify your data (your API keys)
- Pixlie AI uses GLiNER (running locally or on the cloud) to extract semantics from your data
- Pixlie AI can crawl data from the web if you want
- A knowledge graph is created that holds semantic information about your data
- Query the graph visually or programmatically

Start a project: 
![Start a Project](https://pixlie.com/images/PixlieAI-Start-Project.png)

Crawl a website:
![Crawl Workflow](https://pixlie.com/images/PixlieAI-Workflow-links.png)

## How can I use Pixlie AI?
Pixlie AI is open source and is under active development. Our work is done in public, please star this repository, it means a lot to us.
If you want to be notified when Pixlie AI is ready for use, please subscribe to our [insights newsletter](https://pixlie.com/insights).

## Documentation for developers
If you want to develop on Pixlie AI, please see the [DEVELOP.md](DEVELOP.md) file.

## Documentation for users
If you want to use Pixlie AI, please see the [USE.md](USE.md) file.

## REST API
Pixlie AI has a REST API that you can use to interact with the graph. We use Bruno to document the API.
You can find the API spec in the `rest_api` directory.

## Is Pixlie AI an alternative to using vector databases?
Yes, Pixlie AI is an alternative to using vector databases. Vector databases are good for storing and querying semantic data,
but they do not model the underlying data accurately. In Pixlie AI, we use LLMs to classify individual pieces of semantically
meaningful data. Each individual entity, such as a person, place, date, event, etc., is stored separately in the graph, along with its relationships to other entities. This makes our graph based approach better where accuracy is
important.

## License
- Pixlie AI is licensed under the GNU General Public License version 3.0
- See the [LICENSE](LICENSE) file for details
