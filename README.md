# Pixlie AI
Knowledge graphs on your data using LLMs

## What is Pixlie AI?
Pixlie AI helps your create knowledge graphs on your data that can answer semantic questions.
Use LLMs like Anthropic's Claude to extract semantics. Then get insights from the graph.

Here is and overview of how to use Pixlie AI:
- Setup Pixlie AI on your computer (on  on the cloud)
- Start with a problem or question that you want to deep dive into
- Share your data with Pixlie AI or crawl the web
- Pixlie AI uses LLMs (Anthropic's Claude) classify your data (your API keys)
- Pixlie AI uses GLiNER (running locally or on the cloud) to extract semantics from your data
- Pixlie AI can crawl data from the web if youw want
- A knowledge graph is created that holds semantic information about your data
- Query the graph visually or programmatically


## Is Pixlie AI an alternative to using vector databases?
Yes, Pixlie AI is an alternative to using vector databases. Vector databases are good for storing and querying semantic data,
but is they do not model the underlying data accurately. In Pxlie AI, we use LLMs to classify individual pieces of semantically
meaningful data. Each individual entity, like a person, place, date, event, item of interest is stored separately in the graph.
Their relationships to other entitiies and also stored in the graph. This makes our graph based approach better where accuracy is
important.


## License
- Pixlie AI is licensed under the GNU General Public License version 3.0
- See the [LICENSE](LICENSE) file for details
